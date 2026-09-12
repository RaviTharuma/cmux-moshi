# Contributing to cmux-moshi

Thanks for helping. This document covers how to propose changes, report issues,
and get a patch landed.

**Important:** By contributing (issues, PRs, patches, reviews, or comments that
include code), you accept [NOTICE.md](NOTICE.md), [DISCLAIMER.md](DISCLAIMER.md),
[TERMS.md](TERMS.md), and the contribution warranties below. Maintainers do
**not** accept liability for supply-chain issues, credit/spend overages, or
malicious/defective code — including code that is accidentally merged. This is
free hobby software; contributing does not create a paid relationship.

## Before you start

1. Read the [README](README.md), [NOTICE.md](NOTICE.md),
   [DISCLAIMER.md](DISCLAIMER.md), and [TERMS.md](TERMS.md).
2. Skim [AGENTS.md](AGENTS.md) for hard product boundaries (CLI-first Moshi host
   integration; no left-sidebar Moshi product; talk to cmux only through the
   public CLI / `cmux-client`).
3. Search [existing issues](https://github.com/RaviTharuma/cmux-moshi/issues) and
   PRs so we avoid duplicates.

## Contribution warranties (binding)

You represent and warrant that:

- You have the legal right to submit the contribution under the [MIT License](LICENSE).
- Your contribution is **not** knowingly malicious, backdoored, credential-stealing,
  or designed to abuse CI/cloud/credits/quotas.
- You have taken reasonable care that secrets, private keys, and unrelated personal
  data are not included.
- You understand maintainers may merge without a full security audit, and that
  **merge ≠ endorsement or warranty**.

You agree to **indemnify and hold harmless** the authors, copyright holders, and
maintainers from claims and costs arising from your contribution, to the maximum
extent permitted by law. See [TERMS.md](TERMS.md) §6.

Malicious or abusive submissions may be reverted, reported, and result in bans
under [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Ways to contribute

| Kind | Prefer |
| --- | --- |
| Bug fix | Issue first (template), then PR linking the issue |
| Feature / UX | Issue discussion before large code |
| Docs / typos | PR is fine without an issue |
| Security | Private report per [SECURITY.md](SECURITY.md) — not a public issue |

## Development setup

Requirements: Rust **1.88+**, Cargo, and for integration smoke: `cmux`, `tmux`
(optional `mosh`). CI and cloud VMs may lack host tools; keep parse/plan logic
unit-testable via `FakeHost`, and keep `doctor` degrading with warnings.

```bash
git clone https://github.com/RaviTharuma/cmux-moshi.git
cd cmux-moshi
cargo build
./scripts/test.sh
```

`./scripts/test.sh` is the full gate:

1. `cargo fmt --all --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --locked`

**Billing:** any CI minutes, cloud, or agent credits spent while you develop or
test are **your** responsibility ([TERMS.md](TERMS.md) §4).

## Coding guidelines

- Runtime changes live in `src/*.rs`; coverage in unit tests and `tests/*.rs`.
- Tests must use temporary directories — never hardcoded home paths.
- Do not link private cmux frameworks; use public `cmux` CLI or `cmux-client`.
- Sidebar code must not panic when `CMUX_TUI_SOCKET` is unset. Esc clears the
  query and must not exit.
- Keep `cmux-plugin.toml` valid (`kind = "sidebar"`, short name `moshi`).
- Prefer small, focused PRs over mixed refactors + features.
- Do not add dependency or CI changes that intentionally burn quotas or phone
  home without a clear, documented reason in the PR.

## Pull requests

1. Branch from `main`.
2. Make the change; update docs/`CHANGELOG.md` when user-facing behavior changes.
3. Run `./scripts/test.sh` locally before pushing.
4. Open a PR using the repository template. Fill in **what** / **why** / **how tested**.
5. Check the contribution-warranty boxes on the PR template.
6. Link related issues (`Fixes #N` / `Refs #N`).
7. Keep the PR description accurate if you push follow-up commits.

Maintainers may squash-merge, revert, or ignore PRs without explanation.
**Acceptance of a PR does not create maintainer liability** for defects,
malware, or downstream loss. Do not force-push shared `main`.

## Issue reporting

Use the GitHub issue forms:

- **Bug report** — for incorrect behavior, crashes, or install failures
- **Feature request** — for new commands or host UX

See [docs/ISSUE_REPORTING.md](docs/ISSUE_REPORTING.md) for what to include when
the forms do not apply (e.g. discussion).

## Conduct

Participation is governed by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## License

By contributing, you agree that your contributions are licensed under the same
[MIT License](LICENSE) that covers this project, and that [NOTICE.md](NOTICE.md),
[DISCLAIMER.md](DISCLAIMER.md), and [TERMS.md](TERMS.md) apply.
