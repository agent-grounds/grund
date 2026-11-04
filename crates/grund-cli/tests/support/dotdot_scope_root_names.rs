//! Issue #1169's cases, run alone: a version-1 `[scan] include` entry spelled through `..` is
//! named by its lexically folded path under the block root, `.` for the block root itself, in
//! the unread-workspace-block warning and the absorbed-scan error alike (§FS-check.3.30.1,
//! §FS-check.4.10.5).

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const CASES: [&str; 4] = [
    "workspace-include-root-false-unread-dotdot-block-root",
    "workspace-include-root-false-unread-dotdot-climb-back",
    "workspace-include-root-false-unread-dotdot-subtree",
    "workspace-member-absorbs-scan-dotdot",
];

#[test]
fn dotdot_scope_root_names_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("dotdot scope root names", &outcomes);
}
