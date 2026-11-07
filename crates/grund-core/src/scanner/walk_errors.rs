use crate::sources::{Sources, canonical_path, link_metadata, open_for_reading, path_is_dir};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use super::read_predicate::{directory_refusal, ignored, name_refusal};
use super::scan_error::scan_io_reason;
use super::tree::ScanError;
use super::walk::WalkDirFilter;
use crate::config::{Frame, Schema};

/// The scan failures one walker error becomes (§FS-check.2.4), each beside the
/// error it is reported from, which the caller retains (§FS-distribution.3.3.2).
///
/// One error is one failure, located by `walk_error_report`, except the error
/// the walker raises about a directory link whose target cannot be opened: that
/// one names no entry, so the links it can be about are recovered first and each
/// is reported at its own path, never at the walk root (§AR-scanner.1.9). Each recovered
/// link is handed on as an error that names it, carrying the failure its own
/// target opens with, so it takes the branch every link with a path takes. An
/// error about a link already recovered adds nothing, and reaches no fallback.
///
/// `filter` is the walk's own directory filter for this root, `recovery` what this
/// root has recovered so far, and `listing` the directory the walk last descended
/// into under it, `None` before the first.
pub(super) fn walk_error_reports(
    err: ignore::Error,
    filter: &WalkDirFilter,
    recovery: &mut LinkRecovery,
    listing: Option<&Path>,
    schema: &Schema,
    frame: Frame<'_>,
    scan_root: &Path,
) -> Vec<(ScanError, ignore::Error)> {
    let Some(links) = recovery.unopenable_links(&err, listing, scan_root) else {
        return walk_error_report(&err, filter, schema, frame, scan_root)
            .map(|report| (report, err))
            .into_iter()
            .collect();
    };
    links
        .into_iter()
        .map(|(path, io)| ignore::Error::WithPath {
            path,
            err: Box::new(ignore::Error::Io(io)),
        })
        .filter_map(|located| {
            walk_error_report(&located, filter, schema, frame, scan_root)
                .map(|report| (report, located))
        })
        .collect()
}

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
/// resolve — a broken target, a loop, which the `ignore` crate detects and
/// reports rather than recursing into, or a directory whose target cannot be
/// opened, which `walk_error_reports` has already located at the link. The
/// walker applies neither its directory filter nor its ignore files to an error
/// entry, so the tests the link would have faced are asked of the one read
/// predicate instead (§AR-scanner.1.13): the hidden-name and `[scan] exclude`
/// tests for a looping directory, the ignore files for every kind, the name test
/// as well for a broken link, which names a file where a loop names a directory,
/// and the walk's own directory filter for a link whose target is a directory,
/// the project-root fence and the ownership boundaries on its canonical target
/// included (§AR-scanner.1.13). That is what keeps a dangling
/// `docs/logo.png`, a link of any kind that `.gitignore` covers, and a link out
/// of the project, silent — the walk was never going to read them. Anything else
/// — an unreadable directory, a malformed ignore file — is reported as it stands,
/// at the path the error names, and at the walk root only when the error names
/// no entry at all: it is a hole in the walk either way, and a silent one is what
/// §REQ-no-missed-citation.1 rules out.
fn walk_error_report(
    err: &ignore::Error,
    filter: &WalkDirFilter,
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
    if walk_skips_link(path, filter, schema, frame, scan_root) {
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

/// Whether the walk was never going to read through the link at `path`, judged
/// by the rules any other entry faces (§FS-config.3.5.6). A link whose
/// target is a directory is asked what the walk asks a directory: the ignore
/// files, then the walk's own filter, whose `refusal` is the one copy of the name,
/// `[scan] exclude`, unwalked-home and E2E-case tests and of the project-root fence
/// and ownership boundaries, which it judges on the link's canonical target
/// whether or not that target opens (§AR-scanner.1.9, §AR-scanner.1.13). Any other link, broken, is asked the name tests and the
/// ignore files, as the file it names would be.
///
/// The walker never drops its own root, so a link that is the walk root is asked
/// no ignore file, and the filter answers for it as it does for the root.
fn walk_skips_link(
    path: &Path,
    filter: &WalkDirFilter,
    schema: &Schema,
    frame: Frame<'_>,
    scan_root: &Path,
) -> bool {
    if !path_is_dir(path) {
        return name_refusal(path, schema).is_some() || walk_ignores(path, false, schema, frame);
    }
    let depth = path
        .strip_prefix(scan_root)
        .map_or(0, |relative| relative.components().count());
    (depth > 0 && walk_ignores(path, true, schema, frame))
        || filter.refusal(path, depth, true, true).is_some()
}

/// The directory links one walk root has recovered, by the directory listed again
/// to find them and the kind of failure their targets open with (§AR-scanner.1.9).
/// A directory is here once it has been listed, so it is listed once per walk root
/// however many errors it explains, and a link is handed out once. The walk builds
/// this on a root's first error, so a run with no walker error pays nothing for it.
#[derive(Default)]
pub(super) struct LinkRecovery {
    listed: HashMap<PathBuf, UnreportedLinks>,
}

/// The links of one directory not yet handed out. An emptied entry stays: a kind
/// with an entry here is one this directory explains, whatever is left under it.
type UnreportedLinks = HashMap<io::ErrorKind, Vec<(PathBuf, io::Error)>>;

impl LinkRecovery {
    /// The links a walker error that names no entry brings to light, each with the
    /// failure its target opens with (§AR-scanner.1.9, §FS-check.2.4). `None` where
    /// no link explains the error, which any other error is; an empty list where a
    /// link does and every such link has already been handed out.
    ///
    /// The walker opens a directory link's target for its loop check before the entry
    /// is anything it hands over, and the error that open raises carries no path
    /// (§AR-scanner.1.9). The link is an entry of the directory the walk was listing,
    /// and that is `listing` or a directory above it: the sequential walk hands a
    /// directory over before anything under it and leaves it only once its listing is
    /// done, so every directory handed over since is inside it. Those directories, up
    /// to the walk root, are listed again, and each directory link there whose target
    /// fails to open the way the error did is one this error can be about. The walker
    /// lists every entry of each of them and fails on each such link in turn, so a
    /// link named here before its own error comes finds nothing more to add when that
    /// error does, and several links in one directory are each named. A target that
    /// opens, or fails another way, is not what this error is about. Nothing tells
    /// such an error from another pathless one of the same kind, so one raised beside
    /// a barred link is reported as that link.
    ///
    /// Only the error path pays: the walk records `listing` among the directories it
    /// already collects, and nothing here runs until an error names no entry.
    pub(super) fn unopenable_links(
        &mut self,
        err: &ignore::Error,
        listing: Option<&Path>,
        scan_root: &Path,
    ) -> Option<Vec<(PathBuf, io::Error)>> {
        let kind = pathless_io_kind(err)?;
        let mut links = None;
        for dir in listing?
            .ancestors()
            .take_while(|dir| dir.starts_with(scan_root))
        {
            let unreported = self
                .listed
                .entry(dir.to_path_buf())
                .or_insert_with(|| unopenable_links_of(dir));
            if let Some(found) = unreported.get_mut(&kind) {
                links.get_or_insert_with(Vec::new).append(found);
            }
        }
        links
    }
}

/// The directory links directly in `dir` whose targets exist and fail to open, by
/// the kind of that failure. A directory that cannot be listed holds none: it
/// fails on its own path.
fn unopenable_links_of(dir: &Path) -> UnreportedLinks {
    let mut links = UnreportedLinks::new();
    Sources::on_disk()
        .read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .is_ok_and(|file_type| file_type.is_symlink())
        })
        .map(|entry| entry.path())
        .filter(|link| path_is_dir(link))
        .filter_map(|link| open_for_reading(&link).err().map(|error| (link, error)))
        .for_each(|(link, error)| {
            links.entry(error.kind()).or_default().push((link, error));
        });
    links
}

/// The I/O error kind of a walker error that names no entry, the form `ignore`
/// gives the error walkdir raises with no path; `None` for every other error.
fn pathless_io_kind(err: &ignore::Error) -> Option<io::ErrorKind> {
    match err {
        ignore::Error::Io(io) => Some(io.kind()),
        ignore::Error::WithDepth { err, .. } => pathless_io_kind(err),
        _ => None,
    }
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
/// Its typed I/O reason is the same as a file read's (§FS-check.2.4.1).
fn walk_error_reason(err: &ignore::Error) -> String {
    match err.io_error() {
        Some(io) => scan_io_reason(io),
        None => match err {
            ignore::Error::WithPath { err, .. } => err.to_string(),
            other => other.to_string(),
        },
    }
}
