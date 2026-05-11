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

### Changed
- Startup banner now reports the configured reasoning effort,
  instructions length, and enabled tool names.

[Unreleased]: https://github.com/cipher-rc5/gpt55_chat/compare/HEAD...HEAD
