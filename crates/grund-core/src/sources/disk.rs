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
