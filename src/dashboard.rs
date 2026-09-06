//! Interactive menu for Moshi login shells.

use crate::cleanup;
use crate::doctor::is_truthy;
use crate::error::Error;
use crate::host::Host;
use crate::mapping::{self, Snapshot};
use crate::titles::is_default_tty_session;
use crate::tmux;
use std::io::{BufRead, Write};

/// A numbered attach target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    /// 1-based index shown to the user.
    pub index: usize,
    /// Display label.
    pub label: String,
    /// tmux session to attach.
    pub session: String,
}

/// Parsed user choice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Attach to this session name.
    Attach(String),
    /// Rebuild the menu.
    Refresh,
    /// Run orphan cleanup.
    Cleanup,
    /// Drop to a bare login-capable shell.
    BareShell,
    /// Leave the dashboard.
    Quit,
    /// Unrecognized input.
    Invalid(String),
}

/// Builds attach targets: titled cmux workspaces first, then named tmux sessions.
pub fn menu_items(snapshot: &Snapshot) -> Vec<MenuItem> {
    let mut items = Vec::new();
    let mut index = 1usize;

    let mut seen_sessions = std::collections::HashSet::new();
    for row in &snapshot.sessions {
        if let Some(title) = &row.title {
            items.push(MenuItem {
                index,
                label: format!("{title}  [{}]", row.session),
                session: row.session.clone(),
            });
            seen_sessions.insert(row.session.clone());
            index += 1;
        }
    }

    for ws in &snapshot.workspaces {
        let already = snapshot.sessions.iter().any(|row| {
            row.workspace_id.as_deref().is_some_and(|id| {
                crate::cmux::normalize_id(id) == crate::cmux::normalize_id(&ws.id)
            })
        });
        if already {
            continue;
        }
        if let Some(row) = snapshot
            .sessions
            .iter()
            .find(|row| row.title.as_deref() == Some(ws.title.as_str()))
        {
            if seen_sessions.insert(row.session.clone()) {
                items.push(MenuItem {
                    index,
                    label: format!("{}  [{}]", ws.title, row.session),
                    session: row.session.clone(),
                });
                index += 1;
            }
        }
    }

    for row in snapshot.named_sessions() {
        if seen_sessions.insert(row.session.clone()) {
            let flag = if row.attached { "  [attached]" } else { "" };
            items.push(MenuItem {
                index,
                label: format!("{}{flag}", row.session),
                session: row.session.clone(),
            });
            index += 1;
        }
    }

    items
}

/// Renders the dashboard (English).
pub fn render(snapshot: &Snapshot, items: &[MenuItem]) -> String {
    let mut out = String::new();
    out.push_str("===================================================\n");
    out.push_str("  cmux-moshi\n");
    out.push_str("===================================================\n\n");
    out.push_str("  CMUX workspaces\n");
    let workspace_items: Vec<&MenuItem> = items
        .iter()
        .filter(|item| {
            snapshot
                .sessions
                .iter()
                .any(|row| row.session == item.session && row.title.is_some())
                || snapshot
                    .workspaces
                    .iter()
                    .any(|ws| item.label.starts_with(&ws.title))
        })
        .collect();
    if workspace_items.is_empty() {
        out.push_str("    (none mapped yet — run cmux-moshi sync after cmux is reachable)\n");
    } else {
        for item in &workspace_items {
            out.push_str(&format!("    {}) {}\n", item.index, item.label));
        }
    }

    out.push_str("\n  Named tmux sessions\n");
    let named: Vec<&MenuItem> = items
        .iter()
        .filter(|item| {
            !snapshot
                .sessions
                .iter()
                .any(|row| row.session == item.session && row.title.is_some())
                && !is_default_tty_session(&item.session)
        })
        .collect();
    if named.is_empty() {
        out.push_str("    (none)\n");
    } else {
        for item in named {
            out.push_str(&format!("    {}) {}\n", item.index, item.label));
        }
    }

    out.push_str("\n  --------------------------------------------------\n");
    out.push_str("   r) Refresh\n");
    out.push_str("   c) Cleanup orphan ttys* sessions\n");
    out.push_str("   s) Bare shell\n");
    out.push_str("   q) Quit\n\n");
    out.push_str("  Choice: ");
    out
}

