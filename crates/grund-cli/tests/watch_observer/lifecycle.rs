//! Interruption, fatal failure and cleanup boundaries (§FS-check.6.3).
use crate::support::*;
use grund::WatchObservation;
use grund_core::CheckFindingSelection;

#[test]
fn watch_sigint_before_first_completion() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, Some("scanning"));
    h.stage("scanning");
    h.stop(2);
    assert!(
        !h.history
            .iter()
            .any(|e| matches!(e, WatchObservation::Completed(..)))
    );
}

#[test]
fn watch_sigint_during_scan_discards_unpublished() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    h.pause("scanned");
    f.write("src/main.rs", BAD);
    h.stage("scanned");
    h.stop(0);
    assert_eq!(
        h.history
            .iter()
            .filter(|e| matches!(e, WatchObservation::Completed(..)))
            .count(),
        1
    );
}

#[test]
fn watch_sigint_during_publication_finishes_both_streams() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    h.pause("stdout-flushed");
    f.write(".agents/grund.toml", CONFIG);
    f.write("src/main.rs", BAD);
    f.write("src/broken.rs", "// \u{a7}FS-another-missing\n");
    h.stage("stdout-flushed");
    h.control.interrupt();
    h.release();
    let expected = f.ordinary();
    assert_eq!(expected.status, 1);
    assert!(!expected.stdout.is_empty() && !expected.stderr.is_empty());
    h.matches(&expected);
    h.finish(1);
    let stdout = h
        .history
        .iter()
        .rposition(|e| matches!(e, WatchObservation::StdoutFlushed))
        .unwrap();
    let stderr = h
        .history
        .iter()
        .rposition(|e| matches!(e, WatchObservation::StderrFlushed))
        .unwrap();
    let complete = h
        .history
        .iter()
        .rposition(|e| matches!(e, WatchObservation::Completed(..)))
        .unwrap();
    assert!(stdout < stderr && stderr < complete);
}

#[test]
fn watch_sigint_after_status_0_1_2() {
    let _serial = crate::support::serial();
    for status in [0, 1, 2] {
        let f = Fixture::new();
        if status == 1 {
            f.write("src/main.rs", BAD);
        }
        if status == 2 {
            f.write("grund.toml", "grund_config_version = \"bad\"\n");
        }
        let mut h = Harness::start(&f, None, None);
        assert_eq!(h.completed().status, status);
        h.stop(status);
    }
}

#[test]
fn watch_shutdown_joins_workers_and_releases_subscriptions() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    h.stop(0);
    assert!(matches!(h.history.last(), Some(WatchObservation::Stopped)));
    // A second native session can immediately acquire the same roots/adapter.
    let mut again = Harness::start(&f, None, None);
    again.completed();
    again.stop(0);
}

#[test]
fn watch_setup_failure_is_fatal_two() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::options(
        f.opts(),
        CheckFindingSelection::default(),
        None,
        None,
        Some("setup"),
    );
    h.finish(2);
    assert!(
        !h.history
            .iter()
            .any(|e| matches!(e, WatchObservation::Completed(..)))
    );
}

#[test]
fn watch_runtime_subscription_failure_is_fatal_two() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    h.control.fail_next("subscribe");
    f.write("newsrc/main.rs", BAD);
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[scan]\ninclude = [\"newsrc\"]\n"),
    );
    h.finish(2);
}

#[test]
fn watch_failed_recovery_is_fatal_two() {
    let _serial = crate::support::serial();
    for previous in [0, 1] {
        let f = Fixture::new();
        if previous == 1 {
            f.write("src/main.rs", BAD);
        }
        let mut h = Harness::start(&f, None, None);
        assert_eq!(h.completed().status, previous);
        h.control.fail_next("recover");
        h.control.saturate();
        h.finish(2);
    }
}

#[test]
fn watch_replacement_reattaches() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    std::fs::remove_dir_all(f.0.join("src")).unwrap();
    let absent = f.ordinary();
    h.matches(&absent);
    f.write("src/main.rs", BAD);
    h.matches(&f.ordinary());
    f.write("src/main.rs", CLEAN);
    h.matches(&f.ordinary());
    h.stop(0);
    assert!(
        h.history
            .iter()
            .any(|e| matches!(e, WatchObservation::Recovered))
    );
}

#[test]
fn watch_deleted_project_root_recreates_after_initial_config_error() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write("grund.toml", "grund_config_version = \"invalid\"\n");
    let mut opts = f.opts();
    opts.path_provided = true;
    let mut h = Harness::options(
        opts.clone(),
        CheckFindingSelection::default(),
        None,
        None,
        None,
    );
    assert_eq!(h.completed().status, 2);
    std::fs::remove_dir_all(&f.0).unwrap();
    let absent = grund::watch_test::one_shot(opts.clone(), &CheckFindingSelection::default(), None);
    h.matches(&absent);
    f.write("grund.toml", CONFIG);
    f.write("docs/functional-spec/FS-live.md", "# FS-live: Live\n");
    f.write(
        "docs/functional-spec/README.md",
        "# FS index\n\n- [\u{a7}FS-live](FS-live.md#fs-live-live)\n",
    );
    f.write("src/main.rs", BAD);
    let recreated = grund::watch_test::one_shot(opts, &CheckFindingSelection::default(), None);
    assert_eq!(recreated.status, 1);
    h.matches(&recreated);
    h.stop(1);
}

#[test]
fn watch_runtime_notification_failure_is_fatal_two() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    h.control
        .notify(Err(notify::Error::generic("native backend failed")));
    h.finish(2);
    assert!(String::from_utf8_lossy(&h.control.take_output()[1]).contains("native backend failed"));
}

#[test]
fn watch_delete_during_scan_retires_dead_subscription() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    let release = h.pause("scanned");
    f.write("src/main.rs", BAD);
    h.stage("scanned");
    std::fs::remove_dir_all(f.0.join("src")).unwrap();
    release.send(()).unwrap();
    assert_eq!(h.completed().status, 1);
    let absent = f.ordinary();
    h.matches(&absent);
    h.stop(absent.status);
}
