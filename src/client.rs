// file: rust/src/client.rs
// description: async HTTP client for the OpenAI/Azure Responses API
// reference: https://developers.openai.com/api/docs/api-reference/responses

use reqwest::Client;

use crate::types::{
    ChatError, ClientConfig, InputMessage, MessageContent, OutputItem, ReasoningParams,
    ResponsesRequest, ResponsesResponse,
};

/// Build the full request URL including the api-version query param.
fn build_url(config: &ClientConfig) -> String {
    format!("{}?api-version={}", config.endpoint, config.api_version)
}

/// Send a single turn to the Responses API. Pass `previous_response_id` to
/// chain reasoning context across turns without manually replaying items.
pub async fn send_message(
    http: &Client,
    config: &ClientConfig,
    input: &[InputMessage],
    previous_response_id: Option<&str>,
) -> Result<ResponsesResponse, ChatError> {
    let url = build_url(config);

    let reasoning = config.reasoning_effort.map(|effort| ReasoningParams {
        effort,
        summary: config.reasoning_summary,
    });

    let request_body = ResponsesRequest {
        model: &config.model,
        input,
        max_output_tokens: config.max_output_tokens,
        reasoning,
        previous_response_id,
    };

    let response = http
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("api-key", &config.api_key)
        .json(&request_body)
        .send()
        .await?;

    let status = response.status().as_u16();

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_else(|_| "<unreadable>".into());
        return Err(ChatError::Http { status, body });
    }

    let parsed = response.json::<ResponsesResponse>().await?;
    Ok(parsed)
}

/// Concatenate all `output_text` fragments from the first assistant message
/// item in the output array.
pub fn extract_reply(response: &ResponsesResponse) -> String {
    for item in &response.output {
        if let OutputItem::Message(msg) = item {
            let mut buf = String::new();
            for content in &msg.content {
                if let MessageContent::OutputText(text) = content {
                    buf.push_str(&text.text);
                }
            }
            if !buf.is_empty() {
                return buf;
            }
        }
    }
    String::new()
}
