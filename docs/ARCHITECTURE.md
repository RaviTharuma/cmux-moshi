# Architecture

cmux-moshi is a Moshi **host integration** plus optional official mux-plugin
packaging. There is no database. The CLI rebuilds a live snapshot; the
optional sidebar picker talks to the mux control socket.

## Product boundaries

| Surface | Role |
| --- | --- |
| **`cmux-moshi` CLI** | The product on the Mac: `doctor`, `list`, `sync`/`rename`, `dashboard`, `cleanup`, `install-shell`, `uninstall-shell`, `install-launchagent`, `uninstall-launchagent`. |
| **Moshi phone** | Connects over **Mosh/SSH**. Never sees the cmux left sidebar. |
| **cmux left sidebar** | Workspaces + machines navigation only. Not a Moshi panel. |
| **Optional `cmux-moshi-sidebar`** | Packaging `[run]` + optional in-cmux workspace picker via `plugin use`. Not the Moshi phone UX. |
| **Right sidebar** | Exploration / accessory only. This repo does not ship invented right-panel features. |

Session multiplexing for this product is **tmux**. Zellij is out of scope.

```text
Moshi iOS  --Mosh/SSH-->  login shell
                              |
                              +--> MOSHI_CLIENT=1  -->  cmux-moshi dashboard
                              |
cmux.app  (friendly titles live here)
   |
   +-- tmux sessions still named ttysNNN
   |
   +-- cmux-moshi sync  -->  rename only ttys* to the title

Official git install (packaging only)
   cmux sidebar plugin install
        |
        +-- cargo build --release
        +-- verify target/release/cmux-moshi-sidebar
        +-- optional: plugin use moshi  -->  workspace picker PTY
            (Moshi phones never see this)
```

## Layers

| Module | Role |
|---|---|
| `host` | PATH, subprocess, process environ (real + `FakeHost` for tests) |
| `tmux` / `cmux` / `procenv` | Public CLI + env parsers |
| `mapping` | Join session ↔ workspace id ↔ title |
| `sync` | Idempotent `ttys*` rename plan |
| `cleanup` | Kill only idle-shell `ttys*` sessions |
| `dashboard` | Numbered attach menu for Moshi login shells |
| `shell` | Optional zshrc snippet with backup |
| `launchagent` | Optional macOS LaunchAgent install/unload |
| `doctor` | Degrading diagnostics (`cmux`/`tmux`/`mosh` may be missing) |
| `cli` | clap dispatch for the host CLI |
| `sidebar` | Optional workspace picker (plugin `[run]`) |

Every CLI command rebuilds the map live (no cache):

1. `tmux list-sessions` / `tmux list-panes` for names, attach state, pane pid/command
2. Process environment of the active pane (and parents) for `CMUX_WORKSPACE_ID`
3. Public `cmux` CLI: `cmux rpc debug.terminals`, `cmux rpc workspace.list` (doctor also tries `cmux identify --json`)

`sync` only touches default `ttys*` names unless `--force`. `cleanup` only
kills detached `ttys*` sessions whose pane and children are idle shells.

## Packaging

The plugin manager clones this repo and runs `[build]`
(`cargo build --release`) then verifies `[run]`
(`target/release/cmux-moshi-sidebar`). `bin/cmux-moshi` prefers that
release CLI. `bin/cmux-moshi-fetch` downloads versioned GitHub Release assets
when present, otherwise builds from source. It is a contributor helper, not
`[build]`.

Talk to cmux only through the public `cmux` CLI or the public `cmux-client`
crate. Do not link private frameworks. Sidebar code must not panic when
`CMUX_TUI_SOCKET` is unset; Esc clears the query and must not exit.

## Related

- Stack summary: [STACK.md](../STACK.md)
- Plugin contract: [PLUGIN.md](PLUGIN.md)
- Agent gate: [AGENTS.md](../AGENTS.md), `./scripts/test.sh`
