// file: rust/src/types.rs
// description: domain types for the OpenAI/Azure Responses API
// reference: https://developers.openai.com/api/docs/api-reference/responses

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---- shared ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Developer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
}

impl ReasoningEffort {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "none" => Some(Self::None),
            "minimal" => Some(Self::Minimal),
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "xhigh" => Some(Self::Xhigh),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningSummary {
    Auto,
    Concise,
    Detailed,
}

impl ReasoningSummary {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "concise" => Some(Self::Concise),
            "detailed" => Some(Self::Detailed),
            _ => None,
        }
    }
}

// ---- request ----

#[derive(Debug, Clone, Serialize)]
pub struct InputMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ReasoningParams {
    pub effort: ReasoningEffort,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummary>,
}

#[derive(Debug, Serialize)]
pub struct ResponsesRequest<'a> {
    pub model: &'a str,
    pub input: &'a [InputMessage],
    pub max_output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<&'a str>,
}

// ---- response ----

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OutputTextContent {
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum MessageContent {
    OutputText(OutputTextContent),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ReasoningSummaryItem {
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ReasoningOutput {
    pub id: String,
    #[serde(default)]
    pub summary: Vec<ReasoningSummaryItem>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MessageOutput {
    pub id: String,
    pub role: Role,
    #[serde(default)]
    pub content: Vec<MessageContent>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum OutputItem {
    Reasoning(ReasoningOutput),
    Message(MessageOutput),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OutputTokensDetails {
    #[serde(default)]
    pub reasoning_tokens: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    #[serde(default)]
    pub output_tokens_details: Option<OutputTokensDetails>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct IncompleteDetails {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ResponsesResponse {
    pub id: String,
    pub status: String,
    pub model: String,
    #[serde(default)]
    pub output: Vec<OutputItem>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub incomplete_details: Option<IncompleteDetails>,
}

// ---- config ----

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub max_output_tokens: u32,
    pub api_version: String,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub reasoning_summary: Option<ReasoningSummary>,
}

// ---- errors ----

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("HTTP error {status}: {body}")]
    Http { status: u16, body: String },

    #[error("parse error: {0}")]
    Parse(#[from] reqwest::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
