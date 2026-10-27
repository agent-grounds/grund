//! The e2e cases of agent-grounds/grund#151, parked under `tests/e2e/pending/`
//! until the texts they pin exist: `grund config explain` says an obligation
//! key never judges a file of a place or the complement that cites nothing
//! (§FS-config-v2.rules.citations.uncited), and `grund schema`'s legend says so
//! where such a row is ungrounded (§FS-schema.8.3). The implementation moves the
//! three cases into `tests/e2e/cases/`, deletes this module and its `mod` line,
//! and refreshes `schema-table-v2-code-places`, whose `page` row is one.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

#[test]
#[ignore = "red until #151 the explain and schema texts name the uncited file; implement removes this"]
fn pending_uncited_obligation_cases_match_their_goldens() {
    let root = repo_root();
    let outcomes: Vec<_> = [
        "config-explain-v2-obligation-uncited",
        "schema-table-v1-place-ungrounded",
        "schema-table-v2-place-ungrounded",
    ]
    .iter()
    .map(|case| run_case(&root, &root.join("tests/e2e/pending").join(case), E2e))
    .collect();
    assert_every_case_passed("pending #151 cases", &outcomes);
}
