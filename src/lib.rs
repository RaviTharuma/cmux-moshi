//! Official cmux plugin for Moshi clients.
//!
//! Reconstructs the live mapping from tmux session names to cmux workspace ids
//! and friendly titles, then optionally renames default `ttys*` sessions and
//! offers a login-shell dashboard for Moshi.

pub mod cleanup;
pub mod cli;
pub mod cmux;
pub mod dashboard;
pub mod doctor;
pub mod error;
pub mod host;
pub mod mapping;
pub mod procenv;
pub mod shell;
pub mod sync;
pub mod titles;
pub mod tmux;

pub use error::Error;
