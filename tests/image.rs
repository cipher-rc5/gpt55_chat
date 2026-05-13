// file: tests/image.rs
// description: integration tests for image::generate against a wiremock-stubbed Azure image endpoint

use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use gpt55_chat::image::{ImageRequest, generate};
use gpt55_chat::types::{ChatError, ClientConfigBuilder, LogLevel};
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(2))
        .build()
        .expect("build reqwest client")
}

fn unique_dir(label: &str) -> std::path::PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("gpt55-image-test-{label}-{stamp}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Minimal 1x1 PNG bytes for use as a fixture payload (transparent pixel).
const FIXTURE_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

#[tokio::test]
async fn generate_decodes_b64_and_writes_png() {
    let server = MockServer::start().await;
    let out_dir = unique_dir("write");

    let body = json!({
        "data": [{
            "b64_json": BASE64_STANDARD.encode(FIXTURE_PNG)
        }]
    });

    Mock::given(method("POST"))
        .and(path("/openai/deployments/gpt-image-2/images/generations"))
        .and(query_param("api-version", "2024-02-01"))
        .and(header("api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test-key".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .image_deployment("gpt-image-2".to_owned())
        .image_api_version("2024-02-01".to_owned())
        .image_out_dir(out_dir.clone())
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");

    let req = ImageRequest::new("a red fox in an autumn forest");
    let written = generate(&build_http_client(), &config, &req)
        .await
        .expect("generate ok");
    assert_eq!(written.len(), 1);
    let bytes = std::fs::read(&written[0]).expect("read written file");
    assert_eq!(bytes, FIXTURE_PNG);
    assert!(written[0].extension().and_then(|s| s.to_str()) == Some("png"));
}

#[tokio::test]
async fn generate_sends_request_body_with_defaults() {
    let server = MockServer::start().await;
    let out_dir = unique_dir("body");
    let body = json!({
        "data": [{ "b64_json": BASE64_STANDARD.encode(FIXTURE_PNG) }]
    });

    Mock::given(method("POST"))
        .and(path("/openai/deployments/gpt-image-2/images/generations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test-key".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .image_deployment("gpt-image-2".to_owned())
        .image_out_dir(out_dir.clone())
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");

    let req = ImageRequest::new("hello");
    generate(&build_http_client(), &config, &req)
        .await
        .expect("generate ok");

    let requests = server
        .received_requests()
        .await
        .expect("request recording enabled");
    let sent: serde_json::Value = requests[0].body_json().expect("request json");
    assert_eq!(sent["prompt"], "hello");
    assert_eq!(sent["size"], "1024x1024");
    assert_eq!(sent["quality"], "high");
    assert_eq!(sent["n"], 1);
    assert_eq!(sent["output_format"], "png");
}

#[tokio::test]
async fn generate_returns_config_error_without_deployment() {
    let config = ClientConfigBuilder::new()
        .endpoint("https://example.test/openai/responses".to_owned())
        .api_key("test-key".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");

    let err = generate(&build_http_client(), &config, &ImageRequest::new("x"))
        .await
        .expect_err("expected config error");
    match err {
        ChatError::Config(msg) => assert!(msg.contains("AZURE_IMAGE_DEPLOYMENT")),
        other => panic!("expected config error, got {other:?}"),
    }
}

#[tokio::test]
async fn generate_rejects_empty_prompt() {
    let config = ClientConfigBuilder::new()
        .endpoint("https://example.test/openai/responses".to_owned())
        .api_key("test-key".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .image_deployment("gpt-image-2".to_owned())
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");

    let err = generate(&build_http_client(), &config, &ImageRequest::new("   "))
        .await
        .expect_err("expected validation error");
    match err {
        ChatError::Config(msg) => assert!(msg.contains("prompt must not be empty")),
        other => panic!("expected config error, got {other:?}"),
    }
}

#[tokio::test]
async fn generate_propagates_http_error() {
    let server = MockServer::start().await;
    let out_dir = unique_dir("http-err");
    Mock::given(method("POST"))
        .and(path("/openai/deployments/gpt-image-2/images/generations"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad prompt"))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test-key".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .image_deployment("gpt-image-2".to_owned())
        .image_out_dir(out_dir)
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");

    let err = generate(&build_http_client(), &config, &ImageRequest::new("x"))
        .await
        .expect_err("expected HTTP error");
    match err {
        ChatError::Http { status, body } => {
            assert_eq!(status, 400);
            assert_eq!(body, "bad prompt");
        }
        other => panic!("expected HTTP error, got {other:?}"),
    }
}
