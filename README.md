# gpt55-chat

A small, single-binary multi-turn chat CLI for the OpenAI/Azure Responses API.
It chains turns via `previous_response_id`, supports configurable reasoning
effort and summaries, an optional system prompt plus rules file, and a small
set of built-in function tools.

## Installation

Download the archive for your target from the GitHub release, verify it, then
install the binary somewhere on your `PATH`.

```sh
VERSION=0.1.0
TARGET=x86_64-unknown-linux-gnu
curl -LO "https://github.com/cipher-rc5/gpt55_chat/releases/download/v${VERSION}/gpt55-chat-${VERSION}-${TARGET}.tar.gz"
curl -LO "https://github.com/cipher-rc5/gpt55_chat/releases/download/v${VERSION}/gpt55-chat-${VERSION}-${TARGET}.sha256"
shasum -a 256 -c "gpt55-chat-${VERSION}-${TARGET}.sha256"
tar -xzf "gpt55-chat-${VERSION}-${TARGET}.tar.gz"
install "gpt55-chat-${VERSION}-${TARGET}/gpt55-chat" /usr/local/bin/gpt55-chat
```

Windows releases are shipped as `.zip` archives. Verify the SHA256 entry before
adding `gpt55-chat.exe` to your `PATH`.

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
| `OPENAI_PROVIDER`         | `azure` | `azure` uses `api-key`; `openai-compatible` uses `Authorization: Bearer` and `OPENAI_ENDPOINT`.          |
| `OPENAI_LOG`              | `normal` | `quiet`, `normal`, or `verbose`. Verbose diagnostics remain redacted.                                    |
| `OPENAI_TOOLS`            | on      | Set to `off` (or `false`/`0`) to disable all built-in tools.                                             |
| `OPENAI_TOOLS_READ_ROOT`  | unset   | Directory the `read_file` tool may read under. If unset, `read_file` is disabled.                        |
| `AZURE_IMAGE_DEPLOYMENT`  | unset   | Azure deployment name for the image-generation endpoint (e.g. `gpt-image-2`). When unset, `/image` is disabled. |
| `AZURE_IMAGE_API_VERSION` | `2024-02-01` | API version used for image generation.                                                              |
| `OPENAI_IMAGE_OUT_DIR`    | `./images` | Directory into which `/image` and `/svg` write artifacts. Created on first use; output paths are sandboxed inside it. |

For `OPENAI_PROVIDER=openai-compatible`, set `OPENAI_API_KEY`, `OPENAI_ENDPOINT`
pointing at a Responses-compatible endpoint, and `MODEL`. `API_VERSION` is only
used for Azure.

## Slash commands

Available at the `you:` prompt:

- **`/image [flags] <prompt>`** — generate one or more raster images via the
  Azure `images/generations` endpoint and write them under
  `OPENAI_IMAGE_OUT_DIR` (default `./images`). Requires `AZURE_IMAGE_DEPLOYMENT`.

  Flags (all optional): `--size=WxH` (default `1024x1024`), `--quality=low|medium|high`
  (default `high`), `--n=1..10` (default `1`), `--format=png|jpeg|webp`
  (default `png`).

  Prompt forms:
  - Inline: `/image a red fox in an autumn forest, golden hour`
  - From a file: `/image --quality=high @prompts/logo.txt`
  - Multi-line: type `/image` alone, then enter the prompt and finish with a
    single `.` on its own line. Useful for long style-direction prompts.

- **`/svg [--style=editable|fidelity|compact|combined] <path-to-png>`** —
  convert a local PNG to SVG markup via the chat model's vision input. The
  result is written to `<OPENAI_IMAGE_OUT_DIR>/<stem>.svg`. The chat model
  deployment must be vision-capable for this to work.

  Style presets (system-prompt-level direction):
  - `editable` — real editable vector paths, gradients/masks, prioritises
    Figma/Illustrator/Inkscape editability.
  - `fidelity` — closest visual match to the source; paths may be more detailed.
  - `compact` — smallest production SVG with simplified paths.
  - `combined` (default) — balanced fidelity + editability + small size.

- **`/help`** — list commands.

## Built-in tools

- `get_time` — returns the current UTC time as ISO 8601 plus Unix seconds.
- `read_file` — reads a UTF-8 text file (max 64 KiB). Gated behind
  `OPENAI_TOOLS_READ_ROOT`: when that variable is unset, `read_file` is
  omitted from the tool list sent to the model entirely, so the model
  cannot see or call it. When the root IS set, the model-supplied `path`
  argument is resolved relative to the process CWD (not the sandbox
  root), then canonicalised and rejected unless it lands inside the
  sandbox.

## Security model

- API keys are never intentionally logged, and config debug output redacts the
  key field.
- Azure mode sends only the `api-key` header. OpenAI-compatible mode sends only
  `Authorization: Bearer`.
- Tool diagnostics are redacted by default. `OPENAI_LOG=verbose` reports tool
  names and argument/output lengths, not raw file paths or contents.
- `read_file` is not advertised unless `OPENAI_TOOLS_READ_ROOT` is set. File
  paths are canonicalised and rejected unless they remain inside that sandbox;
  file contents are still returned to the model when the tool is enabled.
- Do not point `OPENAI_TOOLS_READ_ROOT` at directories containing secrets unless
  you intend the model to be able to request those files.

## Live smoke test

Use non-production credentials and a low-risk deployment when validating a real
provider. The command below should return one assistant response and exit when
stdin closes.

```sh
AZURE_API_KEY=...
AZURE_RESOURCE=https://my-resource.openai.azure.com
API_VERSION=2025-04-01-preview
MODEL=gpt-5.5
printf 'Say pong only.\nexit\n' | gpt55-chat
```

Expected behavior: the process exits successfully, prints one assistant reply,
and does not print raw credentials.

## Exit behavior

Configuration and HTTP client construction failures exit non-zero before the
chat loop starts. Per-turn API/tool/transport errors are printed and the
interactive session remains alive. EOF or `exit` exits successfully.

## Example session

```text
$ ./target/release/gpt55-chat
gpt-5.5 chat | reasoning=medium | instructions=128 chars | tools=get_time,read_file | type 'exit' to quit
you: what time is it?
assistant: It's 2026-05-10T14:32:07Z (Unix 1778510327).
you: summarise ./notes/todo.txt in one line
assistant: A short checklist of three release-blocking tasks: fix CI fmt, add MSRV job, and ship the v0.2 tag.
you: exit
```

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

# Image generation (optional)
AZURE_IMAGE_DEPLOYMENT=gpt-image-2
AZURE_IMAGE_API_VERSION=2024-02-01
OPENAI_IMAGE_OUT_DIR=./images
```

## License

Dual-licensed under MIT or Apache-2.0.
