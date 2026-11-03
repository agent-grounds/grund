//! Issue #1141's cases, run alone: an absent value home is refused with the hint that
//! names the way through (§FS-fetch.4.1), and a plain kind's absent file home is created
//! by fetch (§FS-fetch.4). Every one is also in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const CASES: [&str; 3] = [
    "fetch-value-home-absent-hint",
    "fetch-value-folder-home-absent-hint",
    "fetch-file-home-absent-created",
];

#[test]
#[ignore = "red until #1141 ends the absent value home refusal with its hint; implement removes this"]
fn value_home_hint_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("absent value home hint", &outcomes);
}
