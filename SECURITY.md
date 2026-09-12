# Security Policy

## Supported versions

Security fixes are applied to the latest release on `main` and, when practical,
to the most recent tagged release. Older tags are generally not backported
unless a critical issue affects a widely used release.

| Version | Supported |
| --- | --- |
| Latest `main` / newest tag | Yes |
| Older tags | Best-effort only |

## Reporting a vulnerability

**Do not open a public GitHub issue for security reports.**

Prefer one of:

1. [GitHub private vulnerability reporting](https://github.com/RaviTharuma/cmux-moshi/security/advisories/new)
   (if enabled on this repository), or
2. A private GitHub message / email to the maintainer
   ([RaviTharuma](https://github.com/RaviTharuma)), with subject
   `cmux-moshi security`.

Please include:

- Description of the issue and impact
- Steps to reproduce or a proof of concept
- Affected version / commit if known
- Whether you are OK being credited

We aim to acknowledge reports within a few business days and to keep you
informed while we investigate.

## Scope notes

cmux-moshi is a local host CLI. Typical concerns include unsafe handling of
tmux/session names, unexpected file writes from shell/LaunchAgent installers,
command injection via untrusted workspace titles, or shipping compromised
release artifacts. Issues in upstream `cmux`, Moshi, or `tmux` themselves
should be reported to those projects.

## Disclosure

Please give us reasonable time to fix and ship a release before public
disclosure. We will credit reporters who wish to be named unless doing so would
increase risk.

## No warranty / no SLA

Security process here is **best-effort only**. Receiving a report, opening an
advisory, or shipping a fix does **not** create a warranty, monitoring duty, or
liability for supply-chain attacks, dependency compromise, CI abuse, credit
overages, or malicious contributions. See [DISCLAIMER.md](DISCLAIMER.md) and
[TERMS.md](TERMS.md).
