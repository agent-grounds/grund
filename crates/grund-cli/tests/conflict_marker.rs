//! The cases of agent-grounds/grund#837, run alone in the three groups its
//! change lands in: `fmt` writes nothing before its last refusal
//! (§FS-fmt.7.4.3), a conflict block is a `check` error
//! (§FS-declarations.checks.conflict-marker), and `fmt` refuses on one
//! (§FS-fmt.3.3). Every case is also in the full e2e pass.

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

/// §FS-fmt.7.4.3: a refusal the lock pass or a shorthand's deferred scan meets
/// finds the tree as the run found it. The workspace control passes before the
/// change too: its preflight already refuses before any write (§FS-fmt.3.3).
#[test]
fn a_refused_fmt_writes_nothing() {
    run(
        "nothing written before the last refusal",
        &[
            "fmt-unreadable-path-lock-pass-write",
            "fmt-unreadable-path-shorthand-write",
            "fmt-conflict-marker-workspace-unreadable-wins",
        ],
    );
}

/// §FS-declarations.body.conflict: what a block is, where it is looked for and
/// whom it names, as `check` reports it.
#[test]
fn a_conflict_block_is_a_check_error() {
    run(
        "conflict-marker check",
        &[
            "check-conflict-marker",
            "check-conflict-marker-json",
            "check-conflict-marker-declaration-line",
            "check-conflict-marker-fenced",
            "check-conflict-marker-docstring",
            "check-conflict-marker-doc-comment-cut",
            "check-conflict-marker-half-resolved",
            "check-conflict-marker-lookalikes",
            "check-conflict-marker-unclosed-fence",
            "check-conflict-marker-lock-and-body",
        ],
    );
}

/// §FS-fmt.7.4, §FS-fmt.3.4: a run that needs the whole declaration set refuses
/// on a block, in both modes and both cross-reference settings, root first in a
/// workspace, and with a lock in conflict beside it (§FS-section-lock.write.rebuild).
#[test]
fn a_conflict_block_refuses_fmt() {
    run(
        "conflict-marker fmt",
        &[
            "fmt-conflict-marker-write-refused",
            "fmt-conflict-marker-write-refused-cross-refs-off",
            "fmt-conflict-marker-check-refused",
            "fmt-conflict-marker-workspace",
            "fmt-conflict-marker-lock-and-body",
        ],
    );
}

/// The lock alone in conflict is rebuilt exactly as before (§FS-section-lock.write.rebuild).
#[test]
fn a_lock_alone_in_conflict_is_still_rebuilt() {
    run(
        "lock rebuild unchanged",
        &[
            "fmt-section-lock-conflict-markers-write",
            "fmt-section-lock-conflict-markers-check",
            "check-section-lock-conflict-markers",
        ],
    );
}
