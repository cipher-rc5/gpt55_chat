# Improvements Checklist

**Generated from:** `_dev/reviews/004/critical_analysis.md`
**Date:** 2026-05-12
**Goal:** Close the remaining production-readiness gaps in the current Rust CLI/library.

## P0 - Release Blockers

- [ ] **[Docs] Fix README auth documentation.** Update `README.md:39` so it no longer claims Azure keys are sent as both bearer and `api-key`; document the implemented provider-specific behavior.
- [ ] **[Security] Actually close the `read_file` TOCTOU gap.** Replace check-then-open in `src/tools.rs:113-124` with an open-and-verify approach, platform-specific no-follow behavior, or remove symlink-following support entirely.
- [ ] **[Reliability] Make retries safe for side-effecting POSTs.** Add idempotency keys/request IDs if supported, or restrict automatic retries to cases that cannot have reached the server. Document the retry safety model.
- [ ] **[Review Hygiene] Correct `_dev/reviews/003/improvements.md`.** Do not mark TOCTOU hardening complete until the implementation is actually race-resistant.

## P1 - Pre-Production Hardening

- [ ] **[Compatibility] Replace placeholder capability validation.** Add explicit provider/API-version/model capability rules for reasoning, reasoning summaries, tools, and `previous_response_id`.
- [ ] **[Config] Fail OpenAI-compatible `API_VERSION` at load time.** Move the current send-time rejection into `load_config()` so bad configuration fails before the chat loop starts.
- [ ] **[Security] Remove `AZURE_API_KEY` fallback for OpenAI-compatible mode.** Require `OPENAI_API_KEY` when `OPENAI_PROVIDER=openai-compatible` to avoid sending the wrong secret to the wrong endpoint.
- [ ] **[API] Reconsider public raw-secret access.** Make `ClientConfig::api_key()` crate-private or wrap the API key in a secret type with explicit expose semantics.
- [ ] **[Testing] Expand retry tests.** Cover `408`, `429`, `503`, retry budget exhaustion, `Retry-After`, non-retryable `400`, and transport retry behavior.
- [ ] **[Testing] Add file-tool symlink and traversal tests.** Cover symlinks inside the sandbox, symlinks pointing outside, relative paths, canonical root failure, and race-resistant behavior where feasible.
- [ ] **[CI] Pin cargo tool versions.** Pin `cargo-audit`, `cargo-deny`, and `cargo-llvm-cov` versions in `.github/workflows/ci.yml` or install them through a locked tool manifest.

## P2 - Production Quality Improvements

- [ ] **[Coverage] Enforce coverage thresholds.** Keep uploading `lcov.info`, but add a minimum line/region threshold or diff coverage gate for critical modules.
- [ ] **[Release] Add a consolidated release manifest.** Publish a single checksum manifest across all artifacts and include SBOM/provenance documentation in the release notes.
- [ ] **[Release] Add ARM targets.** Ship at least `aarch64-apple-darwin` and `aarch64-unknown-linux-gnu` artifacts, or document why they are source-build only.
- [ ] **[Smoke Testing] Add an opt-in live smoke workflow.** Provide a manually-triggered GitHub Actions workflow using repository secrets and a non-production deployment.
- [ ] **[Docs] Clarify `read_file` relative path semantics.** Prefer changing implementation so relative paths resolve under `OPENAI_TOOLS_READ_ROOT`; if not, make the CWD behavior more prominent.
- [ ] **[Compliance] Reduce cargo-deny warning noise.** Remove currently unused license allowances or configure warning handling so CI output remains high signal.

## P3 - Follow-Up Maintenance

- [ ] **[Dependencies] Define dependency update ownership.** Document who reviews Dependabot PRs and how quickly security updates should land.
- [ ] **[Release] Require a clean tagged tree for releases.** Add a release checklist or CI guard that verifies releases are built from a clean, reviewed tag.
- [ ] **[UX] Add non-interactive mode.** Provide a single-prompt mode with clear exit codes for scripting and monitoring.
- [ ] **[Observability] Add structured JSON output mode.** Keep human output by default, but support machine-readable logs/responses for automation.

## Completion Target

Production-grade threshold: all P0 items complete, P1 items substantially complete, clean committed tree, CI green on the target branch, and one successful release candidate with verified artifacts.
