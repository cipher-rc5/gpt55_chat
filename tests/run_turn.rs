// file: tests/run_turn.rs
// description: end-to-end tests for client::run_turn against a wiremock-stubbed Responses API

use std::time::Duration;

use gpt55_chat::client::{extract_reply, run_turn, send_message};
use gpt55_chat::types::{ChatError, ClientConfigBuilder, LogLevel, Provider};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(2))
        .build()
        .expect("build reqwest client")
}

async fn recorded_requests(server: &MockServer) -> Vec<wiremock::Request> {
    server
        .received_requests()
        .await
        .expect("request recording enabled")
}

#[tokio::test]
async fn run_turn_returns_message_on_completed_response() {
    let server = MockServer::start().await;

    let body = json!({
        "id": "resp_1",
        "status": "completed",
        "model": "test-model",
        "output": [
            {
                "type": "message",
                "id": "msg_1",
                "role": "assistant",
                "content": [
                    { "type": "output_text", "text": "hi there" }
                ]
            }
        ]
    });

    Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .build()
        .expect("build config");

    let http = build_http_client();
    let resp = run_turn(&http, &config, "hello", None)
        .await
        .expect("run_turn ok");
    assert_eq!(extract_reply(&resp), Some("hi there".to_owned()));
}

#[tokio::test]
async fn send_message_uses_azure_contract_and_only_api_key_header() {
    let server = MockServer::start().await;
    let body = json!({
        "id": "resp_1",
        "status": "completed",
        "model": "test-model",
        "output": []
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
        .instructions("be precise".to_owned())
        .max_output_tokens(123)
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");

    let http = build_http_client();
    let input = [gpt55_chat::types::InputItem::Message {
        role: gpt55_chat::types::Role::User,
        content: "hello".to_owned(),
    }];
    send_message(&http, &config, &input, Some("prev_1"))
        .await
        .expect("send_message ok");

    let requests = recorded_requests(&server).await;
    let request = &requests[0];
    assert_eq!(request.url.query(), Some("api-version=2025-test"));
    assert_eq!(request.headers.get("api-key").unwrap(), "test-key");
    assert!(request.headers.get("authorization").is_none());

    let sent: serde_json::Value = request.body_json().expect("request json");
    assert_eq!(sent["model"], "test-model");
    assert_eq!(sent["max_output_tokens"], 123);
    assert_eq!(sent["instructions"], "be precise");
    assert_eq!(sent["previous_response_id"], "prev_1");
    assert_eq!(sent["input"][0]["type"], "message");
    assert_eq!(sent["input"][0]["role"], "user");
}

#[tokio::test]
async fn send_message_uses_openai_contract_and_only_bearer_header() {
    let server = MockServer::start().await;
    let body = json!({
        "id": "resp_1",
        "status": "completed",
        "model": "test-model",
        "output": []
    });

    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/v1/responses", server.uri()))
        .api_key("test-key".to_owned())
        .model("test-model".to_owned())
        .provider(Provider::OpenAiCompatible)
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");

    let http = build_http_client();
    let input = [gpt55_chat::types::InputItem::Message {
        role: gpt55_chat::types::Role::User,
        content: "hello".to_owned(),
    }];
    send_message(&http, &config, &input, None)
        .await
        .expect("send_message ok");

    let requests = recorded_requests(&server).await;
    let request = &requests[0];
    assert_eq!(request.url.query(), None);
    assert_eq!(
        request.headers.get("authorization").unwrap(),
        "Bearer test-key"
    );
    assert!(request.headers.get("api-key").is_none());
}

#[tokio::test]
async fn run_turn_handles_function_call_roundtrip() {
    let server = MockServer::start().await;

    let first = json!({
        "id": "resp_1",
        "status": "completed",
        "model": "test-model",
        "output": [
            {
                "type": "function_call",
                "id": "fc_1",
                "call_id": "call_1",
                "name": "get_time",
                "arguments": "{}"
            }
        ]
    });

    let second = json!({
        "id": "resp_2",
        "status": "completed",
        "model": "test-model",
        "output": [
            {
                "type": "message",
                "id": "msg_1",
                "role": "assistant",
                "content": [
                    { "type": "output_text", "text": "all done" }
                ]
            }
        ]
    });

    // First stub: function_call. Scoped so it only matches once.
    let _first_scope = Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(first))
        .up_to_n_times(1)
        .expect(1)
        .mount_as_scoped(&server)
        .await;

    // Second stub: plain message. Mounted normally so it serves the next request.
    Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(second))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .build()
        .expect("build config");

    let http = build_http_client();
    let resp = run_turn(&http, &config, "what time is it?", None)
        .await
        .expect("run_turn ok");
    assert_eq!(extract_reply(&resp), Some("all done".to_owned()));

    let requests = recorded_requests(&server).await;
    let second_request: serde_json::Value = requests[1].body_json().expect("second request json");
    assert_eq!(second_request["previous_response_id"], "resp_1");
    assert_eq!(second_request["input"][0]["type"], "function_call_output");
    assert_eq!(second_request["input"][0]["call_id"], "call_1");
    assert!(
        second_request["input"][0]["output"]
            .as_str()
            .unwrap()
            .contains("iso8601_utc")
    );
}

