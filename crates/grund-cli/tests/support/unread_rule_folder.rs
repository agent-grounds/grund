//! Issue #1183's cases, run alone: a rule kind's folder home the run reads no file of earns
//! the located `unread-scan-root` warning (§FS-scan.checks.unread-scan-root.rule-folder). Both
//! lists are also in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

/// The folders the run reads no file of: each one earns the warning.
const UNREAD: [&str; 11] = [
    "check-v2-rule-folder-md-read-as-json",
    "check-v2-rule-folder-md-read-as-json-cited",
    "check-v2-rule-folder-md-read-by-no-language",
    "check-rule-folder-unlisted-extension",
    "check-rule-folder-unlisted-extension-nested",
    "check-rule-folder-nested-in-another-home",
    "check-rule-folder-shared-with-another-home",
    "check-v2-rule-folder-unread-json",
    "check-v2-rule-folder-only-home",
    "check-v2-rule-folder-two-entries",
    "check-full-rule-folder-unread",
];

/// The controls: each of these earns no warning.
const CONTROLS: [&str; 11] = [
    "check-v2-rule-folder-read-as-markdown",
    "check-rule-folder-shared-read",
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
fn unread_rule_folder_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = UNREAD
        .iter()
        .chain(CONTROLS.iter())
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("unread rule folder", &outcomes);
}
