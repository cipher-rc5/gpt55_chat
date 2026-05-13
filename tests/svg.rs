// file: tests/svg.rs
// description: integration tests for svg::convert against a wiremock-stubbed Responses API

use std::time::Duration;

use gpt55_chat::svg::{SvgStyle, convert};
use gpt55_chat::types::{ClientConfigBuilder, LogLevel};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

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
    let dir = std::env::temp_dir().join(format!("gpt55-svg-test-{label}-{stamp}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[tokio::test]
async fn convert_writes_svg_from_fenced_block() {
    let server = MockServer::start().await;
    let out_dir = unique_dir("write");

    let png_path = out_dir.join("logo.png");
    std::fs::write(&png_path, FIXTURE_PNG).expect("write fixture png");

    let svg_payload = "```svg\n<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 10 10\"><circle cx=\"5\" cy=\"5\" r=\"4\"/></svg>\n```";
    let body = json!({
        "id": "resp_1",
        "status": "completed",
        "model": "test-model",
        "output": [{
            "type": "message",
            "id": "msg_1",
            "role": "assistant",
            "content": [{ "type": "output_text", "text": svg_payload }]
        }]
    });

    Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test-key".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .image_out_dir(out_dir.clone())
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");

    let written = convert(&build_http_client(), &config, &png_path, SvgStyle::Combined)
        .await
        .expect("convert ok");
    assert_eq!(written, out_dir.join("logo.svg"));
    let text = std::fs::read_to_string(&written).expect("read svg");
    assert!(text.starts_with("<svg"));
    assert!(text.contains("</svg>"));

    // Verify the wire request was a message with text + input_image parts.
    let requests = server
        .received_requests()
        .await
        .expect("request recording enabled");
    let sent: serde_json::Value = requests[0].body_json().expect("request json");
    let parts = &sent["input"][0]["content"];
    assert_eq!(sent["input"][0]["type"], "message");
    assert!(parts.is_array());
    assert_eq!(parts[0]["type"], "input_text");
    assert_eq!(parts[1]["type"], "input_image");
    assert!(
        parts[1]["image_url"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
}

#[tokio::test]
async fn convert_errors_when_response_has_no_svg() {
    let server = MockServer::start().await;
    let out_dir = unique_dir("no-svg");
    let png_path = out_dir.join("logo.png");
    std::fs::write(&png_path, FIXTURE_PNG).expect("write fixture png");

    let body = json!({
        "id": "resp_1",
        "status": "completed",
        "model": "test-model",
        "output": [{
            "type": "message",
            "id": "msg_1",
            "role": "assistant",
            "content": [{ "type": "output_text", "text": "sorry I cannot help with that" }]
        }]
    });

    Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test-key".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .image_out_dir(out_dir.clone())
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");

    let err = convert(&build_http_client(), &config, &png_path, SvgStyle::Editable)
        .await
        .expect_err("expected tool error");
    let msg = format!("{err}");
    assert!(msg.contains("did not contain a `<svg>"));
}
