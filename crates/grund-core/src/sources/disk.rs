//! The disk backend of the sources component (§AR-system.2.13): every read,
//! listing and probe of a path the engine makes, each read reported to the
//! run's input observer before it is made (§FS-check.6.1.1, §AR-bindings.3).
//! Nothing here knows of buffers; `Sources` puts them in front of it.

use std::path::{Path, PathBuf};

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
pub(crate) fn path_is_entry(path: impl AsRef<Path>) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

/// Whether `path` is itself a symbolic link (§AR-scanner.1.9).
pub(crate) fn path_is_symlink(path: impl AsRef<Path>) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.is_symlink())
}

/// Whether `path` is a directory, links followed.
pub(crate) fn path_is_dir(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_dir()
}

/// Whether `path` is a file, links followed.
pub(crate) fn path_is_file(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}

/// Whether `path` names something, links followed.
pub(crate) fn path_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// The file's bytes as the disk holds them: what a writer reads of the file it
/// replaces and of the tree it writes into, which no run observes (§FS-fetch.4).
pub(crate) fn read_bytes(path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
}

/// A repository's bookkeeping file, unobserved: git's own metadata is not an
/// input of the run (§FS-check.6.1.1), only the ignore rules it locates are.
pub(crate) fn read_git_metadata(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

/// The path with every link resolved, as the disk answers it: the physical
/// identity a reader compares locations by (§FS-scan.identity.buffers).
pub(crate) fn canonical_path(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(path)
}

/// What the disk says of the entry at `path`, links followed.
pub(crate) fn path_metadata(path: impl AsRef<Path>) -> std::io::Result<std::fs::Metadata> {
    std::fs::metadata(path)
}

/// What the disk says of the entry at `path` itself, a link not followed.
pub(crate) fn link_metadata(path: impl AsRef<Path>) -> std::io::Result<std::fs::Metadata> {
    std::fs::symlink_metadata(path)
}

/// The target a symbolic link at `path` names, as written.
pub(crate) fn read_link(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    std::fs::read_link(path)
}

/// Whether an entry's type, as a listing or a link's metadata reports it, is a
/// regular file.
pub(crate) fn is_regular_file(file_type: std::fs::FileType) -> bool {
    file_type.is_file()
}
