//! Issue #1212's cases, run alone: the `[scan] exclude` refusal says its second repair is for a
//! directory, because a file's name loads and excludes nothing (§FS-config.3.5.16). They are held
//! out of the full pass until the fix lands, and implement removes the hold.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

/// Red until #1212: the full pass skips these; implement removes this.
pub const RED_UNTIL_1212: [&str; 8] = CASES;

const CASES: [&str; 8] = [
    "config-scan-exclude-slash-file-check",
    "config-scan-exclude-slash-mixed-first-check",
    "config-scan-exclude-slash-mixed-first-validate",
    "config-scan-exclude-slash-mixed-last-check",
    "config-scan-exclude-slash-mixed-last-validate",
    "config-scan-exclude-slash-single-check",
    "config-scan-exclude-slash-single-validate",
    "config-scan-exclude-slash-two-entries-check",
];

#[test]
#[ignore = "red until #1212 says the `[scan] exclude` name repair is for a directory; implement removes this"]
fn scan_exclude_repair_wording_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("scan exclude repair wording", &outcomes);
}
