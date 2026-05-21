// file: src/config.rs
// description: loads and validates ClientConfig from environment variables
// reference: https://docs.rs/dotenvy/latest/dotenvy/

use std::fs;
use std::path::PathBuf;

use crate::tools::builtin_tools;
use reqwest::Url;

use crate::types::{
    ChatError, ClientConfig, LogLevel, Provider, ReasoningEffort, ReasoningSummary, Tool,
};

const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 16384;
const MAX_MAX_OUTPUT_TOKENS: u32 = 128_000;

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

/// Parse a rules file: one rule per non-empty, non-comment line.
pub fn load_rules(path: &str) -> Result<Vec<String>, ChatError> {
    let raw = fs::read_to_string(path)
        .map_err(|e| ChatError::Config(format!("failed to read rules file '{path}': {e}")))?;
    Ok(raw
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.trim_start_matches("- ").to_owned())
        .collect())
}

/// Compose `instructions` from an optional system prompt and zero or more rules.
pub fn compose_instructions(system_prompt: Option<String>, rules: Vec<String>) -> Option<String> {
    let body = system_prompt
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());

    if rules.is_empty() {
        return body;
    }

    let mut out = body.unwrap_or_default();
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str("# Rules\n");
    for rule in rules {
        out.push_str("- ");
        out.push_str(&rule);
        out.push('\n');
    }
    Some(out)
}

/// Load configuration from the environment.
///
/// Required: `AZURE_API_KEY`, `AZURE_RESOURCE`, `API_VERSION`, `MODEL`.
/// Optional: `REASONING_EFFORT` (none|minimal|low|medium|high|xhigh),
///           `REASONING_SUMMARY` (auto|concise|detailed),
///           `MAX_OUTPUT_TOKENS` (defaults to 16384),
///           `OPENAI_SYSTEM_PROMPT` (free-form instructions),
///           `OPENAI_RULES_FILE` (path; one rule per line, `#` for comments),
///           `OPENAI_TOOLS` (`off` to disable built-in tools; default: on),
///           `OPENAI_PROVIDER` (`azure` or `openai-compatible`; default `azure`),
///           `OPENAI_LOG` (`quiet|normal|verbose`; default `normal`),
///           `OPENAI_TOOLS_READ_ROOT` (directory; sandbox root for the `read_file`
///           tool — when unset, `read_file` refuses to execute),
///           `AZURE_IMAGE_DEPLOYMENT` (Azure deployment name for image generation,
///           e.g. `gpt-image-2`; when unset, `/image` is disabled),
///           `AZURE_IMAGE_API_VERSION` (defaults to `2024-02-01`),
///           `OPENAI_IMAGE_OUT_DIR` (output directory for `/image` and `/svg`;
///           defaults to `./images`).
pub fn load_config() -> Result<ClientConfig, ChatError> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(e) if e.not_found() => {}
        Err(e) => return Err(ChatError::Config(format!(".env load failed: {e}"))),
    }

    let provider = match optional_env("OPENAI_PROVIDER") {
        Some(raw) => Provider::parse(&raw).ok_or_else(|| {
            ChatError::Config(format!(
                "OPENAI_PROVIDER '{raw}' is invalid (expected azure|openai-compatible)"
            ))
        })?,
        None => Provider::Azure,
    };

    let api_key = match provider {
        Provider::Azure => require_env("AZURE_API_KEY")?,
        Provider::OpenAiCompatible => optional_env("OPENAI_API_KEY")
            .or_else(|| optional_env("AZURE_API_KEY"))
            .ok_or_else(|| {
                ChatError::Config("OPENAI_API_KEY environment variable is not set".to_string())
            })?,
    };
    let resource = match provider {
        Provider::Azure => require_env("AZURE_RESOURCE")?,
        Provider::OpenAiCompatible => optional_env("OPENAI_ENDPOINT").ok_or_else(|| {
            ChatError::Config("OPENAI_ENDPOINT environment variable is not set".to_string())
        })?,
    };
    let api_version = match provider {
        Provider::Azure => require_env("API_VERSION")?,
        Provider::OpenAiCompatible => optional_env("API_VERSION").unwrap_or_default(),
    };
    let model = require_env("MODEL")?;

    let endpoint = build_endpoint(provider, &resource)?;

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
    if max_output_tokens == 0 || max_output_tokens > MAX_MAX_OUTPUT_TOKENS {
        return Err(ChatError::Config(format!(
            "MAX_OUTPUT_TOKENS must be between 1 and {MAX_MAX_OUTPUT_TOKENS}"
        )));
    }

    let system_prompt = optional_env("OPENAI_SYSTEM_PROMPT");
    let rules = match optional_env("OPENAI_RULES_FILE") {
        Some(path) => load_rules(&path)?,
        None => Vec::new(),
    };
    let instructions = compose_instructions(system_prompt, rules);

    let tools_read_root = optional_env("OPENAI_TOOLS_READ_ROOT").map(PathBuf::from);
    let tools: Vec<Tool> = match optional_env("OPENAI_TOOLS").as_deref() {
        Some("off") | Some("false") | Some("0") => Vec::new(),
        _ => builtin_tools(tools_read_root.is_some()),
    };

    let log_level = match optional_env("OPENAI_LOG") {
        Some(raw) => LogLevel::parse(&raw).ok_or_else(|| {
            ChatError::Config(format!(
                "OPENAI_LOG '{raw}' is invalid (expected quiet|normal|verbose)"
            ))
        })?,
        None => LogLevel::Normal,
    };

    let image_deployment = optional_env("AZURE_IMAGE_DEPLOYMENT");
    let image_api_version =
        optional_env("AZURE_IMAGE_API_VERSION").unwrap_or_else(|| "2024-02-01".to_owned());
    let image_out_dir = optional_env("OPENAI_IMAGE_OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("images"));

    Ok(ClientConfig {
        endpoint,
        api_key,
        model,
        max_output_tokens,
        api_version,
        provider,
        reasoning_effort,
        reasoning_summary,
        instructions,
        tools,
        tools_read_root,
        log_level,
        image_deployment,
        image_api_version,
        image_out_dir,
    })
}

fn build_endpoint(provider: Provider, raw: &str) -> Result<String, ChatError> {
    let parsed = Url::parse(raw.trim())
        .map_err(|e| ChatError::Config(format!("endpoint/resource URL is invalid: {e}")))?;
    if parsed.scheme() != "https"
        && parsed.host_str() != Some("127.0.0.1")
        && parsed.host_str() != Some("localhost")
    {
        return Err(ChatError::Config(
            "endpoint/resource URL must use https".to_string(),
        ));
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(ChatError::Config(
            "endpoint/resource URL must not contain query or fragment".to_string(),
        ));
    }

    match provider {
        Provider::Azure => {
            if parsed
                .path()
                .trim_matches('/')
                .ends_with("openai/responses")
            {
                return Err(ChatError::Config(
                    "AZURE_RESOURCE must be the resource root, not /openai/responses".to_string(),
                ));
            }
            let mut endpoint = parsed;
            endpoint
                .path_segments_mut()
                .map_err(|_| ChatError::Config("AZURE_RESOURCE cannot be a base URL".to_string()))?
                .pop_if_empty()
                .push("openai")
                .push("responses");
            Ok(endpoint.to_string())
        }
        Provider::OpenAiCompatible => {
            if !parsed.path().trim_matches('/').ends_with("responses") {
                return Err(ChatError::Config(
                    "OPENAI_ENDPOINT must point at a responses endpoint".to_string(),
                ));
            }
            Ok(parsed.to_string())
        }
    }
}
