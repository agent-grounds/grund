//! Shared agreement-fixture identity and exclusive ownership (§AR-ci.10.1, §AR-ci.10.2).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Only a successful exclusive claimant owns cleanup, including on unwind (§AR-ci.10.2).
#[derive(Debug)]
pub(crate) struct FixtureRoot(PathBuf);

impl FixtureRoot {
    /// A process-wide serial distinguishes sequential and overlapping instances (§AR-ci.10.1).
    pub(crate) fn new(parent: &Path, name: &str) -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        fs::create_dir_all(parent)?;
        Self::claim(parent.join(format!("{name}-{}-{serial}", std::process::id())))
    }

    /// Refuse an occupied leaf without touching it; install cleanup before more work (§AR-ci.10.2).
    fn claim(path: PathBuf) -> io::Result<Self> {
        fs::create_dir(&path)?;
        let mut owner = Self(path);
        owner.0 = owner.0.canonicalize()?;
        Ok(owner)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for FixtureRoot {
    /// Failed claims never construct this guard (§AR-ci.10.2).
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The actual claim refuses collisions and cleans up only successful owners (§AR-ci.10.2).
#[test]
fn grund_977_shared_claim_refuses_occupied_root() {
    let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/command-agreement");
    let owner = FixtureRoot::new(&parent, "977-claim").unwrap();
    let path = owner.path().to_path_buf();
    fs::write(path.join("sentinel"), b"original owner").unwrap();

    let refused = FixtureRoot::claim(path.clone());
    assert_eq!(
        refused.as_ref().unwrap_err().kind(),
        io::ErrorKind::AlreadyExists
    );
    drop(refused);
    assert!(path.is_dir());
    assert_eq!(fs::read(path.join("sentinel")).unwrap(), b"original owner");

    drop(owner);
    assert!(!path.exists(), "successful owner must remove its root");
}

/// A claimed root is also released when initialization unwinds (§AR-ci.10.2).
#[test]
fn grund_977_shared_owner_cleans_up_on_unwind() {
    let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/command-agreement");
    let mut path = PathBuf::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let owner = FixtureRoot::new(&parent, "977-unwind").unwrap();
        path = owner.path().to_path_buf();
        fs::write(path.join("sentinel"), b"initializing").unwrap();
        panic!("initialization failed after claiming root");
    }));
    assert!(result.is_err());
    assert!(!path.exists(), "unwinding owner must remove its root");
}
