//! Moshi host integration for cmux.
//!
//! The product is the `cmux-moshi` CLI: map friendly cmux workspace titles
//! onto tmux session names, plus a login-shell dashboard when
//! `MOSHI_CLIENT=1`. The optional `cmux-moshi-sidebar` binary exists so the
//! official mux sidebar plugin packaging channel has a real `[run]`
//! executable. Moshi phones never see that PTY.

pub mod cleanup;
pub mod cli;
pub mod cmux;
pub mod dashboard;
pub mod doctor;
pub mod error;
pub mod host;
pub mod launchagent;
pub mod mapping;
pub mod procenv;
pub mod shell;
pub mod sidebar;
pub mod sync;
pub mod titles;
pub mod tmux;

pub use error::Error;
