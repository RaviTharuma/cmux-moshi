//! tmux CLI helpers. All calls go through the public `tmux` binary.

use crate::error::Error;
use crate::host::{CommandOutput, Host};

/// One tmux session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TmuxSession {
    /// Session name.
    pub name: String,
    /// True when at least one client is attached.
    pub attached: bool,
}

/// One pane belonging to a session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TmuxPane {
    /// Session name.
    pub session: String,
    /// Pane pid.
    pub pid: u32,
    /// `#{pane_current_command}`.
    pub command: String,
    /// True when `#{pane_active}` is set for this pane.
    pub active: bool,
}

/// Lists sessions via `tmux list-sessions`.
pub fn list_sessions(host: &dyn Host) -> Result<Vec<TmuxSession>, Error> {
    let output = tmux(
        host,
        &[
            "list-sessions",
            "-F",
            "#{session_name}\t#{session_attached}",
        ],
    )?;
    if !output.success() {
        if output
            .stderr
            .to_ascii_lowercase()
            .contains("no server running")
            || output.stderr.to_ascii_lowercase().contains("no sessions")
        {
            return Ok(Vec::new());
        }
        return Err(Error::command(
            "tmux",
            first_line(&output.stderr).unwrap_or("list-sessions failed"),
        ));
    }
    Ok(parse_sessions(&output.stdout))
}

/// Lists panes via `tmux list-panes -a`.
pub fn list_panes(host: &dyn Host) -> Result<Vec<TmuxPane>, Error> {
    let output = tmux(
        host,
        &[
            "list-panes",
            "-a",
            "-F",
            "#{session_name}\t#{pane_pid}\t#{pane_current_command}\t#{pane_active}",
        ],
    )?;
    if !output.success() {
        if output
            .stderr
            .to_ascii_lowercase()
            .contains("no server running")
        {
            return Ok(Vec::new());
        }
        return Err(Error::command(
            "tmux",
            first_line(&output.stderr).unwrap_or("list-panes failed"),
        ));
    }
    Ok(parse_panes(&output.stdout))
}

/// Renames `from` to `to`.
pub fn rename_session(host: &dyn Host, from: &str, to: &str) -> Result<(), Error> {
    let output = tmux(host, &["rename-session", "-t", from, to])?;
    if output.success() {
        Ok(())
    } else {
        Err(Error::command(
            "tmux",
            first_line(&output.stderr).unwrap_or("rename-session failed"),
        ))
    }
}

/// Kills a session by name.
pub fn kill_session(host: &dyn Host, name: &str) -> Result<(), Error> {
    let output = tmux(host, &["kill-session", "-t", name])?;
    if output.success() {
        Ok(())
    } else {
        Err(Error::command(
            "tmux",
            first_line(&output.stderr).unwrap_or("kill-session failed"),
        ))
    }
}

/// Attaches to `name` (inherits the TTY).
pub fn attach_session(host: &dyn Host, name: &str) -> Result<i32, Error> {
    host.run_inherit("tmux", &["attach", "-t", name])
}

/// Parses `list-sessions` format output.
pub fn parse_sessions(stdout: &str) -> Vec<TmuxSession> {
    stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let (name, attached) = match line.split_once('\t') {
                Some(pair) => pair,
                None => (line, "0"),
            };
            if name.is_empty() {
                return None;
            }
            Some(TmuxSession {
                name: name.to_string(),
                attached: attached.trim() == "1" || attached.trim().eq_ignore_ascii_case("true"),
            })
        })
        .collect()
}

/// Parses `list-panes` format output.
///
/// Accepts both the current four-field format
/// (`session`, `pid`, `command`, `active`) and the legacy three-field
/// format without `pane_active` (treated as inactive).
pub fn parse_panes(stdout: &str) -> Vec<TmuxPane> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(4, '\t');
            let session = parts.next()?.trim();
            let pid = parts.next()?.trim().parse().ok()?;
            let command = parts.next().unwrap_or("").trim();
            let active_raw = parts.next().unwrap_or("").trim();
            if session.is_empty() {
                return None;
            }
            Some(TmuxPane {
                session: session.to_string(),
                pid,
                command: command.to_string(),
                active: active_raw == "1" || active_raw.eq_ignore_ascii_case("true"),
            })
        })
        .collect()
}

/// Picks the active pane for a session, falling back to the first pane.
pub fn primary_pane<'a>(panes: &'a [TmuxPane], session: &str) -> Option<&'a TmuxPane> {
    let session_panes: Vec<&TmuxPane> = panes
        .iter()
        .filter(|pane| pane.session == session)
        .collect();
    session_panes
        .iter()
        .copied()
        .find(|pane| pane.active)
        .or_else(|| session_panes.first().copied())
}

fn tmux(host: &dyn Host, args: &[&str]) -> Result<CommandOutput, Error> {
    host.run("tmux", args)
}

fn first_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_session_and_pane_tables() {
        let sessions = parse_sessions("ttys001\t0\naccounting\t1\n");
        assert_eq!(sessions.len(), 2);
        assert!(!sessions[0].attached);
        assert!(sessions[1].attached);

        let panes = parse_panes("ttys001\t4242\tzsh\t0\naccounting\t99\tclaude\t1\n");
        assert_eq!(panes[0].pid, 4242);
        assert!(!panes[0].active);
        assert_eq!(panes[1].command, "claude");
        assert!(panes[1].active);

        let legacy = parse_panes("ttys002\t7\tbash\n");
        assert_eq!(legacy[0].pid, 7);
        assert!(!legacy[0].active);
    }

    #[test]
    fn primary_pane_prefers_active() {
        let panes = vec![
            TmuxPane {
                session: "ttys001".into(),
                pid: 1,
                command: "zsh".into(),
                active: false,
            },
            TmuxPane {
                session: "ttys001".into(),
                pid: 2,
                command: "claude".into(),
                active: true,
            },
        ];
        let primary = primary_pane(&panes, "ttys001").unwrap();
        assert_eq!(primary.pid, 2);
        assert_eq!(primary.command, "claude");
    }
}
