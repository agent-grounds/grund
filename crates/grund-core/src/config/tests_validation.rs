//! Test module: configuration validation (§AR-core-module-layout.1.5)

use super::load_config;
use crate::testing::{test_root, write};

/// §FS-config.3.1.8: the load-time invariant the three budget keys carry —
/// `inline_note_suggested_lines ≤ inline_note_max_lines` — so a config that
/// suggests more lines than it permits is refused rather than loaded.
#[test]
fn inline_note_config_rejects_soft_cap_above_hard_cap() {
    let root = test_root("inline_note_config_rejects_soft_cap_above_hard_cap");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\ninline_note_suggested_lines = 4\ninline_note_max_lines = 3\n",
    );

    let err = match load_config(&root) {
        Ok(_) => panic!("invalid inline-note caps should fail"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("reference.inline_note_suggested_lines must be <= inline_note_max_lines"),
        "unexpected error: {err:#}"
    );
}

/// §FS-config.3: `project_description` parses as optional one-line
/// top-level metadata next to `project_name`.
#[test]
fn config_parses_project_description() {
    let root = test_root("config_parses_project_description");
    write(
        &root.join("grund.toml"),
        "project_name = \"api\"\nproject_description = \"Payment API service\"\n",
    );

    let config = load_config(&root).expect("load config");
    assert_eq!(
        config.project_description.as_deref(),
        Some("Payment API service")
    );
}

/// §FS-config.3: a `project_description` with an embedded line break is a
/// config error at the offending line — the key feeds single-line
/// workspace member bullets.
#[test]
fn config_rejects_multiline_project_description() {
    let root = test_root("config_rejects_multiline_project_description");
    write(
        &root.join("grund.toml"),
        "project_description = \"first\\nsecond\"\n",
    );

    let err = match load_config(&root) {
        Ok(_) => panic!("multi-line project_description should fail"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("project_description must be a single line"),
        "unexpected error: {err:#}"
    );
}
