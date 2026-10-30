//! §AR-scanner.1.1: the ignore mode of a mounted root (§FS-remote-projects.ignore),
//! the one difference between its walk and an ordinary one, kept beside the walk
//! builder and the one-level probe walker that both apply it.

use std::path::Path;

use ignore::WalkBuilder;

use super::global_excludes::GlobalExcludes;
use crate::sources::canonical_path;

/// Put `builder` in the ignore mode of a mounted root (§FS-remote-projects.ignore):
/// it reads no parent directory, no `.git/info/exclude` and no global file, and it
/// reads `.gitignore` whether or not a repository encloses the root.
pub(super) fn mounted_ignore_mode(builder: &mut WalkBuilder) {
    builder
        .parents(false)
        .git_exclude(false)
        .git_global(false)
        .require_git(false);
}

/// The rules a walk from `scan_root` must still ask of each entry, because the
/// builder cannot read them: the ignore files between the mounted root and a scan
/// root below it, which a mounted member's walk starts under. They are asked in the
/// walker's own precedence, each matched from its own directory, through the
/// rule-by-rule predicate; `None` where the walk starts at the mounted root itself
/// and the builder reads every file there is (§GOAL-fast-feedback).
pub(super) fn rules_above(mounted_root: &Path, scan_root: &Path) -> Option<GlobalExcludes> {
    let canonical = canonical_path(scan_root).unwrap_or_else(|_| scan_root.to_path_buf());
    (canonical != mounted_root && canonical.starts_with(mounted_root))
        .then(|| GlobalExcludes::for_mount(mounted_root))
}
