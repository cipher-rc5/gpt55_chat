# Improvements Checklist

**Generated from review:** _dev/reviews/002/critical_analysis.md
**Date:** 2026-05-11

---

## P0 — Blockers

- [ ] **[CI/CD]** Fix the broken `cargo fmt --all -- --check` step — add a `rustfmt.toml` matching the current style and run `cargo fmt --all` to normalise the tree, OR drop the fmt step until the project decides on a Rust formatter — `.github/workflows/ci.yml:30` — Effort: S
- [ ] **[Docs]** Correct the README claim about `read_file` registration — the tool IS registered when `OPENAI_TOOLS` is on; it just refuses at execution time when `OPENAI_TOOLS_READ_ROOT` is unset. Either update the README to match, or have `builtin_tools()` skip `read_file` when the env var is missing (cleaner; recommended) — `README.md:42`, `src/tools.rs:15-46`, `src/config.rs:128-133` — Effort: S

## P1 — Pre-release

- [ ] **[Testing]** Add unit tests for `extract_reply` (empty output, single message, multiple messages, mixed reasoning + message) — `src/client.rs:120-132`, new `tests/extract_reply.rs` — Effort: S
- [ ] **[Testing]** Add a `wiremock` dev-dependency and one integration test that drives `run_turn` end-to-end against a stub Azure endpoint, including the tool-roundtrip path — `Cargo.toml`, new `tests/run_turn.rs` — Effort: M
- [ ] **[Safety]** Document and/or enforce that the `path` argument to `read_file` resolves relative to the sandbox root, not the process CWD — `src/tools.rs:91-100` — Effort: S
- [ ] **[CI/CD]** Add an OS matrix (ubuntu / macOS / windows) to the `check` job — `.github/workflows/ci.yml:13-44` — Effort: S
- [ ] **[CI/CD]** Add a separate MSRV job pinning `1.85` so the declared `rust-version` is actually exercised — `.github/workflows/ci.yml`, `rust-toolchain.toml` — Effort: S
- [ ] **[Docs]** Add a crate-level `//!` summary in `src/lib.rs` and one-line rustdoc on each `pub mod` — `src/lib.rs:1-7` — Effort: S
- [ ] **[Docs]** Add rustdoc on `pub fn builtin_tools` — `src/tools.rs:15` — Effort: S
- [ ] **[API]** Replace the `mod` declarations in `main.rs` with `use gpt55_chat::…` so the source isn't compiled twice — `src/main.rs:5-8`, `src/lib.rs:4-7` — Effort: S

## P2 — Should-fix

- [ ] **[Testing]** Cover `compose_instructions` with a whitespace-only system prompt input — `src/config.rs:50-68`, `tests/rules.rs` — Effort: S
- [ ] **[Testing]** Cover `load_rules` against a non-existent path (should return `ChatError::Config`) — `src/config.rs:38-47`, `tests/rules.rs` — Effort: S
- [ ] **[Error Handling]** Clear or invalidate `previous_response_id` when an HTTP error indicates the previous response itself is at fault (e.g. context-window-exceeded) — `src/main.rs:131-137` — Effort: S
- [ ] **[CI/CD]** Add a `cargo deny check` step using the existing `deny.toml` — `.github/workflows/ci.yml` — Effort: S
- [ ] **[CI/CD]** Add a release workflow (tag-triggered) that runs the full suite and uploads release binaries — `.github/workflows/release.yml` — Effort: M
- [ ] **[Conventions]** Decide on the formatter story: either delete `dprint.json` (TypeScript-only, doesn't apply here) or replace it with a `rustfmt.toml` — `dprint.json`, repo root — Effort: S
- [ ] **[Docs]** Add a short example session transcript or screenshot to the README — `README.md` — Effort: S

## P3 — Nice-to-have

- [ ] **[API]** Rename `ChatError::Parse(reqwest::Error)` to `Transport` (or split into `Network` / `Decode`) so the variant name matches what it catches — `src/types.rs` (ChatError block) — Effort: S
- [ ] **[API]** Provide a `ClientConfigBuilder` so downstream library users don't need to populate every `pub` field by hand — `src/types.rs` (ClientConfig block) — Effort: M
- [ ] **[Safety]** Re-open the canonicalised path under O_NOFOLLOW (or open + fstat instead of stat + read) to close the TOCTOU symlink-swap window in `read_file` — `src/tools.rs:101-125` — Effort: M
- [ ] **[Conventions]** Tighten the body-read error path: log the underlying error from `response.text().await` instead of silently substituting `"<unreadable>"` — `src/client.rs:53-58` — Effort: S
- [ ] **[Dependencies]** Tighten pins to `=x.y.z` if you intend `Cargo.lock` to be the source of truth across machines — `Cargo.toml:19-26` — Effort: S

---

## Progress

**Total items:** 22
**P0:** 2 | **P1:** 8 | **P2:** 7 | **P3:** 5
