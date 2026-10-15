//! §FS-citations.checks.mixed-path, §FS-rules.7.12 and
//! §FS-section-lock.checks.unrecorded-section-citation: the release transition
//! and its resolution, lock-enrollment and move/fingerprint controls.

use crate::case_runner::{CaseKind::E2e, assert_every_case_passed, run_case};

#[test]
fn mixed_path_transition_goldens_and_lock_controls() {
    let root = crate::repo_root();
    let cases = root.join("tests/e2e/cases");
    let names = [
        "check-number-name-citation-missing",
        "check-number-name-rule-mixed-only",
        "check-number-name-complete-unrecorded",
        "check-number-name-relative-unrecorded",
        "check-number-name-stub-target-unrecorded",
        "check-number-name-rule-other-citation",
        "check-number-name-complete",
        "check-number-name-relative",
        "check-number-name-stub-target-written",
        "check-number-name-renumber-leaf",
        "check-number-name-renumber-child",
        "check-number-name-renumber-relative",
        "check-number-name-renumber-child-cited",
        "check-number-name-renumber-child-pointer",
        "check-number-name-renumber-leaf-cited",
        "check-number-name-renumber-leaf-pointer",
        "check-number-name-renumber-relative-cited",
        "check-number-name-renumber-relative-pointer",
        "fmt-number-name-complete-write",
        "fmt-number-name-relative-write",
        "fmt-number-name-stub-target-write",
        "fmt-number-name-renumber-child-write",
        "fmt-number-name-renumber-leaf-write",
        "fmt-number-name-renumber-leaf-cited-write",
        "fmt-number-name-renumber-relative-write",
        "refs-number-name-citation-not-truncated",
        "show-number-name-complete",
        "show-number-name-relative",
    ];
    let outcomes = names
        .into_iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("mixed path transition and controls", &outcomes);
}
