//! Official mux sidebar plugin manifest must stay installable.

use std::fs;

#[test]
fn cmux_plugin_toml_matches_official_shape() {
    let text = fs::read_to_string("cmux-plugin.toml").expect("cmux-plugin.toml");
    assert!(text.contains("name = \"moshi\""));
    assert!(text.contains("kind = \"sidebar\""));
    assert!(text.contains("version = \"0.3.0\""));
    assert!(text.contains("command = [\"target/release/cmux-moshi-sidebar\"]"));
    assert!(text.contains("command = [\"cargo\", \"build\", \"--release\"]"));
    assert!(!text.contains("bin/cmux-moshi-fetch"));
    assert!(!text.contains("bin/cmux-moshi"));
}

#[test]
fn version_files_agree() {
    let version = fs::read_to_string("VERSION").expect("VERSION");
    assert_eq!(version.trim(), "0.3.0");
    let cargo = fs::read_to_string("Cargo.toml").expect("Cargo.toml");
    assert!(cargo.contains("version = \"0.3.0\""));
}

#[test]
fn docs_do_not_claim_moshi_lives_in_the_sidebar() {
    let readme = fs::read_to_string("README.md").expect("README");
    assert!(readme.contains("never sees the cmux left sidebar"));
    assert!(readme.contains("plugin use"));
    assert!(readme.contains("cmux-tui/spec/plugins.md"));
    assert!(!readme.contains("custom sidebar into"));
}

#[test]
fn release_workflow_matches_fetch_asset_names() {
    let workflow = fs::read_to_string(".github/workflows/release.yml").expect("release.yml");
    let fetch = fs::read_to_string("bin/cmux-moshi-fetch").expect("fetch");
    assert!(
        workflow.contains("cmux-moshi-${VERSION}-${{ matrix.target }}")
            || workflow.contains("asset=cmux-moshi-${VERSION}-")
    );
    assert!(workflow.contains("SHA256SUMS"));
    for target in [
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
    ] {
        assert!(workflow.contains(target), "missing {target}");
        assert!(fetch.contains(target), "fetch missing {target}");
    }
    assert!(fetch.contains("SHA256SUMS"));
    let plugin = fs::read_to_string("cmux-plugin.toml").expect("plugin");
    assert!(!plugin.contains("cmux-moshi-fetch"));
}
