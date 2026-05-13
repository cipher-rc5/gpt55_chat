// file: rust/src/client.rs
// description: async HTTP client for the OpenAI/Azure Responses API
// reference: https://developers.openai.com/api/docs/api-reference/responses

use reqwest::Client;
use reqwest::Url;
use tokio::time::{Duration, sleep};

use crate::tools;
use crate::types::{
    ChatError, ClientConfig, InputItem, LogLevel, MessageContent, OutputItem, Provider,
    ReasoningParams, ResponsesRequest, ResponsesResponse, Role,
};

const MAX_TOOL_ROUNDTRIPS: usize = 8;
const MAX_HTTP_ATTEMPTS: usize = 3;
const RETRY_BASE_DELAY_MS: u64 = 100;

/// Build the full request URL including the api-version query param.
fn build_url(config: &ClientConfig) -> Result<Url, ChatError> {
    let mut url = Url::parse(config.endpoint())
        .map_err(|e| ChatError::Config(format!("configured endpoint URL is invalid: {e}")))?;
    if url.query().is_some() {
        return Err(ChatError::Config(
            "configured endpoint URL must not contain query parameters".to_string(),
        ));
    }
    if config.provider() == Provider::Azure {
        if config.api_version().trim().is_empty() {
            return Err(ChatError::Config(
                "API_VERSION must not be empty".to_string(),
            ));
        }
        url.query_pairs_mut()
            .append_pair("api-version", config.api_version());
    }
    Ok(url)
}

/// Send a single request to the Responses API. Pass `previous_response_id`
/// to chain reasoning context across turns without manually replaying items.
pub async fn send_message(
    http: &Client,
    config: &ClientConfig,
    input: &[InputItem],
    previous_response_id: Option<&str>,
) -> Result<ResponsesResponse, ChatError> {
    validate_capabilities(config, previous_response_id)?;
    let url = build_url(config)?;

    let reasoning = config.reasoning_effort().map(|effort| ReasoningParams {
        effort,
        summary: config.reasoning_summary(),
    });

    let request_body = ResponsesRequest {
        model: config.model(),
        input,
        max_output_tokens: config.max_output_tokens(),
        instructions: config.instructions(),
        tools: config.tools(),
        reasoning,
        previous_response_id,
    };

    for attempt in 1..=MAX_HTTP_ATTEMPTS {
        let mut request = http
            .post(url.clone())
            .header("Content-Type", "application/json");
        request = match config.provider() {
            Provider::Azure => request.header("api-key", config.api_key()),
            Provider::OpenAiCompatible => {
                request.header("Authorization", format!("Bearer {}", config.api_key()))
            }
        };

        let response = match request.json(&request_body).send().await {
            Ok(response) => response,
            Err(e) if attempt < MAX_HTTP_ATTEMPTS => {
                sleep(retry_delay(None, attempt)).await;
                if config.log_level() == LogLevel::Verbose {
                    eprintln!("[retry: transport error on attempt {attempt}: {e}]");
                }
                continue;
            }
            Err(e) => return Err(ChatError::Transport(e)),
        };

        let status = response.status().as_u16();
        if response.status().is_success() {
            return Ok(response.json::<ResponsesResponse>().await?);
        }

        if is_retryable_status(status) && attempt < MAX_HTTP_ATTEMPTS {
            let delay = retry_delay(response.headers().get("retry-after"), attempt);
            if config.log_level() == LogLevel::Verbose {
                eprintln!("[retry: HTTP {status} on attempt {attempt}]");
            }
            sleep(delay).await;
            continue;
        }

        let body = match response.text().await {
            Ok(t) => t,
            Err(e) => format!("<failed to read response body: {e}>"),
        };
        return Err(ChatError::Http { status, body });
    }

    unreachable!("HTTP retry loop should return from every attempt")
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
            let output = tools::execute(&name, &arguments, config.tools_read_root());
            if config.log_level() == LogLevel::Verbose {
                eprintln!(
                    "[tool: {name} completed; args={} chars output={} chars]",
                    arguments.chars().count(),
                    output.chars().count()
                );
            }
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
    let mut messages = Vec::new();
    for item in &response.output {
        if let OutputItem::Message(msg) = item {
            let mut buf = String::new();
            for content in &msg.content {
                if let MessageContent::OutputText(text) = content {
                    buf.push_str(&text.text);
                }
            }
            if !buf.is_empty() {
                messages.push(buf);
            }
        }
    }
    if messages.is_empty() {
        None
    } else {
        Some(messages.join("\n\n"))
    }
}

fn validate_capabilities(
    config: &ClientConfig,
    previous_response_id: Option<&str>,
) -> Result<(), ChatError> {
    if config.provider() == Provider::OpenAiCompatible && !config.api_version().trim().is_empty() {
        return Err(ChatError::Config(
            "API_VERSION is only used with the azure provider".to_string(),
        ));
    }
    if previous_response_id.is_some() && config.model().trim().is_empty() {
        return Err(ChatError::Config(
            "response chaining requires a non-empty model".to_string(),
        ));
    }
    Ok(())
}

fn is_retryable_status(status: u16) -> bool {
    status == 408 || status == 429 || (500..=599).contains(&status)
}

fn retry_delay(header: Option<&reqwest::header::HeaderValue>, attempt: usize) -> Duration {
    if let Some(seconds) = header
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
    {
        return Duration::from_secs(seconds.min(5));
    }
    Duration::from_millis(RETRY_BASE_DELAY_MS * attempt as u64)
}
