//! Public `cmux` CLI / RPC client. Never talks to private frameworks.

use crate::error::Error;
use crate::host::Host;
use serde_json::{Map, Value};

/// A workspace title from cmux RPC.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceTitle {
    /// Workspace UUID or other stable id.
    pub id: String,
    /// Friendly title shown in cmux chrome.
    pub title: String,
}

/// Terminal row from `debug.terminals`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DebugTerminal {
    /// TTY name (`ttys001`) when present.
    pub tty: Option<String>,
    /// Workspace id.
    pub workspace_id: Option<String>,
    /// Friendly title when the RPC includes one.
    pub title: Option<String>,
    /// Surface id if present.
    pub surface_id: Option<String>,
}

/// Result of probing public cmux RPC methods.
#[derive(Clone, Debug, Default)]
pub struct RpcProbe {
    /// True when at least one method returned parseable JSON.
    pub reachable: bool,
    /// Human notes for `doctor`.
    pub notes: Vec<String>,
    /// Titles from `workspace.list` (or equivalent).
    pub workspaces: Vec<WorkspaceTitle>,
    /// Rows from `debug.terminals`.
    pub terminals: Vec<DebugTerminal>,
}

/// Invokes documented public `cmux` RPC methods and parses what is available.
pub fn probe_rpc(host: &dyn Host) -> RpcProbe {
    let mut probe = RpcProbe::default();
    if !host.which("cmux") {
        probe.notes.push("cmux is not on PATH".to_string());
        return probe;
    }

    match rpc_json(host, "debug.terminals") {
        Ok(value) => {
            probe.terminals = parse_debug_terminals(&value);
            probe.reachable = true;
            probe.notes.push(format!(
                "cmux rpc debug.terminals: ok ({} terminal rows)",
                probe.terminals.len()
            ));
        }
        Err(err) => probe.notes.push(format!("cmux rpc debug.terminals: {err}")),
    }

    match rpc_json(host, "workspace.list") {
        Ok(value) => {
            probe.workspaces = parse_workspace_list(&value);
            probe.reachable = true;
            probe.notes.push(format!(
                "cmux rpc workspace.list: ok ({} workspaces)",
                probe.workspaces.len()
            ));
        }
        Err(err) => probe.notes.push(format!("cmux rpc workspace.list: {err}")),
    }

    if !probe.reachable {
        match host.run("cmux", &["identify", "--json"]) {
            Ok(out) if out.success() && looks_like_json(&out.stdout) => {
                probe.reachable = true;
                probe.notes.push("cmux identify --json: ok".to_string());
            }
            Ok(out) => probe.notes.push(format!(
                "cmux identify --json: exit {} ({})",
                out.status,
                first_line(&out.stderr).unwrap_or("no JSON")
            )),
            Err(err) => probe.notes.push(format!("cmux identify --json: {err}")),
        }
    }

    probe
}

/// Calls `cmux rpc <method>` and unwraps a JSON result object.
pub fn rpc_json(host: &dyn Host, method: &str) -> Result<Value, Error> {
    let attempts: [&[&str]; 2] = [&["rpc", method], &["rpc", "--method", method]];
    let mut last = None;
    for args in attempts {
        match host.run("cmux", args) {
            Ok(out) if out.success() => {
                return parse_rpc_stdout(&out.stdout);
            }
            Ok(out) => {
                last = Some(Error::command(
                    "cmux",
                    format!(
                        "{} failed: {}",
                        args.join(" "),
                        first_line(&out.stderr)
                            .or_else(|| first_line(&out.stdout))
                            .unwrap_or("error")
                    ),
                ));
            }
            Err(err) => last = Some(err),
        }
    }
    Err(last.unwrap_or_else(|| Error::command("cmux", format!("rpc {method} unavailable"))))
}

/// Parses cmux RPC stdout, accepting a bare object or `{ "result": ... }`.
pub fn parse_rpc_stdout(stdout: &str) -> Result<Value, Error> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err(Error::parse("cmux rpc returned empty stdout"));
    }
    let value: Value = serde_json::from_str(trimmed)
        .or_else(|_| first_json_object(trimmed))
        .map_err(|err| Error::parse(format!("cmux rpc JSON: {err}")))?;
    Ok(unwrap_result(value))
}

/// Extracts workspace id → title from a `workspace.list` payload.
pub fn parse_workspace_list(value: &Value) -> Vec<WorkspaceTitle> {
    let mut out = Vec::new();
    for item in iterate_named_array(value, &["workspaces", "items", "data"]) {
        if let Some(row) = workspace_from_value(item) {
            out.push(row);
        }
    }
    if out.is_empty() {
        if let Some(row) = workspace_from_value(value) {
            out.push(row);
        }
    }
    out
}

