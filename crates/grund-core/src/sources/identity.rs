//! A file's physical identity (§AR-system.2.13, §FS-scan.identity): the key two
//! spellings of one location compare equal by, read from the disk where the
//! disk can say and lexical where the file has yet to be written, so a buffer
//! with no file behind it still has one (§FS-scan.identity.buffers).
//!
//! These came down out of `model/paths.rs` with the sources component: they ask
//! the filesystem, and only sources does (§AR-system.5).

use std::fs;
use std::path::{Path, PathBuf};

use crate::model::normalize_path_lexically;

/// Whether two paths name one file. Canonicalized where the filesystem can say
/// so, which is what makes a declaration reached through a symlinked directory
/// the same home as the one reached directly (§FS-declarations.checks.broken-stub).
pub(crate) fn paths_same_location(left: &Path, right: &Path) -> bool {
    physical_path_key(left) == physical_path_key(right)
}

/// The key a *location* is compared by: the resolved path where it exists, and where it does not,
/// its longest existing prefix resolved with the missing tail kept. So an unsaved `alias/new.md`
/// is one key with `notes/new.md` (§FS-scan.identity.buffers).
pub(crate) fn physical_path_key(path: &Path) -> PathBuf {
    #[cfg(test)]
    RESOLVED.with(|count| count.set(count.get() + 1));
    canonicalize_existing_prefix(path)
}

#[cfg(test)]
thread_local! {
    /// How many keys this thread resolved, for the test that a linkless tree pays no new
    /// `realpath` (§FS-scan.identity.read-once).
    static RESOLVED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// See [`RESOLVED`]. Gated as its only reader, `scanner/tests_walk_names.rs`, is.
#[cfg(all(test, unix))]
pub(crate) fn resolved_paths() -> usize {
    RESOLVED.with(std::cell::Cell::get)
}

/// `path` absolutized and resolved as far as the filesystem can: the canonical
/// form where every component exists, else the canonical form of the longest
/// existing prefix with the missing tail appended. A path that has yet to be
/// written therefore still has one spelling, which is what lets the walk and an
/// editor's not-yet-saved buffer be compared (§AR-scanner.1, §AR-lsp.5). That
/// spelling is the prefix and the unresolved components in order, with nothing
/// appended, and a path that exists but has no canonical form keeps its lexical
/// form the same way (§FS-scan.identity.unresolved).
pub(crate) fn canonicalize_existing_prefix(path: &Path) -> PathBuf {
    let path = if path.is_absolute() {
        normalize_path_lexically(path)
    } else {
        std::env::current_dir()
            .map(|cwd| normalize_path_lexically(&cwd.join(path)))
            .unwrap_or_else(|_| normalize_path_lexically(path))
    };
    if let Ok(canonical) = fs::canonicalize(&path) {
        return canonical;
    }
    // Pushed name by name, never joined as one tail: joining an empty path appends
    // a separator, which would end the text in `/` (§FS-scan.identity.unresolved).
    let mut missing = Vec::new();
    let mut cursor = path.as_path();
    while !cursor.exists() {
        let Some(name) = cursor.file_name() else {
            break;
        };
        missing.push(name);
        let Some(parent) = cursor.parent() else {
            break;
        };
        cursor = parent;
    }
    let mut resolved =
        fs::canonicalize(cursor).unwrap_or_else(|_| normalize_path_lexically(cursor));
    for name in missing.into_iter().rev() {
        resolved.push(name);
    }
    resolved
}

/// Canonicalize `path` to the same normalized form `LspSnapshot` paths carry,
/// so an LSP client's request URI matches the snapshot's declaration, stub,
/// and citation paths (§AR-lsp.5.1). Existing files resolve through
/// `fs::canonicalize`; a not-yet-saved overlay file resolves its existing
/// prefix and appends the missing tail, never a trailing separator
/// (§FS-scan.identity.unresolved) — the same absolutization the snapshot
/// applies — so `grund-lsp` does not need a second, drift-prone copy of this
/// logic.
pub fn canonical_snapshot_path(path: &Path) -> PathBuf {
    canonicalize_existing_prefix(path)
}
