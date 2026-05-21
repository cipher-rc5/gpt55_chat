// file: src/cli.rs
// description: pure parsers for the interactive REPL's slash commands

use std::path::PathBuf;

use crate::svg::SvgStyle;

/// Classified slash command. `rest` is the trimmed text after the command word.
#[derive(Debug, PartialEq, Eq)]
pub enum SlashCommand<'a> {
    /// `/help` or `/?`.
    Help,
    /// `/image …`.
    Image { rest: &'a str },
    /// `/svg …`.
    Svg { rest: &'a str },
    /// Unrecognised `/<name>`.
    Unknown { name: &'a str },
}

/// Split a raw `/foo bar baz` line into a `SlashCommand` and the trimmed tail.
pub fn classify_slash(raw: &str) -> SlashCommand<'_> {
    let (cmd, rest) = match raw.split_once(char::is_whitespace) {
        Some((c, r)) => (c, r.trim()),
        None => (raw, ""),
    };
    match cmd {
        "/help" | "/?" => SlashCommand::Help,
        "/image" => SlashCommand::Image { rest },
        "/svg" => SlashCommand::Svg { rest },
        other => SlashCommand::Unknown { name: other },
    }
}

/// Parsed `/image` flags + remainder text (the part after the flags).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImageArgs {
    pub size: Option<String>,
    pub quality: Option<String>,
    pub n: Option<u32>,
    pub format: Option<String>,
    pub remainder: String,
}

/// Parse `/image` flags. Returns an error string on unknown flag or bad `--n=`.
pub fn parse_image_args(rest: &str) -> Result<ImageArgs, String> {
    let mut args = ImageArgs::default();
    let mut tokens = rest.split_whitespace().peekable();
    while let Some(tok) = tokens.peek().copied() {
        if let Some(value) = tok.strip_prefix("--size=") {
            args.size = Some(value.to_owned());
        } else if let Some(value) = tok.strip_prefix("--quality=") {
            args.quality = Some(value.to_owned());
        } else if let Some(value) = tok.strip_prefix("--n=") {
            args.n = Some(
                value
                    .parse::<u32>()
                    .map_err(|_| "--n must be a positive integer".to_owned())?,
            );
        } else if let Some(value) = tok.strip_prefix("--format=") {
            args.format = Some(value.to_owned());
        } else if tok.starts_with("--") {
            return Err(format!("unknown flag '{tok}'"));
        } else {
            break;
        }
        tokens.next();
    }
    args.remainder = tokens.collect::<Vec<_>>().join(" ");
    Ok(args)
}

/// Where an `/image` prompt body should come from.
#[derive(Debug, PartialEq, Eq)]
pub enum PromptSource {
    /// No remainder text — the caller should read a multi-line prompt from stdin.
    MultiLine,
    /// `@/path/to/file` — read the prompt body from the file.
    File(PathBuf),
    /// Inline prompt text.
    Inline(String),
}

/// Classify a `/image` remainder into a `PromptSource`.
pub fn classify_prompt(remainder: &str) -> PromptSource {
    if remainder.is_empty() {
        PromptSource::MultiLine
    } else if let Some(path) = remainder.strip_prefix('@') {
        PromptSource::File(PathBuf::from(path))
    } else {
        PromptSource::Inline(remainder.to_owned())
    }
}

/// Parsed `/svg` arguments.
#[derive(Debug, PartialEq, Eq)]
pub struct SvgArgs {
    pub style: SvgStyle,
    pub path: PathBuf,
}

/// Parse `/svg [--style=...] <png-path>`. Returns an error string on bad flag.
pub fn parse_svg_args(rest: &str) -> Result<SvgArgs, String> {
    let mut tokens = rest.split_whitespace().peekable();
    let mut style = SvgStyle::Combined;
    while let Some(tok) = tokens.peek().copied() {
        if let Some(value) = tok.strip_prefix("--style=") {
            style = SvgStyle::parse(value).ok_or_else(|| {
                format!("--style must be one of editable|fidelity|compact|combined (got '{value}')")
            })?;
        } else if tok.starts_with("--") {
            return Err(format!("unknown flag '{tok}'"));
        } else {
            break;
        }
        tokens.next();
    }
    let path_str = tokens.collect::<Vec<_>>().join(" ");
    if path_str.is_empty() {
        return Err("usage: /svg [--style=...] <png-path>".to_owned());
    }
    Ok(SvgArgs {
        style,
        path: PathBuf::from(path_str),
    })
}

/// Truncate `body` to at most `max_chars` characters, appending an ellipsis
/// when the original was longer. Used to bound how much of an upstream error
/// body is printed in non-verbose log levels.
pub fn truncate_for_display(body: &str, max_chars: usize) -> String {
    let trimmed = body.trim();
    let mut iter = trimmed.char_indices();
    if let Some((cut, _)) = iter.nth(max_chars) {
        let mut out = trimmed[..cut].to_owned();
        out.push('…');
        out
    } else {
        trimmed.to_owned()
    }
}
