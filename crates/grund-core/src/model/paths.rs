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
