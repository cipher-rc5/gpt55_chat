// file: rust/src/types.rs
// description: domain types for the OpenAI/Azure Responses API
// reference: https://developers.openai.com/api/docs/api-reference/responses

use std::fmt;

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
    Message { role: Role, content: String },
    /// A chat message whose content is an ordered list of multi-modal parts
    /// (text and/or images). Serializes with `"type": "message"`, identical
    /// on the wire to the plain `Message` variant.
    #[serde(rename = "message")]
    MessageParts {
        role: Role,
        content: Vec<InputContentPart>,
    },
    /// The output of a tool call identified by `call_id`.
    FunctionCallOutput { call_id: String, output: String },
}

/// One content fragment inside a multi-modal input message.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputContentPart {
    /// Plain text fragment.
    InputText { text: String },
    /// Image fragment, supplied as a data URL (e.g. `data:image/png;base64,...`)
    /// or a remote https URL.
    InputImage { image_url: String },
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

/// API provider/auth mode used to build request URLs and headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// Azure OpenAI endpoint using the `api-key` header and `api-version` query parameter.
    Azure,
    /// OpenAI-compatible endpoint using `Authorization: Bearer`.
    OpenAiCompatible,
}

impl Provider {
    /// Parse a case-insensitive provider label.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "azure" => Some(Self::Azure),
            "openai" | "openai-compatible" | "compatible" => Some(Self::OpenAiCompatible),
            _ => None,
        }
    }

    /// Lowercase string label for this provider.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Azure => "azure",
            Self::OpenAiCompatible => "openai-compatible",
        }
    }
}

/// Runtime diagnostic verbosity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Suppress startup/token/tool diagnostics; keep user-facing replies and errors.
    Quiet,
    /// Default human-readable diagnostics.
    Normal,
    /// Include additional redacted tool diagnostics.
    Verbose,
}

impl LogLevel {
    /// Parse a case-insensitive log level label.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "quiet" | "off" | "0" => Some(Self::Quiet),
            "normal" | "on" | "1" => Some(Self::Normal),
            "verbose" | "debug" => Some(Self::Verbose),
            _ => None,
        }
    }
}

/// Runtime configuration for the chat client.
#[derive(Clone)]
pub struct ClientConfig {
    pub(crate) endpoint: String,
    pub(crate) api_key: String,
    pub(crate) model: String,
    pub(crate) max_output_tokens: u32,
    pub(crate) api_version: String,
    pub(crate) provider: Provider,
    pub(crate) reasoning_effort: Option<ReasoningEffort>,
    pub(crate) reasoning_summary: Option<ReasoningSummary>,
    pub(crate) instructions: Option<String>,
    pub(crate) tools: Vec<Tool>,
    /// Optional sandbox root for the read_file tool; when None, read_file refuses to execute.
    pub(crate) tools_read_root: Option<std::path::PathBuf>,
    pub(crate) log_level: LogLevel,
    /// Azure deployment name for the image-generation endpoint (e.g. `gpt-image-2`).
    pub(crate) image_deployment: Option<String>,
    /// api-version used for the image-generation endpoint (e.g. `2024-02-01`).
    pub(crate) image_api_version: String,
    /// Directory into which `/image` writes PNG/JPEG outputs and `/svg` writes SVG outputs.
    pub(crate) image_out_dir: std::path::PathBuf,
}

impl ClientConfig {
    /// Responses endpoint URL without query parameters.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Redacted-sensitive API key string.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Model/deployment identifier.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Maximum output tokens per request.
    pub fn max_output_tokens(&self) -> u32 {
        self.max_output_tokens
    }

    /// Azure API version, or empty for OpenAI-compatible providers.
    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    /// Configured API provider/auth mode.
    pub fn provider(&self) -> Provider {
        self.provider
    }

    /// Optional reasoning effort.
    pub fn reasoning_effort(&self) -> Option<ReasoningEffort> {
        self.reasoning_effort
    }

    /// Optional reasoning summary mode.
    pub fn reasoning_summary(&self) -> Option<ReasoningSummary> {
        self.reasoning_summary
    }

    /// Optional instructions sent with requests.
    pub fn instructions(&self) -> Option<&str> {
        self.instructions.as_deref()
    }

    /// Function tools advertised to the model.
    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    /// Optional sandbox root for `read_file`.
    pub fn tools_read_root(&self) -> Option<&std::path::Path> {
        self.tools_read_root.as_deref()
    }

    /// Runtime diagnostic verbosity.
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Azure image deployment name, when configured.
    pub fn image_deployment(&self) -> Option<&str> {
        self.image_deployment.as_deref()
    }

    /// api-version used for the image-generation endpoint.
    pub fn image_api_version(&self) -> &str {
        &self.image_api_version
    }

    /// Output directory for `/image` and `/svg` artifacts.
    pub fn image_out_dir(&self) -> &std::path::Path {
        &self.image_out_dir
    }
}

impl fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientConfig")
            .field("endpoint", &self.endpoint)
            .field("api_key", &"<redacted>")
            .field("model", &self.model)
            .field("max_output_tokens", &self.max_output_tokens)
            .field("api_version", &self.api_version)
            .field("provider", &self.provider)
            .field("reasoning_effort", &self.reasoning_effort)
            .field("reasoning_summary", &self.reasoning_summary)
            .field(
                "instructions",
                &self
                    .instructions
                    .as_ref()
                    .map(|s| format!("{} chars", s.len())),
            )
            .field("tools", &self.tools)
            .field("tools_read_root", &self.tools_read_root)
            .field("log_level", &self.log_level)
            .field("image_deployment", &self.image_deployment)
            .field("image_api_version", &self.image_api_version)
            .field("image_out_dir", &self.image_out_dir)
            .finish()
    }
}

