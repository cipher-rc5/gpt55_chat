# Improvements Checklist

**Generated from:** `_dev/reviews/003/critical_analysis.md`
**Date:** 2026-05-11
**Goal:** Bring the codebase to production-grade standards for a Rust CLI/API client.

## P0 - Release Blockers

- [x] **[CI] Fix the Rust version contradiction.** Choose the real MSRV and make `Cargo.toml:8`, `rust-toolchain.toml:2`, and `.github/workflows/ci.yml:50-67` agree. If the project requires Rust `1.93`, rename the MSRV job or install `1.93`; if the MSRV is `1.85`, lower `rust-version` only after confirming the code and dependencies compile there.
- [x] **[Compliance] Make `cargo deny check` pass.** Review `CDLA-Permissive-2.0` from `webpki-root-certs v1.0.7`; either add it to `deny.toml` after legal approval or change dependencies/features to avoid it.
- [x] **[Security] Redact API keys from debug output.** Remove `Debug` from `ClientConfig`, implement a custom redacted `Debug`, or wrap `api_key` in a secret type that never prints raw credentials.
- [x] **[Testing] Add strict request-contract tests.** Extend `tests/run_turn.rs` or add new wiremock tests that assert query string, auth headers, JSON body fields, tool definitions, `previous_response_id`, and function-call output shape.

## P1 - Pre-Production Hardening

- [x] **[Security] Select one auth scheme per provider.** Add explicit provider/config handling so Azure uses `api-key` and OpenAI-compatible endpoints use `Authorization: Bearer`, rather than sending both headers to every endpoint.
- [x] **[Security] Redact tool diagnostics.** Replace `src/client.rs:106-107` raw argument/output previews with opt-in verbose logging and redact file paths or contents by default.
- [x] **[Security] Close the `read_file` TOCTOU window.** Open files in a way that avoids symlink swapping after validation, such as platform-specific no-follow behavior or open-then-verify metadata/inode under the sandbox.
- [x] **[Correctness] Parse and validate endpoint URLs.** Replace string concatenation in `src/config.rs:95-96` and `src/client.rs:16-18` with URL parsing that rejects existing query strings, invalid schemes, malformed resources, and accidental path duplication.
- [x] **[Correctness] Validate numeric config ranges.** Reject `MAX_OUTPUT_TOKENS=0` and define an upper bound that matches the supported model/API limits.
- [x] **[Reliability] Add bounded retry/backoff.** Retry transient `408`, `429`, and `5xx` responses with jitter and `Retry-After` support; keep non-retryable errors immediate.
- [x] **[Testing] Cover HTTP error paths.** Add tests for non-2xx bodies, malformed JSON responses, transport errors, timeout-like failures, and max tool-roundtrip overflow.
- [x] **[Testing] Cover `load_config()`.** Add isolated environment-variable tests for required variables, optional parsing, invalid values, `OPENAI_TOOLS=off`, and `OPENAI_TOOLS_READ_ROOT` behavior.
- [x] **[Release] Harden artifacts.** Publish archives named by version and target triple, plus SHA256 checksums and ideally signatures or provenance attestations.

## P2 - Production Quality Improvements

- [x] **[Operability] Add logging controls.** Provide quiet/verbose modes and optionally structured JSON logs for automation.
- [x] **[Operability] Define exit behavior.** Decide which errors should keep the interactive session alive and which should exit non-zero; document this for scripted use.
- [x] **[Testing] Add a documented live smoke test.** Provide a gated command or ignored test that can run against a real Azure/OpenAI deployment using test credentials.
- [x] **[Compatibility] Add provider/model capability checks.** Fail early or degrade gracefully when configured API versions/models do not support reasoning, tools, or response chaining.
- [x] **[UX] Improve multi-message formatting.** Consider separators or explicit handling when `extract_reply()` sees multiple assistant message items.
- [x] **[Dependencies] Add update automation.** Exact dependency pins are workable only with a routine update/audit process; add Dependabot/Renovate or a documented release maintenance cadence.
- [x] **[Docs] Document security model.** Add a README section explaining what local tools can access, how sandboxing works, what is logged, and how credentials are handled.

## P3 - Nice-to-Have

- [x] **[API] Introduce typed provider configuration.** Replace raw `endpoint` plus `api_version` strings with an enum or builder that captures Azure/OpenAI differences.
- [x] **[API] Avoid exposing raw mutable config fields long-term.** If the library becomes public-facing, prefer constructors/builders and accessors over fully public `ClientConfig` fields.
- [x] **[Quality] Add coverage reporting.** Track test coverage for `src/client.rs`, `src/config.rs`, and `src/tools.rs` to prevent regression in critical paths.
- [x] **[Release] Add installation documentation.** Document how users install release artifacts, verify checksums, and configure shells or PATH.

## Completion Target

Production-grade threshold: P0 complete, P1 substantially complete, all CI jobs green on a clean branch, and one successful tagged release candidate with verifiable artifacts.
