//! Safe orphan cleanup for idle `ttys*` tmux sessions.

use crate::error::Error;
use crate::host::Host;
use crate::mapping::{self, SessionRow, Snapshot};
use crate::titles::is_default_tty_session;
use crate::tmux;

/// Command names treated as an idle login/interactive shell.
const IDLE_SHELLS: &[&str] = &["zsh", "bash", "sh", "dash", "fish", "ksh"];

/// Planned cleanup action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CleanupPlan {
    /// Session name.
    pub session: String,
    /// True when the session may be killed.
    pub kill: bool,
    /// Human reason.
    pub reason: String,
}

/// Builds a safe cleanup plan: only `ttys*` sessions whose pane tree is an idle shell.
pub fn plan(snapshot: &Snapshot, host: &dyn Host) -> Vec<CleanupPlan> {
    snapshot
        .sessions
        .iter()
        .map(|row| classify(row, host))
        .collect()
}

/// Applies kills (unless dry-run).
pub fn apply(
    host: &dyn Host,
    plans: &[CleanupPlan],
    dry_run: bool,
) -> Result<Vec<CleanupPlan>, Error> {
    for plan in plans {
        if plan.kill && !dry_run {
            tmux::kill_session(host, &plan.session)?;
        }
    }
    Ok(plans.to_vec())
}

/// Collects and applies.
pub fn run(host: &dyn Host, dry_run: bool) -> Result<Vec<CleanupPlan>, Error> {
    let snapshot = mapping::collect(host)?;
    let plans = plan(&snapshot, host);
    apply(host, &plans, dry_run)
}

/// Formats a human report.
pub fn format_report(plans: &[CleanupPlan], dry_run: bool) -> String {
    let verb = if dry_run { "would kill" } else { "killed" };
    let mut lines = Vec::new();
    let mut killed = 0usize;
    for plan in plans {
        if plan.kill {
            killed += 1;
            lines.push(format!("{verb}  {} ({})", plan.session, plan.reason));
        } else {
            lines.push(format!("keep   {} ({})", plan.session, plan.reason));
        }
    }
    if killed == 0 {
        lines.push("no orphan ttys* sessions removed".into());
    }
    lines.join("\n")
}

/// True when `command` is an idle shell (basename, no path).
pub fn is_idle_shell(command: &str) -> bool {
    let base = command
        .rsplit('/')
        .next()
        .unwrap_or(command)
        .trim()
        .trim_start_matches('-');
    IDLE_SHELLS
        .iter()
        .any(|shell| base.eq_ignore_ascii_case(shell))
}

fn classify(row: &SessionRow, host: &dyn Host) -> CleanupPlan {
    if !is_default_tty_session(&row.session) {
        return CleanupPlan {
            session: row.session.clone(),
            kill: false,
            reason: "named session".into(),
        };
    }
    if row.attached {
        return CleanupPlan {
            session: row.session.clone(),
            kill: false,
            reason: "attached session".into(),
        };
    }
    // Never kill when pane state is missing — list-panes races / empty
    // command strings would otherwise look like idle orphans.
    let Some(cmd) = row.pane_command.as_deref() else {
        return CleanupPlan {
            session: row.session.clone(),
            kill: false,
            reason: "pane command unknown".into(),
        };
    };
    if !is_idle_shell(cmd) {
        return CleanupPlan {
            session: row.session.clone(),
            kill: false,
            reason: format!("pane running {cmd}"),
        };
    }
    if let Some(pid) = row.pane_pid {
        if let Some(busy) = busy_descendant(host, pid) {
            return CleanupPlan {
                session: row.session.clone(),
                kill: false,
                reason: format!("child process {busy} still running"),
            };
        }
    } else {
        return CleanupPlan {
            session: row.session.clone(),
            kill: false,
            reason: "pane pid unknown".into(),
        };
    }
    CleanupPlan {
        session: row.session.clone(),
        kill: true,
        reason: "idle shell on ttys* session".into(),
    }
}

fn busy_descendant(host: &dyn Host, pid: u32) -> Option<String> {
    let mut stack = vec![pid];
    let mut seen = 0u8;
    while let Some(current) = stack.pop() {
        if seen > 32 {
            break;
        }
        seen += 1;
        if current != pid {
            if let Ok(comm) = host.process_comm(current) {
                if !is_idle_shell(&comm) {
                    return Some(comm);
                }
            }
        }
        if let Ok(children) = host.child_pids(current) {
            stack.extend(children);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmux::RpcProbe;
    use crate::host::FakeHost;
    use crate::mapping::{SessionRow, Snapshot};

    fn snap(rows: Vec<SessionRow>) -> Snapshot {
        Snapshot {
            sessions: rows,
            workspaces: Vec::new(),
            probe: RpcProbe::default(),
        }
    }

    fn row(session: &str, cmd: &str, pid: u32) -> SessionRow {
        SessionRow {
            session: session.into(),
            attached: false,
            workspace_id: None,
            title: None,
            pane_command: Some(cmd.into()),
            pane_pid: Some(pid),
        }
    }

    #[test]
    fn kills_only_idle_ttys_shells() {
        let mut host = FakeHost::default();
        host.comms.insert(1, "zsh".into());
        host.comms.insert(2, "claude".into());
        host.children.insert(2, vec![22]);
        host.comms.insert(22, "claude".into());
        let snapshot = snap(vec![
            row("ttys001", "zsh", 1),
            row("ttys002", "zsh", 2),
            row("accounting", "zsh", 3),
            row("ttys003", "python", 4),
        ]);
        let plans = plan(&snapshot, &host);
        assert!(plans[0].kill);
        assert!(!plans[1].kill);
        assert!(!plans[2].kill);
        assert!(!plans[3].kill);
    }

    #[test]
    fn keeps_attached_idle_ttys_sessions() {
        let mut host = FakeHost::default();
        host.comms.insert(9, "zsh".into());
        let mut attached = row("ttys009", "zsh", 9);
        attached.attached = true;
        let plans = plan(&snap(vec![attached]), &host);
        assert!(!plans[0].kill);
        assert!(plans[0].reason.contains("attached"));
    }

    #[test]
    fn keeps_ttys_when_pane_state_unknown() {
        let host = FakeHost::default();
        let unknown = SessionRow {
            session: "ttys010".into(),
            attached: false,
            workspace_id: None,
            title: None,
            pane_command: None,
            pane_pid: None,
        };
        let plans = plan(&snap(vec![unknown]), &host);
        assert!(!plans[0].kill);
        assert!(plans[0].reason.contains("unknown"));
    }

    #[test]
    fn idle_shell_matches_login_and_path_forms() {
        assert!(is_idle_shell("-zsh"));
        assert!(is_idle_shell("/bin/bash"));
        assert!(!is_idle_shell("node"));
        assert!(!is_idle_shell("python3"));
    }
}
