//! Observable agreement fixture ownership (§AR-ci.10.1, §AR-ci.10.2).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn usable_cwd(root: &Path) -> bool {
    Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(root)
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Two overlapping allocations on one thread, in both drop orders, followed
/// by sequential allocations. An empty-file constructor must work here too.
pub fn lifetimes<T>(create: impl Fn() -> T, root: impl Fn(&T) -> &Path) {
    let mut safe = true;
    let mut identities = BTreeSet::new();
    for older_first in [false, true] {
        let older = create();
        let older_root = root(&older).to_path_buf();
        identities.insert(older_root.clone());
        fs::write(older_root.join("older-sentinel"), b"older").unwrap();
        let newer = create();
        let newer_root = root(&newer).to_path_buf();
        let distinct = identities.insert(newer_root.clone());
        fs::write(newer_root.join("newer-sentinel"), b"newer").unwrap();
        let preserved =
            fs::read(older_root.join("older-sentinel")).ok().as_deref() == Some(b"older");
        let (survivor, removed, sentinel, bytes) = if older_first {
            drop(older);
            (newer, older_root, "newer-sentinel", b"newer")
        } else {
            drop(newer);
            (older, newer_root, "older-sentinel", b"older")
        };
        let alive = fs::read(root(&survivor).join(sentinel)).ok().as_deref() == Some(bytes);
        let cwd = usable_cwd(root(&survivor));
        let cleaned = !removed.exists();
        let survivor_root = root(&survivor).to_path_buf();
        drop(survivor);
        println!(
            "older_first={older_first} distinct={distinct} constructor_preserved={preserved} \
             survivor_bytes={alive} usable_cwd={cwd} dropped_root_removed={cleaned}"
        );
        safe &= distinct && preserved && alive && cwd && cleaned && !survivor_root.exists();
    }
    for _ in 0..2 {
        let tree = create();
        let fresh = identities.insert(root(&tree).to_path_buf());
        println!("sequential allocation fresh={fresh}");
        safe &= fresh && usable_cwd(root(&tree));
    }
    assert!(
        safe,
        "same-name fixtures must own independent roots through construction and drop"
    );
}

pub fn legacy_candidate(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/command-agreement")
        .join(format!("{name}-{}", std::process::id()))
}

/// Exercises the real caller against the occupied candidate used before #977.
/// A new naming scheme may avoid it; the allocator also needs a direct claim
/// refusal test once implement supplies that seam (§AR-ci.10.2).
pub fn occupied<T>(candidate: PathBuf, create: impl FnOnce() -> T, root: impl Fn(&T) -> &Path) {
    fs::create_dir_all(candidate.parent().unwrap()).unwrap();
    fs::create_dir(&candidate).expect("exclusively create pre-existing owner");
    let candidate = candidate.canonicalize().unwrap();
    fs::write(candidate.join("owner-sentinel"), b"preexisting").unwrap();
    let claimed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(create));
    let refused_or_distinct = match &claimed {
        Err(_) => true,
        Ok(tree) => root(tree) != candidate,
    };
    drop(claimed);
    let preserved =
        fs::read(candidate.join("owner-sentinel")).ok().as_deref() == Some(b"preexisting");
    println!("occupied refused_or_distinct={refused_or_distinct} sentinel_preserved={preserved}");
    let _ = fs::remove_dir_all(&candidate);
    assert!(
        refused_or_distinct && preserved,
        "occupied root was adopted, modified or removed"
    );
}
