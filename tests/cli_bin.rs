//! Hermetic CLI binary checks. No hardcoded home paths.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cmux-moshi"))
}

#[test]
fn version_and_help_list_product_commands() {
    let version = bin().arg("--version").output().expect("version");
    assert!(version.status.success());
    let stdout = String::from_utf8_lossy(&version.stdout);
    assert!(stdout.contains("cmux-moshi"));
    assert!(stdout.contains("0.2.2"));

    let help = bin().arg("--help").output().expect("help");
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    for needle in [
        "doctor",
        "list",
        "sync",
        "dashboard",
        "cleanup",
        "install-shell",
        "uninstall-shell",
    ] {
        assert!(text.contains(needle), "help missing {needle}: {text}");
    }
}

#[test]
fn dashboard_refuses_without_moshi_or_force() {
    let output = bin()
        .arg("dashboard")
        .env_remove("MOSHI_CLIENT")
        .output()
        .expect("dashboard");
    assert!(!output.status.success());
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("MOSHI_CLIENT"), "{err}");
}

#[test]
fn install_and_uninstall_shell_use_temp_rc() {
    let dir = tempfile::tempdir().unwrap();
    let rc = dir.path().join(".zshrc");
    std::fs::write(&rc, "# user config\n").unwrap();

    let install = bin()
        .args(["install-shell", "--rc-file"])
        .arg(&rc)
        .output()
        .expect("install-shell");
    assert!(
        install.status.success(),
        "{}",
        String::from_utf8_lossy(&install.stderr)
    );
    let text = std::fs::read_to_string(&rc).unwrap();
    assert!(text.contains("cmux-moshi dashboard"));
    assert!(text.contains("# user config"));

    let uninstall = bin()
        .args(["uninstall-shell", "--rc-file"])
        .arg(&rc)
        .output()
        .expect("uninstall-shell");
    assert!(uninstall.status.success());
    let after = std::fs::read_to_string(&rc).unwrap();
    assert!(!after.contains(">>> cmux-moshi begin"));
    assert!(after.contains("# user config"));
}

#[test]
fn doctor_runs_without_host_tools() {
    let output = bin()
        .arg("doctor")
        .env("PATH", dir_without_cmux())
        .env_remove("MOSHI_CLIENT")
        .output()
        .expect("doctor");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cmux-moshi doctor"));
    assert!(stdout.contains("MOSHI_CLIENT env toggle"));
    assert!(stdout.contains("cmux sidebar plugin install"));
}

fn dir_without_cmux() -> String {
    let dir = tempfile::tempdir().unwrap();
    // Keep the temp dir alive for the process by leaking it; PATH must exist.
    let path = dir.path().to_path_buf();
    std::mem::forget(dir);
    path.display().to_string()
}
