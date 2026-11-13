//! Behavioral contract compiled beside each temporary-root helper, including
//! canonicalizing helpers (§FS-repository-maintenance.4.1 and
//! §FS-repository-maintenance.4.2). No owner is supplied by this test harness:
//! the helper under test must provide it.

use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

use super::test_root;

#[test]
fn fixture_root_normal_exit_removes_contents() {
    let path;
    {
        let root = test_root("fixture_root_normal_exit_removes_contents");
        path = root.to_path_buf();
        fs::write(root.join("sentinel"), "live fixture").unwrap();
        assert_eq!(
            fs::read_to_string(path.join("sentinel")).unwrap(),
            "live fixture"
        );
        assert!(path.is_dir(), "root must live through its caller's use");
    }
    assert!(
        !path.exists(),
        "normal exit left owned root: {}",
        path.display()
    );
}

#[test]
fn fixture_root_panic_removes_contents() {
    let mut path = PathBuf::new();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let root = test_root("fixture_root_panic_removes_contents");
        path = root.to_path_buf();
        fs::write(root.join("sentinel"), "live before panic").unwrap();
        panic!("fixture-root controlled panic");
    }));
    let panic = result.expect_err("the controlled panic must unwind");
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"fixture-root controlled panic")
    );
    assert!(
        !path.exists(),
        "unwinding left owned root: {}",
        path.display()
    );
}

#[test]
fn fixture_root_repeated_allocations_are_independent() {
    for first_drops_first in [true, false] {
        let first = test_root("fixture_root_repeated_allocations_are_independent");
        let first_path = first.to_path_buf();
        fs::write(first.join("sentinel"), "first fixture").unwrap();
        let second = test_root("fixture_root_repeated_allocations_are_independent");
        let second_path = second.to_path_buf();
        assert_eq!(
            fs::read_to_string(first_path.join("sentinel"))
                .ok()
                .as_deref(),
            Some("first fixture"),
            "second allocation destroyed first fixture"
        );
        assert_ne!(
            first_path, second_path,
            "live fixtures must have distinct roots"
        );
        fs::write(second.join("sentinel"), "second fixture").unwrap();
        if first_drops_first {
            drop(first);
            assert_eq!(
                fs::read_to_string(second.join("sentinel")).unwrap(),
                "second fixture"
            );
            drop(second);
        } else {
            drop(second);
            assert_eq!(
                fs::read_to_string(first.join("sentinel")).unwrap(),
                "first fixture"
            );
            drop(first);
        }
        assert!(
            !first_path.exists() && !second_path.exists(),
            "drop must remove both leaves"
        );
    }
}

#[test]
fn fixture_root_occupied_candidate_is_preserved() {
    // Forget the first owner to model an occupied leaf from a previous caller.
    // A new caller may refuse it or choose another leaf, but must not pre-clear it.
    let occupied = test_root("fixture_root_occupied_candidate_is_preserved");
    let path = occupied.to_path_buf();
    fs::write(path.join("sentinel"), "unowned by next claimant").unwrap();
    std::mem::forget(occupied);
    let claimant = catch_unwind(|| test_root("fixture_root_occupied_candidate_is_preserved"));
    let distinct = claimant
        .as_ref()
        .map_or(true, |root| root.to_path_buf() != path);
    drop(claimant);
    let bytes = fs::read_to_string(path.join("sentinel")).ok();
    // Only this deliberately abandoned leaf is ours to remove. Observe first,
    // so this cleanup cannot make the helper's refusal appear correct.
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    assert_eq!(
        bytes.as_deref(),
        Some("unowned by next claimant"),
        "claiming an occupied root changed its sentinel"
    );
    assert!(distinct, "allocation adopted an occupied root");
}
