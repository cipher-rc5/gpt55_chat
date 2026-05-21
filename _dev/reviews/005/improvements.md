# Improvements Checklist

**Generated from review:** _dev/reviews/005/critical_analysis.md
**Date:** 2026-05-21

---

## P0 — Blockers

- [ ] **[CI/CD]** Run `cargo fmt --all` under the pinned 1.93 toolchain and commit the result; `cargo fmt --check` currently exits 1 on `main` at `src/image.rs:122`, `src/main.rs:44`, `src/main.rs:281`, `src/main.rs:307`. — Effort: S
- [ ] **[Conventions]** Correct the `// file:` header in `src/client.rs:1`, `src/config.rs:1`, `src/lib.rs:1`, `src/main.rs:1`, `src/tools.rs:1`, `src/types.rs:1` to reference the actual `src/...` paths (not the non-existent `rust/src/...`). — Effort: S

## P1 — Pre-release

- [ ] **[Docs]** Add `SECURITY.md` with a coordinated-disclosure contact and supported-versions table; required before publicising the release artefacts. — Effort: S
- [ ] **[CI/CD]** Add an SBOM step to `.github/workflows/release.yml` (e.g. `anchore/sbom-action` or `cargo-cyclonedx`) and attach the SBOM alongside the binary, sha256, and attestation. — Effort: M
- [ ] **[CI/CD]** Have the release workflow re-run `cargo clippy --all-targets -- -D warnings`, `cargo audit`, and `cargo deny check` before the build/publish steps (`.github/workflows/release.yml:30-31` currently only runs `cargo test`). — Effort: S
- [ ] **[CI/CD]** Extend the release matrix in `.github/workflows/release.yml:43-55` to cover `aarch64-apple-darwin` and `aarch64-unknown-linux-gnu`. — Effort: M
- [ ] **[Docs]** Add a `[0.1.0]` row to `CHANGELOG.md` matching `Cargo.toml:3 version = "0.1.0"`, and replace the placeholder `[Unreleased]: …HEAD...HEAD` link with a real compare URL. — Effort: S
- [ ] **[CI/CD]** Pin every GitHub Action in `.github/workflows/ci.yml` and `release.yml` by immutable SHA (currently `@v1`, `@v2`, `@v4`). — Effort: M
- [ ] **[Testing]** Add integration tests for the slash-command parser (`src/main.rs:173-198`, `handle_image` / `handle_svg`) and the multi-line prompt reader (`src/main.rs:316-334`). — Effort: M

## P2 — Should-fix

- [ ] **[API]** Mark `Provider`, `LogLevel`, `ReasoningEffort`, `ReasoningSummary`, and `ChatError` `#[non_exhaustive]` (`src/types.rs:280, 308, 25, 64, 641`) before tagging 1.0. — Effort: S
- [ ] **[Safety]** Add `#[serde(other)]` fallback to `Role` (`src/types.rs:14-20`) so an unrecognised role from the API does not cause a JSON decode failure. — Effort: S
- [ ] **[Concurrency]** Add a ctrl-c handler that aborts the current `run_turn` and returns to the `you:` prompt; today the REPL is blocked on the 300s HTTP timeout (`src/main.rs:21`, `src/main.rs:73-163`). — Effort: M
- [ ] **[Error Handling]** Redact / truncate the upstream body printed at `src/main.rs:156` when `log_level != Verbose` to avoid surfacing future server-echoed credentials. — Effort: S
- [ ] **[Testing]** Add doc-tests to public items in `lib.rs` and the public client/image/svg modules; current doc-test count is 0. — Effort: M
- [ ] **[Dependencies]** Pin `wiremock = "=0.6.5"` (or the current version) in `Cargo.toml:30` to match the runtime pinning policy. — Effort: S
- [ ] **[Dependencies]** Remove unused entries (`BSD-2-Clause`, `Zlib`, `Unicode-DFS-2016`, `MPL-2.0`) from `deny.toml:5-17`, or document why they remain pre-approved. — Effort: S

## P3 — Nice-to-have

- [ ] **[Testing]** Add a property-based test (e.g. `proptest`) for `tools::format_utc` (`src/tools.rs:150`) over arbitrary `u64` inputs. — Effort: M
- [ ] **[Performance]** Add a `criterion` benchmark crate covering `extract_reply` and `format_utc`. — Effort: M
- [ ] **[CI/CD]** Upload `lcov.info` to Codecov (or equivalent) in `.github/workflows/ci.yml:107-132` so coverage trends become visible per PR. — Effort: S
- [ ] **[Conventions]** Audit the nine `#[allow(dead_code)]` markers in `src/types.rs` (lines 82, 187, 194, 197, 205, 207, 215, 228, 266) and either use the fields or remove them. — Effort: M
- [ ] **[Docs]** Add `CONTRIBUTING.md` describing the gate commands so contributors don't rediscover them. — Effort: S
- [ ] **[Docs]** State the library-API semver caveat in `README.md` (today only in `src/lib.rs:20`). — Effort: S
- [ ] **[Performance]** Collapse the duplicate filesystem syscalls in `svg::read_png_as_data_url` (`src/svg.rs:101-133`) to a single read with length check. — Effort: S
- [ ] **[API]** Add an `ImageRequest::output_compression(..)` builder method matching the existing `size/quality/n/output_format` setters (`src/image.rs:52-75`). — Effort: S

---

## Progress

**Total items:** 22
**P0:** 2 | **P1:** 7 | **P2:** 7 | **P3:** 6
