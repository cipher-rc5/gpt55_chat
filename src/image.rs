// file: src/image.rs
// description: async client for the Azure OpenAI image-generation endpoint (gpt-image-2 family)
// reference: https://learn.microsoft.com/azure/ai-services/openai/reference#image-generations

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

use crate::tools::format_utc;
use crate::types::{ChatError, ClientConfig, LogLevel, Provider};

const MAX_HTTP_ATTEMPTS: usize = 3;
const RETRY_BASE_DELAY_MS: u64 = 100;
const MAX_N: u32 = 10;

/// Parameters for a single image-generation request.
#[derive(Debug, Clone)]
pub struct ImageRequest {
    /// Free-form prompt describing the desired image. Sent verbatim to the API.
    pub prompt: String,
    /// Image dimensions as `WxH` (e.g. `1024x1024`).
    pub size: String,
    /// Render quality. `gpt-image-2` accepts `low`, `medium`, `high`.
    pub quality: String,
    /// Number of images to request (`1..=10`).
    pub n: u32,
    /// File container format: `png`, `jpeg`, or `webp`.
    pub output_format: String,
    /// Compression level (0..=100) for lossy formats; `None` to omit.
    pub output_compression: Option<u32>,
}

impl ImageRequest {
    /// Build a request with prompt-only defaults: 1024x1024, high quality,
    /// PNG output, single image.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            size: "1024x1024".to_owned(),
            quality: "high".to_owned(),
            n: 1,
            output_format: "png".to_owned(),
            output_compression: Some(100),
        }
    }

    /// Override the output size (`WxH`).
    pub fn size(mut self, value: impl Into<String>) -> Self {
        self.size = value.into();
        self
    }

    /// Override the render quality (`low|medium|high`).
    pub fn quality(mut self, value: impl Into<String>) -> Self {
        self.quality = value.into();
        self
    }

    /// Override the number of images requested.
    pub fn n(mut self, value: u32) -> Self {
        self.n = value;
        self
    }

    /// Override the output file container.
    pub fn output_format(mut self, value: impl Into<String>) -> Self {
        self.output_format = value.into();
        self
    }
}

#[derive(Serialize)]
struct ImageRequestBody<'a> {
    prompt: &'a str,
    size: &'a str,
    quality: &'a str,
    n: u32,
    output_format: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_compression: Option<u32>,
}

#[derive(Deserialize)]
struct ImageResponse {
    #[serde(default)]
    data: Vec<ImageData>,
}

#[derive(Deserialize)]
struct ImageData {
    #[serde(default)]
    b64_json: Option<String>,
}

/// Send a single image-generation request, decode the base64 payload(s),
/// and write the resulting files into `config.image_out_dir()`. Returns the
/// list of absolute paths written.
pub async fn generate(
    http: &Client,
    config: &ClientConfig,
    request: &ImageRequest,
) -> Result<Vec<PathBuf>, ChatError> {
    if request.prompt.trim().is_empty() {
        return Err(ChatError::Config(
            "image prompt must not be empty".to_owned(),
        ));
    }
    if request.n == 0 || request.n > MAX_N {
        return Err(ChatError::Config(format!(
            "image `n` must be between 1 and {MAX_N}"
        )));
    }
    if config.provider() != Provider::Azure {
        return Err(ChatError::Config(
            "image generation currently supports the azure provider only".to_owned(),
        ));
    }
    let deployment = config.image_deployment().ok_or_else(|| {
        ChatError::Config(
            "image generation disabled: set AZURE_IMAGE_DEPLOYMENT to a deployment name"
                .to_owned(),
        )
    })?;

    let url = build_image_url(config, deployment)?;

    let body = ImageRequestBody {
        prompt: &request.prompt,
        size: &request.size,
        quality: &request.quality,
        n: request.n,
        output_format: &request.output_format,
        output_compression: request.output_compression,
    };

    for attempt in 1..=MAX_HTTP_ATTEMPTS {
        let builder = http
            .post(url.clone())
            .header("Content-Type", "application/json")
            .header("api-key", config.api_key())
            .json(&body);

        let response = match builder.send().await {
            Ok(r) => r,
            Err(e) if attempt < MAX_HTTP_ATTEMPTS => {
                sleep(retry_delay(None, attempt)).await;
                if config.log_level() == LogLevel::Verbose {
                    eprintln!("[image-retry: transport error on attempt {attempt}: {e}]");
                }
                continue;
            }
            Err(e) => return Err(ChatError::Transport(e)),
        };

        let status = response.status().as_u16();
        if response.status().is_success() {
            let parsed: ImageResponse = response.json().await?;
            return write_images(&parsed, &request.output_format, config.image_out_dir());
        }

        if is_retryable_status(status) && attempt < MAX_HTTP_ATTEMPTS {
            let delay = retry_delay(response.headers().get("retry-after"), attempt);
            if config.log_level() == LogLevel::Verbose {
                eprintln!("[image-retry: HTTP {status} on attempt {attempt}]");
            }
            sleep(delay).await;
            continue;
        }

        let text = match response.text().await {
            Ok(t) => t,
            Err(e) => format!("<failed to read response body: {e}>"),
        };
        return Err(ChatError::Http { status, body: text });
    }

    unreachable!("HTTP retry loop should return from every attempt")
}

