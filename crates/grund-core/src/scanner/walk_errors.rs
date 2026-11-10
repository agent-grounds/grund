use crate::sources::{
    Sources, canonical_path, link_metadata, open_for_reading, path_is_dir, physical_path_key,
};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use super::read_predicate::{Unread, ignore_file_unapplied, ignored, name_refusal};
use super::scan_error::{FileKind, Place, ScanFailure, unreadable_link_reason};
use super::tree::ScanError;
use super::walk::WalkDirFilter;
use crate::config::{Frame, Schema, canonical_config_root};
use crate::sources::io_reason;

/// The scan failures one walker error becomes (§FS-check.2.4), each beside the
/// error it is reported from, which the caller retains (§FS-distribution.3.3.2).
///
/// One error is one failure, located by `walk_error_report`, except the error
/// the walker raises about a directory link whose target cannot be opened: that
/// one names no entry, so the links it can be about are recovered first and each
/// is reported at its own path, never at the walk root (§AR-scanner.1.9). Each recovered
/// link is handed on as an error that names it, carrying the failure its own
/// target opens with, so it takes the branch every link with a path takes. An
/// error that a recovered link explains and that has no link left to name adds
/// nothing; one no link explains reaches the walk-root fallback, so a failure may
/// land at a walk root and never disappears.
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
        // §FS-check.2.4.2: errors `ignore` folds into one are each at the file that raised them.
        return split_by_path(err)
            .into_iter()
            .filter_map(|err| {
                walk_error_report(&err, filter, schema, frame, scan_root)
                    .map(|report| (report, err))
            })
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

/// The scan failure at a walk root whose lookup failed for a reason other than absence
/// (§FS-check.2.4), beside the error it is reported from, as a walker error's is.
pub(super) fn root_lookup_failure(
    root: &Path,
    error: io::Error,
    version: u32,
) -> (ScanError, ignore::Error) {
    let located = ignore::Error::WithPath {
        path: root.to_path_buf(),
        err: Box::new(ignore::Error::Io(error)),
    };
    let unread = ScanFailure::Unread {
        place: Place::Entry,
        version,
    };
    let reason = walk_error_reason(&located);
    (ScanError::new(root.to_path_buf(), reason, unread), located)
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
/// predicate instead (§AR-scanner.1.13): the hidden-name and `exclude`
/// tests for a looping directory, the ignore files for every kind, the name test
/// as well for a broken link, which names a file where a loop names a directory,
/// and the walk's own directory filter for a link whose target is a directory,
/// the project-root fence and the ownership boundaries on its canonical target
/// included (§AR-scanner.1.13). That is what keeps a dangling
/// `docs/logo.png`, a link of any kind that `.gitignore` covers, and a link out
/// of the project to a target in no tree the run reads, silent — the walk was never
/// going to read them, and nothing else in the run reads through them. A directory
/// link into a tree another reader of the run holds is pruned from this walk and is
/// still reported (§AR-scanner.1.9). Anything else
/// — an unreadable directory, a malformed ignore file the run applies — is reported as it
/// stands, at the path the error names, and at the walk root only when the error names
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
        return symlink_loop_report(&link, ancestor, filter, schema, frame);
    }
    let path = walk_error_path(err).unwrap_or(scan_root);
    let ignore_file_line = is_ignore_file_line_error(err);
    // §DF-unparsed-ignore-file-at-any-depth: asked where every route a walker error comes by
    // meets, so whether a walk root sits below the file cannot change the answer.
    if ignore_file_line && ignore_file_unapplied(path, filter, frame) {
        return None;
    }
    // §FS-check.2.4.2: an ignore file that is a link was resolved and read, so a line of it
    // that does not parse is no link failure for the rules below to silence.
    let unread = ScanFailure::Unread {
        place: Place::Entry,
        version: frame.version,
    };
    // §FS-check.2.4.3: an ignore file's bad line is the one walker error that read and did not
    // parse; every other is unread, and the walk judges its place once it has them all.
    if ignore_file_line {
        let failure = ScanFailure::Unparsed(FileKind::Ignore);
        return Some(ScanError::new(
            path.to_path_buf(),
            walk_error_reason(err),
            failure,
        ));
    }
    if !link_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Some(ScanError::new(
            path.to_path_buf(),
            walk_error_reason(err),
            unread,
        ));
    }
    if walk_skips_link(path, filter, schema, frame, scan_root) {
        return None;
    }
    let reason = match err.io_error().map(std::io::Error::kind) {
        Some(std::io::ErrorKind::NotFound) => {
            "broken symlink: the target does not exist".to_string()
        }
        _ => unreadable_link_reason(&walk_error_reason(err)),
    };
    Some(ScanError::new(path.to_path_buf(), reason, unread))
}

