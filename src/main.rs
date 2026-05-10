// file: rust/src/main.rs
// description: multi-turn chat CLI entry point using the Responses API
// reference: https://docs.rs/tokio/latest/tokio/

mod client;
mod config;
mod types;

use std::io::{self, BufRead, Write};

use reqwest::Client;

use crate::client::{extract_reply, send_message};
use crate::config::load_config;
use crate::types::{ChatError, InputMessage, Role};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let config = load_config().map_err(|e| anyhow::anyhow!("{e}"))?;
    let http = Client::new();

    let effort_label = config
        .reasoning_effort
        .map(|e| format!("{e:?}").to_ascii_lowercase())
        .unwrap_or_else(|| "default".into());
    println!(
        "{} chat | reasoning={} | type 'exit' to quit\n",
        config.model, effort_label
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

        let input = [InputMessage {
            role: Role::User,
            content: user_input,
        }];

        match send_message(&http, &config, &input, previous_response_id.as_deref()).await {
            Ok(response) => {
                if response.status == "incomplete" {
                    let reason = response
                        .incomplete_details
                        .as_ref()
                        .map(|d| d.reason.as_str())
                        .unwrap_or("unknown");
                    eprintln!("response incomplete: {reason}");
                }

                let reply = extract_reply(&response);
                if reply.is_empty() {
                    eprintln!("(no visible output — model may have spent all tokens on reasoning)");
                } else {
                    println!("\nassistant: {reply}\n");
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
