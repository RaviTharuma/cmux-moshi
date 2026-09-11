//! Optional login-shell snippet for Moshi clients.

use crate::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Marker start written to the rc file.
pub const BEGIN_MARKER: &str = "# >>> cmux-moshi begin";
/// Marker end written to the rc file.
pub const END_MARKER: &str = "# <<< cmux-moshi end";

/// Snippet that execs the dashboard when Moshi exported a truthy `MOSHI_CLIENT`.
pub fn snippet() -> String {
    format!(
        "{BEGIN_MARKER}
# Official cmux-moshi login-shell dashboard for Moshi clients.
# Moshi Settings → MOSHI_CLIENT env toggle exports MOSHI_CLIENT=1.
case \"${{MOSHI_CLIENT:-}}\" in
  1|true|TRUE|yes|YES|on|ON)
    if command -v cmux-moshi >/dev/null 2>&1; then
      exec cmux-moshi dashboard
    fi
    ;;
esac
{END_MARKER}
"
    )
}

/// Result of installing or removing the snippet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellChange {
    /// rc file that was edited.
    pub rc_path: PathBuf,
    /// Backup path, if a backup was written.
    pub backup_path: Option<PathBuf>,
    /// Human summary.
    pub message: String,
}

/// Inserts the snippet at the top of `rc_path`, backing up first.
///
/// Idempotent when the current snippet is already installed. If an older
/// marked block is present (for example the pre-0.2.1 `!= 0` gate), replaces
/// it in place so truthy `MOSHI_CLIENT` matching stays aligned with doctor /
/// dashboard.
pub fn install(rc_path: &Path, now: SystemTime) -> Result<ShellChange, Error> {
    let existing = if rc_path.exists() {
        fs::read_to_string(rc_path)?
    } else {
        String::new()
    };
    let desired = snippet();
    if let Some(current) = extract_snippet_block(&existing) {
        if normalize_snippet_text(&current) == normalize_snippet_text(&desired) {
            return Ok(ShellChange {
                rc_path: rc_path.to_path_buf(),
                backup_path: None,
                message: format!("snippet already present in {}", rc_path.display()),
            });
        }
        let backup = backup_path(rc_path, now);
        fs::copy(rc_path, &backup)?;
        let stripped = strip_snippet(&existing);
        let next = prepend_snippet(&desired, &stripped);
        fs::write(rc_path, next)?;
        return Ok(ShellChange {
            rc_path: rc_path.to_path_buf(),
            backup_path: Some(backup),
            message: format!("upgraded dashboard snippet in {}", rc_path.display()),
        });
    }
    let backup = backup_path(rc_path, now);
    if rc_path.exists() {
        fs::copy(rc_path, &backup)?;
    } else if let Some(parent) = rc_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let next = prepend_snippet(&desired, &existing);
    fs::write(rc_path, next)?;
    Ok(ShellChange {
        rc_path: rc_path.to_path_buf(),
        backup_path: Some(backup),
        message: format!("installed dashboard snippet into {}", rc_path.display()),
    })
}

fn prepend_snippet(desired: &str, existing: &str) -> String {
    let mut next = desired.to_string();
    if !existing.is_empty() {
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(existing);
        if !existing.ends_with('\n') {
            next.push('\n');
        }
    }
    next
}

/// Returns the marked snippet block including begin/end markers, if present.
pub fn extract_snippet_block(source: &str) -> Option<String> {
    let mut out = String::new();
    let mut copying = false;
    let mut found_end = false;
    for line in source.lines() {
        if line.trim() == BEGIN_MARKER {
            copying = true;
            out.push_str(BEGIN_MARKER);
            out.push('\n');
            continue;
        }
        if line.trim() == END_MARKER {
            if copying {
                out.push_str(END_MARKER);
                out.push('\n');
                found_end = true;
            }
            break;
        }
        if copying {
            out.push_str(line);
            out.push('\n');
        }
    }
    if found_end {
        Some(out)
    } else {
        None
    }
}

