# Changelog

## 0.1.0 — 2026-09-06

First public release of **cmux-moshi**, the official cmux plugin for Moshi
clients.

- `doctor` checks `cmux`, `tmux`, optional `mosh`, public `cmux rpc` reachability, and `MOSHI_CLIENT`
- `list` prints the live tmux session ↔ `CMUX_WORKSPACE_ID` ↔ friendly title map
- `sync` / `rename` rename only default `ttys*` sessions to the cmux title (idempotent; `--force` required for already-named sessions)
- `dashboard` interactive menu for Moshi login shells (`MOSHI_CLIENT=1` or `--force`)
- `cleanup` kills only idle-shell `ttys*` sessions
- `install-shell` / `uninstall-shell` manage an optional `~/.zshrc` snippet (backup first)
- Official install path: `cmux sidebar plugin install https://github.com/RaviTharuma/cmux-moshi.git`
- v0.1 fetch script falls back to `cargo build --release` until release assets are published
