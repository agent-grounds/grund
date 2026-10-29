//! The fresh config's one comment (§FS-init.2.4.5), in the sentence of
//! §FS-config.4.2.3.1, through the ordinary e2e runner: the marker is a default,
//! so `grund config show` names it, not the file.

use std::fs;
use std::path::PathBuf;

use super::case_runner::{CaseKind, assert_every_case_passed, run_case_inspecting};

// §FS-examples.5.6.1: read the generated config while its invocation owner lives.
fn assert_fresh_comments(case: &str, comments: usize) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut config = None;
    let outcome = run_case_inspecting(
        &root,
        &root.join("tests/e2e/cases").join(case),
        CaseKind::E2e,
        |repo| {
            config = Some(
                fs::read_to_string(repo.join("grund.toml")).expect("read freshly generated config"),
            );
        },
    );
    let config = config.expect("inspect this invocation's generated config");
    assert_eq!(
        config.lines().next(),
        Some(
            "# Only non-defaults; `grund config show` prints what applies, `grund config explain` every setting."
        ),
        "{case}: the first line says what `show` prints and names `explain` for the rest"
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
