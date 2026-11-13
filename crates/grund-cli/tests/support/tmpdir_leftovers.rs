//! What a passing case leaves in its temporary directory, the observation a fixture's removal is
//! held to (§AR-ci-fixture-cleanup). Named cases of the current test binary run in one child whose
//! temporary directory is private and empty, and whatever that child left there is read back.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

#[path = "fixture_cleanup.rs"]
mod fixture_cleanup;

/// The child's temporary directory. It is built only from a claim that succeeded (§AR-ci.10.2),
/// so dropping it removes nothing this process did not create.
struct Private(PathBuf);

impl Drop for Private {
    fn drop(&mut self) {
        fixture_cleanup::remove_fixture_tree(&self.0, "remove the private temporary directory");
    }
}

/// Runs `cases` of this test binary, each selected by its exact name, in one child whose
/// temporary directory is a fresh, empty one. Asserts that every case ran and passed, since only
/// a passing run says anything about cleanup, and returns the sorted names of the entries the
/// child left in that directory.
pub fn left_by(cases: &[&str]) -> Vec<String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "grund-tmpdir-leftovers-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap_or_else(|err| panic!("claim {}: {err}", path.display()));
    let private = Private(path);
    let output = Command::new(std::env::current_exe().expect("this test binary"))
        .arg("--exact")
        .args(cases)
        .env("TMPDIR", &private.0)
        .env("TMP", &private.0)
        .env("TEMP", &private.0)
        .output()
        .expect("run the cases in a child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains(&format!("ok. {} passed;", cases.len())),
        "{cases:?} must all run and pass before what they leave says anything:\n{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut left: Vec<String> = fs::read_dir(&private.0)
        .expect("read the private temporary directory")
        .map(|entry| {
            let entry = entry.expect("read an entry of the private temporary directory");
            entry.file_name().to_string_lossy().into_owned()
        })
        .collect();
    left.sort();
    left
}
