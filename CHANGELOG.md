# Changelog

## 0.2.1 — 2026-09-11

Reliability and Moshi login UX fixes from upstream cmux / Moshi docs review.

- Parse `debug.terminals` without treating bare `id` as a workspace id; fall
  back to `last_known_workspace_id` when `workspace_id` is absent
- Prefer the active tmux pane (`#{pane_active}`) for workspace env / command
- `cleanup` never kills attached `ttys*` sessions
- `install-shell` snippet matches dashboard/`is_truthy` (`1|true|yes|on`)
- Dashboard: `y` / `sync` renames `ttys*` titles in-place
- Doctor tips use Moshi’s MOSHI_CLIENT env toggle wording

## 0.2.0 — 2026-09-06

Moshi host integration aligned with the official mux sidebar plugin
contract.

- Primary product remains the `cmux-moshi` CLI: doctor, list, sync/rename,
  dashboard, cleanup, install-shell
- `cmux-plugin.toml` matches the official shape: name `moshi`,
  `[run] = target/release/cmux-moshi-sidebar`,
  `[build] = cargo build --release`
- New optional `cmux-moshi-sidebar` workspace picker (cmux-client;
  Esc clears query; Ctrl-C exits; reconnect UI if the socket is missing)
- Docs lead with the host CLI / phone path; `plugin use` is optional
  packaging, not a Moshi sidebar
- `bin/cmux-moshi-fetch` is a contributor helper only

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
