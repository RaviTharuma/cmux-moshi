//! Optional macOS LaunchAgent for periodic `cmux-moshi sync`.

use crate::error::Error;
use crate::host::Host;
use std::fs;
use std::path::{Path, PathBuf};

/// launchd label matching `scripts/com.cmux-moshi.sync.plist`.
pub const LABEL: &str = "com.cmux-moshi.sync";
/// Filename installed under `~/Library/LaunchAgents/`.
pub const PLIST_FILENAME: &str = "com.cmux-moshi.sync.plist";

/// Canonical plist body shipped in the repo.
pub fn plist_body() -> &'static str {
    include_str!("../scripts/com.cmux-moshi.sync.plist")
}

/// Result of installing or removing the LaunchAgent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchAgentChange {
    /// Target plist path (even when skipped).
    pub plist_path: PathBuf,
    /// Human summary.
    pub message: String,
    /// True when `launchctl load` succeeded on this call.
    pub loaded: bool,
}

/// Default LaunchAgents directory from the environment (never a hardcoded home).
pub fn default_agents_dir(env: &dyn Fn(&str) -> Option<String>) -> PathBuf {
    let home = env("HOME").unwrap_or_else(|| ".".to_string());
    PathBuf::from(home).join("Library").join("LaunchAgents")
}

/// True when the host reports Darwin (or cfg is macOS if `uname` is unavailable).
pub fn is_macos(host: &dyn Host) -> bool {
    match host.run("uname", &["-s"]) {
        Ok(out) if out.success() => out.stdout.trim().eq_ignore_ascii_case("Darwin"),
        _ => cfg!(target_os = "macos"),
    }
}

/// Writes the plist under `agents_dir` and loads it with `launchctl` on macOS.
///
/// Idempotent: rewriting the same body and reloading is a no-op for the user.
/// On non-macOS hosts, returns a skip message without touching `launchctl`.
pub fn install(host: &dyn Host, agents_dir: &Path) -> Result<LaunchAgentChange, Error> {
    let plist_path = agents_dir.join(PLIST_FILENAME);
    if !is_macos(host) {
        return Ok(LaunchAgentChange {
            plist_path,
            message: "LaunchAgent install is macOS-only; skipped on this platform \
                      (use a Mac, or copy scripts/com.cmux-moshi.sync.plist manually)"
                .into(),
            loaded: false,
        });
    }

    fs::create_dir_all(agents_dir)?;
    let body = plist_body();
    let already = plist_path.is_file()
        && fs::read_to_string(&plist_path)
            .map(|existing| existing == body)
            .unwrap_or(false);
    fs::write(&plist_path, body)?;

    reload(host, &plist_path)?;

    let message = if already {
        format!(
            "LaunchAgent already installed at {}; reloaded",
            plist_path.display()
        )
    } else {
        format!("installed LaunchAgent at {}", plist_path.display())
    };
    Ok(LaunchAgentChange {
        plist_path,
        message,
        loaded: true,
    })
}

/// Unloads the agent (when macOS) and removes the plist if present.
pub fn uninstall(host: &dyn Host, agents_dir: &Path) -> Result<LaunchAgentChange, Error> {
    let plist_path = agents_dir.join(PLIST_FILENAME);
    if !is_macos(host) {
        if plist_path.is_file() {
            fs::remove_file(&plist_path)?;
            return Ok(LaunchAgentChange {
                plist_path,
                message: "removed LaunchAgent plist (launchctl skipped; not macOS)".into(),
                loaded: false,
            });
        }
        return Ok(LaunchAgentChange {
            plist_path,
            message: "LaunchAgent uninstall is macOS-only; nothing to do on this platform".into(),
            loaded: false,
        });
    }

    if plist_path.is_file() {
        let _ = unload(host, &plist_path);
        fs::remove_file(&plist_path)?;
        return Ok(LaunchAgentChange {
            plist_path: plist_path.clone(),
            message: format!("removed LaunchAgent at {}", plist_path.display()),
            loaded: false,
        });
    }

    // Still try unload by label in case a stale job remains.
    let _ = host.run("launchctl", &["remove", LABEL]);
    Ok(LaunchAgentChange {
        message: format!("no LaunchAgent plist at {}", plist_path.display()),
        plist_path,
        loaded: false,
    })
}

