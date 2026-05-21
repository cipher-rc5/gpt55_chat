// file: src/svg.rs
// description: convert a local PNG to SVG markup via the Responses API vision input
// reference: https://developers.openai.com/api/docs/api-reference/responses

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use reqwest::Client;

use crate::client::{extract_reply, send_message};
use crate::types::{ChatError, ClientConfig, InputContentPart, InputItem, Role};

const MAX_INPUT_PNG_BYTES: u64 = 8 * 1024 * 1024;

/// Style/quality direction for the SVG production prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgStyle {
    /// Real editable paths, gradients, masks. Prioritises editability.
    Editable,
    /// Highest visual fidelity to the source PNG; paths may be more detailed.
    Fidelity,
    /// Smallest, optimised production SVG with simplified paths.
    Compact,
    /// Combined production-ready prompt covering fidelity, editability, and size.
    Combined,
}

impl SvgStyle {
    /// Parse a case-insensitive style label.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "editable" | "edit" | "a" => Some(Self::Editable),
            "fidelity" | "visual" | "b" => Some(Self::Fidelity),
            "compact" | "small" | "c" => Some(Self::Compact),
            "combined" | "default" => Some(Self::Combined),
            _ => None,
        }
    }

    /// System-style instructions sent with the conversion request.
    pub fn prompt(&self) -> &'static str {
        match self {
            Self::Editable => EDITABLE_PROMPT,
            Self::Fidelity => FIDELITY_PROMPT,
            Self::Compact => COMPACT_PROMPT,
            Self::Combined => COMBINED_PROMPT,
        }
    }
}

/// Convert a local PNG to SVG using the configured Responses API model.
/// Reads `png_path`, base64-encodes it as a data URL, sends a vision message
/// with the style-specific prompt, extracts SVG markup from the response, and
/// writes it to `<image_out_dir>/<stem>.svg`. Returns the written path.
pub async fn convert(
    http: &Client,
    config: &ClientConfig,
    png_path: &Path,
    style: SvgStyle,
) -> Result<PathBuf, ChatError> {
    let data_url = read_png_as_data_url(png_path)?;

    let input = [InputItem::MessageParts {
        role: Role::User,
        content: vec![
            InputContentPart::InputText {
                text: style.prompt().to_owned(),
            },
            InputContentPart::InputImage {
                image_url: data_url,
            },
        ],
    }];

    let response = send_message(http, config, &input, None).await?;
    let reply = extract_reply(&response).ok_or_else(|| {
        ChatError::Tool("SVG conversion: model returned no text output".to_owned())
    })?;

    let svg = extract_svg(&reply).ok_or_else(|| {
        ChatError::Tool(
            "SVG conversion: response did not contain a `<svg>...</svg>` block".to_owned(),
        )
    })?;

    let out_path = build_output_path(config.image_out_dir(), png_path)?;
    std::fs::create_dir_all(config.image_out_dir()).map_err(|e| {
        ChatError::Config(format!(
            "failed to create image_out_dir {}: {e}",
            config.image_out_dir().display()
        ))
    })?;
    std::fs::write(&out_path, svg.as_bytes())
        .map_err(|e| ChatError::Config(format!("failed to write {}: {e}", out_path.display())))?;

    Ok(out_path)
}

fn read_png_as_data_url(png_path: &Path) -> Result<String, ChatError> {
    use std::io::Read;

    let canonical = std::fs::canonicalize(png_path)
        .map_err(|e| ChatError::Config(format!("PNG path canonicalize failed: {e}")))?;
    let file = std::fs::File::open(&canonical)
        .map_err(|e| ChatError::Config(format!("PNG open failed: {e}")))?;
    let meta = file
        .metadata()
        .map_err(|e| ChatError::Config(format!("PNG stat failed: {e}")))?;
    if !meta.is_file() {
        return Err(ChatError::Config(format!(
            "not a regular file: {}",
            png_path.display()
        )));
    }
    if meta.len() > MAX_INPUT_PNG_BYTES {
        return Err(ChatError::Config(format!(
            "PNG too large: {} bytes (limit {MAX_INPUT_PNG_BYTES})",
            meta.len()
        )));
    }
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    file.take(MAX_INPUT_PNG_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|e| ChatError::Config(format!("PNG read failed: {e}")))?;
    let mime = match canonical
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "image/png",
    };
    let encoded = BASE64_STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{encoded}"))
}

fn build_output_path(out_dir: &Path, png_path: &Path) -> Result<PathBuf, ChatError> {
    let stem = png_path
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| ChatError::Config("input path has no file stem".to_owned()))?;
    Ok(out_dir.join(format!("{stem}.svg")))
}

