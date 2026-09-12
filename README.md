<h1 align="center">cmux-moshi</h1>
<p align="center"><strong>Moshi host integration for cmux</strong></p>
<p align="center">
  Map friendly cmux workspace titles onto tmux session names.
  Login-shell dashboard when <code>MOSHI_CLIENT=1</code>.
  Official mux sidebar plugin packaging; optional workspace picker.
</p>

<p align="center">
  <a href="https://github.com/RaviTharuma/cmux-moshi/actions/workflows/ci.yml"><img src="https://github.com/RaviTharuma/cmux-moshi/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/runtime-Rust-brown.svg" alt="Rust binary" /></a>
</p>

<p align="center">
  English ·
  Kurz auf Deutsch unten ·
  <a href="CHANGELOG.md">changelog</a> ·
  <a href="docs/PLUGIN.md">plugin contract</a> ·
  <a href="CONTRIBUTING.md">contributing</a> ·
  <a href="DISCLAIMER.md">disclaimer</a>
</p>

**cmux-moshi** is the Moshi **host integration** for
[cmux](https://github.com/manaflow-ai/cmux). Moshi is a phone client that
connects over Mosh or SSH. It never sees the cmux left sidebar. What the phone
needs on the Mac is:

1. Friendly cmux workspace titles copied onto tmux session names (`ttys*` → title)
2. A login-shell dashboard when Moshi exports `MOSHI_CLIENT=1`
3. `doctor`, `cleanup`, optional `install-shell`, and optional LaunchAgent helpers

The product is the **`cmux-moshi` CLI** (and an optional LaunchAgent that runs
`sync`). Official cmux only ships git plugins through the
[mux sidebar plugin](https://github.com/manaflow-ai/cmux/blob/main/cmux-tui/spec/plugins.md)
channel, so this repo also carries a valid `cmux-plugin.toml`. That packaging
can install the repo; `plugin use` is optional and hosts a small generic
workspace picker. It is not a Moshi panel.

Current source version: **v0.3.0**.

> **Deutsch (kurz):** Moshi-Host-Integration, keine Sidebar-App. Die Phone-App
> sieht die cmux-Linke-Sidebar nie. Produkt ist die CLI (`sync`, Dashboard,
> doctor). `cmux sidebar plugin install` ist nur der offizielle
> Verteilkanal; `plugin use` ist optional (Workspace-Picker in cmux).

## Install (host CLI)

cmux clones this repo, runs `cargo build --release`, and verifies the
sidebar `[run]` binary. After that, the host CLI is at
`target/release/cmux-moshi` inside the plugin directory (or on PATH if you
copy it).

```bash
cmux sidebar plugin install https://github.com/RaviTharuma/cmux-moshi.git
# Some cmux builds also accept the older alias:
#   cmux-tui plugin install https://github.com/RaviTharuma/cmux-moshi.git
```

That clones into `$XDG_DATA_HOME/cmux/mux-plugins/moshi` (or
`~/.local/share/cmux/mux-plugins/moshi`). Plugin name is `moshi`.

Then use the CLI:

```bash
cmux-moshi doctor
cmux-moshi list
cmux-moshi sync
```

On the iPhone / iPad, open Moshi → **Settings** and enable the
**MOSHI_CLIENT** env toggle (exports `MOSHI_CLIENT=1` on connect). Then either:

```bash
cmux-moshi install-shell    # optional: exec dashboard from ~/.zshrc
# or, once connected:
cmux-moshi dashboard
```

`install-shell` backs up the rc file first. Remove the snippet with
`cmux-moshi uninstall-shell`.

Periodic rename without a login snippet (macOS):

```bash
cmux-moshi install-launchagent     # write + launchctl load
cmux-moshi uninstall-launchagent   # unload + remove plist
```

This installs [`scripts/com.cmux-moshi.sync.plist`](scripts/com.cmux-moshi.sync.plist)
into `$HOME/Library/LaunchAgents/` and loads it. It runs `cmux-moshi sync`
every five minutes. On non-macOS hosts the commands skip with a clear message.
Manual `cp` + `launchctl load` still works if you prefer.

Contributors can also build locally:

```bash
cargo build --release
./target/release/cmux-moshi doctor
```

`bin/cmux-moshi-fetch` is a contributor helper. On tagged releases it downloads
a per-OS/arch binary and verifies `SHA256SUMS`; otherwise it falls back to a
source build. It is **not** the official `[build]` command.

## Optional: in-cmux workspace picker

`plugin use` is **optional**. It does not put Moshi in the sidebar. It hosts
a small generic picker that lists workspaces by friendly title and selects
one through `cmux-client`.

```bash
cmux sidebar plugin use moshi
cmux server reload-config
```

(`cmux-tui plugin use moshi` if your build still uses that alias.)

Return to the built-in sidebar with `cmux sidebar plugin use --builtin`.

Standalone development (reconnect UI if the socket is missing; never panics):

```bash
CMUX_TUI_SOCKET=/path/to/cmux-tui.sock cargo run --bin cmux-moshi-sidebar
# or
CMUX_TUI_SOCKET=/path/to/cmux-tui.sock cargo run --bin cmux-moshi-sidebar -- --probe
```

Keys: type to filter, Enter selects the workspace, Esc clears the query
(does **not** exit — cmux owns prefix-escape), Ctrl-C exits.

Contract details: [docs/PLUGIN.md](docs/PLUGIN.md) and
[cmux-tui/spec/plugins.md](https://github.com/manaflow-ai/cmux/blob/main/cmux-tui/spec/plugins.md).

This repo does **not** install interpreted sidebars under
`~/.config/cmux/sidebars/`, HTML/WebView chrome, or a
`cmux sidebar select` / `cmux sidebar open` product path.

## Commands

| Command | What it does |
|---|---|
| `doctor` | Check `cmux` on PATH, `tmux`, optional `mosh`, `cmux rpc` reachability, `MOSHI_CLIENT` tips |
| `list` | Live map: tmux session ↔ `CMUX_WORKSPACE_ID` ↔ friendly title (`--json`) |
| `sync` / `rename` | Rename only sessions still named `ttys*` to the cmux title (idempotent; `--force`, `--dry-run`) |
| `dashboard` | Numbered menu for Moshi login shells (`MOSHI_CLIENT=1` or `--force`): attach, refresh, sync titles, cleanup, bare shell, quit |
| `cleanup` | Kill detached idle-shell `ttys*` sessions; keep attached / `claude`/`node`/`python`/… |
| `install-shell` | Insert a marked snippet into `~/.zshrc` (or `--rc-file`); backup first |
| `uninstall-shell` | Remove the marked snippet |
| `install-launchagent` | macOS: install + `launchctl load` periodic sync (`--agents-dir`) |
| `uninstall-launchagent` | macOS: unload + remove the LaunchAgent plist |
| `--version` / `--help` | Version and command list |

```bash
cmux-moshi --help
cmux-moshi list --json
cmux-moshi sync --dry-run
cmux-moshi cleanup --dry-run
cmux-moshi dashboard --force
```

## How it works

Every CLI command rebuilds the map live. No cache.

1. `tmux list-sessions` and `tmux list-panes` for names, attach state, active pane pid/command
2. Process environment of the active pane (and parents) for `CMUX_WORKSPACE_ID`
3. Public `cmux` CLI: `cmux rpc debug.terminals`, `cmux rpc workspace.list` (doctor also tries `cmux identify --json`)

`sync` only touches default `ttys*` names unless you pass `--force`.
`cleanup` only kills detached `ttys*` sessions whose pane and children are idle shells.
The dashboard can run sync (`y`) and cleanup (`c`) without leaving the menu.

The optional picker talks to the mux control socket (`CMUX_TUI_SOCKET`,
legacy `CMUX_MUX_SOCKET`) through the `cmux-client` crate: `identify`,
`list_workspaces`, `select_workspace`.

## Requirements

- [cmux](https://cmux.com) with `cmux sidebar plugin` on PATH (or the
  `cmux-tui plugin` alias on older builds)
- `tmux`
- `mosh` / `mosh-server` optional (Moshi can use SSH)
- [Moshi](https://getmoshi.app) with the MOSHI_CLIENT env toggle enabled for the dashboard
- Contributors building from source: Rust 1.88+ / Cargo

## Development

```bash
./scripts/test.sh          # cargo fmt --check, clippy -D warnings, cargo test --locked
cargo build --release
./target/release/cmux-moshi --version
./target/release/cmux-moshi-sidebar --probe
```

Layout: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
Plugin contract: [docs/PLUGIN.md](docs/PLUGIN.md).
Agent notes: [AGENTS.md](AGENTS.md).

## Community & policies

| Doc | Purpose |
| --- | --- |
| [DISCLAIMER.md](DISCLAIMER.md) | Unofficial status, trademarks, no warranty |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Dev setup, PR process, coding guidelines |
| [docs/ISSUE_REPORTING.md](docs/ISSUE_REPORTING.md) | What to include in bug/feature reports |
| [SUPPORT.md](SUPPORT.md) | Where to get help |
| [SECURITY.md](SECURITY.md) | Private vulnerability reporting |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Contributor Covenant |

Use the GitHub **Bug report** / **Feature request** forms when opening issues.

## License

[MIT](LICENSE) © 2026 Ravi Tharuma. See also [DISCLAIMER.md](DISCLAIMER.md).