fn normalize_snippet_text(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Removes the marked snippet block.
pub fn uninstall(rc_path: &Path, now: SystemTime) -> Result<ShellChange, Error> {
    if !rc_path.exists() {
        return Ok(ShellChange {
            rc_path: rc_path.to_path_buf(),
            backup_path: None,
            message: format!("{} does not exist", rc_path.display()),
        });
    }
    let existing = fs::read_to_string(rc_path)?;
    if !existing.contains(BEGIN_MARKER) {
        return Ok(ShellChange {
            rc_path: rc_path.to_path_buf(),
            backup_path: None,
            message: format!("no cmux-moshi snippet in {}", rc_path.display()),
        });
    }
    let backup = backup_path(rc_path, now);
    fs::copy(rc_path, &backup)?;
    let stripped = strip_snippet(&existing);
    fs::write(rc_path, stripped)?;
    Ok(ShellChange {
        rc_path: rc_path.to_path_buf(),
        backup_path: Some(backup),
        message: format!("removed dashboard snippet from {}", rc_path.display()),
    })
}

/// Drops the marked block, preserving surrounding text.
pub fn strip_snippet(source: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;
    for line in source.lines() {
        if line.trim() == BEGIN_MARKER {
            skipping = true;
            continue;
        }
        if line.trim() == END_MARKER {
            skipping = false;
            continue;
        }
        if !skipping {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn backup_path(rc_path: &Path, now: SystemTime) -> PathBuf {
    let secs = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let name = rc_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("zshrc");
    rc_path.with_file_name(format!("{name}.bak-{secs}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn install_is_idempotent_and_uninstall_restores_user_text() {
        let dir = tempfile::tempdir().unwrap();
        let rc = dir.path().join(".zshrc");
        fs::write(&rc, "export PATH=/tmp/bin:$PATH\n").unwrap();
        let t0 = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let first = install(&rc, t0).unwrap();
        assert!(first.backup_path.is_some());
        let text = fs::read_to_string(&rc).unwrap();
        assert!(text.contains(BEGIN_MARKER));
        assert!(text.contains("export PATH=/tmp/bin:$PATH"));
        assert!(text.contains("case \"${MOSHI_CLIENT:-}\" in"));
        assert!(text.contains("1|true|TRUE|yes|YES|on|ON)"));
        assert!(!text.contains("[ \"${MOSHI_CLIENT}\" != \"0\" ]"));
        let again = install(&rc, t0).unwrap();
        assert!(again.message.contains("already present"));
        let removed = uninstall(&rc, UNIX_EPOCH + Duration::from_secs(1_700_000_001)).unwrap();
        assert!(removed.backup_path.is_some());
        let after = fs::read_to_string(&rc).unwrap();
        assert!(!after.contains(BEGIN_MARKER));
        assert!(after.contains("export PATH=/tmp/bin:$PATH"));
    }

    #[test]
    fn install_upgrades_stale_marked_snippet() {
        let dir = tempfile::tempdir().unwrap();
        let rc = dir.path().join(".zshrc");
        let stale = format!(
            "{BEGIN_MARKER}
# old gate
if [ \"${{MOSHI_CLIENT}}\" != \"0\" ]; then
  exec cmux-moshi dashboard
fi
{END_MARKER}
# keep me
"
        );
        fs::write(&rc, &stale).unwrap();
        let t0 = UNIX_EPOCH + Duration::from_secs(1_700_000_100);
        let change = install(&rc, t0).unwrap();
        assert!(change.message.contains("upgraded"));
        assert!(change.backup_path.is_some());
        let text = fs::read_to_string(&rc).unwrap();
        assert!(text.contains("1|true|TRUE|yes|YES|on|ON)"));
        assert!(!text.contains("[ \"${MOSHI_CLIENT}\" != \"0\" ]"));
        assert!(text.contains("# keep me"));
        let again = install(&rc, t0).unwrap();
        assert!(again.message.contains("already present"));
    }
}
