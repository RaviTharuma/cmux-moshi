//! Tmux session-name helpers.

/// Returns true when the session still uses a default TTY-derived name (`ttys` + digits).
pub fn is_default_tty_session(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("ttys") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

/// Turns a cmux workspace title into a tmux-safe session name.
///
/// tmux rejects `.` and `:` in session names. Spaces and path separators become `-`.
pub fn sanitize_session_name(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut last_dash = false;
    for ch in title.chars() {
        let mapped = match ch {
            c if c.is_ascii_alphanumeric() || c == '_' => Some(c),
            '.' | ':' | '/' | '\\' | ' ' | '\t' | '\n' | '\r' | '-' => Some('-'),
            _ => Some('-'),
        };
        if let Some(c) = mapped {
            if c == '-' {
                if !last_dash && !out.is_empty() {
                    out.push('-');
                    last_dash = true;
                }
            } else {
                out.push(c);
                last_dash = false;
            }
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("workspace");
    }
    if out.len() > 80 {
        out.truncate(80);
        while out.ends_with('-') {
            out.pop();
        }
    }
    out
}

/// Picks a unique session name, appending a short workspace-id suffix on collision.
pub fn unique_session_name(base: &str, workspace_id: Option<&str>, taken: &[String]) -> String {
    if !taken.iter().any(|name| name == base) {
        return base.to_string();
    }
    let suffix = workspace_id
        .map(short_id)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "2".to_string());
    let candidate = format!("{base}-{suffix}");
    if !taken.iter().any(|name| name == &candidate) {
        return candidate;
    }
    for n in 3..100 {
        let candidate = format!("{base}-{n}");
        if !taken.iter().any(|name| name == &candidate) {
            return candidate;
        }
    }
    format!("{base}-x")
}

fn short_id(workspace_id: &str) -> String {
    workspace_id
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(4)
        .collect::<String>()
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_ttys_default_names_only() {
        assert!(is_default_tty_session("ttys001"));
        assert!(is_default_tty_session("ttys012"));
        assert!(!is_default_tty_session("ttys"));
        assert!(!is_default_tty_session("ttysABC"));
        assert!(!is_default_tty_session("accounting"));
        assert!(!is_default_tty_session("tty001"));
    }

    #[test]
    fn sanitizes_titles_for_tmux() {
        assert_eq!(sanitize_session_name("accounting"), "accounting");
        assert_eq!(
            sanitize_session_name("client call: prep"),
            "client-call-prep"
        );
        assert_eq!(sanitize_session_name("  ..  "), "workspace");
        assert_eq!(sanitize_session_name("docs/v2"), "docs-v2");
    }

    #[test]
    fn unique_name_appends_suffix_on_collision() {
        let taken = vec!["accounting".to_string()];
        assert_eq!(
            unique_session_name("accounting", Some("A1B2C3"), &taken),
            "accounting-a1b2"
        );
        assert_eq!(unique_session_name("inbox", Some("abcd"), &[]), "inbox");
    }
}