/// Whether the walk was never going to read through the link at `path`, judged
/// by the rules any other entry faces (§FS-config.3.5.6). A link whose
/// target is a directory is asked what the walk asks a directory: the ignore
/// files, then the walk's own filter, whose `refusal` is the one copy of the name,
/// `[scan] exclude`, unwalked-home and E2E-case tests and of the project-root fence
/// and ownership boundaries, which it judges on the link's canonical target
/// whether or not that target opens (§AR-scanner.1.9, §AR-scanner.1.13). Two of its
/// answers are not silence. `Unread::AnotherProject`: the walk does not read
/// through such a link, but the run reads the tree it lands in, so a failure there
/// is a hole in what the run answers from (§AR-scanner.1.9).
/// `Unread::UnwalkedHome`: a link standing at a home the config lists without walking
/// is in no declared blind spot (§AR-scanner.1.9). Any
/// other link, broken, is asked the name tests and the ignore files, as the file it
/// names would be.
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
        || filter
            .refusal(path, depth, true, true)
            .is_some_and(|reason| !matches!(reason, Unread::AnotherProject | Unread::UnwalkedHome))
}

/// The directory links one walk root has recovered, by the directory listed again
/// to find them and the kind of failure their targets open with (§AR-scanner.1.9).
/// A directory is here once it has been listed, so it is listed once per walk root
/// however many errors it explains, and a link is handed out once. The walk builds
/// this on a root's first error, so a run with no walker error pays nothing for it.
#[derive(Default)]
pub(super) struct LinkRecovery {
    listed: HashMap<PathBuf, HashMap<io::ErrorKind, ListedLinks>>,
}

/// One directory's links of one failure kind: those not yet handed out, and how many
/// walker errors they can still explain, one per link (§AR-scanner.1.9). The two
/// differ because several links can be handed out on one error, and it is the credits
/// that tell an error with nothing new to name from an error no link explains.
#[derive(Default)]
struct ListedLinks {
    unreported: Vec<(PathBuf, io::Error)>,
    credits: usize,
}

impl LinkRecovery {
    /// The links a walker error that names no entry brings to light, each with the
    /// failure its target opens with (§AR-scanner.1.9, §FS-check.2.4). `None` where
    /// no link is left to explain the error, which any other error is; an empty list
    /// where one is and every such link has already been handed out.
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
    /// opens, or fails another way, is not what this error is about.
    ///
    /// Nothing tells such an error from another pathless one of the same kind, so each
    /// link answers for one error, whichever comes first. The error beyond the last
    /// credit returns `None` and reaches the walk-root fallback, which keeps it from
    /// disappearing beside a link the rules silence (§REQ-no-missed-citation.1).
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
        let mut links = Vec::new();
        let mut explained = false;
        for dir in listing?
            .ancestors()
            .take_while(|dir| dir.starts_with(scan_root))
        {
            let listed = self
                .listed
                .entry(dir.to_path_buf())
                .or_insert_with(|| unopenable_links_of(dir));
            let Some(found) = listed.get_mut(&kind) else {
                continue;
            };
            links.append(&mut found.unreported);
            if !explained && found.credits > 0 {
                found.credits -= 1;
                explained = true;
            }
        }
        explained.then_some(links)
    }
}

