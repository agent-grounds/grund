//! The path keys every component compares a file by (§AR-system.2.2): the
//! lexical normalization, the physical (symlink-resolved) key, and the
//! scanned-relative form a configured home is matched against.
//!
//! Tiny helpers over `Path` and nothing else, which is why they are model's:
//! the scanner keys its walk and its value spans by them (§AR-scanner.1), the
//! checker matches `[[kinds]]` homes with them (§FS-check.3.7), and `show` and
//! the api compare a requested file to a recorded one. They sat in
//! `checker/homes.rs` while the checker was a file-name category, which had the
//! scanner reading a path helper out of the component above it
//! (§AR-system.4).
//!
//! The pair that matters is *scanned* against *physical*: a configured home is
//! a prefix of the path the walk recorded, not of the symlink target it resolves
//! to (§FS-config.3.4), while two paths name one location only when their
//! resolved forms agree.
//!
//! Three report spellings came down beside them when §AR-system.2.9 became a
//! module: `format_path`, `sort_path_key` and `relative_from_base`, which sat in
//! the deprecated path's `output` category while config, workspace, the scanner,
//! the checker, the queries and the writers all read them upward
//! (§AR-system.4). They are functions of a `Path` and nothing else; what needs a
//! `Config` to answer — which base a report spells a path against
//! (§FS-config.3.6) — stayed one level up, in `config/report_paths.rs`. The
//! canonicalization an editor's request URI is matched by came the same way, out
//! of `scanner/tree.rs`: the queries and the writers are siblings, so the
//! snapshot path they both rebase against had to sit below the pair of them
//! (§AR-lsp.5).

use std::borrow::Cow;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// `path` with `.` dropped and `..` applied textually — no filesystem access, so
/// it answers for a path that does not exist and never follows a link.
pub(crate) fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// Whether two paths name one file. Canonicalized where the filesystem can say
/// so, which is what makes a declaration reached through a symlinked directory
/// the same home as the one reached directly (§FS-check.3.4).
pub(crate) fn paths_same_location(left: &Path, right: &Path) -> bool {
    physical_path_key(left) == physical_path_key(right)
}

/// The key a *location* is compared by: the resolved path where it exists, the
/// lexical form where it does not, so a path that has yet to be written still
/// has a key.
pub(crate) fn physical_path_key(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| normalize_path_lexically(path))
}

/// The key a *scanned* path is compared by: lexical only, because the walk
/// records what the configuration wrote and a configured home is a prefix of
/// that rather than of its target (§FS-config.3.4).
pub(crate) fn scanned_path_key(path: &Path) -> PathBuf {
    normalize_path_lexically(path)
}

/// The same key for a home written in `grund.toml` as a relative string.
pub(crate) fn configured_home_path_key(home: &str) -> PathBuf {
    scanned_path_key(Path::new(home))
}

/// `path` relative to the config root, under whichever spelling of that root
/// reaches it — the physical one for a path the walk canonicalized, the
/// configured one for a path recorded as written — and `None` when neither does,
/// which is a file outside the project (§FS-check.3.7).
pub(crate) fn scanned_decl_relative_path<'a>(
    path: &'a Path,
    configured_root: &Path,
    physical_root: &Path,
) -> Option<Cow<'a, Path>> {
    if let Ok(relative) = path.strip_prefix(physical_root) {
        return Some(Cow::Borrowed(relative));
    }
    if let Ok(relative) = path.strip_prefix(configured_root) {
        return Some(Cow::Owned(scanned_path_key(relative)));
    }

    let path = scanned_path_key(path);
    if let Ok(relative) = path.strip_prefix(physical_root) {
        return Some(Cow::Owned(scanned_path_key(relative)));
    }
    if let Ok(relative) = path.strip_prefix(configured_root) {
        return Some(Cow::Owned(scanned_path_key(relative)));
    }
    None
}

/// One path spelled the way every report spells it: forward slashes, whatever
/// the platform's separator is, so a reported path is one string on every OS
/// (§FS-errors.4).
pub(crate) fn format_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// The key a list of paths is ordered by (§FS-errors.4). The same spelling
/// `format_path` renders, so what a reader sorts by is what they see.
pub(crate) fn sort_path_key(path: &Path) -> String {
    format_path(path)
}

/// `path` expressed relative to `base`, walking up with `..` for a target that
/// lies *outside* it (§FS-errors.4: a report never carries an absolute path, and a
/// reader resolves what it prints against the directory they are standing in). The
/// case that needs it is a config file **above** the run's root: an enclosing
/// workspace's `members` line, reported at a run narrowed into one of its members
/// (§FS-workspace.6.1). Both paths are canonical there. A pair with no shared
/// component at all — different Windows prefixes — has no relative form, so the
/// target is returned unchanged.
pub(crate) fn relative_from_base(base: &Path, path: &Path) -> PathBuf {
    if let Ok(inside) = path.strip_prefix(base) {
        return inside.to_path_buf();
    }
    let base_parts: Vec<_> = base.components().collect();
    let path_parts: Vec<_> = path.components().collect();
    let shared = base_parts
        .iter()
        .zip(&path_parts)
        .take_while(|(a, b)| a == b)
        .count();
    if shared == 0 {
        return path.to_path_buf();
    }
    let mut relative = PathBuf::new();
    for _ in shared..base_parts.len() {
        relative.push("..");
    }
    relative.extend(&path_parts[shared..]);
    relative
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
/// and citation paths (§AR-lsp.5). Existing files resolve through
/// `fs::canonicalize`; a not-yet-saved overlay file resolves its existing
/// prefix and appends the missing tail — the same absolutization the snapshot
/// applies — so `grund-lsp` does not need a second, drift-prone copy of this
/// logic.
pub fn canonical_snapshot_path(path: &Path) -> PathBuf {
    canonicalize_existing_prefix(path)
}
