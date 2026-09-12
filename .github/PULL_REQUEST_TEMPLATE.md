## Summary

<!-- What does this PR change, and why? -->

## Type of change

- [ ] Bug fix
- [ ] Feature
- [ ] Docs / community health
- [ ] Refactor / chore
- [ ] CI / release

## Contributor warranties (required)

By opening this PR I confirm:

- [ ] I accept [NOTICE.md](../NOTICE.md), [DISCLAIMER.md](../DISCLAIMER.md), and [TERMS.md](../TERMS.md)
- [ ] I have the right to submit this under the MIT License
- [ ] This change is **not** knowingly malicious, backdoored, or designed to steal secrets / burn credits / abuse CI
- [ ] I understand maintainers may merge **without** a full security audit, and merge does **not** make them liable for defects or supply-chain harm
- [ ] I agree to the indemnification terms in [TERMS.md](../TERMS.md) §6 and [CONTRIBUTING.md](../CONTRIBUTING.md)

## Checklist

- [ ] I read [CONTRIBUTING.md](../CONTRIBUTING.md)
- [ ] `./scripts/test.sh` passes locally (or CI is green)
- [ ] User-facing changes are reflected in `CHANGELOG.md` / README when needed
- [ ] Tests use temporary directories (no hardcoded home paths)
- [ ] No private cmux frameworks; public `cmux` CLI / `cmux-client` only
- [ ] Product boundaries respected (CLI-first; not a left-sidebar Moshi panel)
- [ ] No unexplained new network calls, dependency churn, or CI matrix explosions

## How tested

<!-- Commands, platforms, FakeHost/unit vs live cmux/tmux -->

## Related issues

<!-- Fixes #N / Refs #N -->
