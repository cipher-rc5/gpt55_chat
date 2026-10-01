# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `cli` module exposing pure parsers for `/image`, `/svg`, and slash-command
  classification, with unit tests in `tests/cli.rs`.
- ctrl-c handler that cancels an in-flight chat turn and returns to the prompt
  instead of waiting for the HTTP timeout.
- `ImageRequest::output_compression(..)` builder method for lossy formats.
- `SECURITY.md` and `CONTRIBUTING.md`.
- Property-based tests for `tools::format_utc` (year-range bounds and
  monotonicity).
- Criterion benches for `tools::format_utc` and `client::extract_reply`.
- Local CI/CD via a `justfile`: `just ci` (fmt, check, clippy, tests,
  doctests, rustdoc, `cargo audit`, `cargo deny`, betterleaks history scan),
  `just precommit`, `just hooks`, `just update`, `just release-build`, and
  `just release-publish`.
- `.githooks/pre-push` runs `just ci`; `.githooks/pre-commit` now runs
  `just precommit` (rustfmt check plus a staged-change secret scan) and fails
  when a required tool is missing instead of skipping the scan.
- Release archives for `aarch64-apple-darwin`, `x86_64-apple-darwin`,
  `x86_64-unknown-linux-gnu`, and `aarch64-unknown-linux-gnu` (glibc 2.28+),
  built locally, with per-archive SHA256 files, a combined `SHA256SUMS`, and a
  CycloneDX SBOM.

### Changed
- Toolchain pinned to Rust **1.99.0**, the latest stable release (was 1.93);
  `rust-version` raised to match. The `justfile` puts the pinned toolchain
  first on `PATH`, so another Rust install (such as Homebrew's `rust`) cannot
  silently replace it.
- Benches use `std::hint::black_box` instead of the deprecated
  `criterion::black_box`, so `-D warnings` builds of all targets pass.
- Upstream HTTP error bodies are truncated to 256 chars at non-verbose log
  levels to prevent future server-echoed credentials from surfacing.
- `Role`, `Provider`, `LogLevel`, `ReasoningEffort`, `ReasoningSummary`, and
  `ChatError` are now `#[non_exhaustive]`. `Role` also accepts unrecognised
  variants via `#[serde(other)]` to avoid hard JSON decode failures.
- `wiremock` dev-dependency now pinned to an exact version.
- `deny.toml` allowlist trimmed to the licenses actually encountered.
- Dependencies refreshed to their latest releases: every direct dependency is
  on its newest crates.io version, and `Cargo.lock` is re-resolved for Rust
  1.99.0 (25 transitive updates).

### Removed
- GitHub Actions workflows (CI, Release, Secret Scan); CI/CD now runs locally.
- Dependabot version-update configuration; update dependencies with
  `just update`.
- `.pre-commit-config.yaml`; `.githooks/` is the single hook mechanism.
- Windows (`x86_64-pc-windows-msvc`) release archives; Windows users build
  from source.
- GitHub build-provenance attestations for release artefacts.

### Fixed
- Stale `// file: rust/src/…` headers across 6 source files now reference the
  correct `src/…` paths.
- `Cargo.lock` no longer resolves the yanked `yoke-derive` 0.8.3 (now
  0.8.4), so `cargo audit --deny warnings` passes.

## [0.1.0] - 2026-05-21

### Added
- Initial multi-turn chat CLI for the OpenAI/Azure Responses API.
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
- Startup banner reporting the configured reasoning effort, instructions
  length, and enabled tool names.
- HTTP requests use bounded retry/backoff for transient failures.
- Release artifacts are archived by version/target and include SHA256 sums.

[Unreleased]: https://github.com/cipher-rc5/gpt55_chat/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/cipher-rc5/gpt55_chat/releases/tag/v0.1.0
