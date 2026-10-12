//! The disk backend of the sources component (§AR-system.2.13): every read,
//! listing and probe of a path the engine makes, each read reported to the
//! run's input observer before it is made (§FS-check.6.1.1, §AR-bindings.3).
//! Nothing here knows of buffers; `Sources` puts them in front of it.

use std::path::Path;

use super::observe::observe_input;

/// The file's bytes as text, observed first (§FS-check.6.1.3).
pub(crate) fn input_read_to_string(path: impl AsRef<Path>) -> std::io::Result<String> {
    if !observe_input(path.as_ref(), false) {
        return Err(std::io::Error::other("watch input coverage failed"));
    }
    std::fs::read_to_string(path)
}

/// The directory's entries, observed first, the tree under it included
/// (§FS-check.6.1.3).
pub(crate) fn input_read_dir(path: impl AsRef<Path>) -> std::io::Result<std::fs::ReadDir> {
    if !observe_input(path.as_ref(), true) {
        return Err(std::io::Error::other("watch input coverage failed"));
    }
    std::fs::read_dir(path)
}

/// Whether anything is at `path`, a dangling link included, without following
/// it: the probe the read predicate asks before it lets the walker answer
/// (§AR-scanner.1.13).
pub(crate) fn path_is_entry(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

/// Whether `path` is itself a symbolic link (§AR-scanner.1.9).
pub(crate) fn path_is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.is_symlink())
}

/// Whether `path` is a directory, links followed.
pub(crate) fn path_is_dir(path: &Path) -> bool {
    path.is_dir()
}

/// Whether `path` is a file, links followed.
pub(crate) fn path_is_file(path: &Path) -> bool {
    path.is_file()
}

/// Whether `path` names something, links followed.
pub(crate) fn path_exists(path: &Path) -> bool {
    path.exists()
}

/// A repository's bookkeeping file, unobserved: git's own metadata is not an
/// input of the run (§FS-check.6.1.1), only the ignore rules it locates are.
pub(crate) fn read_git_metadata(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}
