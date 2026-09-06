//! Picker state machine. Connecting without a socket is a reconnect
//! status, never a panic.

use crate::sidebar::filter::{filter_titles, MatchResult};
use crate::sidebar::keys::{interpret_key, KeyAction};
use crate::sidebar::socket::{missing_socket_message, socket_from_process_env};
use crate::sidebar::workspaces::{rows_from_tree, WorkspaceRow};
use cmux_client::{ClientConfig, CmuxClient, Tree};
use crossterm::event::KeyEvent;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const REFRESH_EVERY: Duration = Duration::from_secs(2);
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_millis(500);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(8);

/// Connection / readiness status shown in the PTY.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Socket is connected and a workspace list is available.
    Ready,
    /// Waiting to retry. Always used when the socket is missing or drops.
    Reconnecting { message: String },
}

/// Snapshot consumed by the renderer.
#[derive(Clone, Debug)]
pub struct PickerView<'a> {
    /// Current filter query.
    pub query: &'a str,
    /// Visible rows after filtering.
    pub rows: Vec<VisibleRow<'a>>,
    /// Selected index into `rows`.
    pub selected: usize,
    /// Unfiltered workspace count.
    pub total_rows: usize,
    /// Connection status.
    pub status: &'a Status,
}

/// One rendered row.
#[derive(Clone, Debug)]
pub struct VisibleRow<'a> {
    /// Underlying workspace.
    pub row: &'a WorkspaceRow,
    /// Match highlights.
    pub match_positions: &'a [usize],
}

/// In-memory picker. Safe to construct with no socket and no TTY.
pub struct Picker {
    query: String,
    rows: Vec<WorkspaceRow>,
    filtered: Vec<FilteredRow>,
    selected: usize,
    client: Option<CmuxClient>,
    socket_path: Option<PathBuf>,
    status: Status,
    last_refresh: Instant,
    next_reconnect: Instant,
    reconnect_delay: Duration,
}

#[derive(Clone, Debug)]
struct FilteredRow {
    row_index: usize,
    match_positions: Vec<usize>,
}

impl Picker {
    /// Empty picker in the reconnecting state.
    pub fn new() -> Self {
        Self {
            query: String::new(),
            rows: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            client: None,
            socket_path: None,
            status: Status::Reconnecting {
                message: "connecting".to_string(),
            },
            last_refresh: Instant::now(),
            next_reconnect: Instant::now(),
            reconnect_delay: INITIAL_RECONNECT_DELAY,
        }
    }

    /// Builds a picker that never talks to a socket. Used by tests.
    pub fn disconnected(message: impl Into<String>) -> Self {
        let mut picker = Self::new();
        picker.status = Status::Reconnecting {
            message: message.into(),
        };
        picker
    }

    /// Seed rows without a live client (unit tests and offline preview).
    pub fn with_rows(rows: Vec<WorkspaceRow>) -> Self {
        let mut picker = Self::new();
        picker.status = Status::Ready;
        picker.replace_rows(rows);
        picker
    }

    /// Resolves the process env and either connects or schedules reconnect.
    /// Missing `CMUX_TUI_SOCKET` does not panic.
    pub fn connect_or_schedule(&mut self) {
        let socket_path = match socket_from_process_env() {
            Some(path) => path,
            None => {
                self.socket_path = None;
                self.disconnect_with_backoff(missing_socket_message());
                return;
            }
        };
        self.try_connect(socket_path);
    }

