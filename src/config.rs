// file: rust/src/config.rs
// description: loads and validates ClientConfig from environment variables
// reference: https://docs.rs/dotenvy/latest/dotenvy/

use std::fs;
use std::path::PathBuf;

use crate::tools::builtin_tools;
use crate::types::{ChatError, ClientConfig, ReasoningEffort, ReasoningSummary, Tool};

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
    let body = system_prompt.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());

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
///           `OPENAI_TOOLS_READ_ROOT` (directory; sandbox root for the `read_file`
///           tool — when unset, `read_file` refuses to execute).
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

    let system_prompt = optional_env("OPENAI_SYSTEM_PROMPT");
    let rules = match optional_env("OPENAI_RULES_FILE") {
        Some(path) => load_rules(&path)?,
        None => Vec::new(),
    };
    let instructions = compose_instructions(system_prompt, rules);

    let tools: Vec<Tool> = match optional_env("OPENAI_TOOLS").as_deref() {
        Some("off") | Some("false") | Some("0") => Vec::new(),
        _ => builtin_tools(),
    };

    let tools_read_root = optional_env("OPENAI_TOOLS_READ_ROOT").map(PathBuf::from);

    Ok(ClientConfig {
        endpoint,
        api_key,
        model,
        max_output_tokens,
        api_version,
        reasoning_effort,
        reasoning_summary,
        instructions,
        tools,
        tools_read_root,
    })
}
