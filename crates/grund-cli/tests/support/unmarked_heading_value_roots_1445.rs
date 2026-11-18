//! Issue #1445's cases, run alone: under a value root inside a declaration, an embedded root
//! (§FS-values.2.4) or a chapter root (§FS-values.2.5), unmarked-heading shows a repair, and a
//! relative heading's form, only where the shown heading is the component the root's run expects
//! at that line, so writing it never trades the error for the root's own refusal
//! (§FS-declarations.checks.unmarked-heading.2.3, §FS-declarations.checks.unmarked-heading.2.1,
//! §FS-values.2.4.3). Each case keeps its
//! positive controls beside the shapes that must lose their clause. The full pass runs them
//! too; this runs them without the rest of the corpus.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const CASES: [&str; 2] = [
    "check-unmarked-heading-fault-guard-value-root-chapter",
    "check-unmarked-heading-fault-guard-value-root-embedded",
];

#[test]
fn unmarked_heading_value_roots_1445_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("unmarked heading under a value root (#1445)", &outcomes);
}
