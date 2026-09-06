//! Library-level mapping, sync, and cleanup flows with a fake host.

use cmux_moshi::cleanup;
use cmux_moshi::cmux::{self, HashMapIndex};
use cmux_moshi::dashboard::{self, Choice};
use cmux_moshi::host::FakeHost;
use cmux_moshi::mapping::{self, SessionRow};
use cmux_moshi::sync::{self, SyncOptions};
use cmux_moshi::tmux::{TmuxPane, TmuxSession};
use std::collections::HashMap;

#[test]
fn list_table_includes_workspace_id_and_title() {
    let mut host = FakeHost::default();
    host.environs.insert(
        7,
        HashMap::from([("CMUX_WORKSPACE_ID".into(), "WS-1".into())]),
    );
    let mut index = HashMapIndex::default();
    index.by_id.insert("ws-1".into(), "client-call-prep".into());
    let rows = mapping::join_rows(
        &[TmuxSession {
            name: "ttys012".into(),
            attached: false,
        }],
        &[TmuxPane {
            session: "ttys012".into(),
            pid: 7,
            command: "zsh".into(),
        }],
        &index,
        &host,
    );
    let table = mapping::format_table(&rows);
    assert!(table.contains("ttys012"));
    assert!(table.contains("WS-1"));
    assert!(table.contains("client-call-prep"));
}

#[test]
fn sync_is_idempotent_across_two_plans() {
    let snapshot = snapshot(vec![SessionRow {
        session: "ttys001".into(),
        attached: false,
        workspace_id: Some("abcd".into()),
        title: Some("docs v2".into()),
        pane_command: Some("zsh".into()),
        pane_pid: Some(1),
    }]);
    let first = sync::plan(&snapshot, SyncOptions::default());
    assert_eq!(first[0].to, "docs-v2");
    let renamed = snapshot_renamed(&snapshot, &first);
    let second = sync::plan(&renamed, SyncOptions::default());
    assert_eq!(second[0].skip.as_deref(), Some("already named"));
}

#[test]
fn cleanup_keeps_busy_python_on_ttys() {
    let mut host = FakeHost::default();
    host.comms.insert(4, "python3".into());
    let snapshot = snapshot(vec![SessionRow {
        session: "ttys003".into(),
        attached: false,
        workspace_id: None,
        title: None,
        pane_command: Some("python3".into()),
        pane_pid: Some(4),
    }]);
    let plans = cleanup::plan(&snapshot, &host);
    assert!(!plans[0].kill);
}

#[test]
fn dashboard_choice_refresh_and_quit() {
    let items = Vec::new();
    assert!(matches!(
        dashboard::parse_choice("R", &items),
        Choice::Refresh
    ));
    assert!(matches!(dashboard::parse_choice("Q", &items), Choice::Quit));
}

#[test]
fn rpc_parser_accepts_workspace_list_variants() {
    let value = cmux::parse_rpc_stdout(
        r#"{"result":{"workspaces":[{"workspace_id":"x","name":"INBOX"}]}}"#,
    )
    .unwrap();
    let list = cmux::parse_workspace_list(&value);
    assert_eq!(list[0].id, "x");
    assert_eq!(list[0].title, "INBOX");
}

fn snapshot(sessions: Vec<SessionRow>) -> cmux_moshi::mapping::Snapshot {
    cmux_moshi::mapping::Snapshot {
        sessions,
        workspaces: Vec::new(),
        probe: cmux_moshi::cmux::RpcProbe::default(),
    }
}

fn snapshot_renamed(
    snapshot: &cmux_moshi::mapping::Snapshot,
    plans: &[cmux_moshi::sync::RenamePlan],
) -> cmux_moshi::mapping::Snapshot {
    let mut next = snapshot.clone();
    for plan in plans {
        if plan.skip.is_none() {
            if let Some(row) = next
                .sessions
                .iter_mut()
                .find(|row| row.session == plan.from)
            {
                row.session = plan.to.clone();
            }
        }
    }
    next
}
