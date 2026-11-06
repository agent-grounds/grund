//! Git ref fixtures obey §AR-ci.10.6 without suppressing §FS-distribution.4.1.1.

#[path = "non_utf8_refusal.rs"]
mod refusal;

fn survives_refusal(test: &str, checked_ref: &[u8]) {
    let probe = refusal::Probe::new();
    let result = probe.run(test, "encoding");
    result.assert_available("encoding");
    result.assert_invoked(checked_ref);
    assert!(
        result.output.contains("1 passed; 0 failed; 0 ignored"),
        "{}",
        result.output
    );
    let denied = probe.run(test, "permission");
    assert!(
        !denied.success,
        "Git/I/O failure was swallowed: {}",
        denied.output
    );
    assert!(
        denied.trace.contains("refused:permission"),
        "{}",
        denied.trace
    );
}

#[test]
fn refused_names_retain_branch_contract() {
    survives_refusal(
        "non_utf8_refs::a_git_valid_non_utf8_branch_keeps_its_commit",
        b"check-ref-format\0refs/heads/nonutf/branch-\xff\0",
    );
}

#[test]
fn refused_names_retain_symbolic_ref_contract() {
    survives_refusal(
        "non_utf8_refs::retargeting_a_non_utf8_symbolic_ref_updates_the_label",
        b"check-ref-format\0refs/heads/aliases/alias-\xff\0",
    );
}
