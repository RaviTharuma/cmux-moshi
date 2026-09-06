//! Sidebar binary stays usable without a mux socket.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cmux-moshi-sidebar"))
}

#[test]
fn probe_without_socket_exits_zero() {
    let output = bin()
        .arg("--probe")
        .env_remove("CMUX_TUI_SOCKET")
        .env_remove("CMUX_MUX_SOCKET")
        .output()
        .expect("probe");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status=missing"), "{stdout}");
    assert!(stdout.contains("CMUX_TUI_SOCKET"), "{stdout}");
}

#[test]
fn probe_reports_legacy_socket_env() {
    let output = bin()
        .arg("--probe")
        .env_remove("CMUX_TUI_SOCKET")
        .env("CMUX_MUX_SOCKET", "/tmp/cmux-moshi-test.sock")
        .output()
        .expect("probe legacy");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status=resolved"), "{stdout}");
    assert!(stdout.contains("/tmp/cmux-moshi-test.sock"), "{stdout}");
}

#[test]
fn help_mentions_packaging_not_moshi_panel() {
    let output = bin().arg("--help").output().expect("help");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("cmux-moshi-sidebar"));
    assert!(text.contains("probe"));
}
