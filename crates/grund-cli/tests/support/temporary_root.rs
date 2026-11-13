//! Test-only temporary-root ownership shared by the core, CLI and LSP fixtures
//! (§FS-repository-maintenance.4.1, §AR-ci-fixture-cleanup.3).

use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[path = "fixture_cleanup.rs"]
mod fixture_cleanup;

/// A claimed leaf, kept alive independently of any copied paths
/// (§FS-repository-maintenance.4.1). Deliberately not `Clone`: copying a path
/// never transfers ownership or extends the lifetime of its fixture.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TemporaryRoot {
    owned: PathBuf,
    path: PathBuf,
}

impl TemporaryRoot {
    /// Exclusively claim a fresh leaf; an occupied candidate is never touched
    /// (§FS-repository-maintenance.4.2). Only shared parents use `create_dir_all`.
    pub(crate) fn new(parent: &Path, name: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        fs::create_dir_all(parent).expect("create fixture parent");
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!("{name}-{}-{serial}", std::process::id()));
        fs::create_dir(&path).unwrap_or_else(|error| {
            panic!("claim fixture root {}: {error}", path.display());
        });
        Self {
            owned: path.clone(),
            path,
        }
    }

    /// Keep the owner installed while resolving platform path spelling
    /// (§FS-repository-maintenance.4.1); failure unwinds through its cleanup.
    #[allow(dead_code)] // Only canonicalizing boundaries need physical spelling.
    pub(crate) fn physical(mut self) -> Self {
        self.path = fs::canonicalize(&self.path).expect("canonicalize fixture root");
        self
    }

    /// A builder may expose a descendant while still owning the whole claimed leaf
    /// (§FS-repository-maintenance.4.1). Cleanup always targets `owned`.
    #[allow(dead_code)] // Only builders exposing a nested project need this adapter.
    pub(crate) fn child(mut self, relative: &str) -> Self {
        self.path = self.path.join(relative);
        self
    }

    /// Keep a path consumer and its owner together until the consumer is dropped
    /// (§FS-repository-maintenance.4.1). Fields drop in order: consumer, then root.
    #[allow(dead_code)] // Core config builders return consumers; other boundaries return roots.
    pub(crate) fn hold<T>(self, value: T) -> Rooted<T> {
        Rooted { value, _root: self }
    }
}

/// A test-only path consumer with the fixture lifetime attached
/// (§FS-repository-maintenance.4.1).
pub(crate) struct Rooted<T> {
    value: T,
    _root: TemporaryRoot,
}

impl<T> Deref for Rooted<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Rooted<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl Deref for TemporaryRoot {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.path
    }
}

impl AsRef<Path> for TemporaryRoot {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<std::ffi::OsStr> for TemporaryRoot {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.path.as_os_str()
    }
}

impl Drop for TemporaryRoot {
    /// Remove only our leaf, with the shared strict/unwinding policy
    /// (§FS-repository-maintenance.4.1, §AR-ci-fixture-cleanup.3).
    fn drop(&mut self) {
        fixture_cleanup::remove_fixture_tree(&self.owned, "remove temporary fixture root");
    }
}
