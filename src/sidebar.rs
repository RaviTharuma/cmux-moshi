//! Optional in-cmux workspace picker hosted as a mux sidebar plugin.
//!
//! Official cmux only distributes git plugins through the mux sidebar plugin
//! channel (`cmux-plugin.toml`, `kind = "sidebar"`). This module is the
//! `[run]` binary that channel verifies. It is **not** the Moshi product:
//! the Moshi phone app never sees this PTY. The host integration is the
//! `cmux-moshi` CLI (`doctor`, `list`, `sync`, `dashboard`, `cleanup`,
//! `install-shell`).

pub mod filter;
pub mod keys;
pub mod picker;
pub mod socket;
pub mod ui;
pub mod workspaces;

pub use filter::{fuzzy_match, MatchResult};
pub use keys::{interpret_key, KeyAction};
pub use picker::{Picker, PickerView, Status};
pub use socket::{missing_socket_message, probe_report, socket_from_env};
pub use workspaces::{rows_from_tree, WorkspaceRow};
