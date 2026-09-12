# Issue reporting guidelines

Use these guidelines whenever you open an issue. Prefer the GitHub **issue
forms** (Bug report / Feature request) so required fields are not skipped.

## Before filing

1. Update to the latest release or `main` and retry.
2. Run `cmux-moshi doctor` and keep the output (redact secrets / hostnames if needed).
3. Search [open and closed issues](https://github.com/RaviTharuma/cmux-moshi/issues)
   for duplicates.
4. Confirm the problem is in **cmux-moshi**, not upstream cmux / Moshi / tmux /
   mosh / SSH. Upstream bugs belong in those projects; you can still open a
   tracking issue here that links upstream if integration is affected.
5. Read [DISCLAIMER.md](../DISCLAIMER.md) so expectations around warranty and
   affiliation are clear.

## Bug reports

Include:

- **cmux-moshi version** (`cmux-moshi --version`) and install method (plugin
  install, local `cargo build`, release binary)
- **OS** and versions of `cmux`, `tmux`, and optionally `mosh`
- **Exact commands** you ran and the full relevant terminal output
- **Expected vs actual** behavior
- Whether `MOSHI_CLIENT` is set (dashboard / login-shell issues)
- For sync/cleanup: session names before/after (anonymize if needed) and whether
  you used `--dry-run` / `--force`

Do **not** paste private keys, tokens, full `~/.ssh` configs, or unrelated
personal file contents.

## Feature requests

Describe:

- The **problem** you are solving (not only the preferred API)
- Who benefits (Moshi phone user, Mac host, contributor)
- Whether it stays within product boundaries (host CLI / optional packaging;
  not a left-sidebar Moshi panel — see [AGENTS.md](../AGENTS.md))
- Any alternatives you considered

## Security

Do not file public issues for vulnerabilities. Follow [SECURITY.md](../SECURITY.md).

## After filing

- Stay responsive to maintainer questions
- Update the issue if you find a workaround or confirm a fix on `main`
- Close the issue yourself if you discover it was user error or upstream-only
  (a short note helps the next person)
