// file: rust/src/types.rs
// description: domain types for the OpenAI/Azure Responses API
// reference: https://developers.openai.com/api/docs/api-reference/responses

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---- shared ----

/// Author role for a chat message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Developer,
}

/// Reasoning effort level requested from the model.
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
    /// Parse a case-insensitive string into a `ReasoningEffort`.
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

    /// Lowercase string label for this variant.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        }
    }
}

/// Verbosity of the reasoning summary returned to the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningSummary {
    Auto,
    Concise,
    Detailed,
}

impl ReasoningSummary {
    /// Parse a case-insensitive string into a `ReasoningSummary`.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "concise" => Some(Self::Concise),
            "detailed" => Some(Self::Detailed),
            _ => None,
        }
    }

    /// Lowercase string label for this variant.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Concise => "concise",
            Self::Detailed => "detailed",
        }
    }
}

// ---- request ----

/// One item in the `input` array of a Responses API request.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputItem {
    /// A chat message authored by `role` with text `content`.
    Message {
        role: Role,
        content: String,
    },
    /// The output of a tool call identified by `call_id`.
    FunctionCallOutput {
        call_id: String,
        output: String,
    },
}

/// Reasoning configuration block for a request.
#[derive(Debug, Serialize)]
pub struct ReasoningParams {
    pub effort: ReasoningEffort,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummary>,
}

/// Definition of a function-style tool the model may invoke.
#[derive(Debug, Clone, Serialize)]
pub struct FunctionTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// A tool exposed to the model.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Tool {
    /// A locally-executed function tool.
    Function(FunctionTool),
}

/// Wire format for a single Responses API request.
#[derive(Debug, Serialize)]
pub struct ResponsesRequest<'a> {
    pub model: &'a str,
    pub input: &'a [InputItem],
    pub max_output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<&'a str>,
    #[serde(skip_serializing_if = "<[Tool]>::is_empty")]
    pub tools: &'a [Tool],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<&'a str>,
}

// ---- response ----

/// A text fragment emitted as part of a message output.
#[derive(Debug, Deserialize)]
pub struct OutputTextContent {
    pub text: String,
}

/// One piece of content inside a `MessageOutput`.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum MessageContent {
    /// A plain-text output fragment.
    OutputText(OutputTextContent),
    /// Any content type the client does not recognise.
    #[serde(other)]
    Other,
}

/// One entry inside a `ReasoningOutput.summary` array.
#[derive(Debug, Deserialize)]
pub struct ReasoningSummaryItem {
    #[serde(default)]
    #[allow(dead_code)]
    pub text: String,
}

/// A `reasoning` item in the model's `output` array.
#[derive(Debug, Deserialize)]
pub struct ReasoningOutput {
    #[allow(dead_code)]
    pub id: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub summary: Vec<ReasoningSummaryItem>,
}

/// A `message` item in the model's `output` array.
#[derive(Debug, Deserialize)]
pub struct MessageOutput {
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub role: Role,
    #[serde(default)]
    pub content: Vec<MessageContent>,
}

/// A `function_call` item in the model's `output` array.
#[derive(Debug, Deserialize)]
pub struct FunctionCall {
    #[allow(dead_code)]
    pub id: String,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}

/// One item in a Responses API response's `output` array.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum OutputItem {
    /// A reasoning trace item.
    Reasoning(#[allow(dead_code)] ReasoningOutput),
    /// An assistant message item.
    Message(MessageOutput),
    /// A function-call request from the model.
    FunctionCall(FunctionCall),
    /// Any item type the client does not recognise.
    #[serde(other)]
    Unknown,
}

/// Token-usage breakdown for the `output` portion of a response.
#[derive(Debug, Deserialize)]
pub struct OutputTokensDetails {
    #[serde(default)]
    pub reasoning_tokens: u32,
}

/// Total token usage reported for a response.
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    #[serde(default)]
    pub output_tokens_details: Option<OutputTokensDetails>,
}

/// Details attached when a response's `status` is `incomplete`.
#[derive(Debug, Deserialize)]
pub struct IncompleteDetails {
    pub reason: String,
}

/// Top-level body of a Responses API response.
#[derive(Debug, Deserialize)]
pub struct ResponsesResponse {
    pub id: String,
    pub status: String,
    #[allow(dead_code)]
    pub model: String,
    #[serde(default)]
    pub output: Vec<OutputItem>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub incomplete_details: Option<IncompleteDetails>,
}

// ---- config ----

/// Runtime configuration for the chat client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub max_output_tokens: u32,
    pub api_version: String,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub reasoning_summary: Option<ReasoningSummary>,
    pub instructions: Option<String>,
    pub tools: Vec<Tool>,
    /// Optional sandbox root for the read_file tool; when None, read_file refuses to execute.
    pub tools_read_root: Option<std::path::PathBuf>,
}

// ---- errors ----

/// Error type for all chat client operations.
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

    #[error("tool error: {0}")]
    Tool(String),
}
