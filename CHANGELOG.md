# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Optional system prompt via `OPENAI_SYSTEM_PROMPT`, sent as the
  Responses API `instructions` field on every turn.
- Optional rules file via `OPENAI_RULES_FILE`. Lines are trimmed, blank
  lines and `#` comments are ignored, and a leading `- ` is stripped.
  Rules are appended to the system prompt under a `# Rules` heading.
- Built-in function tool `get_time` returning UTC time as ISO 8601 and
  Unix seconds.
- Built-in function tool `read_file` for reading UTF-8 text files (max
  64 KiB). Gated behind `OPENAI_TOOLS_READ_ROOT`: if the variable is
  unset, `read_file` is not registered and cannot be invoked.
- `OPENAI_TOOLS=off` (or `false`/`0`) disables all built-in tools.
- Tool-call roundtrip loop in the client (bounded to 8 iterations per
  user turn).
- `OPENAI_PROVIDER` selects Azure `api-key` auth or OpenAI-compatible bearer auth.
- `OPENAI_LOG` controls quiet, normal, and verbose redacted diagnostics.
- `/image` slash command generates raster images via the Azure
  `images/generations` endpoint (`gpt-image-2` family) with configurable
  size, quality, count, and output format. Gated behind
  `AZURE_IMAGE_DEPLOYMENT`; outputs are written under `OPENAI_IMAGE_OUT_DIR`
  (default `./images`) with sandboxed paths.
- `/svg` slash command converts a local PNG to SVG markup via the chat
  model's vision input, with four style presets (editable, fidelity,
  compact, combined).
- `AZURE_IMAGE_API_VERSION` overrides the image-endpoint API version
  (default `2024-02-01`).
- Multi-modal input support in the request types (`InputItem::MessageParts`
  with text and image content parts).

### Changed
- Startup banner now reports the configured reasoning effort,
  instructions length, and enabled tool names.
- HTTP requests now use bounded retry/backoff for transient failures.
- Release artifacts are archived by version/target and include SHA256 sums.

[Unreleased]: https://github.com/cipher-rc5/gpt55_chat/compare/HEAD...HEAD
