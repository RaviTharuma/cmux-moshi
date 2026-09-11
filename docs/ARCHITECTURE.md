# Architecture

cmux-moshi is a Moshi **host integration** plus official mux-plugin
packaging. There is no database. The CLI rebuilds a live snapshot; the
optional sidebar picker talks to the mux control socket.

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

Layers:

| Module | Role |
|---|---|
| `host` | PATH, subprocess, process environ (real + fake) |
| `tmux` / `cmux` / `procenv` | Public CLI + env parsers |
| `mapping` | Join session ↔ workspace id ↔ title |
| `sync` | Idempotent `ttys*` rename plan |
| `cleanup` | Kill only idle-shell `ttys*` sessions |
| `dashboard` | Numbered attach menu for Moshi login shells |
| `shell` | Optional zshrc snippet with backup |
| `launchagent` | Optional macOS LaunchAgent install/unload |
| `doctor` | Degrading diagnostics |
| `cli` | clap dispatch for the host CLI |
| `sidebar` | Optional workspace picker (plugin `[run]`) |

The plugin manager clones this repo and runs `[build]`
(`cargo build --release`) then verifies `[run]`
(`target/release/cmux-moshi-sidebar`). `bin/cmux-moshi` prefers that
release CLI. `bin/cmux-moshi-fetch` downloads versioned release assets when
present, otherwise builds from source. It is a contributor helper, not `[build]`.
