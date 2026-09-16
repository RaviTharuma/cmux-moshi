# Stack

Technology and runtime surface for **cmux-moshi** (v0.3.0+). This is a
Rust host CLI, not a Zellij integration and not a left-sidebar Moshi UI.

## Core

| Piece | Role |
| --- | --- |
| **Rust / Cargo** | Language and build. Edition 2021, `rust-version = "1.88"`. Bins: `cmux-moshi` (product CLI), `cmux-moshi-sidebar` (optional plugin `[run]`). |
| **clap** | Host CLI parsing (`doctor`, `list`, `sync`/`rename`, `dashboard`, `cleanup`, `install-shell`, `uninstall-shell`, `install-launchagent`, `uninstall-launchagent`). |
| **cmux public CLI** | Live workspace/terminal data via `cmux rpc …` (and doctor’s `cmux identify --json`). No private frameworks. |
| **cmux-client** | Optional workspace picker talks to the mux control socket (`CMUX_TUI_SOCKET` / legacy `CMUX_MUX_SOCKET`). |
| **tmux** | Session list, pane state, rename, attach, idle-shell cleanup. Session multiplexer for this product is **tmux only** — not Zellij. |
| **Mosh / SSH** | Moshi phone path to the Mac login shell. Phones never see the cmux left sidebar. |
| **MOSHI_CLIENT=1** | Moshi Settings env toggle; triggers the login-shell dashboard path. |

## Host helpers (optional)

| Piece | Role |
| --- | --- |
| **zshrc snippet** | `install-shell` / `uninstall-shell` — exec dashboard when `MOSHI_CLIENT=1` (backup first). |
| **macOS LaunchAgent** | `install-launchagent` / `uninstall-launchagent` — periodic `cmux-moshi sync` via `scripts/com.cmux-moshi.sync.plist`. Non-macOS hosts skip with a clear message. |

## Packaging & CI

| Piece | Role |
| --- | --- |
| **`cmux-plugin.toml`** | Official mux sidebar plugin channel (`kind = "sidebar"`, name `moshi`). `[build] = cargo build --release`, `[run] = target/release/cmux-moshi-sidebar`. Installs the repo; `plugin use` is optional packaging for a generic picker — **not** a Moshi panel. |
| **GitHub Actions CI** | `.github/workflows/ci.yml` — fmt, clippy, tests (see `./scripts/test.sh`). |
| **GitHub Actions release** | `.github/workflows/release.yml` — tagged `v*` builds of `cmux-moshi` for macOS/Linux (aarch64/x86_64) plus `SHA256SUMS`. Used by contributor helper `bin/cmux-moshi-fetch`; official plugin install still builds from source. |

## UI crates (picker only)

`ratatui` + `crossterm` power the optional `cmux-moshi-sidebar` PTY picker.
That binary exists so the official plugin manager has a real `[run]` target.
It is not the Moshi phone experience and must not replace left-sidebar
workspaces/machines navigation.

## Out of stack / out of product

- Zellij (or any non-tmux multiplexer) as the Moshi host path
- Interpreted left sidebars under `~/.config/cmux/sidebars/` as the product
- HTML/WebView chrome, Bonsplit panes, or `cmux sidebar select` /
  `cmux sidebar open` as the user-facing Moshi path
- Invented shipped right-panel features (right sidebar remains exploration only)

## Related docs

- Product shape: [README.md](README.md), [AGENTS.md](AGENTS.md)
- Layers: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Plugin contract: [docs/PLUGIN.md](docs/PLUGIN.md)
- Liability: [NOTICE.md](NOTICE.md), [DISCLAIMER.md](DISCLAIMER.md), [TERMS.md](TERMS.md)