/// Doctor-style status for the optional LaunchAgent.
pub fn status_check(host: &dyn Host) -> crate::doctor::Check {
    use crate::doctor::{Check, Status};

    if !is_macos(host) {
        return Check {
            name: "LaunchAgent".into(),
            status: Status::Warn,
            detail: "macOS only (optional periodic sync)".into(),
        };
    }

    let path = default_agents_dir(&|key| host.env(key)).join(PLIST_FILENAME);
    if path.is_file() {
        let loaded = launchctl_lists_label(host);
        Check {
            name: "LaunchAgent".into(),
            status: Status::Ok,
            detail: if loaded {
                format!("installed and loaded ({LABEL}) at {}", path.display())
            } else {
                format!(
                    "plist present at {}; not listed in launchctl (try install-launchagent)",
                    path.display()
                )
            },
        }
    } else {
        Check {
            name: "LaunchAgent".into(),
            status: Status::Warn,
            detail: "not installed (optional: cmux-moshi install-launchagent)".into(),
        }
    }
}

fn reload(host: &dyn Host, plist: &Path) -> Result<(), Error> {
    let _ = unload(host, plist);
    let path = plist.to_string_lossy();
    let out = host.run("launchctl", &["load", path.as_ref()])?;
    if !out.success() {
        let detail = if out.stderr.trim().is_empty() {
            format!("exit {}", out.status)
        } else {
            out.stderr.trim().to_string()
        };
        return Err(Error::command(
            "launchctl",
            format!("load failed: {detail}"),
        ));
    }
    Ok(())
}

fn unload(host: &dyn Host, plist: &Path) -> Result<(), Error> {
    let path = plist.to_string_lossy();
    let _ = host.run("launchctl", &["unload", path.as_ref()]);
    Ok(())
}

fn launchctl_lists_label(host: &dyn Host) -> bool {
    match host.run("launchctl", &["list", LABEL]) {
        Ok(out) => out.success(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::Status;
    use crate::host::FakeHost;
    use std::collections::HashMap;

    fn darwin_host(home: &Path) -> FakeHost {
        let mut host = FakeHost {
            env: HashMap::from([("HOME".into(), home.display().to_string())]),
            ..FakeHost::default()
        };
        host.ok("uname", &["-s"], "Darwin\n");
        host.ok(
            "launchctl",
            &[
                "unload",
                &home
                    .join("Library/LaunchAgents")
                    .join(PLIST_FILENAME)
                    .display()
                    .to_string(),
            ],
            "",
        );
        host.ok(
            "launchctl",
            &[
                "load",
                &home
                    .join("Library/LaunchAgents")
                    .join(PLIST_FILENAME)
                    .display()
                    .to_string(),
            ],
            "",
        );
        host.ok(
            "launchctl",
            &["list", LABEL],
            "PID\tStatus\tLabel\n-\t0\tcom.cmux-moshi.sync\n",
        );
        host.ok("launchctl", &["remove", LABEL], "");
        host
    }

    #[test]
    fn install_is_idempotent_and_uninstall_removes_plist() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path();
        let agents = home.join("Library").join("LaunchAgents");
        let host = darwin_host(home);

        let first = install(&host, &agents).unwrap();
        assert!(first.loaded);
        assert!(first.message.contains("installed"));
        let plist = agents.join(PLIST_FILENAME);
        assert_eq!(fs::read_to_string(&plist).unwrap(), plist_body());

        let again = install(&host, &agents).unwrap();
        assert!(again.loaded);
        assert!(again.message.contains("already installed"));

        let removed = uninstall(&host, &agents).unwrap();
        assert!(!plist.exists());
        assert!(removed.message.contains("removed"));
    }

    #[test]
    fn non_macos_install_skips_without_writing() {
        let dir = tempfile::tempdir().unwrap();
        let agents = dir.path().join("Library").join("LaunchAgents");
        let mut host = FakeHost::default();
        host.ok("uname", &["-s"], "Linux\n");
        let change = install(&host, &agents).unwrap();
        assert!(!change.loaded);
        assert!(change.message.contains("macOS-only"));
        assert!(!agents.join(PLIST_FILENAME).exists());
    }

    #[test]
    fn status_check_warns_when_missing_on_macos() {
        let dir = tempfile::tempdir().unwrap();
        let mut host = FakeHost {
            env: HashMap::from([("HOME".into(), dir.path().display().to_string())]),
            ..FakeHost::default()
        };
        host.ok("uname", &["-s"], "Darwin\n");
        let check = status_check(&host);
        assert_eq!(check.status, Status::Warn);
        assert!(check.detail.contains("install-launchagent"));
    }

    #[test]
    fn default_agents_dir_uses_home_env() {
        let env = |key: &str| match key {
            "HOME" => Some("/tmp/cmux-moshi-home".to_string()),
            _ => None,
        };
        assert_eq!(
            default_agents_dir(&env),
            PathBuf::from("/tmp/cmux-moshi-home/Library/LaunchAgents")
        );
    }
}
