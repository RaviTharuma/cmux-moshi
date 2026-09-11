# AGENTS.md

`cmux-moshi` is a **Moshi host integration** implemented in Rust. The
product is the `cmux-moshi` CLI (`doctor`, `list`, `sync`, `dashboard`,
`cleanup`, `install-shell`, `install-launchagent`). Moshi phones connect over Mosh/SSH and never
see the cmux left sidebar.

Official cmux only distributes git plugins via the mux sidebar plugin
channel. Keep a valid `cmux-plugin.toml` (`kind = "sidebar"`, short name
`moshi`, `[run] = target/release/cmux-moshi-sidebar`,
`[build] = cargo build --release`). `plugin use` is optional packaging;
do not treat the picker as a Moshi panel.

Do not add interpreted sidebars under `~/.config/cmux/sidebars/`, extra
Bonsplit panes, HTML/WebView chrome, or `cmux sidebar select` /
`cmux sidebar open` as the user-facing product.

Runtime source lives under `src/*.rs`. Integration tests live under
`tests/*.rs`. End users install via the cmux plugin manager; contributors
need Rust/Cargo.

### Lint / test / build / run

- The complete Rust verification gate is `./scripts/test.sh`. It runs, in
  order, `cargo fmt --all --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo test --locked`.
- Keep runtime changes in `src/*.rs` and coverage in `src/*` unit tests plus
  `tests/*.rs`. Tests must use temporary directories — never hardcoded home
  paths.
- Talk to cmux only through the public `cmux` CLI (`cmux rpc …`, `cmux --help`)
  or the public `cmux-client` crate on the sidebar socket. Do not link
  private frameworks.
- Sidebar code must not panic when `CMUX_TUI_SOCKET` is unset. Esc clears
  the query and must not exit.

### Host tools on CI / cloud VMs

`cmux`, `tmux`, and `mosh` may be missing. Commands that only parse or plan
must stay unit-testable through `FakeHost`. `doctor` must degrade with clear
warnings instead of crashing.
