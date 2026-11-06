//! Filesystem capability for invalid-byte fixture names (§AR-ci.10.6).

use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Probe the exact basename beside its intended destination (§AR-ci.10.6).
/// Isolation keeps altered spellings from contaminating ordinary control files.
/// Only an encoding refusal or changed spelling makes coverage unavailable;
/// permissions, missing parents and enumeration errors remain fixture failures.
pub fn can_store(path: &Path, coverage: &str) -> bool {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let parent = path.parent().expect("fixture name has a parent");
    let name = path.file_name().expect("fixture path names a file");
    assert!(name.to_str().is_none(), "probe a non-UTF-8 basename");
    let directory = parent.join(format!(
        ".grund-name-probe-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).expect("create capability probe on the fixture filesystem");
    let probe = Probe(directory);
    match fs::write(probe.0.join(name), []) {
        Ok(()) => {}
        Err(error) if error.raw_os_error() == Some(libc::EILSEQ) => {
            eprintln!(
                "unavailable: {coverage}; filesystem refuses non-UTF-8 name {path:?}: {error}"
            );
            return false;
        }
        Err(error) => panic!("create non-UTF-8 fixture probe for {path:?}: {error}"),
    }
    let entries = fs::read_dir(&probe.0)
        .expect("enumerate capability probe directory")
        .map(|entry| entry.expect("read capability probe entry").file_name())
        .collect::<Vec<_>>();
    if entries.len() == 1 && entries[0].as_bytes() == name.as_bytes() {
        true
    } else {
        eprintln!(
            "unavailable: {coverage}; filesystem changed non-UTF-8 name {path:?}: {entries:?}"
        );
        false
    }
}

/// Remove even an altered spelling before independent controls run (§AR-ci.10.6).
struct Probe(PathBuf);

impl Drop for Probe {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove capability probe directory");
    }
}