/// Extracts terminal rows from a `debug.terminals` payload.
pub fn parse_debug_terminals(value: &Value) -> Vec<DebugTerminal> {
    iterate_named_array(value, &["terminals", "items", "data"])
        .filter_map(terminal_from_value)
        .collect()
}

/// Builds an id → title map from RPC probe data.
pub fn title_index(probe: &RpcProbe) -> HashMapIndex {
    let mut index = HashMapIndex::default();
    for ws in &probe.workspaces {
        index.by_id.insert(normalize_id(&ws.id), ws.title.clone());
    }
    for term in &probe.terminals {
        if let (Some(id), Some(title)) = (&term.workspace_id, &term.title) {
            index
                .by_id
                .entry(normalize_id(id))
                .or_insert_with(|| title.clone());
        }
        if let (Some(tty), Some(title)) = (&term.tty, &term.title) {
            index.by_tty.insert(normalize_tty(tty), title.clone());
        }
        if let (Some(tty), Some(id)) = (&term.tty, &term.workspace_id) {
            index.tty_to_id.insert(normalize_tty(tty), id.clone());
        }
    }
    index
}

/// Lookups collected from cmux RPC.
#[derive(Clone, Debug, Default)]
pub struct HashMapIndex {
    /// Workspace id → title.
    pub by_id: std::collections::HashMap<String, String>,
    /// TTY name → title.
    pub by_tty: std::collections::HashMap<String, String>,
    /// TTY name → workspace id.
    pub tty_to_id: std::collections::HashMap<String, String>,
}

fn unwrap_result(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            if let Some(result) = map.get("result") {
                return result.clone();
            }
            Value::Object(map)
        }
        other => other,
    }
}

fn iterate_named_array<'a>(
    value: &'a Value,
    keys: &'a [&'a str],
) -> Box<dyn Iterator<Item = &'a Value> + 'a> {
    if let Some(arr) = value.as_array() {
        return Box::new(arr.iter());
    }
    if let Some(obj) = value.as_object() {
        for key in keys {
            if let Some(arr) = obj.get(*key).and_then(Value::as_array) {
                return Box::new(arr.iter());
            }
        }
    }
    Box::new(std::iter::empty())
}

fn workspace_from_value(value: &Value) -> Option<WorkspaceTitle> {
    let obj = value.as_object()?;
    let id = string_field(obj, &["id", "workspace_id", "workspaceId"])?;
    let title = string_field(obj, &["title", "name", "workspace_title", "workspaceTitle"])
        .unwrap_or_else(|| id.clone());
    Some(WorkspaceTitle { id, title })
}

fn terminal_from_value(value: &Value) -> Option<DebugTerminal> {
    let obj = value.as_object()?;
    Some(DebugTerminal {
        tty: string_field(obj, &["tty", "tty_name", "name"]),
        workspace_id: string_field(obj, &["workspace_id", "workspaceId", "id"]),
        title: string_field(
            obj,
            &[
                "workspace_title",
                "workspaceTitle",
                "title",
                "workspace_name",
            ],
        ),
        surface_id: string_field(obj, &["surface_id", "surfaceId"]),
    })
}

fn string_field(obj: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(Value::String(s)) = obj.get(*key) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn first_json_object(text: &str) -> Result<Value, serde_json::Error> {
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('{') || line.starts_with('[') {
            return serde_json::from_str(line);
        }
    }
    serde_json::from_str(text)
}

fn looks_like_json(text: &str) -> bool {
    let t = text.trim();
    t.starts_with('{') || t.starts_with('[')
}

fn first_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|s| !s.is_empty())
}

/// Normalizes workspace ids for map lookup.
pub fn normalize_id(id: &str) -> String {
    id.trim().to_ascii_lowercase()
}

fn normalize_tty(tty: &str) -> String {
    tty.trim().trim_start_matches("/dev/").to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwraps_result_envelope_and_parses_lists() {
        let raw = r#"{"ok":true,"result":{"workspaces":[{"id":"AAA","title":"accounting"}]}}"#;
        let value = parse_rpc_stdout(raw).unwrap();
        let list = parse_workspace_list(&value);
        assert_eq!(list[0].title, "accounting");

        let terminals = parse_debug_terminals(&serde_json::json!({
            "terminals": [{
                "tty": "/dev/ttys001",
                "workspace_id": "AAA",
                "title": "accounting"
            }]
        }));
        assert_eq!(terminals[0].tty.as_deref(), Some("/dev/ttys001"));
    }
}
