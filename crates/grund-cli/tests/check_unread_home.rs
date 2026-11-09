//! §FS-check.2.4: a citation of a fetching kind earns no finding while this run's own `error:`
//! lines hold a failure at or under the kind's home.
//!
//! Each case is an e2e fixture held out of the sweep while it is red, because the sweep has no
//! per-case skip and the commit hook runs it. The fix moves every directory under
//! `tests/e2e/held-1215/` to `tests/e2e/cases/` and deletes this file.

use std::path::PathBuf;

#[path = "support/case_runner.rs"]
#[allow(dead_code)]
mod case_runner;

use case_runner::{CaseKind, assert_every_case_passed, run_case};

fn held(name: &str) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let case = root.join("tests/e2e/held-1215").join(name);
    assert_every_case_passed(name, &[run_case(&root, &case, CaseKind::E2e)]);
}

#[test]
#[ignore = "red until #1215 withholds the finding; implement removes this"]
fn a_value_source_that_does_not_parse_withholds() {
    held("check-unread-home-value-source-withholds");
}

#[test]
#[ignore = "red until #1215 withholds the finding; implement removes this"]
fn a_markdown_source_that_is_not_utf8_withholds() {
    held("check-unread-home-markdown-source-withholds");
}

#[test]
#[ignore = "red until #1215 withholds the finding; implement removes this"]
fn a_file_home_that_does_not_parse_withholds() {
    held("check-unread-home-file-home-withholds");
}

#[test]
#[ignore = "red until #1215 withholds the finding; implement removes this"]
fn must_resolution_withholds_too() {
    held("check-unread-home-must-withholds");
}

#[test]
#[ignore = "red until #1215 withholds the finding; implement removes this"]
fn a_failure_in_the_target_member_withholds() {
    held("check-unread-home-workspace-member-withholds");
}