/// The directory links directly in `dir` whose targets exist and fail to open, by
/// the kind of that failure. A directory that cannot be listed holds none: it
/// fails on its own path.
fn unopenable_links_of(dir: &Path) -> HashMap<io::ErrorKind, ListedLinks> {
    let mut links = HashMap::<io::ErrorKind, ListedLinks>::new();
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
            let listed = links.entry(error.kind()).or_default();
            listed.unreported.push((link, error));
            listed.credits += 1;
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
/// `exclude` tests — a version 1 name or a version 2 pattern, asked of `filter`,
/// the walk's own directory filter, which never applies them to an error entry
/// (§FS-config-v2.schema.sources.exclude.roots) — and the ignore files, which
/// reach a link the walker could not resolve no other way.
///
/// `ancestor` is the directory the link reaches back into, under any spelling that
/// reaches it: `loop_ancestor_name` decides what the reason calls it, so neither of
/// the two ways a loop is found does. Those are the walker's own detection, and the
/// filter pruning a link whose target is at or above the walk root — which the walker
/// cannot see without descending a second copy of the tree first (§AR-scanner.1.9).
pub(super) fn symlink_loop_report(
    link: &Path,
    ancestor: &Path,
    filter: &WalkDirFilter,
    schema: &Schema,
    frame: Frame<'_>,
) -> Option<ScanError> {
    // §FS-config-v2.schema.sources.exclude.roots: a pattern matching the link keeps its loop out.
    if filter.kept_out_by_exclude(link) || walk_ignores(link, true, schema, frame) {
        return None;
    }
    // Naming the directory is the useful half of the message, and it is available
    // only where that directory has a name *in the report* — the config root
    // renders as nothing at all (§FS-config.3.6).
    let ancestor = Some(loop_ancestor_name(ancestor, frame)).filter(|name| !name.is_empty());
    let reason = match ancestor {
        Some(name) => format!("symlink loop: the target is the ancestor directory {name}"),
        None => "symlink loop: the target contains the link".to_string(),
    };
    // §FS-check.2.4.3: the walk judges where the link is once it has all its failures.
    let failure = ScanFailure::Unread {
        place: Place::Entry,
        version: frame.version,
    };
    Some(ScanError::new(link.to_path_buf(), reason, failure))
}

/// The one name a loop's reason gives the directory the link reaches back into, the
/// same on every route to the link (§FS-check.2.4.2): its physical location under the
/// config root, and never the name a walk root or a link gave it on the way. `frame` is
/// the walk's, so a workspace member or an import spells the location from its own root
/// even where the line's path is spelled from the workspace root, unless the run reports
/// from the invocation. The directory it is spelled from renders empty, which the caller
/// reads as having no name.
///
/// The config root is re-joined because it may itself be spelled through a link, and
/// the report strips that spelling, not the physical one. Outside the config root there
/// is no such location, so the name keeps each walk root's own spelling, which is
/// relative for an included root: a link out of the project is fenced, so only a walk
/// root reaches that tree, and the filter's prune never hands one in (§AR-scanner.1.2).
/// Where the directory has gone since the walk met it, the comparison is lexical
/// (`physical_path_key`), the same comparison a path that has no resolved form gets
/// everywhere else.
fn loop_ancestor_name(ancestor: &Path, frame: Frame<'_>) -> String {
    let root = frame.root();
    let location = physical_path_key(ancestor);
    match location.strip_prefix(canonical_config_root(root)) {
        Ok(rest) => frame.display_path(&root.join(rest)),
        Err(_) => frame.display_path(ancestor),
    }
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
pub(super) fn link_identity(path: &Path) -> Option<PathBuf> {
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

/// Whether a walker error is about lines of an ignore file, the only errors `ignore` tags with a
/// line number, a `Partial` being several of them (§FS-check.2.4.2).
fn is_ignore_file_line_error(err: &ignore::Error) -> bool {
    match err {
        ignore::Error::WithLineNumber { .. } => true,
        ignore::Error::WithPath { err, .. } | ignore::Error::WithDepth { err, .. } => {
            is_ignore_file_line_error(err)
        }
        ignore::Error::Partial(errs) => errs.iter().all(is_ignore_file_line_error),
        _ => false,
    }
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

/// A walker error as one error per path it names (§FS-check.2.4.2). The `ignore` crate folds
/// every malformed line of a directory's ignore files into one `Partial`, which names no path of
/// its own; each inner error names its file, so the lines are grouped by it, in the order they
/// came, and each group is one failure at that file. Any other error stands as it is.
fn split_by_path(err: ignore::Error) -> Vec<ignore::Error> {
    let errs = match err {
        ignore::Error::Partial(errs) => errs,
        ignore::Error::WithDepth { depth, err } => match *err {
            ignore::Error::Partial(errs) => errs,
            err => {
                return vec![ignore::Error::WithDepth {
                    depth,
                    err: Box::new(err),
                }];
            }
        },
        other => return vec![other],
    };
    let mut groups: Vec<(Option<PathBuf>, Vec<ignore::Error>)> = Vec::new();
    for err in errs {
        let path = walk_error_path(&err).map(Path::to_path_buf);
        match groups.iter_mut().find(|(at, _)| *at == path) {
            Some((_, group)) => group.push(err),
            None => groups.push((path, vec![err])),
        }
    }
    groups
        .into_iter()
        .map(|(_, mut group)| match group.len() {
            1 => group.remove(0),
            _ => ignore::Error::Partial(group),
        })
        .collect()
}

/// The in-tree path a walker error is about, at any nesting depth: for a `Partial`, the one path
/// every error it folds names, which `split_by_path` makes it have (§FS-check.2.4.2).
pub(super) fn walk_error_path(err: &ignore::Error) -> Option<&Path> {
    match err {
        ignore::Error::WithPath { path, .. } => Some(path),
        ignore::Error::WithDepth { err, .. } => walk_error_path(err),
        ignore::Error::Partial(errs) => {
            let first = walk_error_path(errs.first()?)?;
            errs.iter()
                .all(|err| walk_error_path(err) == Some(first))
                .then_some(first)
        }
        _ => None,
    }
}

/// A walker error's reason, without the path `ignore` writes into its own text:
/// the diagnostic already carries the path (§FS-errors.2.2.1), and the one `ignore`
/// writes is absolute, which a report may not contain (§FS-errors.4).
/// Its typed I/O reason is the same as a file read's (§FS-check.2.4.1).
fn walk_error_reason(err: &ignore::Error) -> String {
    match err.io_error() {
        Some(io) => io_reason(io),
        None => match err {
            ignore::Error::WithPath { err, .. } => err.to_string(),
            ignore::Error::WithDepth { err, .. } => walk_error_reason(err),
            // §FS-errors.4: each folded line without the absolute path `ignore` writes before it,
            // joined the way `ignore` joins them.
            ignore::Error::Partial(errs) => errs
                .iter()
                .map(walk_error_reason)
                .collect::<Vec<_>>()
                .join("\n"),
            other => other.to_string(),
        },
    }
}
