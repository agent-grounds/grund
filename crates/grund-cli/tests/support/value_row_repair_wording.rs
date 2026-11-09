//! The cases of a value row's repair, run by name: it says what removal costs and names the
//! rewrite (§FS-config-migrate.3.1.7.2).

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const CASES: [&str; 3] = [
    "config-migrate-refuse-value-chapter",
    "config-migrate-refuse-values-fetched",
    "config-migrate-refuse-values-guidance",
];

#[test]
fn value_row_repair_wording_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("value row repair wording", &outcomes);
}
