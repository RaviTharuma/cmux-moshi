//! Rename default `ttys*` sessions to friendly cmux titles.

use crate::error::Error;
use crate::host::Host;
use crate::mapping::{self, SessionRow, Snapshot};
use crate::titles::{is_default_tty_session, sanitize_session_name, unique_session_name};
use crate::tmux;

/// One planned or applied rename.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenamePlan {
    /// Current session name.
    pub from: String,
    /// Target session name.
    pub to: String,
    /// Why this row was skipped, when applicable.
    pub skip: Option<String>,
}

/// Options for `sync` / `rename`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SyncOptions {
    /// Also rename already-named sessions.
    pub force: bool,
    /// Print the plan only.
    pub dry_run: bool,
}

/// Builds an idempotent rename plan.
pub fn plan(snapshot: &Snapshot, options: SyncOptions) -> Vec<RenamePlan> {
    let mut taken: Vec<String> = snapshot
        .sessions
        .iter()
        .map(|r| r.session.clone())
        .collect();
    let mut plans = Vec::new();
    for row in &snapshot.sessions {
        plans.push(plan_row(row, options, &mut taken));
    }
    plans
}

fn plan_row(row: &SessionRow, options: SyncOptions, taken: &mut [String]) -> RenamePlan {
    let Some(title) = row.title.as_deref().filter(|s| !s.is_empty()) else {
        return RenamePlan {
            from: row.session.clone(),
            to: row.session.clone(),
            skip: Some("no friendly cmux title".into()),
        };
    };
    let base = sanitize_session_name(title);
    if row.session == base {
        return RenamePlan {
            from: row.session.clone(),
            to: row.session.clone(),
            skip: Some("already named".into()),
        };
    }
    if !is_default_tty_session(&row.session) && !options.force {
        return RenamePlan {
            from: row.session.clone(),
            to: row.session.clone(),
            skip: Some("named session left unchanged (pass --force to rename)".into()),
        };
    }
    let others: Vec<String> = taken
        .iter()
        .filter(|name| *name != &row.session)
        .cloned()
        .collect();
    let to = unique_session_name(&base, row.workspace_id.as_deref(), &others);
    if let Some(pos) = taken.iter().position(|n| n == &row.session) {
        taken[pos] = to.clone();
    }
    RenamePlan {
        from: row.session.clone(),
        to,
        skip: None,
    }
}

/// Applies the plan (unless dry-run).
pub fn apply(
    host: &dyn Host,
    plans: &[RenamePlan],
    dry_run: bool,
) -> Result<Vec<RenamePlan>, Error> {
    let mut applied = Vec::new();
    for plan in plans {
        if plan.skip.is_some() || plan.from == plan.to {
            applied.push(plan.clone());
            continue;
        }
        if !dry_run {
            tmux::rename_session(host, &plan.from, &plan.to)?;
        }
        applied.push(plan.clone());
    }
    Ok(applied)
}

/// Collects, plans, and applies.
pub fn run(host: &dyn Host, options: SyncOptions) -> Result<Vec<RenamePlan>, Error> {
    let snapshot = mapping::collect(host)?;
    let plans = plan(&snapshot, options);
    apply(host, &plans, options.dry_run)
}

/// Formats a human report.
pub fn format_report(plans: &[RenamePlan], dry_run: bool) -> String {
    let verb = if dry_run { "would rename" } else { "renamed" };
    let mut lines = Vec::new();
    let mut changed = 0usize;
    for plan in plans {
        if let Some(reason) = &plan.skip {
            lines.push(format!("skip  {} ({reason})", plan.from));
        } else if plan.from == plan.to {
            lines.push(format!("ok    {}", plan.from));
        } else {
            changed += 1;
            lines.push(format!("{verb}  {} -> {}", plan.from, plan.to));
        }
    }
    if changed == 0 {
        lines.push("no sessions renamed".into());
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmux::RpcProbe;
    use crate::mapping::{SessionRow, Snapshot};

    fn snap(rows: Vec<SessionRow>) -> Snapshot {
        Snapshot {
            sessions: rows,
            workspaces: Vec::new(),
            probe: RpcProbe::default(),
        }
    }

    fn row(session: &str, title: Option<&str>) -> SessionRow {
        SessionRow {
            session: session.into(),
            attached: false,
            workspace_id: Some("aaa1".into()),
            title: title.map(ToString::to_string),
            pane_command: Some("zsh".into()),
            pane_pid: Some(1),
        }
    }

    #[test]
    fn renames_only_ttys_sessions_without_force() {
        let snapshot = snap(vec![
            row("ttys001", Some("accounting")),
            row("project-alpha", Some("should-not-move")),
        ]);
        let plans = plan(&snapshot, SyncOptions::default());
        assert_eq!(plans[0].to, "accounting");
        assert!(plans[0].skip.is_none());
        assert!(plans[1].skip.is_some());
    }

    #[test]
    fn is_idempotent_when_already_named() {
        let snapshot = snap(vec![row("accounting", Some("accounting"))]);
        let plans = plan(&snapshot, SyncOptions::default());
        assert_eq!(plans[0].skip.as_deref(), Some("already named"));
    }

    #[test]
    fn force_renames_named_sessions() {
        let snapshot = snap(vec![row("old-name", Some("inbox"))]);
        let plans = plan(
            &snapshot,
            SyncOptions {
                force: true,
                dry_run: true,
            },
        );
        assert_eq!(plans[0].to, "inbox");
        assert!(plans[0].skip.is_none());
    }
}
