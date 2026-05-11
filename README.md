# gpt55-chat

A small, single-binary multi-turn chat CLI for the OpenAI/Azure Responses API.
It chains turns via `previous_response_id`, supports configurable reasoning
effort and summaries, an optional system prompt plus rules file, and a small
set of built-in function tools.

## Quick start

```sh
cargo build --release
./target/release/gpt55-chat
```

Type messages at the `you:` prompt. Enter `exit` (or send EOF) to quit.

## Required environment variables

| Variable         | Description                                                   |
|------------------|---------------------------------------------------------------|
| `AZURE_API_KEY`  | API key sent as both `Authorization: Bearer` and `api-key`.   |
| `AZURE_RESOURCE` | Base resource URL, e.g. `https://my-resource.openai.azure.com`. |
| `API_VERSION`    | Azure API version, e.g. `2025-04-01-preview`.                 |
| `MODEL`          | Deployment / model name to invoke.                            |

## Optional environment variables

| Variable                  | Default | Description                                                                                              |
|---------------------------|---------|----------------------------------------------------------------------------------------------------------|
| `REASONING_EFFORT`        | unset   | One of `none`, `minimal`, `low`, `medium`, `high`, `xhigh`.                                              |
| `REASONING_SUMMARY`       | unset   | One of `auto`, `concise`, `detailed`.                                                                    |
| `MAX_OUTPUT_TOKENS`       | `16384` | Hard cap on output tokens per turn.                                                                      |
| `OPENAI_SYSTEM_PROMPT`    | unset   | Free-form `instructions` text prepended to every turn.                                                   |
| `OPENAI_RULES_FILE`       | unset   | Path to a rules file. One rule per line; blank lines and `#` comments are ignored.                       |
| `OPENAI_TOOLS`            | on      | Set to `off` (or `false`/`0`) to disable all built-in tools.                                             |
| `OPENAI_TOOLS_READ_ROOT`  | unset   | Directory the `read_file` tool may read under. If unset, `read_file` is disabled.                        |

## Built-in tools

- `get_time` — returns the current UTC time as ISO 8601 plus Unix seconds.
- `read_file` — reads a UTF-8 text file (max 64 KiB). Gated behind
  `OPENAI_TOOLS_READ_ROOT`: if that variable is unset, the tool is not
  registered and the model cannot call it.

## Example `.env`

```dotenv
AZURE_API_KEY=sk-...
AZURE_RESOURCE=https://my-resource.openai.azure.com
API_VERSION=2025-04-01-preview
MODEL=gpt-5.5

REASONING_EFFORT=medium
REASONING_SUMMARY=auto
MAX_OUTPUT_TOKENS=16384

OPENAI_SYSTEM_PROMPT=You are a concise senior engineer.
OPENAI_RULES_FILE=./rules.txt
OPENAI_TOOLS=on
OPENAI_TOOLS_READ_ROOT=/Users/me/projects/notes
```

## License

Dual-licensed under MIT or Apache-2.0.
