//! §AR-ci-fixture-cleanup held on `Configured`, the fixture of agent-grounds/grund#1226: a test that fails
//! while its fixture cannot be removed is one failed test with its own panic, and the binary runs
//! on; a passing test whose fixture cannot be removed still fails.
//!
//! Each case holds a real `Configured` on a thread of its own and makes its removal fail by taking
//! write permission from a directory inside the fixture root, so nothing outside that root is
//! touched. The case joins the thread, gives the permission back and removes the tree itself.
//! Where a read-only directory does not stop this user, as for root, the case says so and skips.

use super::Configured;
use std::any::Any;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::panic::resume_unwind;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

/// The panic a failing test raises while its fixture is still alive.
const FIRST: &str = "the test's own assertion failed while its fixture was alive";

/// The directory inside a `Configured` root whose files cannot be removed once it is read-only.
const BARRED: &str = "docs/requirements";

fn set_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("set the barred mode");
}

/// How a thread holding a `Configured` whose removal is refused ended, and the root it left; `None`
/// where the refusal cannot be arranged for this user.
fn hold_unremovable(fail_first: bool) -> Option<(thread::Result<()>, PathBuf)> {
    let (sender, receiver) = mpsc::channel();
    let holder = thread::spawn(move || {
        let fixture = Configured::new();
        let barred = fixture.0.join(BARRED);
        set_mode(&barred, 0o555);
        let binds = fs::write(barred.join("permission-probe"), "").is_err();
        if !binds {
            set_mode(&barred, 0o755);
        }
        sender
            .send((fixture.0.clone(), binds))
            .expect("report the fixture root");
        if binds && fail_first {
            panic!("{FIRST}");
        }
    });
    let ended = holder.join();
    let Ok((root, binds)) = receiver.recv() else {
        match ended {
            Err(payload) => resume_unwind(payload),
            Ok(()) => panic!("the holding thread ended without building its fixture"),
        }
    };
    if !binds {
        if let Err(payload) = ended {
            resume_unwind(payload);
        }
        eprintln!(
            "skipped: a read-only directory does not stop this user, so {} could not be made \
             unremovable (§AR-ci-fixture-cleanup is not exercised here)",
            root.display()
        );
        return None;
    }
    let left = root.exists();
    if left {
        set_mode(&root.join(BARRED), 0o755);
        fs::remove_dir_all(&root).expect("remove the fixture tree the case kept");
    }
    assert!(left, "the refused removal left {} in place", root.display());
    Some((ended, root))
}

fn message(payload: &(dyn Any + Send)) -> Option<&str> {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
}

/// §AR-ci-fixture-cleanup.1: the join hands back the test's own panic, so the binary is still running to report
/// it. Today the fixture's `Drop` panics again while unwinding and the whole binary aborts.
#[test]
#[ignore = "red until #1226 stops a fixture Drop raising while its thread unwinds; implement removes this"]
fn a_failing_test_whose_configured_cannot_be_removed_reports_its_own_panic() {
    let Some((ended, root)) = hold_unremovable(true) else {
        return;
    };
    let payload = ended.expect_err("the holding thread failed its test");
    assert_eq!(
        message(&*payload),
        Some(FIRST),
        "the join returns the test's own panic, not the fixture's ({})",
        root.display()
    );
}

/// §AR-ci-fixture-cleanup.2: on the passing path a refused removal still fails the test, with the error.
#[test]
fn a_passing_test_whose_configured_cannot_be_removed_still_fails() {
    let Some((ended, root)) = hold_unremovable(false) else {
        return;
    };
    let payload = ended.expect_err("a refused removal fails a passing test");
    let message = message(&*payload).unwrap_or_default();
    assert!(
        message.contains("Permission denied"),
        "the fixture's panic carries the removal error for {}: {message}",
        root.display()
    );
}