/// Find the SVG block in `reply`. Prefers a fenced ```svg``` block when
/// present, otherwise falls back to the first balanced `<svg>...</svg>` span.
fn extract_svg(reply: &str) -> Option<String> {
    if let Some(start) = reply.find("```svg") {
        let after = &reply[start + "```svg".len()..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim().to_owned());
        }
    }
    if let Some(start) = reply.find("```xml") {
        let after = &reply[start + "```xml".len()..];
        if let Some(end) = after.find("```") {
            let body = after[..end].trim();
            if body.contains("<svg") {
                return Some(body.to_owned());
            }
        }
    }
    if let Some(start) = reply.find("<svg")
        && let Some(end_rel) = reply[start..].find("</svg>")
    {
        let end = start + end_rel + "</svg>".len();
        return Some(reply[start..end].trim().to_owned());
    }
    None
}

const EDITABLE_PROMPT: &str = "You are a senior vector logo production designer.\n\nUsing the provided image as the source of truth, recreate the icon/logo as a clean, optimized, EDITABLE SVG.\n\nRequirements:\n- Remove all text, taglines, and background elements; preserve only the central logomark.\n- Match original proportions, geometry, spacing, negative space, segment breaks, curves, and silhouette.\n- Use real SVG vector paths, masks, clip paths, and gradients.\n- Do NOT embed the source image as base64. Do NOT use raster fills.\n- Use a tight viewBox around the logomark and a transparent background.\n- Minimise groups, metadata, editor comments, and redundant path data; 2–3 decimals.\n- The output must render correctly in browsers, Figma, Illustrator, and Inkscape.\n\nReturn only valid SVG markup, inside a single ```svg fenced code block. No explanation before or after.";

const FIDELITY_PROMPT: &str = "You are a senior vector logo production designer.\n\nRecreate the central logomark from the provided image as an SVG that VISUALLY MATCHES the source as closely as possible.\n\nRequirements:\n- Prioritise visual fidelity over path simplicity.\n- May use detailed paths, gradients, masks, and clip paths.\n- Do NOT use base64 raster embedding unless absolutely necessary.\n- Remove all text and background; keep only the logomark.\n- Preserve original proportions, negative space, segment breaks, and brand colors.\n- Use a tight viewBox and transparent background.\n\nReturn only valid SVG markup, inside a single ```svg fenced code block.";

const COMPACT_PROMPT: &str = "You are a senior SVG production specialist.\n\nFrom the provided image, output a COMPACT production SVG of the central logomark.\n\nRequirements:\n- Simplify paths while preserving visual identity.\n- Optimise for small file size.\n- Remove metadata, comments, hidden layers, unused defs, and redundant groups.\n- Use transparent background and a tight viewBox.\n- Real vector geometry only; no base64 raster.\n- Remove all text and background; keep only the logomark.\n\nReturn only valid SVG markup, inside a single ```svg fenced code block.";

const COMBINED_PROMPT: &str = "Using the provided image as visual reference, recreate ONLY the central logomark as a clean, production-ready SVG.\n\nRemove all text, taglines, background patterns, shadows outside the icon, and canvas artifacts.\n\nThe SVG must be real editable vector geometry, not an embedded raster image. Use paths, gradients, masks, clip paths, and geometric primitives to preserve the original mark's silhouette, segmentation, negative space, gradients, and proportions.\n\nOptimise for:\n- visual fidelity to the source\n- clean editability in Figma, Illustrator, and Inkscape\n- small file size\n- transparent background\n- tight viewBox\n- scalable rendering at small and large sizes\n\nAvoid:\n- base64 raster images\n- thousands of tiny auto-traced fragments\n- generic replacement shapes\n- text, background, metadata, or unused defs\n\nReturn only valid SVG markup, inside a single ```svg fenced code block. No explanation before or after.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_svg_prefers_fenced_block() {
        let reply = "here you go:\n```svg\n<svg width=\"10\"></svg>\n```\nhope this helps";
        assert_eq!(
            extract_svg(reply),
            Some("<svg width=\"10\"></svg>".to_owned())
        );
    }

    #[test]
    fn extract_svg_falls_back_to_raw_tags() {
        let reply = "raw output: <svg><path d=\"M0 0\"/></svg> trailing";
        assert_eq!(
            extract_svg(reply),
            Some("<svg><path d=\"M0 0\"/></svg>".to_owned())
        );
    }

    #[test]
    fn extract_svg_returns_none_for_plain_text() {
        assert_eq!(extract_svg("sorry, I can't help with that."), None);
    }

    #[test]
    fn svg_style_parses_aliases() {
        assert_eq!(SvgStyle::parse("editable"), Some(SvgStyle::Editable));
        assert_eq!(SvgStyle::parse("A"), Some(SvgStyle::Editable));
        assert_eq!(SvgStyle::parse("combined"), Some(SvgStyle::Combined));
        assert_eq!(SvgStyle::parse("nope"), None);
    }
}
