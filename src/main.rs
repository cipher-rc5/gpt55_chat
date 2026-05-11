// file: rust/src/main.rs
// description: multi-turn chat CLI entry point using the Responses API
// reference: https://docs.rs/tokio/latest/tokio/

mod client;
mod config;
mod tools;
mod types;

use std::io::{self, BufRead, Write};

use reqwest::Client;

use crate::client::{extract_reply, run_turn};
use crate::config::load_config;
use crate::types::{ChatError, Tool};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let config = load_config().map_err(|e| anyhow::anyhow!("{e}"))?;
    let http = Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| anyhow::anyhow!("failed to build HTTP client: {e}"))?;

    let effort_label = config
        .reasoning_effort
        .map(|e| e.as_str().to_owned())
        .unwrap_or_else(|| "default".into());

    let tool_names: Vec<&str> = config
        .tools
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
    let instructions_label = match &config.instructions {
        Some(s) => format!("{} chars", s.len()),
        None => "none".to_string(),
    };

    println!(
        "{} chat | reasoning={} | instructions={} | tools={} | type 'exit' to quit\n",
        config.model, effort_label, instructions_label, tools_label
    );

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

                if let Some(usage) = &response.usage {
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
            }
            Err(e) => {
                eprintln!("error: {e}");
            }
        }
    }

    Ok(())
}
