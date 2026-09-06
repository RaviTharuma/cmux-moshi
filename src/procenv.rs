//! Process environment parsers.

use std::collections::HashMap;

/// Parses a Linux `/proc/pid/environ` NUL-separated blob.
pub fn parse_nul_environ(raw: &[u8]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for chunk in raw.split(|b| *b == 0) {
        if chunk.is_empty() {
            continue;
        }
        if let Some((key, value)) = split_env_pair(&String::from_utf8_lossy(chunk)) {
            map.insert(key, value);
        }
    }
    map
}

/// Extracts `KEY=value` pairs from a `ps eww` command line dump.
pub fn parse_ps_eww_env(blob: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for token in tokenize_ps_eww(blob) {
        if let Some((key, value)) = split_env_pair(&token) {
            map.insert(key, value);
        }
    }
    map
}

/// Reads `CMUX_WORKSPACE_ID` from an env map (case-sensitive).
pub fn workspace_id_from_env(env: &HashMap<String, String>) -> Option<String> {
    env.get("CMUX_WORKSPACE_ID")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

fn split_env_pair(token: &str) -> Option<(String, String)> {
    let (key, value) = token.split_once('=')?;
    if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    Some((key.to_string(), value.to_string()))
}

fn tokenize_ps_eww(blob: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    for ch in blob.chars() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            c if c.is_whitespace() && !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nul_environ() {
        let raw = b"HOME=/tmp/x\0CMUX_WORKSPACE_ID=abc-123\0PATH=/bin\0";
        let map = parse_nul_environ(raw);
        assert_eq!(
            map.get("CMUX_WORKSPACE_ID").map(String::as_str),
            Some("abc-123")
        );
    }

    #[test]
    fn parses_ps_eww_workspace_id() {
        let blob = "tmux new-session -d HOME=/tmp/x CMUX_WORKSPACE_ID=ws-9 TERM=xterm";
        let map = parse_ps_eww_env(blob);
        assert_eq!(workspace_id_from_env(&map).as_deref(), Some("ws-9"));
    }
}
