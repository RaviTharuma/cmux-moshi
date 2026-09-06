# Architecture

cmux-moshi is a small Rust CLI. It does not persist a database. Each command
rebuilds a live snapshot from three public sources:

1. `tmux list-sessions` / `tmux list-panes` — session names, attach state, pane pid + command
2. Process environment of the pane pid (and parents) — `CMUX_WORKSPACE_ID`
3. Public `cmux` CLI RPC — `debug.terminals` and `workspace.list` (plus `identify --json` as a reachability fallback)

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
```

Layers:

| Module | Role |
|---|---|
| `host` | PATH, subprocess, process environ (real + fake) |
| `tmux` / `cmux` / `procenv` | Public CLI + env parsers |
| `mapping` | Join session ↔ workspace id ↔ title |
| `sync` | Idempotent `ttys*` rename plan |
| `cleanup` | Kill only idle-shell `ttys*` sessions |
| `dashboard` | Numbered attach menu |
| `shell` | Optional zshrc snippet with backup |
| `doctor` | Degrading diagnostics |
| `cli` | clap dispatch |

The plugin manager clones this repo and runs `[build]` (`bin/cmux-moshi-fetch`)
then `[run]` (`bin/cmux-moshi doctor`). Those `bin/` files are POSIX-sh
launchers for the Rust binary under `.cmux-moshi/bin/`.
