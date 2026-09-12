# Contributing to cmux-moshi

Thanks for helping. This document covers how to propose changes, report issues,
and get a patch landed.

## Before you start

1. Read the [README](README.md) product summary and [DISCLAIMER.md](DISCLAIMER.md).
2. Skim [AGENTS.md](AGENTS.md) for hard product boundaries (CLI-first Moshi host
   integration; no left-sidebar Moshi product; talk to cmux only through the
   public CLI / `cmux-client`).
3. Search [existing issues](https://github.com/RaviTharuma/cmux-moshi/issues) and
   PRs so we avoid duplicates.

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

## Coding guidelines

- Runtime changes live in `src/*.rs`; coverage in unit tests and `tests/*.rs`.
- Tests must use temporary directories — never hardcoded home paths.
- Do not link private cmux frameworks; use public `cmux` CLI or `cmux-client`.
- Sidebar code must not panic when `CMUX_TUI_SOCKET` is unset. Esc clears the
  query and must not exit.
- Keep `cmux-plugin.toml` valid (`kind = "sidebar"`, short name `moshi`).
- Prefer small, focused PRs over mixed refactors + features.

## Pull requests

1. Branch from `main`.
2. Make the change; update docs/`CHANGELOG.md` when user-facing behavior changes.
3. Run `./scripts/test.sh` locally before pushing.
4. Open a PR using the repository template. Fill in **what** / **why** / **how tested**.
5. Link related issues (`Fixes #N` / `Refs #N`).
6. Keep the PR description accurate if you push follow-up commits.

Maintainers may squash-merge. Do not force-push shared `main`.

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
[MIT License](LICENSE) that covers this project.
