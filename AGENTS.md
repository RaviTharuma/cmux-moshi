# AGENTS.md

`cmux-moshi` is an official **cmux plugin for Moshi** implemented in Rust.
The product is the `cmux-moshi` CLI plus an optional login-shell dashboard.
Do not add a custom sidebar under `~/.config/cmux/sidebars/`, extra Bonsplit
panes, HTML/WebView chrome, or `cmux sidebar select` / `cmux sidebar open` as
the user-facing product.

Runtime source lives under `src/*.rs`. Integration tests live under `tests/*.rs`.
End users install via the cmux plugin manager; contributors need Rust/Cargo.

### Lint / test / build / run

- The complete Rust verification gate is `./scripts/test.sh`. It runs, in
  order, `cargo fmt --all --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo test --locked`.
- Keep runtime changes in `src/*.rs` and coverage in `src/*` unit tests plus
  `tests/*.rs`. Tests must use temporary directories — never hardcoded home
  paths.
- Talk to cmux only through the public `cmux` CLI (`cmux rpc …`, `cmux --help`).
  Do not link private frameworks.
- `kind = "sidebar"` in `cmux-plugin.toml` is plugin-manager packaging only.
  `[run]` launches `bin/cmux-moshi doctor`.

### Host tools on CI / cloud VMs

`cmux`, `tmux`, and `mosh` may be missing. Commands that only parse or plan
must stay unit-testable through `FakeHost`. `doctor` must degrade with clear
warnings instead of crashing.
