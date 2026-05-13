// file: rust/src/main.rs
// description: multi-turn chat CLI entry point using the Responses API
// reference: https://docs.rs/tokio/latest/tokio/

use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use reqwest::Client;

use gpt55_chat::client::{extract_reply, run_turn};
use gpt55_chat::config::load_config;
use gpt55_chat::image::{ImageRequest, generate as generate_image};
use gpt55_chat::svg::{SvgStyle, convert as convert_svg};
use gpt55_chat::types::{ChatError, ClientConfig, LogLevel, Tool};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let config = load_config().map_err(|e| anyhow::anyhow!("{e}"))?;
    let http = Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| anyhow::anyhow!("failed to build HTTP client: {e}"))?;

    let effort_label = config
        .reasoning_effort()
        .map(|e| e.as_str().to_owned())
        .unwrap_or_else(|| "default".into());

    let tool_names: Vec<&str> = config
        .tools()
        .iter()
        .map(|t| match t {
            Tool::Function(f) => f.name.as_str(),
        })
        .collect();
    let tools_label = if tool_names.is_empty() {
        "off".to_string()
    } else {
        tool_names.join(",")
    };
    let instructions_label = match config.instructions() {
        Some(s) => format!("{} chars", s.len()),
        None => "none".to_string(),
    };
    let media_label = match config.image_deployment() {
        Some(name) => format!(
            "{} -> {}",
            name,
            config.image_out_dir().display()
        ),
        None => "off".to_string(),
    };

    if config.log_level() != LogLevel::Quiet {
        println!(
            "{} chat | provider={} | reasoning={} | instructions={} | tools={} | media={} | /help for commands | type 'exit' to quit\n",
            config.model(),
            config.provider().as_str(),
            effort_label,
            instructions_label,
            tools_label,
            media_label
        );
    }

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout().lock();
    let mut line = String::with_capacity(256);
    let mut previous_response_id: Option<String> = None;

    loop {
        stdout.write_all(b"you: ")?;
        stdout.flush()?;

        line.clear();
        let bytes_read = stdin.read_line(&mut line)?;

        if bytes_read == 0 {
            println!("goodbye.");
            break;
        }

        let user_input = line.trim().to_owned();

        if user_input.is_empty() {
            continue;
        }

        if user_input.eq_ignore_ascii_case("exit") {
            println!("goodbye.");
            break;
        }

        if user_input.starts_with('/') {
            match handle_slash_command(&http, &config, &user_input, &mut stdin).await {
                Ok(SlashOutcome::Handled) => {}
                Ok(SlashOutcome::Help) => print_help(),
                Err(e) => eprintln!("error: {e}"),
            }
            continue;
        }

        match run_turn(&http, &config, &user_input, previous_response_id.as_deref()).await {
            Ok(response) => {
                if response.status == "incomplete" {
                    let reason = response
                        .incomplete_details
                        .as_ref()
                        .map(|d| d.reason.as_str())
                        .unwrap_or("unknown");
                    eprintln!("response incomplete: {reason}");
                }

                match extract_reply(&response) {
                    Some(reply) => println!("\nassistant: {reply}\n"),
                    None => {
                        if response.status != "incomplete" {
                            let all_reasoning = response.usage.as_ref().is_some_and(|u| {
                                let r = u
                                    .output_tokens_details
                                    .as_ref()
                                    .map(|d| d.reasoning_tokens)
                                    .unwrap_or(0);
                                r > 0 && u.output_tokens == r
                            });
                            if all_reasoning {
                                eprintln!(
                                    "(no visible output — model spent all output tokens on reasoning)"
                                );
                            } else {
                                eprintln!("(no visible output)");
                            }
                        }
                    }
                }

                if config.log_level() != LogLevel::Quiet
                    && let Some(usage) = &response.usage
                {
                    let r = usage
                        .output_tokens_details
                        .as_ref()
                        .map(|d| d.reasoning_tokens)
                        .unwrap_or(0);
                    eprintln!(
                        "[tokens: in={} out={} reasoning={} total={}]",
                        usage.input_tokens, usage.output_tokens, r, usage.total_tokens
                    );
                }

                previous_response_id = Some(response.id);
            }
            Err(ChatError::Http { status, body }) => {
                eprintln!("api error {status}: {body}");
                previous_response_id = None;
            }
            Err(e) => {
                eprintln!("error: {e}");
            }
        }
    }

    Ok(())
}

enum SlashOutcome {
    Handled,
    Help,
}

async fn handle_slash_command(
    http: &Client,
    config: &ClientConfig,
    raw: &str,
    stdin: &mut impl BufRead,
) -> anyhow::Result<SlashOutcome> {
    let (cmd, rest) = match raw.split_once(char::is_whitespace) {
        Some((c, r)) => (c, r.trim()),
        None => (raw, ""),
    };

    match cmd {
        "/help" | "/?" => Ok(SlashOutcome::Help),
        "/image" => {
            handle_image(http, config, rest, stdin).await?;
            Ok(SlashOutcome::Handled)
        }
        "/svg" => {
            handle_svg(http, config, rest).await?;
            Ok(SlashOutcome::Handled)
        }
        other => Err(anyhow::anyhow!(
            "unknown command '{other}' — type /help to list commands"
        )),
    }
}

