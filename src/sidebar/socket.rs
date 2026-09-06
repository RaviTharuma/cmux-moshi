//! Control-socket discovery for the optional workspace picker.
//!
//! cmux sets `CMUX_TUI_SOCKET` (legacy `CMUX_MUX_SOCKET`) when it hosts a
//! sidebar plugin PTY. Standalone development can set the same variable.
//! Missing the socket is a reconnect state, never a panic: Moshi phone
//! clients never launch this binary.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

/// Resolves the mux control socket from an environment lookup.
///
/// Prefers `CMUX_TUI_SOCKET`, then the legacy `CMUX_MUX_SOCKET` alias.
pub fn socket_from_env(env: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    for key in ["CMUX_TUI_SOCKET", "CMUX_MUX_SOCKET"] {
        if let Some(value) = env(key) {
            if !value.is_empty() {
                return Some(PathBuf::from(value));
            }
        }
    }
    None
}

/// Resolves the socket from the process environment.
pub fn socket_from_process_env() -> Option<PathBuf> {
    socket_from_env(|key| env::var_os(key))
}

/// Human message used when no socket env var is set.
pub fn missing_socket_message() -> String {
    "CMUX_TUI_SOCKET is not set. Launch via `cmux sidebar plugin use moshi`, or run standalone with CMUX_TUI_SOCKET=/path/to/cmux-tui.sock.".to_string()
}

/// One-shot probe used by `--probe` and tests. Never panics.
pub fn probe_report(env: impl Fn(&str) -> Option<OsString>) -> String {
    match socket_from_env(env) {
        Some(path) => format!("socket={}\nstatus=resolved\n", path.display()),
        None => format!(
            "socket=\nstatus=missing\nnote={}\n",
            missing_socket_message()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn prefers_tui_socket_over_legacy_alias() {
        let env = |key: &str| match key {
            "CMUX_TUI_SOCKET" => Some(OsString::from("/tmp/cmux-tui.sock")),
            "CMUX_MUX_SOCKET" => Some(OsString::from("/tmp/legacy.sock")),
            _ => None,
        };
        assert_eq!(
            socket_from_env(env).as_deref(),
            Some(std::path::Path::new("/tmp/cmux-tui.sock"))
        );
    }

    #[test]
    fn falls_back_to_legacy_mux_socket() {
        let env = |key: &str| match key {
            "CMUX_MUX_SOCKET" => Some(OsString::from("/tmp/legacy.sock")),
            _ => None,
        };
        assert_eq!(
            socket_from_env(env).as_deref(),
            Some(std::path::Path::new("/tmp/legacy.sock"))
        );
    }

    #[test]
    fn empty_or_missing_socket_is_none() {
        let env = |key: &str| match key {
            "CMUX_TUI_SOCKET" => Some(OsString::from("")),
            _ => None,
        };
        assert!(socket_from_env(env).is_none());
        assert!(socket_from_env(|_| None).is_none());
    }

    #[test]
    fn probe_report_never_panics_without_socket() {
        let text = probe_report(|_| None);
        assert!(text.contains("status=missing"));
        assert!(text.contains("CMUX_TUI_SOCKET"));
    }
}
