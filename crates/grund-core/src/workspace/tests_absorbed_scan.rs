//! Test module: a `[workspace]` block whose members cover every one of its own
//! walk roots (§FS-workspace.2.1), the config error it became in grund 0.16.0
//! (§FS-check.3.30).
//!
//! The behaviour itself is pinned end to end, in `tests/e2e/cases/`, because the
//! error is a property of a whole run rather than of one function: it is raised
//! where a run populates a block's member boundary, so `list`, `check`, `refs`,
//! `cover` and `fmt` each refuse with it. What is left for a unit test is that
//! the sentence is assembled from the covered pairs and that the spec shows the
//! bytes the golden pins.

use std::path::{Path, PathBuf};

/// The case whose golden holds the shipped message byte for byte.
const GOLDEN: &str = "tests/e2e/cases/workspace-member-absorbs-scan-list/expected.stderr";

/// The spec point that documents the same message as a worked example.
const SPEC: &str = "docs/functional-spec/FS-check.md";

fn repo_file(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

/// Absent when the tests run from a packaged crate rather than the
/// workspace; in the repository — where the promise can be broken — both
/// files are always present.
fn repo_text(relative: &str) -> Option<String> {
    std::fs::read_to_string(repo_file(relative)).ok()
}

/// §FS-check.3.30.1, §FS-workspace.2.1.1: the whole sentence, assembled from
/// the covered pairs the rule found — each covered root named beside the member
/// entry it is inside — with the golden's `members`-line breadcrumb taken off
/// the front. Held here as well as end to end because this is where a failure
/// names the sentence rather than a whole run's stderr.
#[test]
fn the_message_is_assembled_from_the_covered_pairs() {
    let Some(golden) = repo_text(GOLDEN) else {
        return;
    };
    let shipped = golden.trim_end_matches('\n');
    let sentence = shipped
        .strip_prefix("error: grund.toml:16: ")
        .unwrap_or_else(|| panic!("{GOLDEN} is not the `members`-line config error:\n{shipped}"));
    assert_eq!(
        super::findings::absorbed_scan_warning(&["`docs` in `docs`".to_string()]),
        sentence
    );
}

/// §FS-check.3.30.1: the message the spec shows and the message the binary
/// prints are one string. Without this the release the error landed in could be
/// kept in the golden and stale in the document a reader reaches by citation.
#[test]
fn the_documented_message_is_the_shipped_message() {
    let (Some(golden), Some(spec)) = (repo_text(GOLDEN), repo_text(SPEC)) else {
        return;
    };
    let shipped = golden.trim_end_matches('\n');
    assert!(
        spec.lines().any(|line| line == shipped),
        "{SPEC} does not show the error {GOLDEN} pins:\n{shipped}"
    );
}
