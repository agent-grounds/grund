//! Issue #1149's cases, run alone: a version-2 rule kind's `file` home is Markdown by
//! the language `[schema.sources]` reads it as, not by its name (§FS-config-v2.schema.sources.rules).
//! Every one is also in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const CASES: [&str; 4] = [
    "check-v2-rule-home-md-read-as-json",
    "check-v2-rule-home-json-refused",
    "check-v2-rule-home-md-read-by-no-language",
    "check-v2-rule-home-txt-read-as-markdown",
];

#[test]
#[ignore = "red until #1149 asks [schema.sources] whether a v2 rule home is Markdown; implement removes this"]
fn v2_rule_home_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("v2 rule home", &outcomes);
}
