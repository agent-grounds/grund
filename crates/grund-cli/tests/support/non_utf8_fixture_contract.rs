//! Controlled refusal exercises §AR-ci.10.6 without requiring a Darwin filesystem.
//! The children execute the existing §FS-cli.5.1 end-to-end cases unchanged.

#[path = "non_utf8_refusal.rs"]
mod refusal;

#[test]
fn refused_names_retain_cli_contract() {
    cli_contract("encoding");
    let probe = refusal::Probe::new();

    // Permission failure at the same invalid-name write is not a capability refusal.
    let denied = probe.run("contract::existing_non_utf8_path_is_refused", "permission");
    assert!(
        !denied.success,
        "unexpected I/O failure was swallowed: {}",
        denied.output
    );
    assert!(
        denied.trace.contains("refused:permission"),
        "{}",
        denied.trace
    );
}

#[test]
fn rewritten_names_retain_cli_contract() {
    cli_contract("spelling");
}

fn cli_contract(mode: &str) {
    let probe = refusal::Probe::new();
    let result = probe.run("contract::", mode);
    result.assert_available(mode);
    // Observe real subprocess argv, rather than accepting an early-returning test.
    for argv in [
        b"check\0docs/FACTS.\xff\0".as_slice(),
        b"check\0docs/notes\xff.md\0",
        b"check\0docs/notes\xef\xbf\xbd.md\0",
        b"fmt\0--write\0docs/notesX.md\0",
        b"fmt\0--write\0docs/notes\xff.md\0",
        b"check\0nope.\xff\0",
        b"--version\0\xff\0",
        b"\xff\0--version\0",
        b"-V\0\xff\0",
        b"\xff\0-V\0",
    ] {
        result.assert_invoked(argv);
    }
    assert!(
        result.output.contains("13 passed; 0 failed; 0 ignored"),
        "{}",
        result.output
    );
}
