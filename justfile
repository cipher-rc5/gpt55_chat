# Local CI/CD for gpt55-chat. Everything that used to run on GitHub Actions
# runs here instead. `just --list` shows the recipes.
#
#   just hooks            one-time per clone: enable .githooks (pre-commit, pre-push)
#   just ci               full gate (run automatically by the pre-push hook)
#   just update           cargo update, then the full gate
#   just release-build    build + checksum + SBOM release archives into dist/
#   just release-publish  tag the release and upload dist/ to GitHub Releases

set shell := ["bash", "-euo", "pipefail", "-c"]

# Put the toolchain pinned in rust-toolchain.toml first on PATH, so cargo,
# rustc, clippy, rustfmt, and cargo-zigbuild's inner cargo all use it even when
# another Rust install (e.g. Homebrew's `rust`) shadows the rustup proxies.
# Without rustup, fall back to whichever cargo is on PATH.
export PATH := `dirname "$(rustup which cargo 2>/dev/null || command -v cargo)"` + ":" + env("PATH")

# Warnings are errors for every build the gate runs (same flags CI used).
export RUSTFLAGS := "-D warnings"
export RUSTDOCFLAGS := "-D warnings"

mac_targets := "aarch64-apple-darwin x86_64-apple-darwin"
linux_targets := "x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu"
# Minimum glibc for the Linux archives (cargo-zigbuild target suffix).
linux_glibc := "2.28"

# List the available recipes.
default:
    @just --list

# Point git at the repo's hooks: pre-commit runs `precommit`, pre-push runs `ci`.
hooks:
    git config core.hooksPath .githooks
    @echo "hooks enabled: pre-commit -> just precommit, pre-push -> just ci"

# Fast gate for every commit: formatting plus a secret scan of staged changes.
precommit: (require "betterleaks")
    cargo fmt --all -- --check
    betterleaks git --pre-commit --staged --redact --no-banner

# Full gate. Validates the working tree as it is right now.
ci: (require "cargo-audit" "cargo-deny" "betterleaks")
    @echo "toolchain: $(rustc --version)"
    cargo fmt --all -- --check
    cargo check --all-targets --locked
    cargo clippy --all-targets --all-features --locked -- -D warnings
    cargo test --all-targets --locked
    cargo test --doc --locked
    cargo doc --no-deps --locked
    cargo audit --deny warnings
    cargo deny check
    betterleaks git . --redact --no-banner
    @echo "ci: all gates passed"

# Refresh Cargo.lock within the semver ranges in Cargo.toml, then run the gate.
update:
    cargo update
    just ci

# Build release archives for macOS and Linux into dist/ (requires a clean tree).
release-build: (require "rustup" "cargo-zigbuild" "zig" "cargo-cyclonedx" "shasum")
    #!/usr/bin/env bash
    set -euo pipefail
    version="$(cargo pkgid | sed -E 's/.*[#@]//')"
    tag="v${version}"
    if [ -n "$(git status --porcelain)" ]; then
        echo "error: working tree is not clean; commit or stash before building a release" >&2
        exit 1
    fi
    if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
        echo "error: tag ${tag} already exists; bump the version in Cargo.toml first" >&2
        exit 1
    fi
    just ci
    just _dist

# Tag HEAD as v<version> and publish dist/ as a GitHub Release.
[confirm("Tag this commit and publish dist/ to GitHub Releases? [y/N]")]
release-publish: (require "gh")
    #!/usr/bin/env bash
    set -euo pipefail
    version="$(cargo pkgid | sed -E 's/.*[#@]//')"
    tag="v${version}"
    head="$(git rev-parse HEAD)"
    if [ -n "$(git status --porcelain)" ]; then
        echo "error: working tree is not clean" >&2
        exit 1
    fi
    if [ ! -f dist/COMMIT ] || [ "$(cat dist/COMMIT)" != "${head}" ]; then
        echo "error: dist/ was not built from HEAD; run 'just release-build' first" >&2
        exit 1
    fi
    if [ "$(cat dist/VERSION)" != "${version}" ]; then
        echo "error: dist/ holds version $(cat dist/VERSION), Cargo.toml says ${version}" >&2
        exit 1
    fi
    (cd dist && shasum -a 256 -c SHA256SUMS)
    git fetch --quiet origin main
    if ! git merge-base --is-ancestor HEAD origin/main; then
        echo "error: HEAD is not on origin/main; push it before releasing" >&2
        exit 1
    fi
    git tag -a "${tag}" -m "gpt55-chat ${tag}"
    git push origin "${tag}"
    gh release create "${tag}" --verify-tag --title "${tag}" --generate-notes \
        dist/*.tar.gz dist/*.sha256 dist/SHA256SUMS dist/*.cdx.json

# Build, archive, checksum, and SBOM every release target into dist/.
_dist:
    #!/usr/bin/env bash
    set -euo pipefail
    version="$(cargo pkgid | sed -E 's/.*[#@]//')"
    rm -rf dist
    mkdir -p dist
    rustup target add {{ mac_targets }} {{ linux_targets }}
    for target in {{ mac_targets }}; do
        cargo build --release --locked --target "${target}"
    done
    for target in {{ linux_targets }}; do
        cargo zigbuild --release --locked --target "${target}.{{ linux_glibc }}"
    done
    for target in {{ mac_targets }} {{ linux_targets }}; do
        name="gpt55-chat-${version}-${target}"
        mkdir "dist/${name}"
        cp "target/${target}/release/gpt55-chat" "dist/${name}/"
        cp README.md LICENSE LICENSE-MIT SECURITY.md "dist/${name}/"
        tar -C dist -czf "dist/${name}.tar.gz" "${name}"
        rm -rf "dist/${name}"
        (cd dist && shasum -a 256 "${name}.tar.gz" > "${name}.sha256")
    done
    (cd dist && cat ./*.sha256 > SHA256SUMS && shasum -a 256 -c SHA256SUMS)
    cargo cyclonedx --format json --override-filename "gpt55-chat-${version}.cdx"
    mv "gpt55-chat-${version}.cdx.json" dist/
    # A dirty-tree build gets a suffix so release-publish can never match it.
    commit="$(git rev-parse HEAD)"
    [ -z "$(git status --porcelain)" ] || commit="${commit}-dirty"
    printf '%s\n' "${commit}" > dist/COMMIT
    printf '%s' "${version}" > dist/VERSION
    echo "dist/ ready for ${version}:"
    ls -1 dist

# Fail with an install hint when a required tool is missing.
[private]
require +tools:
    #!/usr/bin/env bash
    set -euo pipefail
    missing=0
    for tool in {{ tools }}; do
        command -v "${tool}" >/dev/null 2>&1 && continue
        case "${tool}" in
            cargo-audit|cargo-deny|cargo-cyclonedx) hint="cargo install ${tool} --locked" ;;
            cargo-zigbuild|zig) hint="brew install cargo-zigbuild zig" ;;
            betterleaks|gh|rustup) hint="brew install ${tool}" ;;
            *) hint="install ${tool}" ;;
        esac
        echo "error: required tool '${tool}' is not installed (${hint})" >&2
        missing=1
    done
    exit "${missing}"
