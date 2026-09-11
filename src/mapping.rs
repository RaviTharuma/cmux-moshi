//! Live mapping: tmux session ↔ CMUX_WORKSPACE_ID ↔ friendly title.

use crate::cmux::{self, HashMapIndex, RpcProbe, WorkspaceTitle};
use crate::error::Error;
use crate::host::Host;
use crate::procenv::workspace_id_from_env;
use crate::tmux::{self, TmuxPane, TmuxSession};
use serde::Serialize;

/// One mapped tmux session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SessionRow {
    /// tmux session name.
    pub session: String,
    /// Whether a client is attached.
    pub attached: bool,
    /// `CMUX_WORKSPACE_ID` from the pane process tree, if found.
    pub workspace_id: Option<String>,
    /// Friendly cmux title, if known.
    pub title: Option<String>,
    /// Primary pane command.
    pub pane_command: Option<String>,
    /// Primary pane pid.
    pub pane_pid: Option<u32>,
}

/// Snapshot used by `list`, `sync`, `dashboard`, and `cleanup`.
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// Sessions currently known to tmux.
    pub sessions: Vec<SessionRow>,
    /// Workspaces reported by cmux RPC.
    pub workspaces: Vec<WorkspaceTitle>,
    /// RPC probe details.
    pub probe: RpcProbe,
}

impl Snapshot {
    /// Named (non-`ttys*`) sessions.
    pub fn named_sessions(&self) -> impl Iterator<Item = &SessionRow> {
        self.sessions
            .iter()
            .filter(|row| !crate::titles::is_default_tty_session(&row.session))
    }
}

/// Builds a live snapshot from `tmux` + process env + `cmux` RPC.
pub fn collect(host: &dyn Host) -> Result<Snapshot, Error> {
    if !host.which("tmux") {
        return Err(Error::command("tmux", "not found on PATH"));
    }
    let sessions = tmux::list_sessions(host)?;
    let panes = tmux::list_panes(host).unwrap_or_default();
    let probe = cmux::probe_rpc(host);
    let index = cmux::title_index(&probe);
    let rows = join_rows(&sessions, &panes, &index, host);
    Ok(Snapshot {
        sessions: rows,
        workspaces: probe.workspaces.clone(),
        probe,
    })
}

/// Joins tmux state with RPC titles and process-env workspace ids.
pub fn join_rows(
    sessions: &[TmuxSession],
    panes: &[TmuxPane],
    index: &HashMapIndex,
    host: &dyn Host,
) -> Vec<SessionRow> {
    sessions
        .iter()
        .map(|session| {
            let pane = tmux::primary_pane(panes, &session.name);
            let pane_pid = pane.map(|p| p.pid);
            let pane_command = pane.map(|p| p.command.clone()).filter(|s| !s.is_empty());
            let mut workspace_id = pane_pid.and_then(|pid| walk_workspace_id(host, pid));
            if workspace_id.is_none() {
                workspace_id = index.tty_to_id.get(&normalize_tty(&session.name)).cloned();
            }
            let mut title = workspace_id
                .as_deref()
                .and_then(|id| index.by_id.get(&cmux::normalize_id(id)).cloned());
            if title.is_none() {
                title = index.by_tty.get(&normalize_tty(&session.name)).cloned();
            }
            SessionRow {
                session: session.name.clone(),
                attached: session.attached,
                workspace_id,
                title,
                pane_command,
                pane_pid,
            }
        })
        .collect()
}

/// Walks pid → parent looking for `CMUX_WORKSPACE_ID`.
pub fn walk_workspace_id(host: &dyn Host, start: u32) -> Option<String> {
    let mut current = Some(start);
    let mut seen = 0u8;
    while let Some(pid) = current {
        if seen > 16 {
            break;
        }
        seen += 1;
        if let Ok(env) = host.read_environ(pid) {
            if let Some(id) = workspace_id_from_env(&env) {
                return Some(id);
            }
        }
        current = host.parent_pid(pid).ok().flatten();
    }
    None
}

