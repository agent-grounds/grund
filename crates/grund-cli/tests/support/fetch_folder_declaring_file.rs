//! Issue #1213's cases, run alone: a file in a `folder` home that fetch does not replace and that
//! declares the requested ID stops the fetch, with exit 2, the file named from the report base and
//! the tree byte-identical (§FS-fetch.5.2). They are held out of the full pass until the fix
//! lands. Implement removes the hold, and with it this module and its `mod` line in `e2e.rs`,
//! because the full pass then runs them. The other `fetch-folder-source-*` cases are guards that
//! pass today, so they stay in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

/// Red until #1213: the full pass skips these; implement removes this.
pub const RED_UNTIL_1213: [&str; 18] = CASES;

const CASES: [&str; 18] = [
    "fetch-folder-source-after-markdown-refused",
    "fetch-folder-source-before-markdown-refused",
    "fetch-folder-source-beside-other-content-refused",
    "fetch-folder-source-markdown-by-mapping-refused",
    "fetch-folder-source-stub-target-refused",
    "fetch-folder-source-two-files-refused",
    "fetch-folder-source-undecodable-beside-markdown-refused",
    "fetch-folder-source-undecodable-refused",
    "fetch-folder-source-v1-python-refused",
    "fetch-folder-source-v1-rust-refused",
    "fetch-folder-source-v1-txt-refused",
    "fetch-folder-source-v2-go-refused",
    "fetch-folder-source-v2-python-function-refused",
    "fetch-folder-source-v2-python-nested-refused",
    "fetch-folder-source-v2-python-refused",
    "fetch-folder-source-v2-ruby-refused",
    "fetch-folder-source-v2-rust-refused",
    "fetch-folder-source-workspace-member-refused",
];

#[test]
#[ignore = "red until #1213 refuses a file in the home that declares the ID; implement removes this"]
fn fetch_folder_declaring_file_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("fetch folder declaring file", &outcomes);
}
