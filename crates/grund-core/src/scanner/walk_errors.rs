use crate::sources::{canonical_path, link_metadata};
use std::path::{Path, PathBuf};

use super::read_predicate::{directory_refusal, ignored, name_refusal};
use super::tree::ScanError;
use crate::config::{Frame, Schema};

/// The per-file scan failure a walker error becomes (§FS-check.2.4), or `None` when
/// the walk was never going to read through the path it names (§FS-config.3.5.6).
///
/// What the walk does with a path it cannot read (§AR-scanner.1.9, §FS-check.2.4):
/// which walker errors become per-file scan failures, which stay silent because
/// the ordinary walk was never going to read the path anyway, and which link a
/// loop is reported at. It sits beside `walk.rs` because it is the other half of
/// the walk §AR-scanner.1 describes — the traversal there, its scan error
/// handling here — and the two meet only at the error list one hands the other.
///
/// `follow_links` hands back an error in place of the entry for a link it cannot
/// resolve — a broken target, or a loop, which the `ignore` crate detects and
/// reports rather than recursing into. The walker applies neither its directory
/// filter nor its ignore files to an error entry, so the tests the link would
/// have faced are asked of the one read predicate instead (§AR-scanner.1.13):
/// the hidden-name and `[scan] exclude` tests for a looping directory, the ignore
/// files for either kind, and the name test as well for a broken link, which
/// names a file where a loop names a directory. That is what keeps a dangling `docs/logo.png`, and a link
/// of either kind that `.gitignore` covers, silent — the walk was never going to
/// read them. Anything else — an unreadable directory, a malformed ignore file —
/// is reported as it stands, at the walk root when the error names no path of its
/// own: it is a hole in the walk either way, and a silent one is what
/// §REQ-no-missed-citation.1 rules out.
pub(super) fn walk_error_report(
    err: &ignore::Error,
    schema: &Schema,
    frame: Frame<'_>,
    scan_root: &Path,
) -> Option<ScanError> {
    if let Some((child, ancestor)) = walk_error_loop(err) {
        // §FS-config.3.5.2: the finding names the link, and the path the walker
        // hands back is not always it — a loop met one link deep is reported at
        // the doubled spelling the descent produced.
        let link = shortest_link_spelling(child, scan_root).unwrap_or_else(|| child.to_path_buf());
        // Naming the ancestor is the useful half of the message, but it says
        // nothing when the ancestor *is* the link: the target then reaches back
        // over the walk root, which no in-tree name below it describes.
        let ancestor = (!ancestor.starts_with(&link)).then_some(ancestor);
        return symlink_loop_report(&link, ancestor, schema, frame);
    }
    let path = walk_error_path(err).unwrap_or(scan_root);
    if !link_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Some((path.to_path_buf(), walk_error_reason(err)));
    }
    if name_refusal(path, schema).is_some() || walk_ignores(path, false, schema, frame) {
        return None;
    }
    let reason = match err.io_error().map(std::io::Error::kind) {
        Some(std::io::ErrorKind::NotFound) => {
            "broken symlink: the target does not exist".to_string()
        }
        _ => format!("unreadable symlink: {}", walk_error_reason(err)),
    };
    Some((path.to_path_buf(), reason))
}

/// The scan error a looping directory link earns (§FS-config.3.5.5), or `None`
/// where the walk was never going to read through it: the hidden-name and
/// `[scan] exclude` tests the walker's own directory filter never applies to an
/// error entry, and the ignore files, which reach a link the walker could not
/// resolve no other way.
///
/// `ancestor` is the directory the link reaches back into, and `None` where the
/// target reaches over the walk root, which no in-tree name below it describes. Shared by the two ways a loop is found: the walker's
/// own detection, and the filter pruning a link whose target is at or above the
/// walk root — which the walker cannot see without descending a second copy of
/// the tree first (§AR-scanner.1.9).
///
/// Why the ancestor name can be missing: the config root renders as nothing at
/// all, and a target at or above the walk root has no in-tree name below it
/// either. Both read the same way — the target is not somewhere inside the tree,
/// it is the tree.
pub(super) fn symlink_loop_report(
    link: &Path,
    ancestor: Option<&Path>,
    schema: &Schema,
    frame: Frame<'_>,
) -> Option<ScanError> {
    if directory_refusal(link, &schema.sources.exclude).is_some()
        || walk_ignores(link, true, schema, frame)
    {
        return None;
    }
    // Naming the directory is the useful half of the message, and it is available
    // only where that directory has a name *in the report* — the config root
    // renders as nothing at all (§FS-config.3.6).
    let ancestor = ancestor
        .map(|ancestor| frame.display_path(ancestor))
        .filter(|name| !name.is_empty());
    let reason = match ancestor {
        Some(name) => format!("symlink loop: the target is the ancestor directory {name}"),
        None => "symlink loop: the target contains the link".to_string(),
    };
    Some((link.to_path_buf(), reason))
}

