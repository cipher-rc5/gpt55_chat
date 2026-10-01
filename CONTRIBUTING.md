# Contributing to gpt55-chat

Thanks for your interest in contributing.

## Toolchain

The project pins **Rust 1.98.1** via `rust-toolchain.toml` (also the
`rust-version` in `Cargo.toml`). If you use `rustup`, the correct toolchain is
installed automatically the first time you run a `cargo` command in this
directory. If you build outside `rustup`, install 1.98.1 manually.

`just` recipes always run the pinned toolchain, even when another Rust install
(such as Homebrew's `rust`) comes earlier on `PATH`. Bare `cargo` commands use
whichever `cargo` is first on `PATH`, so prefer the recipes for gate checks.

## Local CI

CI runs on your machine, not on a hosted service. The recipes live in the
`justfile`; install the tools once:

```sh
brew install just betterleaks cargo-zigbuild zig
cargo install cargo-audit cargo-deny cargo-cyclonedx --locked
```

Then enable the git hooks once per clone:

```sh
just hooks   # sets core.hooksPath=.githooks
```

| Hook / command | Runs |
|---|---|
| `pre-commit` hook (`just precommit`) | `cargo fmt --check` and a betterleaks scan of staged changes |
| `pre-push` hook (`just ci`) | the full gate below |
| `just ci` | fmt, `check` + `clippy` on all targets with `-D warnings`, tests, doctests, `doc` with `-D warnings`, `cargo audit`, `cargo deny check`, betterleaks over the git history |

Every change must pass `just ci`. The gate checks the working tree as it is,
so commit or stash unrelated edits before pushing. Missing tools fail the gate
with an install hint. `--no-verify` bypasses a hook in an emergency; prefer
fixing the cause.

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

  Keep the `// file:` path in sync with the file's real location.

- `cargo fmt` is mandatory. The `rustfmt.toml` is intentionally minimal so the
  pinned toolchain's default style is the source of truth.

- Public items should carry rustdoc, ideally with a runnable `///` example.

- Avoid `.unwrap()` and `.expect()` outside tests. Use `?` and the typed
  `ChatError` variants.

## Dependency policy

- Runtime dependencies in `[dependencies]` are pinned to exact versions
  (`=x.y.z`).
- Dev-dependencies are also exact-pinned for reproducible test runs.
- New runtime dependencies need a one-line justification in the PR description.
- `cargo deny check` and `cargo audit` must remain green.
- Update dependencies locally with `just update` (`cargo update` followed by
  the full gate). There are no Dependabot version-update PRs; GitHub
  Dependabot security alerts remain enabled.

## Commits and PRs

- Commit messages: short imperative subject (`feat: add X`, `fix: handle Y`).
- Reference issues in the body, not the subject.
- Open a draft PR if you want early feedback; mark ready-for-review once
  `just ci` passes locally.

## Releases

Releases are built and published from a maintainer's macOS machine in two
steps, so the artefacts can be inspected before anything goes public:

1. Bump `version` in `Cargo.toml`, update `CHANGELOG.md`, commit, and push to
   `main`.
2. `just release-build` refuses a dirty tree or an existing tag, runs
   `just ci`, then writes to `dist/`:
   - `.tar.gz` archives (binary, README, licenses, SECURITY.md) for
     `aarch64-apple-darwin`, `x86_64-apple-darwin`,
     `x86_64-unknown-linux-gnu`, and `aarch64-unknown-linux-gnu` (glibc 2.28+,
     cross-built with `cargo-zigbuild`);
   - a `.sha256` file per archive plus a combined `SHA256SUMS`;
   - a CycloneDX SBOM (`gpt55-chat-<version>.cdx.json`).
3. `just release-publish` asks for confirmation, checks that `dist/` was built
   from `HEAD` and that `HEAD` is on `origin/main`, re-verifies the checksums,
   creates and pushes the annotated `v<version>` tag, and uploads `dist/` to a
   GitHub Release with generated notes.

Windows binaries are not published; Windows users build from source. Release
artefacts carry SHA256 checksums and an SBOM but no signature or build
provenance attestation.

## Security

If you find a vulnerability, follow `SECURITY.md` instead of opening a public
issue.
