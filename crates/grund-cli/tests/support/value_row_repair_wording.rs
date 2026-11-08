//! Issue #1232's cases, run alone: the repair of a value row says what removal costs and names
//! the rewrite (§FS-config-migrate.3.1.7.2). They are held out of the full pass until the fix
//! lands, and implement removes the hold.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

/// Red until #1232: the full pass skips these; implement removes this.
pub const RED_UNTIL_1232: [&str; 3] = CASES;

const CASES: [&str; 3] = [
    "config-migrate-refuse-value-chapter",
    "config-migrate-refuse-values-fetched",
    "config-migrate-refuse-values-guidance",
];

#[test]
#[ignore = "red until #1232 says what removing a value key costs; implement removes this"]
fn value_row_repair_wording_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("value row repair wording", &outcomes);
}
