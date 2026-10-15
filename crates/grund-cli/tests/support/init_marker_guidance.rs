//! The fresh config's one comment (§FS-init.2.4.5) through the ordinary e2e
//! runner: the marker is a default, so `grund config show` names it, not the file.

use std::fs;
use std::path::PathBuf;

use super::case_runner::{CaseKind, assert_every_case_passed, run_case};

fn assert_fresh_comments(case: &str, comments: usize) {
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
    assert_eq!(
        config.lines().next(),
        Some(
            "# Only what differs from grund's defaults; `grund config show` prints every setting."
        ),
        "{case}: the first line says where every setting is printed"
    );
    assert!(
        !config.lines().any(|line| line.contains("marker")),
        "{case}: the marker is left at its default, unwritten and unannotated:\n{config}"
    );
    let written = config.lines().filter(|line| line.starts_with('#')).count();
    assert_eq!(written, comments, "{case}: comment lines:\n{config}");
    assert_every_case_passed("fresh config comments", &[outcome]);
}

/// §FS-init.2.4.5: the default template keeps its first-line comment alone.
#[test]
fn fresh_default_config_keeps_one_comment() {
    assert_fresh_comments("init-bare-empty-repo", 1);
}

/// §FS-init.2.4.5, §FS-init.2.4.9: the `none` template adds the two-line
/// comment on declaring a first kind, and nothing about the marker.
#[test]
fn fresh_none_config_keeps_its_comments() {
    assert_fresh_comments("init-template-none", 3);
}
