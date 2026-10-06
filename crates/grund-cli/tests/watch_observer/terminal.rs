//! Exact output writes of the owned-screen state machine (§FS-check.6.2.2).
//! Real PTYs independently pin terminal eligibility in test_watch_terminal.py.
use crate::support::*;

const ENTER: &[u8] = b"\x1b[?1049h\x1b[H\x1b[2J";
const CLEAR: &[u8] = b"\x1b[H\x1b[2J";
const RESTORE: &[u8] = b"\x1b[?1049l";

#[test]
fn watch_text_json_text_format_transitions() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::screen(&f, true);
    assert_eq!(h.completed().stdout, [ENTER, b"success\n"].concat());
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[output]\nformat = \"json\"\n"),
    );
    let json = h.completed();
    assert_eq!(json.stdout, RESTORE);
    assert!(json.stderr.is_empty());
    f.write("grund.toml", CONFIG);
    assert_eq!(h.completed().stdout, [ENTER, b"success\n"].concat());
    h.stop(0);
    assert_eq!(h.control.take_output()[0], RESTORE);
}

#[test]
fn watch_fatal_error_restores_owned_screen() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::screen(&f, true);
    h.completed();
    h.control.fail_next("recover");
    h.control.saturate();
    h.finish(2);
    let output = h.control.take_output();
    assert_eq!(output[0], RESTORE);
    assert!(String::from_utf8_lossy(&output[1]).contains("error: watch: recover:"));
}

#[test]
fn watch_shared_terminal_stderr_clears_with_report() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::screen(&f, true);
    h.completed();
    f.write("grund.toml", "grund_config_version = \"invalid\"\n");
    let failure = h.completed();
    assert_eq!(failure.stdout, CLEAR);
    assert_eq!(failure.stderr, f.ordinary().stderr);
    assert_eq!(failure.status, 2);
    h.stop(2);
    assert_eq!(h.control.take_output()[0], RESTORE);
}

#[test]
fn watch_redirected_stderr_appends_across_runs() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write("grund.toml", "grund_config_version = \"invalid\"\n");
    let mut h = Harness::screen(&f, true);
    let first = h.completed();
    assert_eq!(first.stdout, ENTER);
    assert!(!first.stderr.is_empty());
    f.write("grund.toml", "grund_config_version = \"another-invalid\"\n");
    let second = h.completed();
    assert_eq!(second.stdout, CLEAR);
    assert!(!second.stderr.is_empty());
    assert!(![first.stderr, second.stderr].concat().contains(&0x1b));
    h.stop(2);
}
