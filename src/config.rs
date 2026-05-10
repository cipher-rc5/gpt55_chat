// file: rust/src/config.rs
// description: loads and validates ClientConfig from environment variables
// reference: https://docs.rs/dotenvy/latest/dotenvy/

use crate::types::{ChatError, ClientConfig, ReasoningEffort, ReasoningSummary};

const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 16384;

/// Read a required, non-empty env var.
fn require_env(key: &str) -> Result<String, ChatError> {
    let value = std::env::var(key)
        .map_err(|_| ChatError::Config(format!("{key} environment variable is not set")))?;

    if value.trim().is_empty() {
        return Err(ChatError::Config(format!("{key} must not be empty")));
    }

    Ok(value)
}

/// Read an optional, non-empty env var.
fn optional_env(key: &str) -> Option<String> {
    std::env::var(key).ok().and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

/// Load configuration from the environment.
///
/// Required: `AZURE_API_KEY`, `AZURE_RESOURCE`, `API_VERSION`, `MODEL`.
/// Optional: `REASONING_EFFORT` (none|minimal|low|medium|high|xhigh),
///           `REASONING_SUMMARY` (auto|concise|detailed),
///           `MAX_OUTPUT_TOKENS` (defaults to 16384).
pub fn load_config() -> Result<ClientConfig, ChatError> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(e) if e.not_found() => {}
        Err(e) => return Err(ChatError::Config(format!(".env load failed: {e}"))),
    }

    let api_key = require_env("AZURE_API_KEY")?;
    let resource = require_env("AZURE_RESOURCE")?;
    let api_version = require_env("API_VERSION")?;
    let model = require_env("MODEL")?;

    let resource = resource.trim_end_matches('/');
    let endpoint = format!("{resource}/openai/responses");

    let reasoning_effort = match optional_env("REASONING_EFFORT") {
        Some(raw) => Some(ReasoningEffort::parse(&raw).ok_or_else(|| {
            ChatError::Config(format!(
                "REASONING_EFFORT '{raw}' is invalid (expected none|minimal|low|medium|high|xhigh)"
            ))
        })?),
        None => None,
    };

    let reasoning_summary = match optional_env("REASONING_SUMMARY") {
        Some(raw) => Some(ReasoningSummary::parse(&raw).ok_or_else(|| {
            ChatError::Config(format!(
                "REASONING_SUMMARY '{raw}' is invalid (expected auto|concise|detailed)"
            ))
        })?),
        None => None,
    };

    let max_output_tokens = match optional_env("MAX_OUTPUT_TOKENS") {
        Some(raw) => raw.parse::<u32>().map_err(|_| {
            ChatError::Config(format!("MAX_OUTPUT_TOKENS '{raw}' is not a valid u32"))
        })?,
        None => DEFAULT_MAX_OUTPUT_TOKENS,
    };

    Ok(ClientConfig {
        endpoint,
        api_key,
        model,
        max_output_tokens,
        api_version,
        reasoning_effort,
        reasoning_summary,
    })
}
