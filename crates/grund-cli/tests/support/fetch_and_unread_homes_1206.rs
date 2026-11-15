//! Issue #1206's second pull request, its cases run alone: folder fetch follows how the
//! configuration reads a file (§FS-fetch.5.2, §FS-fetch.5.5, §FS-fetch.admission.1), a folder
//! home nothing in which is read is named (§FS-scan.checks.unread-scan-root.folder), and the
//! reason a name is not read says a data reading truly (§FS-scan.checks.unread-scan-root.message,
//! §FS-check.2.2.2.2, §FS-fields.forms). They are held out of the full pass until the fix lands,
//! and implement removes the hold.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

/// Red until #1206: the full pass skips these; implement removes this.
pub const RED_UNTIL_1206: [&str; 27] = CASES;

const CASES: [&str; 27] = [
    "check-empty-scan-v2-names-languages",
    "check-empty-scan-v2-names-md-mapping",
    "check-rule-folder-home-unread-cited",
    "check-rule-folder-home-unread-nolang",
    "check-rule-folder-home-unread-report",
    "check-stub-target-read-as-data-reason",
    "check-unread-root-data-language-defined",
    "check-unread-root-data-language-hidden",
    "check-unread-root-data-language-mapped",
    "check-unread-root-data-language-omitted",
    "check-unread-root-data-language-report",
    "check-unread-root-remapped-md-reason",
    "check-v1-folder-home-unread-warns",
    "check-v2-folder-home-unread-warns",
    "config-rule-home-refusal-names-entry-mapped",
    "config-rule-home-refusal-names-entry-second-entry",
    "fetch-file-home-unread-refused-v2-json",
    "fetch-folder-home-id-md-not-markdown-refused-mapped",
    "fetch-folder-home-md-read-as-source-refused",
    "fetch-folder-home-read-as-markdown-beside",
    "fetch-folder-home-read-as-markdown-nested",
    "fetch-folder-home-read-as-markdown-other-name",
    "fetch-folder-home-read-as-markdown-report",
    "fetch-folder-home-read-as-markdown-upper-case",
    "fetch-folder-source-markdown-by-mapping-replaced",
    "fetch-folder-source-stub-target-is-the-snapshot-replaced",
    "fetch-folder-source-stub-under-another-name-refused",
];

#[test]
#[ignore = "red until #1206 has folder fetch and the unread-home reasons follow the reading; implement removes this"]
fn fetch_and_unread_homes_1206_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("fetch and unread homes (#1206)", &outcomes);
}
