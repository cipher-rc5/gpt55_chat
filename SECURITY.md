# Security Policy

## Supported Versions

Only the latest released `0.x` line receives security fixes. The library API
is not yet semver-stable and may change between minor versions — see the
[library notes in `src/lib.rs`](src/lib.rs) for details.

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅        |
| < 0.1   | ❌        |

## Reporting a Vulnerability

Please do **not** open a public GitHub issue for security reports.

Use GitHub's private vulnerability reporting:

1. Navigate to the [Security tab](https://github.com/cipher-rc5/gpt55_chat/security)
   of the repository.
2. Click **Report a vulnerability**.
3. Include reproduction steps, affected version (`gpt55-chat --version` or the
   commit SHA), and any proof-of-concept input.

You should receive an acknowledgement within **5 business days**. We will work
with you on a coordinated-disclosure timeline (default: 90 days from triage to
public advisory, accelerated for actively exploited issues).

## Scope

In scope:

- Logic bugs in the binary's path-handling, sandboxing, or credential
  redaction paths (`src/tools.rs`, `src/image.rs`, `src/svg.rs`,
  `src/types.rs`).
- Vulnerabilities introduced via third-party dependencies that affect this
  crate's runtime behavior. Please file these with the upstream first when
  possible.
- Issues in the release artefacts (signed binaries, SHA256 sums, attestation).

Out of scope:

- Issues that require the attacker to control `OPENAI_TOOLS_READ_ROOT` (the
  sandbox is explicitly opt-in; pointing it at sensitive data is documented as
  unsafe in `README.md`).
- Misuse of an attacker-controlled `.env` file (treated as trusted local
  configuration).
- Behavior of the upstream OpenAI / Azure Responses API itself.

## Disclosure

Once a fix lands on `main`, we will:

1. Publish a tagged release with the patch.
2. Open a GitHub Security Advisory describing the issue and the fix.
3. Credit the reporter unless they request anonymity.

## PGP / Encrypted Mail

We do not currently publish a PGP key. GitHub's private-vulnerability-reporting
channel encrypts reports in transit.
