//! Fresh marker guidance (§FS-init.2.4.5) through the ordinary e2e runner.

use std::fs;
use std::path::PathBuf;

use super::case_runner::{CaseKind, assert_every_case_passed, run_case};

fn assert_marker_guidance(case: &str) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let outcome = run_case(
        &root,
        &root.join("tests/e2e/cases").join(case),
        CaseKind::E2e,
    );
    let config = fs::read_to_string(
        root.join("target/e2e-work")
            .join(case)
            .join("repo/grund.toml"),
    )
    .expect("read freshly generated config");
    let marker_lines = config
        .lines()
        .filter(|line| line.starts_with("marker ="))
        .collect::<Vec<_>>();
    assert_eq!(marker_lines.len(), 1, "{case}: one marker setting");
    let actual = marker_lines[0];
    assert_eq!(
        actual.split('#').next().unwrap().trim(),
        "marker = \"§\"",
        "{case}: the configured marker must remain the built-in default"
    );
    assert_eq!(
        actual, "marker = \"§\" # § | ∵ | ⊢ | ※ | ⏚ | any rare character",
        "{case}: fresh config must teach the open-ended marker examples"
    );
    assert_every_case_passed("fresh marker guidance", &[outcome]);
}

/// §FS-init.2.4.5: the default template teaches marker choices inline.
#[test]
fn fresh_default_config_names_marker_alternatives() {
    assert_marker_guidance("init-bare-empty-repo");
}

/// §FS-init.2.4.5: the `none` template inherits the same marker guidance.
#[test]
fn fresh_none_config_names_marker_alternatives() {
    assert_marker_guidance("init-template-none");
}