/// Construct the image-generation URL from a config whose `endpoint` was
/// canonicalised by [`crate::config`] to end with `/openai/responses`. The
/// resource root is recovered by stripping that suffix, then the
/// deployment-specific image path and api-version query are appended.
fn build_image_url(config: &ClientConfig, deployment: &str) -> Result<Url, ChatError> {
    let endpoint = config.endpoint().trim_end_matches('/');
    let resource = endpoint
        .strip_suffix("/openai/responses")
        .ok_or_else(|| ChatError::Config(format!("unexpected endpoint shape: {endpoint}")))?;
    if config.image_api_version().trim().is_empty() {
        return Err(ChatError::Config(
            "AZURE_IMAGE_API_VERSION must not be empty".to_owned(),
        ));
    }
    let raw = format!("{resource}/openai/deployments/{deployment}/images/generations");
    let mut url =
        Url::parse(&raw).map_err(|e| ChatError::Config(format!("image URL is invalid: {e}")))?;
    url.query_pairs_mut()
        .append_pair("api-version", config.image_api_version());
    Ok(url)
}

fn write_images(
    response: &ImageResponse,
    format: &str,
    out_dir: &Path,
) -> Result<Vec<PathBuf>, ChatError> {
    if response.data.is_empty() {
        return Err(ChatError::Tool(
            "image API returned an empty `data` array".to_owned(),
        ));
    }

    std::fs::create_dir_all(out_dir).map_err(|e| {
        ChatError::Config(format!(
            "failed to create image_out_dir {}: {e}",
            out_dir.display()
        ))
    })?;
    let canonical_root = std::fs::canonicalize(out_dir)
        .map_err(|e| ChatError::Config(format!("image_out_dir canonicalize failed: {e}")))?;

    let extension = match format {
        "jpeg" | "jpg" => "jpg",
        "webp" => "webp",
        _ => "png",
    };

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| ChatError::Config(e.to_string()))?
        .as_secs();
    let stamp = format_utc(secs).replace(':', "-");

    let mut written = Vec::with_capacity(response.data.len());
    for (idx, item) in response.data.iter().enumerate() {
        let encoded = item
            .b64_json
            .as_deref()
            .ok_or_else(|| ChatError::Tool("image data item missing `b64_json`".to_owned()))?;
        let bytes = BASE64_STANDARD
            .decode(encoded)
            .map_err(|e| ChatError::Tool(format!("image base64 decode failed: {e}")))?;

        let filename = if response.data.len() == 1 {
            format!("{stamp}.{extension}")
        } else {
            format!("{stamp}-{}.{extension}", idx + 1)
        };
        let candidate = canonical_root.join(&filename);
        if !candidate.starts_with(&canonical_root) {
            return Err(ChatError::Config(format!(
                "image output path is outside {} sandbox",
                canonical_root.display()
            )));
        }
        std::fs::write(&candidate, &bytes).map_err(|e| {
            ChatError::Config(format!("failed to write {}: {e}", candidate.display()))
        })?;
        written.push(candidate);
    }

    Ok(written)
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