/// Parses a line of menu input.
pub fn parse_choice(input: &str, items: &[MenuItem]) -> Choice {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Choice::Invalid(String::new());
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "q" | "quit" => Choice::Quit,
        "r" | "refresh" => Choice::Refresh,
        "c" | "cleanup" => Choice::Cleanup,
        "s" | "shell" => Choice::BareShell,
        other => {
            if let Ok(n) = other.parse::<usize>() {
                if let Some(item) = items.iter().find(|item| item.index == n) {
                    return Choice::Attach(item.session.clone());
                }
            }
            Choice::Invalid(trimmed.to_string())
        }
    }
}

/// True when the dashboard should start (Moshi client or --force).
pub fn allowed(host: &dyn Host, force: bool) -> bool {
    force || host.env("MOSHI_CLIENT").as_deref().is_some_and(is_truthy)
}

/// Interactive loop. Returns when the user quits or a bare shell is requested.
pub fn run(
    host: &dyn Host,
    force: bool,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
) -> Result<(), Error> {
    if !allowed(host, force) {
        return Err(Error::usage(
            "dashboard is for Moshi login shells (MOSHI_CLIENT=1). Pass --force to run anyway.",
        ));
    }
    loop {
        let snapshot = mapping::collect(host)?;
        let items = menu_items(&snapshot);
        write!(output, "{}", render(&snapshot, &items))?;
        output.flush()?;
        let mut line = String::new();
        let n = input.read_line(&mut line)?;
        if n == 0 {
            return Ok(());
        }
        match parse_choice(&line, &items) {
            Choice::Quit => return Ok(()),
            Choice::Refresh => continue,
            Choice::Cleanup => {
                let plans = cleanup::run(host, false)?;
                writeln!(output, "\n{}\n", cleanup::format_report(&plans, false))?;
            }
            Choice::BareShell => {
                let shell = host
                    .env("SHELL")
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "/bin/sh".to_string());
                host.run_inherit(&shell, &["-l"])?;
                return Ok(());
            }
            Choice::Attach(session) => {
                writeln!(
                    output,
                    "attaching {session} (detach with the tmux prefix + d)"
                )?;
                output.flush()?;
                tmux::attach_session(host, &session)?;
            }
            Choice::Invalid(raw) => {
                if !raw.is_empty() {
                    writeln!(output, "unknown choice: {raw}")?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmux::{RpcProbe, WorkspaceTitle};
    use crate::mapping::{SessionRow, Snapshot};

    fn snapshot() -> Snapshot {
        Snapshot {
            sessions: vec![
                SessionRow {
                    session: "ttys001".into(),
                    attached: false,
                    workspace_id: Some("aaa".into()),
                    title: Some("accounting".into()),
                    pane_command: Some("zsh".into()),
                    pane_pid: Some(1),
                },
                SessionRow {
                    session: "project-alpha".into(),
                    attached: true,
                    workspace_id: None,
                    title: None,
                    pane_command: Some("zsh".into()),
                    pane_pid: Some(2),
                },
            ],
            workspaces: vec![WorkspaceTitle {
                id: "aaa".into(),
                title: "accounting".into(),
            }],
            probe: RpcProbe::default(),
        }
    }

    #[test]
    fn numbers_workspaces_then_named_sessions() {
        let items = menu_items(&snapshot());
        assert_eq!(items[0].session, "ttys001");
        assert_eq!(items[1].session, "project-alpha");
        assert!(matches!(parse_choice("1", &items), Choice::Attach(s) if s == "ttys001"));
        assert!(matches!(parse_choice("q", &items), Choice::Quit));
        assert!(matches!(parse_choice("r", &items), Choice::Refresh));
        assert!(matches!(parse_choice("c", &items), Choice::Cleanup));
        assert!(matches!(parse_choice("s", &items), Choice::BareShell));
    }

    #[test]
    fn render_lists_english_actions() {
        let snap = snapshot();
        let text = render(&snap, &menu_items(&snap));
        assert!(text.contains("CMUX workspaces"));
        assert!(text.contains("Named tmux sessions"));
        assert!(text.contains("Cleanup orphan"));
        assert!(text.contains("Bare shell"));
    }
}
