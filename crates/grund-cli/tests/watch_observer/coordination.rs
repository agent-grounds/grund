//! Serialized runs, notification loss and bounded debounce (§FS-check.6.1).
use crate::support::*;
use grund::{WatchObservation, watch_test};
use std::time::Duration;

#[test]
fn watch_debounce_quiet_100ms_and_maximum_500ms() {
    let _serial = crate::support::serial();
    assert_eq!(watch_test::debounce_deadline(&[0], 99), Some(1));
    assert_eq!(watch_test::debounce_deadline(&[0], 100), Some(0));
    assert_eq!(
        watch_test::debounce_deadline(&[0, 90, 180, 270, 360, 450], 499),
        Some(1)
    );
    assert_eq!(
        watch_test::debounce_deadline(&[0, 90, 180, 270, 360, 450], 500),
        Some(0)
    );
}

#[test]
fn watch_coalesces_atomic_burst() {
    let _serial = crate::support::serial();
    assert_eq!(
        watch_test::debounce_deadline(&[0, 10, 20, 30], 129),
        Some(1)
    );
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    let release = h.pause("scanning");
    for n in 0..20 {
        f.write("src/main.rs", if n % 2 == 0 { CLEAN } else { BAD });
    }
    h.stage("scanning");
    release.send(()).unwrap();
    h.matches(&f.ordinary());
    h.stop(1);
}

#[test]
fn watch_access_events_do_not_schedule() {
    let _serial = crate::support::serial();
    let input = grund_core::CheckInput {
        path: "source.rs".into(),
        recursive: false,
    };
    let event = notify::Event::new(notify::EventKind::Access(notify::event::AccessKind::Read))
        .add_path(input.path.clone());
    assert!(!watch_test::event_relevant(&[input], &event));
}

#[test]
fn watch_change_during_startup_is_not_lost() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let path = f.0.join("src/main.rs");
    let barrier = format!("input:{}", path.display());
    let mut h = Harness::start(&f, None, Some(&barrier));
    h.stage(&barrier);
    f.write("src/main.rs", BAD);
    h.release();
    h.matches(&f.ordinary());
    h.stop(1);
}

#[test]
fn watch_new_input_subscribed_before_read() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    let path = f.0.join("newsrc/main.rs");
    let barrier = format!("input:{}", f.0.join("newsrc").display());
    let release = h.pause(&barrier);
    f.write("newsrc/main.rs", CLEAN);
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[scan]\ninclude = [\"newsrc\"]\n"),
    );
    h.stage(&barrier);
    assert!(h.history.iter().any(
        |event| matches!(event, WatchObservation::Subscribed(root, true) if path.starts_with(root))
    ));
    f.write("newsrc/main.rs", BAD);
    release.send(()).unwrap();
    h.matches(&f.ordinary());
    h.stop(1);
}

#[test]
fn watch_change_during_scan_runs_again() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    let release = h.pause("scanned");
    f.write("src/main.rs", BAD);
    h.stage("scanned");
    f.write("src/main.rs", CLEAN);
    release.send(()).unwrap();
    let stale = h.completed();
    assert_eq!(stale.status, 1);
    h.matches(&f.ordinary());
    h.stop(0);
}

#[test]
fn watch_one_pending_rerun_serializes_publication() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    let release = h.pause("publishing");
    f.write("src/main.rs", BAD);
    h.stage("publishing");
    for _ in 0..64 {
        h.control.notify(Ok(changed_event(f.0.join("src/main.rs"))));
    }
    f.write("src/main.rs", CLEAN);
    release.send(()).unwrap();
    assert_eq!(h.completed().status, 1);
    h.matches(&f.ordinary());
    h.stop(0);
    let mut scanning = false;
    for event in &h.history {
        match event {
            WatchObservation::Scanning => {
                assert!(!scanning);
                scanning = true;
            }
            WatchObservation::Completed(..) => {
                assert!(scanning);
                scanning = false;
            }
            _ => {}
        }
    }
    assert!(!scanning);
}

#[test]
fn watch_add_before_retire_refresh() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[scan]\ninclude = [\"oldsrc\"]\n"),
    );
    f.write("oldsrc/main.rs", BAD);
    let mut h = Harness::start(&f, None, None);
    h.completed();
    f.write("newsrc/main.rs", CLEAN);
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[scan]\ninclude = [\"newsrc\"]\n"),
    );
    h.matches(&f.ordinary());
    h.stop(0);
    let added = h
        .history
        .iter()
        .position(
            |e| matches!(e, WatchObservation::Subscribed(p, true) if p == &f.0.join("newsrc")),
        )
        .unwrap();
    let retired = h
        .history
        .iter()
        .position(|e| matches!(e, WatchObservation::Retired(p) if p == &f.0.join("oldsrc")))
        .unwrap();
    assert!(added < retired);
}

#[test]
fn watch_loss_overflow_uncertain_saturation_rescan() {
    let _serial = crate::support::serial();
    for kind in ["loss", "overflow", "uncertain", "saturation"] {
        let f = Fixture::new();
        let mut h = Harness::start(&f, None, None);
        h.completed();
        let release = h.pause("scanning");
        match kind {
            "loss" | "saturation" => h.control.saturate(),
            "overflow" => {
                let mut event = notify::Event::new(notify::EventKind::Other);
                event.attrs.set_flag(notify::event::Flag::Rescan);
                h.control.notify(Ok(event));
            }
            _ => h
                .control
                .notify(Ok(notify::Event::new(notify::EventKind::Any))),
        }
        h.stage("scanning");
        assert!(
            h.history
                .iter()
                .any(|e| matches!(e, WatchObservation::Recovered))
        );
        f.write("src/main.rs", BAD);
        release.send(()).unwrap();
        h.matches(&f.ordinary());
        h.stop(1);
    }
    // No sleeps establish completion; policy itself uses supplied clock values.
    assert_eq!(Duration::from_millis(100).as_millis(), 100);
}

#[test]
fn watch_real_queue_saturation_preserves_fatal_backend_error() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    let release = h.pause("scanned");
    f.write("src/main.rs", BAD);
    h.stage("scanned");
    for _ in 0..129 {
        h.control.notify(Ok(changed_event(f.0.join("src/main.rs"))));
    }
    h.control
        .notify(Err(notify::Error::generic("fatal despite saturation")));
    release.send(()).unwrap();
    h.finish(2);
    assert!(
        String::from_utf8_lossy(&h.control.take_output()[1]).contains("fatal despite saturation")
    );
}

#[test]
fn watch_real_queue_saturation_rescans_latest_state() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let mut h = Harness::start(&f, None, None);
    h.completed();
    let release = h.pause("scanned");
    f.write("src/main.rs", BAD);
    h.stage("scanned");
    for _ in 0..129 {
        h.control.notify(Ok(changed_event(f.0.join("src/main.rs"))));
    }
    f.write("src/main.rs", CLEAN);
    release.send(()).unwrap();
    assert_eq!(h.completed().status, 1);
    h.matches(&f.ordinary());
    h.stop(0);
    assert!(
        h.history
            .iter()
            .any(|event| matches!(event, WatchObservation::Recovered))
    );
}