/// Formats the mapping as a stable table.
pub fn format_table(rows: &[SessionRow]) -> String {
    let mut lines = vec![format!(
        "{:<20} {:<38} {:<24} {}",
        "SESSION", "CMUX_WORKSPACE_ID", "TITLE", "PANE"
    )];
    for row in rows {
        lines.push(format!(
            "{:<20} {:<38} {:<24} {}",
            row.session,
            row.workspace_id.as_deref().unwrap_or("-"),
            row.title.as_deref().unwrap_or("-"),
            row.pane_command.as_deref().unwrap_or("-")
        ));
    }
    if rows.is_empty() {
        lines.push("(no tmux sessions)".to_string());
    }
    lines.join("\n")
}

/// Serializes the mapping as JSON.
pub fn format_json(rows: &[SessionRow]) -> Result<String, Error> {
    serde_json::to_string_pretty(rows).map_err(|err| Error::parse(err.to_string()))
}

fn normalize_tty(name: &str) -> String {
    name.trim().trim_start_matches("/dev/").to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::FakeHost;
    use crate::tmux::{TmuxPane, TmuxSession};
    use std::collections::HashMap;

    #[test]
    fn joins_workspace_id_from_parent_env_and_rpc_title() {
        let mut host = FakeHost::default();
        host.environs.insert(
            10,
            HashMap::from([("CMUX_WORKSPACE_ID".into(), "AAA-BBB".into())]),
        );
        host.parents.insert(20, 10);
        let mut index = HashMapIndex::default();
        index.by_id.insert("aaa-bbb".into(), "accounting".into());
        let rows = join_rows(
            &[TmuxSession {
                name: "ttys001".into(),
                attached: false,
            }],
            &[TmuxPane {
                session: "ttys001".into(),
                pid: 20,
                command: "zsh".into(),
                active: true,
            }],
            &index,
            &host,
        );
        assert_eq!(rows[0].workspace_id.as_deref(), Some("AAA-BBB"));
        assert_eq!(rows[0].title.as_deref(), Some("accounting"));
    }

    #[test]
    fn prefers_active_pane_for_workspace_env() {
        let mut host = FakeHost::default();
        host.environs.insert(
            20,
            HashMap::from([("CMUX_WORKSPACE_ID".into(), "idle-ws".into())]),
        );
        host.environs.insert(
            21,
            HashMap::from([("CMUX_WORKSPACE_ID".into(), "active-ws".into())]),
        );
        let mut index = HashMapIndex::default();
        index
            .by_id
            .insert("active-ws".into(), "active-title".into());
        index.by_id.insert("idle-ws".into(), "idle-title".into());
        let rows = join_rows(
            &[TmuxSession {
                name: "ttys001".into(),
                attached: false,
            }],
            &[
                TmuxPane {
                    session: "ttys001".into(),
                    pid: 20,
                    command: "zsh".into(),
                    active: false,
                },
                TmuxPane {
                    session: "ttys001".into(),
                    pid: 21,
                    command: "claude".into(),
                    active: true,
                },
            ],
            &index,
            &host,
        );
        assert_eq!(rows[0].workspace_id.as_deref(), Some("active-ws"));
        assert_eq!(rows[0].title.as_deref(), Some("active-title"));
        assert_eq!(rows[0].pane_command.as_deref(), Some("claude"));
    }

    #[test]
    fn falls_back_to_tty_index_when_process_env_missing() {
        let host = FakeHost::default();
        let mut index = HashMapIndex::default();
        index.tty_to_id.insert("ttys001".into(), "from-tty".into());
        index.by_id.insert("from-tty".into(), "tty-title".into());
        let rows = join_rows(
            &[TmuxSession {
                name: "ttys001".into(),
                attached: false,
            }],
            &[TmuxPane {
                session: "ttys001".into(),
                pid: 99,
                command: "zsh".into(),
                active: true,
            }],
            &index,
            &host,
        );
        assert_eq!(rows[0].workspace_id.as_deref(), Some("from-tty"));
        assert_eq!(rows[0].title.as_deref(), Some("tty-title"));
    }
}
