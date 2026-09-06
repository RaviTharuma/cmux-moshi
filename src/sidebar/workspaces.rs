//! Workspace rows for the optional picker.
//!
//! Rows use the friendly workspace title from the mux tree. This picker
//! does not flatten screens or panes; that chrome belongs to a dedicated
//! fuzzy-finder plugin, not this host-integration package.

use cmux_client::{Tree, Workspace};

/// One selectable workspace in the sidebar picker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceRow {
    /// Index passed to `select_workspace`.
    pub index: usize,
    /// Mux workspace id.
    pub id: u64,
    /// Friendly title shown in cmux.
    pub title: String,
    /// True when this workspace is the mux active workspace.
    pub active: bool,
    /// Number of screens (shown as an attach hint).
    pub screen_count: usize,
}

impl WorkspaceRow {
    /// Builds a row from a mux workspace and its list index.
    pub fn from_workspace(index: usize, workspace: &Workspace) -> Self {
        Self {
            index,
            id: workspace.id,
            title: workspace.name.clone(),
            active: workspace.active,
            screen_count: workspace.screens.len(),
        }
    }

    /// Compact hint: active marker and screen count.
    pub fn hint(&self) -> String {
        let screens = match self.screen_count {
            1 => "1 screen".to_string(),
            n => format!("{n} screens"),
        };
        if self.active {
            format!("active · {screens}")
        } else {
            screens
        }
    }
}

/// Extracts workspace-only rows from a mux tree.
pub fn rows_from_tree(tree: &Tree) -> Vec<WorkspaceRow> {
    tree.workspaces
        .iter()
        .enumerate()
        .map(|(index, workspace)| WorkspaceRow::from_workspace(index, workspace))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmux_client::{Layout, Screen, Workspace};

    fn workspace(id: u64, name: &str, active: bool, screens: usize) -> Workspace {
        Workspace {
            id,
            name: name.to_string(),
            active,
            screens: (0..screens)
                .map(|i| Screen {
                    id: 10 + i as u64,
                    name: Some(format!("s{i}")),
                    active: i == 0,
                    active_pane: 1,
                    layout: Layout::Leaf { pane: 1 },
                    panes: Vec::new(),
                })
                .collect(),
        }
    }

    #[test]
    fn lists_friendly_titles_without_screen_or_pane_rows() {
        let tree = Tree {
            workspaces: vec![
                workspace(1, "accounting", true, 2),
                workspace(2, "inbox", false, 1),
            ],
        };
        let rows = rows_from_tree(&tree);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title, "accounting");
        assert_eq!(rows[0].index, 0);
        assert!(rows[0].active);
        assert_eq!(rows[0].hint(), "active · 2 screens");
        assert_eq!(rows[1].title, "inbox");
        assert_eq!(rows[1].hint(), "1 screen");
    }
}