fn print_help() {
    println!(
        "\ncommands:\n  /image [--size=WxH] [--quality=low|medium|high] [--n=1..10] [--format=png|jpeg|webp] <prompt>\n           generate one or more images via the configured AZURE_IMAGE_DEPLOYMENT and\n           save them under {}. Prompt forms: inline, '@path/to/prompt.txt', or omit\n           the prompt to enter multi-line mode (terminate with '.' on its own line).\n  /svg [--style=editable|fidelity|compact|combined] <path-to-png>\n           convert a local PNG to SVG via the chat model's vision input.\n  /help    show this help.\n  exit     quit the REPL.\n",
        std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    );
}

async fn handle_image(
    http: &Client,
    config: &ClientConfig,
    rest: &str,
    stdin: &mut impl BufRead,
) -> anyhow::Result<()> {
    let mut tokens = rest.split_whitespace().peekable();
    let mut size: Option<String> = None;
    let mut quality: Option<String> = None;
    let mut n: Option<u32> = None;
    let mut format: Option<String> = None;

    while let Some(tok) = tokens.peek().copied() {
        if let Some(value) = tok.strip_prefix("--size=") {
            size = Some(value.to_owned());
        } else if let Some(value) = tok.strip_prefix("--quality=") {
            quality = Some(value.to_owned());
        } else if let Some(value) = tok.strip_prefix("--n=") {
            n = Some(
                value
                    .parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("--n must be a positive integer"))?,
            );
        } else if let Some(value) = tok.strip_prefix("--format=") {
            format = Some(value.to_owned());
        } else if tok.starts_with("--") {
            return Err(anyhow::anyhow!("unknown flag '{tok}'"));
        } else {
            break;
        }
        tokens.next();
    }

    let remainder: String = tokens.collect::<Vec<_>>().join(" ");
    let prompt = if remainder.is_empty() {
        read_multiline_prompt(stdin)?
    } else if let Some(path) = remainder.strip_prefix('@') {
        std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read prompt file '{path}': {e}"))?
            .trim()
            .to_owned()
    } else {
        remainder
    };

    if prompt.is_empty() {
        return Err(anyhow::anyhow!("image prompt was empty"));
    }

    let mut request = ImageRequest::new(prompt);
    if let Some(s) = size {
        request = request.size(s);
    }
    if let Some(q) = quality {
        request = request.quality(q);
    }
    if let Some(count) = n {
        request = request.n(count);
    }
    if let Some(f) = format {
        request = request.output_format(f);
    }

    eprintln!(
        "[generating {} image(s) at {} quality={} format={}…]",
        request.n, request.size, request.quality, request.output_format
    );
    let written = generate_image(http, config, &request).await?;
    for path in &written {
        println!("wrote {}", path.display());
    }
    Ok(())
}

async fn handle_svg(
    http: &Client,
    config: &ClientConfig,
    rest: &str,
) -> anyhow::Result<()> {
    let mut tokens = rest.split_whitespace().peekable();
    let mut style = SvgStyle::Combined;
    while let Some(tok) = tokens.peek().copied() {
        if let Some(value) = tok.strip_prefix("--style=") {
            style = SvgStyle::parse(value).ok_or_else(|| {
                anyhow::anyhow!(
                    "--style must be one of editable|fidelity|compact|combined (got '{value}')"
                )
            })?;
        } else if tok.starts_with("--") {
            return Err(anyhow::anyhow!("unknown flag '{tok}'"));
        } else {
            break;
        }
        tokens.next();
    }
    let path_str = tokens.collect::<Vec<_>>().join(" ");
    if path_str.is_empty() {
        return Err(anyhow::anyhow!("usage: /svg [--style=...] <png-path>"));
    }
    let png_path = PathBuf::from(path_str);
    eprintln!("[converting {} → SVG using style={:?}…]", png_path.display(), style);
    let written = convert_svg(http, config, &png_path, style).await?;
    println!("wrote {}", written.display());
    Ok(())
}

/// Read a multi-line prompt from `stdin`, terminated by a line containing only
/// `.` or by EOF.
fn read_multiline_prompt(stdin: &mut impl BufRead) -> io::Result<String> {
    eprintln!("(enter prompt; end with a single '.' on its own line)");
    let mut buf = String::new();
    let mut line = String::new();
    loop {
        line.clear();
        let read = stdin.read_line(&mut line)?;
        if read == 0 {
            break;
        }
        if line.trim() == "." {
            break;
        }
        buf.push_str(&line);
    }
    Ok(buf.trim().to_owned())
}
