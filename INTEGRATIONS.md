# Integrations

External software this project talks to. No secret values here — credentials are declared (by name only) in Varlock's `.env.schema` or the platform's credential store.

| System | Purpose | How |
| --- | --- | --- |
| cmux | Workspace titles source; sidebar plugin host | `cmux-client` |
| tmux | Session names are the mapping target | tmux CLI |
| Moshi | Remote client that shows friendly names / dashboard | `MOSHI_CLIENT=1` login-shell hook |
| macOS launchd | Periodic sync | `scripts/com.cmux-moshi.sync.plist` |
