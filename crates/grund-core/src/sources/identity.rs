//! A file's physical identity (§AR-system.2.13, §FS-scan.identity): the key two
//! spellings of one location compare equal by, read from the disk where the
//! disk can say and lexical where the file has yet to be written, so a buffer
//! with no file behind it still has one (§FS-scan.identity.buffers).
//!
//! These came down out of `model/paths.rs` with the sources component: they ask
//! the filesystem, and only sources does (§AR-system.5).

use std::fs;
use std::path::{Component, Path, PathBuf};

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
    super::realpath_count::note_realpath();
    canonicalize_existing_prefix(path)
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

/// Where `path` goes as far as the disk answers, for a path that cannot be looked up
/// (§FS-scan.scope.homes.lookup.outward): each link on the way is read, and a component
/// behind a directory that refuses the question keeps its written name. Hops are bounded
/// as the kernel bounds them, so a link cycle ends as written rather than never.
pub(crate) fn resolved_as_far_as_answered(path: &Path) -> PathBuf {
    let absolute =
        std::env::current_dir().map_or_else(|_| path.to_path_buf(), |cwd| cwd.join(path));
    let mut pending: Vec<PathBuf> = components_reversed(&absolute);
    let mut resolved = PathBuf::new();
    let mut hops = 40;
    while let Some(next) = pending.pop() {
        if next == Path::new("..") {
            resolved.pop();
            continue;
        }
        let candidate = resolved.join(&next);
        match fs::read_link(&candidate) {
            Ok(target) if hops > 0 => {
                hops -= 1;
                if target.is_absolute() {
                    resolved.clear();
                }
                pending.extend(components_reversed(&target));
            }
            _ => resolved = candidate,
        }
    }
    resolved
}

fn components_reversed(path: &Path) -> Vec<PathBuf> {
    path.components()
        .filter(|component| !matches!(component, Component::CurDir))
        .map(|component| PathBuf::from(component.as_os_str()))
        .rev()
        .collect()
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
