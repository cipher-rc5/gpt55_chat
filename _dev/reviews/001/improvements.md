# Improvements Checklist

**Generated from review:** _dev/reviews/001/critical_analysis.md
**Date:** 2026-05-10

---

## P0 — Blockers

- [ ] **[Testing]** Add a `tests/` directory with at least table-driven coverage for `format_utc`, `compose_instructions`, `load_rules`, `ReasoningEffort::parse`, `ReasoningSummary::parse`, and `tools::execute` (get_time + read_file size/missing-arg cases) — `tests/` — Effort: M
- [ ] **[CI/CD]** Add `.github/workflows/ci.yml` running `cargo check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo doc -D warnings`, and `cargo audit` on push/PR — `.github/workflows/ci.yml` — Effort: M
- [ ] **[CI/CD]** Add a `LICENSE` file and set `[package].license` in `Cargo.toml` — `Cargo.toml:1-9` — Effort: S
- [ ] **[Safety]** Sandbox `tools::read_file` (or gate it behind an explicit `OPENAI_TOOLS_READ_ROOT` env var that scopes reads under one directory) to prevent prompt-injected reads of `~/.ssh/`, dotenv, etc. — `src/tools.rs:55-78` — Effort: M
- [ ] **[Safety]** Set HTTP client timeouts: `Client::builder().connect_timeout(…).timeout(…).build()` so a hung Azure endpoint doesn't wedge the chat loop — `src/main.rs:21` — Effort: S

## P1 — Pre-release

- [ ] **[Docs]** Add a `README.md` covering required env vars (`AZURE_API_KEY`, `AZURE_RESOURCE`, `API_VERSION`, `MODEL`) and the new optional ones (`OPENAI_SYSTEM_PROMPT`, `OPENAI_RULES_FILE`, `OPENAI_TOOLS`) with an example session — `README.md` — Effort: S
- [ ] **[Error Handling]** Surface JSON-parse errors from tool arguments instead of silently falling back to `Value::Null` — `src/tools.rs:48` — Effort: S
- [ ] **[Error Handling]** Add `ChatError::Tool(String)` variant and use it for the tool-roundtrip overflow instead of `ChatError::Config` — `src/client.rs:114-116`, `src/types.rs:201-213` — Effort: S
- [ ] **[API]** Mark `OutputItem` and `MessageContent` `#[non_exhaustive]` to keep future variants non-breaking if this code is lifted into a library — `src/types.rs:96-105, 161-171` — Effort: S
- [ ] **[Docs]** Add rustdoc summaries to every public type in `types.rs` (Role, InputItem, Tool, FunctionTool, ResponsesRequest, OutputItem, MessageContent, FunctionCall, ChatError, ClientConfig) — `src/types.rs:1-213` — Effort: M
- [ ] **[Dependencies]** Switch reqwest to `default-features = false, features = ["json", "rustls-tls"]` for a portable, vendored-TLS build — `Cargo.toml:13` — Effort: S

## P2 — Should-fix

- [ ] **[Docs]** Add `description`, `license`, `repository`, `keywords`, `rust-version` to `[package]` — `Cargo.toml:1-9` — Effort: S
- [ ] **[Docs]** Start a `CHANGELOG.md` capturing the system-prompt / rules / tools feature work — `CHANGELOG.md` — Effort: S
- [ ] **[API]** Change `extract_reply` to return `Option<String>` so callers can distinguish "no reply" from "empty reply" — `src/client.rs:124-138` — Effort: S
- [ ] **[Error Handling]** Distinguish "no text because incomplete" vs "no text because all tokens went to reasoning" in the main-loop log — `src/main.rs:80-90` — Effort: S
- [ ] **[Safety]** Use a saturating cast in `format_utc` (`unix_secs.min(i64::MAX as u64) as i64`) — `src/tools.rs:85` — Effort: S
- [ ] **[CI/CD]** Add `rust-toolchain.toml` pinning the toolchain to a known-good stable version — `rust-toolchain.toml` — Effort: S

## P3 — Nice-to-have

- [ ] **[Conventions]** Replace blanket `#[allow(dead_code)]` on response structs with field-level annotations on truly unread fields — `src/types.rs:91-179` — Effort: S
- [ ] **[Conventions]** Add `ReasoningEffort::as_str` and `ReasoningSummary::as_str` instead of `format!("{e:?}").to_ascii_lowercase()` — `src/main.rs:34`, `src/types.rs:19-42` — Effort: S
- [ ] **[Performance]** None identified — current allocation profile is already lean — Effort: —
- [ ] **[Dependencies]** Add `deny.toml` for `cargo-deny` to gate transitive-license drift — `deny.toml` — Effort: M
- [ ] **[API]** Concatenate multiple non-empty `Message` items in `extract_reply` rather than only returning the first — `src/client.rs:124-138` — Effort: S

---

## Progress

**Total items:** 22
**P0:** 5 | **P1:** 6 | **P2:** 6 | **P3:** 5 — see improvements.md