/// The link `path` names, spelled as the walk root reaches it (§FS-config.3.5.2).
/// A walk that descends through a link meets one link under two spellings —
/// `docs/up -> ..` is met again as `docs/up/docs/up`, and `docs/a/link -> ../b`
/// with `docs/b/link -> ../a` is met as `docs/a/link/link` — and the one to
/// report is the name the user can go and fix.
///
/// The link is identified by the canonical *directory* holding it plus its own
/// name, so a link two spellings deep is recognized as the different link it is:
/// with `docs/shared -> ../shared-docs` and a `self -> .` inside it, the loop is
/// at `docs/shared/self` and `docs/shared` is innocent. That identity is then
/// re-expressed under the walk root, which is what finds the *sibling* spelling
/// a prefix search cannot: `docs/a/link/link` is `docs/b/link`, and no prefix of
/// the reported path is. A link whose canonical directory lies outside the walk
/// root has no in-tree name of its own, so the reported path stands. Only the
/// error path pays for any of it.
fn shortest_link_spelling(path: &Path, scan_root: &Path) -> Option<PathBuf> {
    let identity = link_identity(path)?;
    let canonical_root = canonical_path(scan_root).ok()?;
    let relative = identity.strip_prefix(&canonical_root).ok()?;
    let candidate = scan_root.join(relative);
    // The canonical directory may be reached under several in-tree names; this
    // one is the walk root's own, and it counts only if it really is the link.
    (link_identity(&candidate)? == identity).then_some(candidate)
}

/// Which directory entry a path names, independent of how the walk spelled the
/// directories above it: the canonical parent plus the final component. Unlike
/// `canonicalize` on the whole path, this does not resolve the entry itself, so a
/// link is identified as the link rather than as its target.
fn link_identity(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?;
    Some(canonical_path(path.parent()?).ok()?.join(name))
}

/// Whether the ignore files drop the link at `path` had it resolved — the half
/// of the question the name tests cannot answer (§FS-config.3.5.6,
/// §AR-scanner.1.1). The walker applies `.gitignore` and friends to the entries
/// it *yields*, and a link it cannot resolve arrives as an error instead, so the
/// predicate's `ignored` asks the walker about the link as an ordinary entry of
/// its own directory. Only a positive "the walker filtered this out" suppresses
/// the report: a silent skip is the failure §REQ-no-missed-citation.1 rules out.
fn walk_ignores(path: &Path, is_dir: bool, schema: &Schema, frame: Frame<'_>) -> bool {
    schema.sources.respect_gitignore
        && ignored(path, is_dir, frame.run.workspace.mounted_root.as_deref())
}

/// The `(link, ancestor)` of a symlink loop, at any nesting depth of the error.
fn walk_error_loop(err: &ignore::Error) -> Option<(&Path, &Path)> {
    match err {
        ignore::Error::Loop { ancestor, child } => Some((child, ancestor)),
        ignore::Error::WithPath { err, .. } | ignore::Error::WithDepth { err, .. } => {
            walk_error_loop(err)
        }
        _ => None,
    }
}

/// The in-tree path a walker error is about, at any nesting depth.
fn walk_error_path(err: &ignore::Error) -> Option<&Path> {
    match err {
        ignore::Error::WithPath { path, .. } => Some(path),
        ignore::Error::WithDepth { err, .. } => walk_error_path(err),
        _ => None,
    }
}

/// A walker error's reason, without the path `ignore` writes into its own text:
/// the diagnostic already carries the path (§FS-errors.2.2.1), and the one `ignore`
/// writes is absolute, which a report may not contain (§FS-errors.4).
fn walk_error_reason(err: &ignore::Error) -> String {
    match err.io_error() {
        Some(io) => std::io::Error::from(io.kind()).to_string(),
        None => match err {
            ignore::Error::WithPath { err, .. } => err.to_string(),
            other => other.to_string(),
        },
    }
}
