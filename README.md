<h1 align="center">cmux-moshi</h1>
<p align="center"><strong>Official cmux plugin for Moshi</strong></p>
<p align="center">
  Friendly cmux workspace titles on tmux sessions over Mosh/SSH.
  CLI + optional login-shell dashboard. No custom sidebar.
</p>

<p align="center">
  <a href="https://github.com/RaviTharuma/cmux-moshi/actions/workflows/ci.yml"><img src="https://github.com/RaviTharuma/cmux-moshi/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/runtime-Rust-brown.svg" alt="Rust binary" /></a>
  <a href="https://github.com/topics/plugin"><img src="https://img.shields.io/badge/kind-cmux%20plugin-4c71f2.svg" alt="cmux plugin" /></a>
</p>

<p align="center">
  English ·
  Kurz auf Deutsch unten ·
  <a href="CHANGELOG.md">changelog</a>
</p>

**cmux-moshi** is the official [cmux](https://github.com/manaflow-ai/cmux) plugin
for [Moshi](https://getmoshi.app) clients. Moshi connects over Mosh or SSH.
cmux keeps friendly workspace titles in its own UI and does not copy those
titles onto tmux session names, so remote clients only see `ttys001`,
`ttys002`, … This plugin reconstructs the live mapping and optionally renames
those default sessions.

The product is the `cmux-moshi` CLI and an optional login-shell dashboard.
It does **not** install a custom sidebar into `~/.config/cmux/sidebars/`, add
extra Bonsplit panes, or replace `cmux sidebar select` / `cmux sidebar open`.
`kind = "sidebar"` in `cmux-plugin.toml` is plugin-manager packaging only.

Current source version: **v0.1.0**.

> **Deutsch (kurz):** Offizielles cmux-Plugin für Moshi. Installation über den
> cmux Plugin-Manager. Produkt ist die CLI plus optionales Login-Dashboard —
> keine eigene Sidebar.

## Install

```bash
cmux sidebar plugin install https://github.com/RaviTharuma/cmux-moshi.git
cmux sidebar plugin use cmux-moshi
cmux sidebar plugin update cmux-moshi
cmux sidebar plugin remove cmux-moshi
```

That clones into `$XDG_DATA_HOME/cmux/mux-plugins/cmux-moshi` (or
`~/.local/share/cmux/mux-plugins/cmux-moshi`). The plugin-manager build step
runs `bin/cmux-moshi-fetch`. For v0.1.0, release binaries may not exist yet;
the fetch script then builds with `cargo build --release --locked` when Cargo
is available. `bin/cmux-moshi` is a thin POSIX-sh launcher for the installed
binary.

### After install

```bash
cmux-moshi doctor
cmux-moshi list
cmux-moshi sync
```

On the iPhone / iPad, open Moshi → **Settings → Integrations → Export ENV**
(this sets `MOSHI_CLIENT=1` on connect). Then either:

```bash
cmux-moshi install-shell    # optional: exec dashboard from ~/.zshrc
# or, once connected:
cmux-moshi dashboard
```

`install-shell` backs up the rc file first. Remove the snippet with
`cmux-moshi uninstall-shell`.

Periodic rename without a login snippet: copy
[`scripts/com.cmux-moshi.sync.plist`](scripts/com.cmux-moshi.sync.plist) to
`~/Library/LaunchAgents/` and `launchctl load` it (macOS). It runs
`cmux-moshi sync` every five minutes.

## Commands

| Command | What it does |
|---|---|
| `doctor` | Check `cmux` on PATH, `tmux`, optional `mosh`, `cmux rpc` reachability, `MOSHI_CLIENT` tips |
| `list` | Live map: tmux session ↔ `CMUX_WORKSPACE_ID` ↔ friendly title (`--json`) |
| `sync` / `rename` | Rename only sessions still named `ttys*` to the cmux title (idempotent; `--force`, `--dry-run`) |
| `dashboard` | Numbered menu for Moshi login shells (`MOSHI_CLIENT=1` or `--force`): attach, refresh, cleanup, bare shell, quit |
| `cleanup` | Kill `ttys*` sessions whose pane tree is only an idle shell (`zsh`/`bash`/`sh`); keep `claude`/`node`/`python`/… |
| `install-shell` | Insert a marked snippet into `~/.zshrc` (or `--rc-file`); backup first |
| `uninstall-shell` | Remove the marked snippet |
| `--version` / `--help` | Version and command list |

```bash
cmux-moshi --help
cmux-moshi list --json
cmux-moshi sync --dry-run
cmux-moshi cleanup --dry-run
cmux-moshi dashboard --force
```

## How it works

Every command rebuilds the map live. No cache.

1. `tmux list-sessions` and `tmux list-panes` for names, attach state, pane pid, and pane command
2. Process environment of the pane (and parents) for `CMUX_WORKSPACE_ID`
3. Public `cmux` CLI: `cmux rpc debug.terminals`, `cmux rpc workspace.list` (doctor also tries `cmux identify --json`)

`sync` only touches default `ttys*` names unless you pass `--force`.
`cleanup` only kills `ttys*` sessions whose pane and children are idle shells.

## Requirements

- [cmux](https://cmux.com) with `cmux sidebar plugin` on PATH
- `tmux`
- `mosh` / `mosh-server` optional (Moshi can use SSH)
- [Moshi](https://getmoshi.app) with Export ENV enabled for the dashboard
- Contributors building from source: Rust 1.85+ / Cargo

v0.1.0 documents `cargo build --release` as the fetch fallback until GitHub
Release assets (`cmux-moshi-<version>-<target>` + `SHA256SUMS`) are published.

## Development

```bash
./scripts/test.sh          # cargo fmt --check, clippy -D warnings, cargo test --locked
./bin/cmux-moshi --version # after fetch/build
```

Layout: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Agent notes: [AGENTS.md](AGENTS.md).

## License

[MIT](LICENSE) © 2026 Ravi Tharuma.
