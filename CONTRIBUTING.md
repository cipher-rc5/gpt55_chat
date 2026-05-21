# Contributing to gpt55-chat

Thanks for your interest in contributing.

## Toolchain

The project pins **Rust 1.95** via `rust-toolchain.toml`. If you use `rustup`,
the correct toolchain is installed automatically the first time you run a
`cargo` command in this directory. If you build outside `rustup`, install
1.95 manually — older toolchains will fail the MSRV CI job.

## Development gates

Every change must pass the same gates CI runs. From the repo root:

```sh
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo audit
cargo deny check
```

If you change anything performance-sensitive, also run the benches:

```sh
cargo bench
```

## Code style

- Every `.rs` file begins with two lines:

  ```rust
  // file: <relative path from repo root>
  // description: <one short line>
  ```

  CI verifies that the `// file:` path matches the file's real location.

- `cargo fmt` is mandatory. The `rustfmt.toml` is intentionally minimal so the
  pinned toolchain's default style is the source of truth.

- Public items should carry rustdoc, ideally with a runnable `///` example.

- Avoid `.unwrap()` and `.expect()` outside tests. Use `?` and the typed
  `ChatError` variants.

## Dependency policy

- Runtime dependencies in `[dependencies]` are pinned to exact versions
  (`=x.y.z`). Dependabot will open PRs to bump them; review and merge.
- Dev-dependencies are also exact-pinned for reproducible test runs.
- New runtime dependencies need a one-line justification in the PR description.
- `cargo deny check` and `cargo audit` must remain green.

## Commits and PRs

- Commit messages: short imperative subject (`feat: add X`, `fix: handle Y`).
- Reference issues in the body, not the subject.
- Open a draft PR if you want early feedback; mark ready-for-review when CI is
  green.

## Releases

Releases are cut by pushing a `v*` tag. The `Release` workflow:

1. Re-runs `cargo fmt --check`, `cargo clippy`, `cargo audit`, `cargo deny check`,
   and `cargo test` on Linux, macOS, and Windows.
2. Builds release binaries for the platforms listed in
   `.github/workflows/release.yml`.
3. Produces SHA256 checksums, a CycloneDX SBOM, and a GitHub-provenance
   attestation for every artefact.
4. Publishes the GitHub Release with auto-generated notes.

## Security

If you find a vulnerability, follow `SECURITY.md` instead of opening a public
issue.
