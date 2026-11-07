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

/// The key a *location* is compared by: the resolved path where it exists, the
/// lexical form where it does not, so a path that has yet to be written still
/// has a key.
pub(crate) fn physical_path_key(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| normalize_path_lexically(path))
}

/// `path` absolutized and resolved as far as the filesystem can: the canonical
/// form where every component exists, else the canonical form of the longest
/// existing prefix with the missing tail appended. A path that has yet to be
/// written therefore still has one spelling, which is what lets the walk and an
/// editor's not-yet-saved buffer be compared (§AR-scanner.1, §AR-lsp.5).
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
    let mut suffix = PathBuf::new();
    let mut cursor = path.as_path();
    while !cursor.exists() {
        let Some(name) = cursor.file_name() else {
            break;
        };
        suffix = Path::new(name).join(suffix);
        let Some(parent) = cursor.parent() else {
            break;
        };
        cursor = parent;
    }
    fs::canonicalize(cursor)
        .unwrap_or_else(|_| normalize_path_lexically(cursor))
        .join(suffix)
}

/// Canonicalize `path` to the same normalized form `LspSnapshot` paths carry,
/// so an LSP client's request URI matches the snapshot's declaration, stub,
/// and citation paths (§AR-lsp.5.1). Existing files resolve through
/// `fs::canonicalize`; a not-yet-saved overlay file resolves its existing
/// prefix and appends the missing tail — the same absolutization the snapshot
/// applies — so `grund-lsp` does not need a second, drift-prone copy of this
/// logic.
pub fn canonical_snapshot_path(path: &Path) -> PathBuf {
    canonicalize_existing_prefix(path)
}
