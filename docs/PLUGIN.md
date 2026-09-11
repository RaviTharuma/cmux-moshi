# Plugin contract

cmux-moshi is a **Moshi host integration** packaged as an official mux
sidebar plugin. The phone Moshi app never sees the cmux left sidebar. This
document is the packaging contract, not a product claim that Moshi lives in
the sidebar.

Authoritative host spec:
[cmux-tui/spec/plugins.md](https://github.com/manaflow-ai/cmux/blob/main/cmux-tui/spec/plugins.md).

## Why `kind = "sidebar"`

Official cmux only installs git plugins through the mux sidebar plugin
manager. A repo must ship `cmux-plugin.toml` with `kind = "sidebar"`, a
`[build]` command, and a `[run]` executable the manager can verify.

That channel can still install this repo. It does **not** make the left
sidebar the Moshi product. `cmux sidebar plugin use moshi` is optional.

Forbidden in this repo:

- Interpreted custom sidebars (`~/.config/cmux/sidebars/*.js|swift`)
- `cmux sidebar select` / `cmux sidebar open` as a product path
- HTML / WebView chrome
- CmuxExtensionKit Swift App Extensions (out of scope)

## Manifest

```toml
[plugin]
name = "moshi"          # short [a-z0-9-_]+ ; repo name stays cmux-moshi
kind = "sidebar"
version = "0.2.2"
description = "Moshi host CLI plus an optional workspace picker"

[run]
command = ["target/release/cmux-moshi-sidebar"]

[build]
command = ["cargo", "build", "--release"]
```

Install layout (from the spec):

```text
~/.local/share/cmux/mux-plugins/moshi
# or $XDG_DATA_HOME/cmux/mux-plugins/moshi
```

`[build]` is the official `cargo build --release`. `bin/cmux-moshi-fetch` is
a contributor helper only.

## Install / use / reload

```bash
cmux sidebar plugin install https://github.com/RaviTharuma/cmux-moshi.git
cmux sidebar plugin use moshi          # optional picker
cmux server reload-config              # apply to a running session
```

Some cmux builds still document the older `cmux-tui plugin …` alias. Use
whichever your `cmux --help` shows. `plugin use` writes
`sidebar.plugin.command` / `cwd` into `cmux-tui.json` (or legacy `mux.json`).
It does not reload by itself.

`plugin use` is optional. Daily Moshi use is the host CLI / LaunchAgent
`sync`, not the picker.

## Environment (sidebar PTY)

When cmux hosts `[run]`, the child receives:

| Variable | Value |
|---|---|
| `CMUX_TUI_SOCKET` | Server process control socket (JSON-lines) |
| `CMUX_MUX_SOCKET` | Legacy alias for `CMUX_TUI_SOCKET` |
| `CMUX_SIDEBAR` | `1` |
| `TERM` | Same TERM as ordinary PTY surfaces |

Resize is normal PTY / `SIGWINCH`. There is no plugin-specific resize
protocol. Focus uses the same PTY input path as panes. cmux owns the prefix
escape chord; the picker must not exit on `Esc`.

## Sidebar binary behavior

`cmux-moshi-sidebar` is a generic workspace picker:

- Lists workspaces by friendly title via `cmux-client`
- Optional fuzzy filter
- Enter → `select_workspace`
- Esc clears the query; does not exit
- Ctrl-C exits cleanly
- Missing or dropped socket → reconnect UI, never a panic

Standalone:

```bash
CMUX_TUI_SOCKET=/path/to/cmux-tui.sock cargo run --bin cmux-moshi-sidebar
cargo run --bin cmux-moshi-sidebar -- --probe
```

`--probe` prints socket resolution and exits 0 even when the socket is
unset (used by tests).

`cmux-client` calls used: `identify`, `list_workspaces`, `select_workspace`.
The picker does not flatten screens or panes.

## Host CLI (the product)

`target/release/cmux-moshi` (also launched via `bin/cmux-moshi` after a
release build):

| Command | Role |
|---|---|
| `doctor` | PATH / RPC / `MOSHI_CLIENT` diagnostics |
| `list` | Live tmux ↔ workspace id ↔ title map |
| `sync` / `rename` | `ttys*` → sanitized title |
| `dashboard` | Moshi login-shell menu |
| `cleanup` | Idle `ttys*` shells only |
| `install-shell` / `uninstall-shell` | Optional `~/.zshrc` snippet |

Phone clients attach over Mosh/SSH and use this CLI. They do not get the
mux sidebar PTY.
