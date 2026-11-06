//! Empty-extension fixtures distinguish capability refusal from errors (§AR-ci.10.6).

#[path = "non_utf8_refusal.rs"]
mod refusal;

#[test]
fn refused_names_retain_extension_contract() {
    let probe = refusal::Probe::new();
    // Includes the ordinary trailing-dot control alongside the invalid-name case.
    let result = probe.run("the_entry_reads_a_", "encoding");
    result.assert_available("encoding");
    result.assert_invoked(b"list\0");
    assert!(
        result.output.contains("2 passed; 0 failed; 0 ignored"),
        "{}",
        result.output
    );
    let denied = probe.run(
        "the_entry_reads_a_walked_file_whose_extension_is_not_utf8",
        "permission",
    );
    assert!(
        !denied.success,
        "permission failure was silently treated as unsupported names:\n{}",
        denied.output
    );
    assert!(
        denied.trace.contains("refused:permission"),
        "{}",
        denied.trace
    );
}
