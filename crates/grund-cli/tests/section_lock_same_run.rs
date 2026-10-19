//! The cases of agent-grounds/grund#847, run alone: one `fmt` run judges the
//! section lock against the tree as it leaves it, not as it read it
//! (§FS-section-lock.write.same-run), so a `--write` that marks a trigger is a
//! fixed point (§FS-fmt.requirements.fixed-point) and the dry run lists the
//! rows that write makes (§FS-fmt.7.3). Every case is also in the full e2e pass.

use std::path::PathBuf;

#[path = "support/case_runner.rs"]
#[allow(dead_code)]
mod case_runner;

use case_runner::{CaseKind::E2e, assert_every_case_passed, run_case};

fn run(label: &str, names: &[&str]) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = root.join("tests/e2e/cases");
    let outcomes = names
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed(label, &outcomes);
}

/// A trigger in another file that the run marks into a citation of a locked
/// section adds its entry in that run, with cross-references off and on, and
/// the dry run lists the add beside the rewrite.
#[test]
fn a_citation_the_run_writes_is_entered_in_that_run() {
    run(
        "same-run add",
        &[
            "fmt-section-lock-trigger-add-write",
            "fmt-section-lock-trigger-add-write-cross-refs",
            "fmt-section-lock-trigger-add-check",
            "fmt-section-lock-trigger-add-check-cross-refs",
        ],
    );
}

/// A locked section whose text the run rewrites is fingerprinted as rewritten:
/// refreshed to that text, or left alone where the lock already records it.
#[test]
fn a_section_the_run_rewrites_is_fingerprinted_as_rewritten() {
    run(
        "same-run refresh",
        &[
            "fmt-section-lock-trigger-refresh-write",
            "fmt-section-lock-trigger-refresh-write-cross-refs",
            "fmt-section-lock-trigger-keep-check",
        ],
    );
}

/// A workspace-root run judges the root's entry for a citation into a member
/// against the member's text as the same run leaves it
/// (§FS-section-lock.file.members).
#[test]
fn a_member_the_run_rewrites_is_fingerprinted_as_rewritten() {
    run(
        "same-run member refresh",
        &["fmt-section-lock-trigger-member-refresh-write"],
    );
}