/// Builder for `ClientConfig`. Use this when constructing the config
/// programmatically instead of via `config::load_config`.
#[derive(Debug, Default, Clone)]
pub struct ClientConfigBuilder {
    endpoint: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    max_output_tokens: Option<u32>,
    api_version: Option<String>,
    provider: Option<Provider>,
    reasoning_effort: Option<ReasoningEffort>,
    reasoning_summary: Option<ReasoningSummary>,
    instructions: Option<String>,
    tools: Vec<Tool>,
    tools_read_root: Option<std::path::PathBuf>,
    log_level: Option<LogLevel>,
    image_deployment: Option<String>,
    image_api_version: Option<String>,
    image_out_dir: Option<std::path::PathBuf>,
}

impl ClientConfigBuilder {
    /// Create a new empty builder with all fields unset.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the Responses API endpoint URL (required).
    pub fn endpoint(mut self, value: String) -> Self {
        self.endpoint = Some(value);
        self
    }

    /// Set the API key used for authentication (required).
    pub fn api_key(mut self, value: String) -> Self {
        self.api_key = Some(value);
        self
    }

    /// Set the model identifier (required).
    pub fn model(mut self, value: String) -> Self {
        self.model = Some(value);
        self
    }

    /// Set the maximum number of output tokens (defaults to 16384 if unset).
    pub fn max_output_tokens(mut self, value: u32) -> Self {
        self.max_output_tokens = Some(value);
        self
    }

    /// Set the API version query parameter (required).
    pub fn api_version(mut self, value: String) -> Self {
        self.api_version = Some(value);
        self
    }

    /// Set the API provider/auth mode.
    pub fn provider(mut self, value: Provider) -> Self {
        self.provider = Some(value);
        self
    }

    /// Set the requested reasoning effort level.
    pub fn reasoning_effort(mut self, value: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(value);
        self
    }

    /// Set the reasoning summary verbosity.
    pub fn reasoning_summary(mut self, value: ReasoningSummary) -> Self {
        self.reasoning_summary = Some(value);
        self
    }

    /// Set the system instructions string.
    pub fn instructions(mut self, value: String) -> Self {
        self.instructions = Some(value);
        self
    }

    /// Replace the entire tools list.
    pub fn tools(mut self, value: Vec<Tool>) -> Self {
        self.tools = value;
        self
    }

    /// Append a single tool to the tools list.
    pub fn add_tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Set the sandbox root directory for the `read_file` tool.
    pub fn tools_read_root(mut self, value: std::path::PathBuf) -> Self {
        self.tools_read_root = Some(value);
        self
    }

    /// Set runtime diagnostic verbosity.
    pub fn log_level(mut self, value: LogLevel) -> Self {
        self.log_level = Some(value);
        self
    }

    /// Set the Azure image deployment name (e.g. `gpt-image-2`).
    pub fn image_deployment(mut self, value: String) -> Self {
        self.image_deployment = Some(value);
        self
    }

    /// Set the api-version used for the image-generation endpoint.
    pub fn image_api_version(mut self, value: String) -> Self {
        self.image_api_version = Some(value);
        self
    }

    /// Set the output directory for image and SVG artifacts.
    pub fn image_out_dir(mut self, value: std::path::PathBuf) -> Self {
        self.image_out_dir = Some(value);
        self
    }

    /// Consume the builder and produce a `ClientConfig`, or fail if a required field is missing.
    pub fn build(self) -> Result<ClientConfig, ChatError> {
        let endpoint = self.endpoint.ok_or_else(|| {
            ChatError::Config(format!(
                "ClientConfigBuilder missing required field: {name}",
                name = "endpoint"
            ))
        })?;
        let api_key = self.api_key.ok_or_else(|| {
            ChatError::Config(format!(
                "ClientConfigBuilder missing required field: {name}",
                name = "api_key"
            ))
        })?;
        let model = self.model.ok_or_else(|| {
            ChatError::Config(format!(
                "ClientConfigBuilder missing required field: {name}",
                name = "model"
            ))
        })?;
        let provider = self.provider.unwrap_or(Provider::Azure);
        let api_version = match (provider, self.api_version) {
            (Provider::Azure, Some(value)) => value,
            (Provider::Azure, None) => {
                return Err(ChatError::Config(format!(
                    "ClientConfigBuilder missing required field: {name}",
                    name = "api_version"
                )));
            }
            (Provider::OpenAiCompatible, Some(value)) => value,
            (Provider::OpenAiCompatible, None) => String::new(),
        };

        Ok(ClientConfig {
            endpoint,
            api_key,
            model,
            max_output_tokens: self.max_output_tokens.unwrap_or(16384),
            api_version,
            provider,
            reasoning_effort: self.reasoning_effort,
            reasoning_summary: self.reasoning_summary,
            instructions: self.instructions,
            tools: self.tools,
            tools_read_root: self.tools_read_root,
            log_level: self.log_level.unwrap_or(LogLevel::Normal),
            image_deployment: self.image_deployment,
            image_api_version: self
                .image_api_version
                .unwrap_or_else(|| "2024-02-01".to_owned()),
            image_out_dir: self
                .image_out_dir
                .unwrap_or_else(|| std::path::PathBuf::from("images")),
        })
    }
}

// ---- errors ----

/// Error type for all chat client operations.
#[derive(Debug, Error)]
pub enum ChatError {
    #[error("HTTP error {status}: {body}")]
    Http { status: u16, body: String },

    #[error("transport error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("tool error: {0}")]
    Tool(String),
}
