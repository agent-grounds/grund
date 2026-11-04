//! Issue #1183's cases, run alone: a rule kind's folder home the run reads no file of earns
//! the located `unread-scan-root` warning (§FS-scan.checks.unread-scan-root.rule-folder). The
//! controls are also in the full pass; the red cases are held out of it until the fix lands,
//! and implement removes the hold and the mark.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;
use std::path::PathBuf;

/// The red cases, until #1183: the full pass skips them; implement removes this.
pub const RED_UNTIL_1183: [&str; 9] = [
    "check-v2-rule-folder-md-read-as-json",
    "check-v2-rule-folder-md-read-as-json-cited",
    "check-v2-rule-folder-md-read-by-no-language",
    "check-rule-folder-unlisted-extension",
    "check-rule-folder-unlisted-extension-nested",
    "check-v2-rule-folder-unread-json",
    "check-v2-rule-folder-only-home",
    "check-v2-rule-folder-two-entries",
    "check-full-rule-folder-unread",
];

/// The full pass's cases without the red ones, until #1183: implement removes this.
pub fn without_red(cases: Vec<PathBuf>) -> Vec<PathBuf> {
    cases
        .into_iter()
        .filter(|case| !RED_UNTIL_1183.iter().any(|red| case.ends_with(red)))
        .collect()
}

/// The controls: they pass before and after the fix, so they carry no mark.
const CONTROLS: [&str; 10] = [
    "check-v2-rule-folder-read-as-markdown",
    "check-v2-rule-folder-absent",
    "check-v2-rule-folder-hidden-only",
    "check-v2-rule-folder-ignored-only",
    "check-v2-rule-folder-excluded-subdirectory",
    "check-v2-rule-folder-mixed-read",
    "check-v2-rule-folder-md-as-json-beside-rust",
    "check-v2-rule-folder-read-as-rust",
    "check-v2-folder-home-reads-nothing",
    "list-v2-rule-folder-unread",
];

#[test]
#[ignore = "red until #1183 names a rule folder the run reads no file of; implement removes this"]
fn unread_rule_folder_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = RED_UNTIL_1183
        .iter()
        .chain(CONTROLS.iter())
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("unread rule folder", &outcomes);
}