    /// Applies a key. Returns `true` when the process should exit.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match interpret_key(key) {
            KeyAction::Quit => true,
            KeyAction::ClearQuery => {
                if !self.query.is_empty() {
                    self.query.clear();
                    self.apply_filter();
                }
                false
            }
            KeyAction::Move(delta) => {
                self.move_selection(delta);
                false
            }
            KeyAction::Activate => {
                self.activate_selected();
                false
            }
            KeyAction::Type(ch) => {
                self.query.push(ch);
                self.apply_filter();
                false
            }
            KeyAction::Backspace => {
                self.query.pop();
                self.apply_filter();
                false
            }
            KeyAction::Ignore => false,
        }
    }

    /// Periodic refresh / reconnect. Safe with no socket.
    pub fn tick(&mut self) {
        let now = Instant::now();
        if self.client.is_none() {
            if now >= self.next_reconnect {
                self.connect_or_schedule();
            }
            return;
        }
        if now.duration_since(self.last_refresh) >= REFRESH_EVERY {
            self.refresh_tree();
        }
    }

    /// Renderer snapshot.
    pub fn view(&self) -> PickerView<'_> {
        let rows = self
            .filtered
            .iter()
            .map(|filtered| VisibleRow {
                row: &self.rows[filtered.row_index],
                match_positions: &filtered.match_positions,
            })
            .collect();
        PickerView {
            query: &self.query,
            rows,
            selected: self.selected,
            total_rows: self.rows.len(),
            status: &self.status,
        }
    }

    /// Current status (tests).
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Current query (tests).
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Selected workspace, if any (tests).
    pub fn selected_row(&self) -> Option<&WorkspaceRow> {
        self.filtered
            .get(self.selected)
            .map(|filtered| &self.rows[filtered.row_index])
    }

    fn try_connect(&mut self, socket_path: PathBuf) {
        self.socket_path = Some(socket_path.clone());
        match CmuxClient::connect(ClientConfig::from_socket_path(socket_path)) {
            Ok(mut client) => match client.identify().and_then(|_| client.list_workspaces()) {
                Ok(tree) => self.become_ready(client, tree),
                Err(err) => self.disconnect_with_backoff(format!("cmux did not respond: {err}")),
            },
            Err(err) => self.disconnect_with_backoff(format!("cannot connect to cmux: {err}")),
        }
    }

    fn become_ready(&mut self, client: CmuxClient, tree: Tree) {
        self.client = Some(client);
        self.status = Status::Ready;
        self.reconnect_delay = INITIAL_RECONNECT_DELAY;
        self.last_refresh = Instant::now();
        self.replace_rows(rows_from_tree(&tree));
    }

    fn refresh_tree(&mut self) {
        let result = match self.client.as_mut() {
            Some(client) => client.list_workspaces(),
            None => return,
        };
        match result {
            Ok(tree) => {
                self.last_refresh = Instant::now();
                self.replace_rows(rows_from_tree(&tree));
            }
            Err(err) => self.disconnect(format!("cmux socket dropped: {err}")),
        }
    }

    fn activate_selected(&mut self) {
        let Some(row) = self.selected_row().cloned() else {
            return;
        };
        let result = match self.client.as_mut() {
            Some(client) => client.select_workspace(Some(row.index), None),
            None => return,
        };
        match result {
            Ok(()) => self.refresh_tree(),
            Err(err) => self.disconnect(format!("cmux command failed: {err}")),
        }
    }

    fn disconnect(&mut self, message: String) {
        self.client = None;
        self.disconnect_with_backoff(message);
    }

    fn disconnect_with_backoff(&mut self, message: String) {
        self.status = Status::Reconnecting { message };
        self.next_reconnect = Instant::now() + self.reconnect_delay;
        self.reconnect_delay = (self.reconnect_delay * 2).min(MAX_RECONNECT_DELAY);
    }

    fn replace_rows(&mut self, rows: Vec<WorkspaceRow>) {
        self.rows = rows;
        self.apply_filter();
    }

    fn apply_filter(&mut self) {
        let matches: Vec<(usize, MatchResult)> =
            filter_titles(&self.rows, &self.query, |row| row.title.as_str());
        self.filtered = matches
            .into_iter()
            .map(|(row_index, matched)| FilteredRow {
                row_index,
                match_positions: matched.positions,
            })
            .collect();
        if self.filtered.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.filtered.len() - 1);
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let len = self.filtered.len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        self.selected = self.selected.saturating_add_signed(delta).min(len - 1);
    }
}

impl Default for Picker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sidebar::keys::interpret_key;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn row(title: &str, index: usize) -> WorkspaceRow {
        WorkspaceRow {
            index,
            id: index as u64 + 1,
            title: title.to_string(),
            active: index == 0,
            screen_count: 1,
        }
    }

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn missing_socket_is_reconnect_not_panic() {
        let mut picker = Picker::new();
        picker.disconnect_with_backoff(missing_socket_message());
        match picker.status() {
            Status::Reconnecting { message } => {
                assert!(message.contains("CMUX_TUI_SOCKET"));
            }
            Status::Ready => panic!("expected reconnecting"),
        }
        let view = picker.view();
        assert!(view.rows.is_empty());
    }

    #[test]
    fn esc_clears_query_without_quitting() {
        let mut picker = Picker::with_rows(vec![row("accounting", 0), row("inbox", 1)]);
        assert!(!picker.handle_key(press(KeyCode::Char('a'))));
        assert_eq!(picker.query(), "a");
        assert!(!picker.handle_key(press(KeyCode::Esc)));
        assert_eq!(picker.query(), "");
        assert!(matches!(picker.status(), Status::Ready));
    }

    #[test]
    fn ctrl_c_requests_exit() {
        let mut picker = Picker::disconnected("offline");
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(picker.handle_key(key));
        assert_eq!(interpret_key(key), KeyAction::Quit);
    }

    #[test]
    fn filter_and_arrows_keep_selection_in_range() {
        let mut picker = Picker::with_rows(vec![row("accounting", 0), row("inbox", 1)]);
        picker.handle_key(press(KeyCode::Down));
        assert_eq!(
            picker.selected_row().map(|r| r.title.as_str()),
            Some("inbox")
        );
        picker.handle_key(press(KeyCode::Char('a')));
        picker.handle_key(press(KeyCode::Char('c')));
        assert_eq!(
            picker.selected_row().map(|r| r.title.as_str()),
            Some("accounting")
        );
        picker.handle_key(press(KeyCode::Down));
        assert_eq!(
            picker.selected_row().map(|r| r.title.as_str()),
            Some("accounting")
        );
    }
}
