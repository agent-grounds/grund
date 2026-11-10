//! The one decision a fixture `Drop` makes when its tree cannot be removed
//! (§AR-ci-fixture-cleanup.3).

use std::fs;
use std::path::Path;

/// Removes the tree a fixture created. If the removal fails while the thread is already
/// panicking, the error is written to stderr, naming the tree, and nothing is raised, because a
/// second panic would abort the whole test binary (§AR-ci-fixture-cleanup.1). On the passing path
/// the same failure panics, with no retry and nothing ignored (§AR-ci-fixture-cleanup.2).
pub fn remove_fixture_tree(path: &Path, what: &str) {
    if let Err(err) = fs::remove_dir_all(path) {
        let message = format!("{what} {}: {err}", path.display());
        if std::thread::panicking() {
            eprintln!("{message}");
        } else {
            panic!("{message}");
        }
    }
}
