// file: src/main.rs
// description: multi-turn chat CLI entry point using the Responses API
// reference: https://docs.rs/tokio/latest/tokio/

use std::io::{self, BufRead, Write};

use reqwest::Client;

use gpt55_chat::cli::{
    ImageArgs, PromptSource, SlashCommand, SvgArgs, classify_prompt, classify_slash,
    parse_image_args, parse_svg_args, truncate_for_display,
};
use gpt55_chat::client::{extract_reply, run_turn};
use gpt55_chat::config::load_config;
use gpt55_chat::image::{ImageRequest, generate as generate_image};
use gpt55_chat::svg::convert as convert_svg;
use gpt55_chat::types::{ChatError, ClientConfig, LogLevel, Tool};

const ERROR_BODY_DISPLAY_CHARS: usize = 256;

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
        Some(name) => format!("{} -> {}", name, config.image_out_dir().display()),
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
            if let Err(e) = dispatch_slash(&http, &config, &user_input, &mut stdin).await {
                eprintln!("error: {e}");
            }
            continue;
        }

        // Cancel the in-flight turn on ctrl-c so the user can recover the prompt
        // without waiting for the HTTP timeout. SIGINT outside this scope falls
        // through to the default handler (process exit).
        let turn = run_turn(&http, &config, &user_input, previous_response_id.as_deref());
        tokio::select! {
            outcome = turn => match outcome {
                Ok(response) => {
                    handle_response(&config, response, &mut previous_response_id);
                }
                Err(ChatError::Http { status, body }) => {
                    let display = if config.log_level() == LogLevel::Verbose {
                        body
                    } else {
                        truncate_for_display(&body, ERROR_BODY_DISPLAY_CHARS)
                    };
                    eprintln!("api error {status}: {display}");
                    previous_response_id = None;
                }
                Err(e) => {
                    eprintln!("error: {e}");
                }
            },
            _ = tokio::signal::ctrl_c() => {
                eprintln!("\n(interrupted; returning to prompt)");
            }
        }
    }

    Ok(())
}

fn handle_response(
    config: &ClientConfig,
    response: gpt55_chat::types::ResponsesResponse,
    previous_response_id: &mut Option<String>,
) {
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
                    eprintln!("(no visible output — model spent all output tokens on reasoning)");
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

    *previous_response_id = Some(response.id);
}

async fn dispatch_slash(
    http: &Client,
    config: &ClientConfig,
    raw: &str,
    stdin: &mut impl BufRead,
) -> anyhow::Result<()> {
    match classify_slash(raw) {
        SlashCommand::Help => {
            print_help();
            Ok(())
        }
        SlashCommand::Image { rest } => handle_image(http, config, rest, stdin).await,
        SlashCommand::Svg { rest } => handle_svg(http, config, rest).await,
        SlashCommand::Unknown { name } => Err(anyhow::anyhow!(
            "unknown command '{name}' — type /help to list commands"
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
    let ImageArgs {
        size,
        quality,
        n,
        format,
        remainder,
    } = parse_image_args(rest).map_err(|e| anyhow::anyhow!(e))?;

    let prompt = match classify_prompt(&remainder) {
        PromptSource::MultiLine => read_multiline_prompt(stdin)?,
        PromptSource::File(path) => std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("failed to read prompt file '{}': {e}", path.display()))?
            .trim()
            .to_owned(),
        PromptSource::Inline(text) => text,
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

async fn handle_svg(http: &Client, config: &ClientConfig, rest: &str) -> anyhow::Result<()> {
    let SvgArgs { style, path } = parse_svg_args(rest).map_err(|e| anyhow::anyhow!(e))?;
    eprintln!(
        "[converting {} → SVG using style={:?}…]",
        path.display(),
        style
    );
    let written = convert_svg(http, config, &path, style).await?;
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
