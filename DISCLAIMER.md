# Disclaimer

**cmux-moshi** is an independent, community-maintained host integration. It is
**not** an official product of, and is **not** affiliated with, endorsed by, or
sponsored by:

- [cmux](https://cmux.com) / [manaflow-ai/cmux](https://github.com/manaflow-ai/cmux)
- [Moshi](https://getmoshi.app) or its publishers
- Apple, or any other third-party whose trademarks appear in documentation

“cmux”, “Moshi”, and other product names are trademarks of their respective
owners. They are used here only to describe compatibility.

## No warranty

This software is provided **as is**, without warranty of any kind. See the
[MIT License](LICENSE) for the full legal text. Among other things, that means:

- We do not guarantee compatibility with every cmux, Moshi, tmux, or OS version.
- Host helpers (`sync`, `cleanup`, `install-shell`, LaunchAgent installers) can
  rename tmux sessions, kill idle sessions, and edit shell rc / LaunchAgent
  files. Review commands and prefer `--dry-run` where available before applying
  changes on machines you care about.
- You are responsible for backups, access control, and validating behavior in
  your environment.

## Security and privacy

cmux-moshi talks to local host tools (`tmux`, the public `cmux` CLI / socket) and
optionally installs shell or LaunchAgent hooks. It is not a network service and
does not claim to harden your SSH/Mosh setup. Report suspected vulnerabilities
via [SECURITY.md](SECURITY.md).

## Scope

Phones connecting over Mosh/SSH never see the cmux left sidebar. The product of
this repo is the **host CLI** (and optional packaging). Do not treat optional
`plugin use` workspace pickers as a Moshi panel. See
[AGENTS.md](AGENTS.md) and the README for product boundaries.