#[tokio::test]
async fn send_message_returns_http_error_with_status_and_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad request"))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .build()
        .expect("build config");
    let input = [gpt55_chat::types::InputItem::Message {
        role: gpt55_chat::types::Role::User,
        content: "hello".to_owned(),
    }];
    let err = send_message(&build_http_client(), &config, &input, None)
        .await
        .expect_err("expected HTTP error");
    match err {
        ChatError::Http { status, body } => {
            assert_eq!(status, 400);
            assert_eq!(body, "bad request");
        }
        other => panic!("expected HTTP error, got {other:?}"),
    }
}

#[tokio::test]
async fn send_message_returns_transport_error_for_malformed_success_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .build()
        .expect("build config");
    let input = [gpt55_chat::types::InputItem::Message {
        role: gpt55_chat::types::Role::User,
        content: "hello".to_owned(),
    }];
    let err = send_message(&build_http_client(), &config, &input, None)
        .await
        .expect_err("expected transport/decode error");
    match err {
        ChatError::Transport(_) => {}
        other => panic!("expected transport error, got {other:?}"),
    }
}

#[tokio::test]
async fn send_message_retries_500_then_succeeds() {
    let server = MockServer::start().await;
    let fail = json!({"error":"temporary"});
    let ok = json!({
        "id": "resp_2",
        "status": "completed",
        "model": "test-model",
        "output": []
    });

    let _fail_scope = Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .respond_with(ResponseTemplate::new(500).set_body_json(fail))
        .up_to_n_times(1)
        .expect(1)
        .mount_as_scoped(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok))
        .expect(1)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");
    let input = [gpt55_chat::types::InputItem::Message {
        role: gpt55_chat::types::Role::User,
        content: "hello".to_owned(),
    }];
    send_message(&build_http_client(), &config, &input, None)
        .await
        .expect("retry succeeds");
}

#[tokio::test]
async fn run_turn_errors_after_max_tool_roundtrips() {
    let server = MockServer::start().await;
    let function_call = json!({
        "id": "resp_loop",
        "status": "completed",
        "model": "test-model",
        "output": [{
            "type": "function_call",
            "id": "fc_1",
            "call_id": "call_1",
            "name": "get_time",
            "arguments": "{}"
        }]
    });

    Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_json(function_call))
        .expect(8)
        .mount(&server)
        .await;

    let config = ClientConfigBuilder::new()
        .endpoint(format!("{}/openai/responses", server.uri()))
        .api_key("test".to_owned())
        .model("test-model".to_owned())
        .api_version("2025-test".to_owned())
        .log_level(LogLevel::Quiet)
        .build()
        .expect("build config");
    let err = run_turn(&build_http_client(), &config, "loop", None)
        .await
        .expect_err("expected tool loop error");
    match err {
        ChatError::Tool(msg) => assert!(msg.contains("exceeded 8 tool-call roundtrips")),
        other => panic!("expected tool error, got {other:?}"),
    }
}
