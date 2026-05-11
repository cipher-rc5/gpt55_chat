// file: rust/src/client.rs
// description: async HTTP client for the OpenAI/Azure Responses API
// reference: https://developers.openai.com/api/docs/api-reference/responses

use reqwest::Client;

use crate::tools;
use crate::types::{
    ChatError, ClientConfig, InputItem, MessageContent, OutputItem, ReasoningParams, Role,
    ResponsesRequest, ResponsesResponse,
};

const MAX_TOOL_ROUNDTRIPS: usize = 8;

/// Build the full request URL including the api-version query param.
fn build_url(config: &ClientConfig) -> String {
    format!("{}?api-version={}", config.endpoint, config.api_version)
}

/// Send a single request to the Responses API. Pass `previous_response_id`
/// to chain reasoning context across turns without manually replaying items.
pub async fn send_message(
    http: &Client,
    config: &ClientConfig,
    input: &[InputItem],
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
        instructions: config.instructions.as_deref(),
        tools: &config.tools,
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

/// Drive a full user turn: send the user message, then while the model
/// returns `function_call` items, execute them locally and feed the
/// `function_call_output` items back. Stops when the model returns a normal
/// message (or after `MAX_TOOL_ROUNDTRIPS` to bound runaway loops).
pub async fn run_turn(
    http: &Client,
    config: &ClientConfig,
    user_input: &str,
    previous_response_id: Option<&str>,
) -> Result<ResponsesResponse, ChatError> {
    let mut input: Vec<InputItem> = vec![InputItem::Message {
        role: Role::User,
        content: user_input.to_owned(),
    }];
    let mut prev: Option<String> = previous_response_id.map(str::to_owned);

    for _ in 0..MAX_TOOL_ROUNDTRIPS {
        let response = send_message(http, config, &input, prev.as_deref()).await?;

        let calls: Vec<(String, String, String)> = response
            .output
            .iter()
            .filter_map(|item| match item {
                OutputItem::FunctionCall(c) => {
                    Some((c.call_id.clone(), c.name.clone(), c.arguments.clone()))
                }
                _ => None,
            })
            .collect();

        if calls.is_empty() {
            return Ok(response);
        }

        let next_id = response.id.clone();
        let mut next_input = Vec::with_capacity(calls.len());
        for (call_id, name, arguments) in calls {
            let output = tools::execute(&name, &arguments, config.tools_read_root.as_deref());
            let preview: String = output.chars().take(80).collect();
            eprintln!("[tool: {name}({arguments}) → {preview}]");
            next_input.push(InputItem::FunctionCallOutput { call_id, output });
        }

        input = next_input;
        prev = Some(next_id);
    }

    Err(ChatError::Tool(format!(
        "exceeded {MAX_TOOL_ROUNDTRIPS} tool-call roundtrips in a single turn"
    )))
}

/// Concatenate all `output_text` fragments across every assistant `Message`
/// item in the output array. Returns `None` when the result is empty (no
/// message items, or all message items were empty / non-text).
pub fn extract_reply(response: &ResponsesResponse) -> Option<String> {
    let mut buf = String::new();
    for item in &response.output {
        if let OutputItem::Message(msg) = item {
            for content in &msg.content {
                if let MessageContent::OutputText(text) = content {
                    buf.push_str(&text.text);
                }
            }
        }
    }
    if buf.is_empty() { None } else { Some(buf) }
}
