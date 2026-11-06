//! Clean and warm candidate parents preserve exclusive leaf ownership (§AR-ci.10.2).

use super::FixtureRoot;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn outer_root() -> FixtureRoot {
    let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/command-agreement");
    FixtureRoot::new(&parent, "1279-parents").expect("exclusively claim fresh outer root")
}

/// Port the triage probe's distinct identities and owner-only leaf cleanup (§AR-ci.10.1,
/// §AR-ci.10.2); no parent preparation here may mask the allocator's missing-parent defect.
fn allocate_and_drop(parent: &Path, name: &str, seen: &mut HashSet<PathBuf>) {
    let owner = FixtureRoot::new(parent, name).expect("claim fixture root");
    let root = owner.path().to_path_buf();
    assert!(seen.insert(root.clone()), "fixture identity was reused");
    fs::write(root.join("owned-sentinel"), b"owned").unwrap();
    drop(owner);
    assert!(!root.exists(), "successful owner must remove only its leaf");
}

fn clean_parent(leaf: &str) {
    let outer = outer_root();
    let intermediate = outer.path().join("escaped");
    assert!(!intermediate.exists(), "the regression must start clean");
    // On Windows these are the reported escaped\held and escaped\saved names (§AR-ci.10.2).
    let name = format!("escaped{}{leaf}", std::path::MAIN_SEPARATOR);
    allocate_and_drop(outer.path(), &name, &mut HashSet::new());
    assert!(intermediate.is_dir(), "leaf cleanup must leave its parent");
}

/// A fresh outer root prevents cache warmth from hiding a missing parent (§AR-ci.10.2).
#[test]
#[ignore = "red until #1279 prepares intermediate candidate parents; implement removes this"]
fn grund_1279_clean_held_parent() {
    clean_parent("held");
}

/// Saved roots independently require the same clean-parent preparation (§AR-ci.10.2).
#[test]
#[ignore = "red until #1279 prepares intermediate candidate parents; implement removes this"]
fn grund_1279_clean_saved_parent() {
    clean_parent("saved");
}

/// Prior parent bytes survive repeated names and successful owner cleanup (§AR-ci.10.2).
#[test]
fn grund_1279_warm_parent_preserves_sentinel() {
    let outer = outer_root();
    let intermediate = outer.path().join("escaped");
    fs::create_dir(&intermediate).unwrap();
    let sentinel = intermediate.join("prior-sentinel");
    fs::write(&sentinel, b"prior owner").unwrap();
    let mut seen = HashSet::new();
    for leaf in ["held", "saved", "held", "saved"] {
        let name = format!("escaped{}{leaf}", std::path::MAIN_SEPARATOR);
        allocate_and_drop(outer.path(), &name, &mut seen);
        assert_eq!(fs::read(&sentinel).unwrap(), b"prior owner");
    }
    assert_eq!(seen.len(), 4);
    assert_eq!(fs::read_dir(&intermediate).unwrap().count(), 1);
}

/// Unix backslashes remain filename bytes rather than intermediate directories (§AR-ci.10.2).
#[cfg(unix)]
#[test]
fn grund_1279_unix_literal_backslash_names() {
    let outer = outer_root();
    let mut seen = HashSet::new();
    for name in [r"escaped\held", r"escaped\saved"] {
        allocate_and_drop(outer.path(), name, &mut seen);
        assert!(!outer.path().join("escaped").exists());
    }
}
