//! Environment diagnostics for Moshi + cmux + tmux.

use crate::cmux;
use crate::host::Host;
use std::fmt::Write as _;

/// One doctor check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    /// Short name.
    pub name: String,
    /// `ok`, `warn`, or `fail`.
    pub status: Status,
    /// Detail line.
    pub detail: String,
}

/// Check severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Required piece is present.
    Ok,
    /// Optional or degraded.
    Warn,
    /// Required piece missing.
    Fail,
}

/// Runs all doctor checks.
pub fn run(host: &dyn Host) -> Vec<Check> {
    let mut checks = vec![
        path_check(host, "cmux", true),
        path_check(host, "tmux", true),
        mosh_check(host),
        rpc_check(host),
        moshi_client_check(host),
    ];
    if host.which("cmux") {
        if let Ok(out) = host.run("cmux", &["--version"]) {
            let version = out.stdout.lines().next().unwrap_or("").trim();
            if !version.is_empty() {
                checks.push(Check {
                    name: "cmux version".into(),
                    status: Status::Ok,
                    detail: version.to_string(),
                });
            }
        }
        if let Ok(out) = host.run("cmux", &["--help"]) {
            let help = out.stdout.to_ascii_lowercase();
            let has_plugin = help.contains("sidebar") && help.contains("plugin");
            checks.push(Check {
                name: "cmux plugin manager".into(),
                status: if has_plugin { Status::Ok } else { Status::Warn },
                detail: if has_plugin {
                    "cmux --help mentions sidebar plugin commands".into()
                } else {
                    "could not confirm `cmux sidebar plugin` from --help; install still uses that path".into()
                },
            });
        }
    }
    checks
}

/// Formats checks as a table and returns the process exit code (1 if any fail).
pub fn format_report(checks: &[Check]) -> (String, i32) {
    let mut out = String::from("cmux-moshi doctor\n");
    let mut failed = false;
    for check in checks {
        let mark = match check.status {
            Status::Ok => "ok  ",
            Status::Warn => "warn",
            Status::Fail => {
                failed = true;
                "fail"
            }
        };
        let _ = writeln!(out, "  [{mark}] {:<22} {}", check.name, check.detail);
    }
    out.push_str("\nTips\n");
    out.push_str("  • Host CLI (the product): cmux-moshi doctor|list|sync|dashboard|cleanup\n");
    out.push_str(
        "  • Install packaging: cmux sidebar plugin install https://github.com/RaviTharuma/cmux-moshi.git\n",
    );
    out.push_str(
        "  • Optional picker: cmux sidebar plugin use moshi && cmux server reload-config\n",
    );
    out.push_str(
        "  • Moshi: Settings → enable MOSHI_CLIENT env toggle (exports MOSHI_CLIENT=1), then reconnect\n",
    );
    out.push_str("  • Then run: cmux-moshi dashboard   or   cmux-moshi install-shell\n");
    out.push_str("  • Dashboard: y syncs ttys* titles; c cleans idle orphans\n");
    (out, if failed { 1 } else { 0 })
}

fn path_check(host: &dyn Host, name: &str, required: bool) -> Check {
    if host.which(name) {
        Check {
            name: name.into(),
            status: Status::Ok,
            detail: format!("{name} is on PATH"),
        }
    } else {
        Check {
            name: name.into(),
            status: if required { Status::Fail } else { Status::Warn },
            detail: format!("{name} is not on PATH"),
        }
    }
}

fn mosh_check(host: &dyn Host) -> Check {
    let present = host.which("mosh") || host.which("mosh-server");
    Check {
        name: "mosh".into(),
        status: if present { Status::Ok } else { Status::Warn },
        detail: if present {
            "mosh or mosh-server is on PATH (optional for Moshi)".into()
        } else {
            "mosh not found (optional; Moshi can also use SSH)".into()
        },
    }
}

fn rpc_check(host: &dyn Host) -> Check {
    if !host.which("cmux") {
        return Check {
            name: "cmux rpc".into(),
            status: Status::Fail,
            detail: "skipped because cmux is missing".into(),
        };
    }
    let probe = cmux::probe_rpc(host);
    let detail = if probe.notes.is_empty() {
        "no RPC methods responded".into()
    } else {
        probe.notes.join("; ")
    };
    Check {
        name: "cmux rpc".into(),
        status: if probe.reachable {
            Status::Ok
        } else {
            Status::Warn
        },
        detail,
    }
}

fn moshi_client_check(host: &dyn Host) -> Check {
    match host.env("MOSHI_CLIENT") {
        Some(value) if is_truthy(&value) => Check {
            name: "MOSHI_CLIENT".into(),
            status: Status::Ok,
            detail: format!("MOSHI_CLIENT={value} (Moshi login shell detected)"),
        },
        Some(value) => Check {
            name: "MOSHI_CLIENT".into(),
            status: Status::Warn,
            detail: format!(
                "MOSHI_CLIENT={value} (not treated as a Moshi client; dashboard needs 1 or --force)"
            ),
        },
        None => Check {
            name: "MOSHI_CLIENT".into(),
            status: Status::Warn,
            detail: "unset. In Moshi enable Settings → MOSHI_CLIENT env toggle, then reconnect"
                .into(),
        },
    }
}

/// True when the Moshi client env flag is enabled.
pub fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::FakeHost;

    #[test]
    fn reports_missing_required_tools_and_moshi_tips() {
        let host = FakeHost::default();
        let checks = run(&host);
        assert!(checks
            .iter()
            .any(|c| c.name == "cmux" && c.status == Status::Fail));
        assert!(checks
            .iter()
            .any(|c| c.name == "tmux" && c.status == Status::Fail));
        assert!(checks
            .iter()
            .any(|c| c.name == "MOSHI_CLIENT" && c.status == Status::Warn));
        let (report, code) = format_report(&checks);
        assert_eq!(code, 1);
        assert!(report.contains("MOSHI_CLIENT env toggle"));
    }

    #[test]
    fn detects_moshi_client_env() {
        let mut host = FakeHost {
            binaries: vec!["cmux".into(), "tmux".into()],
            ..FakeHost::default()
        };
        host.env.insert("MOSHI_CLIENT".into(), "1".into());
        host.ok("cmux", &["rpc", "debug.terminals"], r#"{"terminals":[]}"#);
        host.ok("cmux", &["rpc", "workspace.list"], r#"{"workspaces":[]}"#);
        host.ok("cmux", &["--version"], "cmux 0.64\n");
        host.ok("cmux", &["--help"], "sidebar plugin install\n");
        let checks = run(&host);
        assert!(checks
            .iter()
            .any(|c| c.name == "MOSHI_CLIENT" && c.status == Status::Ok));
        assert!(checks
            .iter()
            .any(|c| c.name == "cmux rpc" && c.status == Status::Ok));
    }
}
