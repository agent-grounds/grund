//! §AR-scanner.1.1: the ignore mode of a mounted root (§FS-remote-projects.ignore),
//! the one difference between its walk and an ordinary one, kept beside the walk
//! builder and the one-level probe walker that both apply it.

use std::path::Path;

use ignore::WalkBuilder;

use crate::sources::{canonical_path, path_is_file};

/// The ignore mode of a mounted root (§FS-remote-projects.ignore).
/// The builder reads no parent directory, no `.git/info/exclude` and no global
/// file, and reads `.gitignore` whether or not a repository encloses the root;
/// the ignore files between the mounted root and a scan root below it, which a
/// mounted member's walk starts under, are handed to it explicitly, each still
/// matched from its own directory.
pub(super) fn mounted_ignore_files(
    builder: &mut WalkBuilder,
    mounted_root: &Path,
    scan_root: &Path,
) {
    builder
        .parents(false)
        .git_exclude(false)
        .git_global(false)
        .require_git(false);
    let canonical = canonical_path(scan_root).unwrap_or_else(|_| scan_root.to_path_buf());
    let Ok(below) = canonical.strip_prefix(mounted_root) else {
        return;
    };
    let mut dir = mounted_root.to_path_buf();
    let mut levels = vec![dir.clone()];
    for component in below.components() {
        dir.push(component);
        levels.push(dir.clone());
    }
    // The scan root's own files are the walker's to read; only those above it are handed over.
    levels.pop();
    for level in levels {
        for name in [".gitignore", ".ignore"] {
            let file = level.join(name);
            if path_is_file(&file) {
                let _ = builder.add_ignore(file);
            }
        }
    }
}
