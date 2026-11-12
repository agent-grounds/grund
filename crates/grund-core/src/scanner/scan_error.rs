//! A scan failure and its published form (§AR-system.2.5, §AR-scanner.1.failure): the record the
//! walk collects (§FS-check.2.4), and the rendering whose path is spelled the way the report
//! spells it.
//!
//! It sat in the api's contract file while the queries and the writers read it
//! upward — the size catalog, the formatter's walk, its completeness proof and
//! its refusal all carry one (§AR-system.4, §FS-fmt.3, §FS-list.3.4). A scan
//! error belongs to the component that produces it, which is the walk, and both
//! readers sit above the scanner, so the record and the one line that renders it
//! live here beside the walk that raises it (§AR-scanner.1).

use std::io::Error;
use std::path::{Path, PathBuf};

use super::walk::scan_roots;
use super::walk_errors::{BROKEN_LINK, link_identity, unresolved_link_reason};
use crate::config::{Frame, Schema};
use crate::model::{Code, FailureNames, ValueFormat, remedies};
use crate::sources::{
    io_reason, path_is_symlink, path_metadata, paths_same_location, physical_path_key,
};

/// Where a path that could not be read is, which decides what can keep it out of a run
/// (§FS-check.2.4.3): each is a case of `io`'s *Fix* (§FS-errors.5.5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Place {
    Passed,
    Readable,
    Value,
    Root,
    Entry,
}

/// The kind of a file that was read and does not parse, each a case of `unparsable-file`'s *Fix*
/// (§FS-check.2.4.3).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FileKind {
    Exit,
    Value(ValueFormat),
    Ignore,
}

/// What failed, said where the failure is raised and never read back out of a message
/// (§AR-scanner.1.failure, §FS-check.2.4.1). `version` is the config format the fix is worded for
/// (§FS-errors.5.5.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScanFailure {
    Unread { place: Place, version: u32 },
    Unparsed(FileKind),
}

/// A path the scan could not read, or a file it read and could not parse. The walk continues past
/// it (§FS-check.2.4); callers that are point queries treat any entry as fatal, `check` and `refs`
/// report it and exit 2 with a still-printed report (§AR-scanner.1.failure).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScanError {
    pub(crate) path: PathBuf,
    pub(crate) reason: String,
    failure: ScanFailure,
}

impl ScanError {
    pub(crate) fn new(path: PathBuf, reason: String, failure: ScanFailure) -> Self {
        Self {
            path,
            reason,
            failure,
        }
    }

    /// An ordinary unread path, the record most tests of the list's own handling need.
    #[cfg(test)]
    pub(crate) fn unread_entry(path: PathBuf, reason: &str) -> Self {
        let failure = ScanFailure::Unread {
            place: Place::Entry,
            version: 2,
        };
        Self::new(path, reason.to_string(), failure)
    }

    /// Whether the walk reported a link here whose target does not exist (§FS-check.2.4.1): the
    /// record `fetch` decides a dangling link at its target by (§FS-fetch.links).
    pub(crate) fn is_broken_link(&self) -> bool {
        self.reason == BROKEN_LINK
    }

    /// The code this failure is published under (§FS-check.2.4.3).
    pub(crate) fn code(&self) -> Code {
        match self.failure {
            ScanFailure::Unread { .. } => Code::IO,
            ScanFailure::Unparsed(_) => Code::UNPARSABLE_FILE,
        }
    }

    /// Whether this failure is at the path the run was handed (§FS-check.2.4.3), which a workspace
    /// run walks no project from (§AR-scanner.1.6).
    pub(crate) fn at_passed_path(&self) -> bool {
        matches!(
            self.failure,
            ScanFailure::Unread {
                place: Place::Passed,
                ..
            }
        )
    }

    /// The *Fix* case that works for this failure at its path (§FS-check.2.4.3, §FS-errors.5.5).
    fn fix(&self) -> String {
        match self.failure {
            ScanFailure::Unread { place, version } => match place {
                Place::Passed => remedies::io_passed(version),
                Place::Readable => remedies::io_readable(version),
                Place::Value => remedies::io_value(version),
                Place::Root => remedies::io_root(version),
                Place::Entry => remedies::io_entry(version),
            },
            ScanFailure::Unparsed(FileKind::Exit) => remedies::unparsable_file_exit(),
            ScanFailure::Unparsed(FileKind::Value(format)) => {
                remedies::unparsable_file_value(format.name())
            }
            ScanFailure::Unparsed(FileKind::Ignore) => remedies::unparsable_file_ignore(),
        }
    }

    /// This failure judged at its path: a parse failure's kind does not depend on where it is.
    pub(super) fn placed(mut self, at: &FailureRoots) -> Self {
        if let ScanFailure::Unread { place, .. } = &mut self.failure {
            *place = failure_place(&self.path, at);
        }
        self
    }
}

/// What a failed path is compared against (§FS-check.2.4.3), built only once a pass has a failure
/// to judge, so a clean tree pays nothing (§GOAL-fast-feedback).
pub(super) struct FailureRoots {
    roots: Vec<PathBuf>,
    passed: Option<PathBuf>,
    config_root: PathBuf,
}

impl FailureRoots {
    #[cfg(all(test, unix))]
    pub(super) fn of(roots: Vec<PathBuf>, passed: Option<PathBuf>, config_root: PathBuf) -> Self {
        Self {
            roots,
            passed,
            config_root,
        }
    }

    /// What `scan_roots` answered, whose first entry is the path a narrowed run was handed
    /// (§FS-check.1.3.6.1). `whole` says the scope is the config root, which is no narrowing.
    pub(super) fn new(
        roots: Vec<PathBuf>,
        whole: bool,
        explicit_scope: bool,
        config_root: &Path,
    ) -> Self {
        let passed = (explicit_scope && !whole)
            .then(|| roots.first().cloned())
            .flatten();
        Self {
            roots,
            passed,
            config_root: config_root.to_path_buf(),
        }
    }

    /// The same answer for a pass that does not hold the walk's roots. A scope that no longer
    /// resolves judges every path an `entry`.
    pub(super) fn resolve(
        schema: &Schema,
        frame: Frame<'_>,
        scope: Option<&Path>,
        explicit_scope: bool,
    ) -> Self {
        let (roots, whole) = scan_roots(schema, frame, scope, explicit_scope).unwrap_or_default();
        Self::new(roots, whole, explicit_scope, frame.root())
    }

    /// The same answer for a path a pass reads whether or not the run was given one, which no
    /// longer passing would not stop reading, so it is never `passed` (§FS-check.2.4.3). `new`
    /// keeps the handed path first in the roots, so what is left are the ordinary ones.
    pub(super) fn unpassed(mut self) -> Self {
        if self.passed.take().is_some() {
            self.roots.remove(0);
        }
        self
    }
}

/// Where `path` is among the run's roots, by directory entry so a second spelling of its parents
/// answers the same and a link is the path it is, the first case of §FS-check.2.4.3 that holds. The
/// value and section-lock passes know their answer without asking (§AR-scanner.1.failure).
pub(super) fn failure_place(path: &Path, at: &FailureRoots) -> Place {
    let same = |root: &Path| same_entry(path, root);
    if at.passed.as_deref().is_some_and(same) {
        Place::Passed
    } else if same(&at.config_root) {
        Place::Readable
    } else if at.roots.iter().any(|root| same(root)) {
        Place::Root
    } else {
        Place::Entry
    }
}

/// Whether `path` is the entry `root` names (§AR-scanner.1.entry). One spelled below the root is
/// below it, though a loop may have walked it back to the root's own entry. The final component is
/// never resolved, so names that differ cannot be one entry and cost no `realpath`
/// (§AR-scanner.1.failure).
fn same_entry(path: &Path, root: &Path) -> bool {
    if path == root {
        return true;
    }
    if path.starts_with(root) {
        return false;
    }
    if let (Some(left), Some(right)) = (path.file_name(), root.file_name())
        && left != right
    {
        return false;
    }
    entry_key(path) == entry_key(root)
}

/// The key `link_identity` gives a path. One with no final name, `.` or `..`, is resolved whole,
/// and one whose parent is gone compares as written (§AR-scanner.1.entry).
fn entry_key(path: &Path) -> PathBuf {
    match path.file_name() {
        Some(_) => link_identity(path).unwrap_or_else(|| path.to_path_buf()),
        None => physical_path_key(path),
    }
}

/// One file the walk could not read, as a caller of the embedding API sees it:
/// the path rendered against the run's report base (§FS-config.3.6), the reason
/// followed by `; ` and the failure's *Fix* (§FS-check.2.4.3), and the code that
/// tells what failed without parsing the message (§AR-bindings.2).
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ApiScanError {
    pub path: String,
    pub message: String,
    pub code: String,
}

/// Render one [`ScanError`](super::tree::ScanError) for a caller: the same path
/// spelling `check` prints, so an embedder and the terminal name one file the
/// same way (§FS-errors.4), and the report base `.` (§FS-cli.3.4).
pub(crate) fn api_scan_error(frame: Frame<'_>, failure: &ScanError) -> ApiScanError {
    ApiScanError {
        path: frame.display_location(&failure.path),
        message: scan_error_message(failure),
        code: failure.code().as_str().to_string(),
    }
}

/// The text after `error: <path>: ` for a scan failure: the reason, then the *Fix* of its failure
/// and place, built here once so every command that prints the line, and `check`'s own report,
/// print the same bytes (§FS-check.2.4.3, §FS-workspace.8.7.3).
pub(crate) fn scan_error_message(failure: &ScanError) -> String {
    format!("{}; {}", failure.reason, failure.fix())
}

/// The reason a pass that reads `path` itself gives for `error`: a link that does not resolve
/// carries the reason the walk gives the same link (§FS-declarations.e2e-case.failure), and every
/// other failure, a resolved link's target that will not open among them, the reason its I/O kind
/// derives (§FS-check.2.4.1).
pub(super) fn read_failure_reason(path: &Path, error: &Error) -> String {
    match path_metadata(path) {
        Err(unresolved) if path_is_symlink(path) => unresolved_link_reason(&unresolved),
        _ => io_reason(error),
    }
}

/// The words the walk keeps around the reason of a link it cannot resolve for lack of access to
/// its target (§AR-scanner.1.9, §FS-check.2.4.1).
const UNREADABLE_LINK: &str = "unreadable symlink: ";

/// The walk's report of a link whose target fails to open with `reason`.
pub(super) fn unreadable_link_reason(reason: &str) -> String {
    format!("{UNREADABLE_LINK}{reason}")
}

/// A pass's failure to list `listed`, a directory the walk lists too, published at `spelled`
/// unless the scan's reports so far already hold the same failure (§FS-check.2.4.2,
/// §AR-scanner.1.relisted): the same physical location and the bare reason derived from the
/// error's kind, as the walk derives its own. `context`, the pass's words around that reason,
/// is published with it and never compared (§FS-check.2.4.1), and neither are the walk's around
/// a link: a directory the pass lists through a link is the link the walk reported. Only this
/// one failure is compared: nothing here deduplicates the list.
pub(super) fn publish_unlisted(
    listed: &Path,
    spelled: PathBuf,
    error: &Error,
    context: Option<&str>,
    failure: ScanFailure,
    reported: &mut Vec<ScanError>,
) {
    publish_unless_reported(spelled, error, context, failure, reported, |path, _| {
        paths_same_location(path, listed)
    });
}

/// A case's failure to list a fixture directory, published at `spelled` unless a report already
/// says it there: that path as the tree spells it, or a name the report kept for it
/// (§AR-scanner.1.entry), and the same bare reason. The walk spells below the canonical root as the
/// pass does, so a plain tree meets here. A report at another spelling it did not keep answers no
/// run holding this one, and a kept one does, because `holds_failure` asks the names
/// (§AR-scanner.1.relisted). The walk's words around a fixture directory it reports as a link are
/// not compared either, and a kept name is found under those words, so asked with them.
pub(super) fn publish_unlisted_fixture(
    spelled: PathBuf,
    error: &Error,
    names: &FailureNames,
    failure: ScanFailure,
    reported: &mut Vec<ScanError>,
) {
    let at = spelled.clone();
    publish_unless_reported(spelled, error, None, failure, reported, |path, said| {
        names.of(path, said).any(|name| name == at)
    });
}

/// Select the whole-run name of a file failure from the run's ordered file names
/// (§FS-check.2.4.2). The ordinary file pass, case files and ignore files share this selection.
/// Every alias this pass introduces is retained for membership; CLI reporting alone selects an
/// anchor name. A file absent from the table keeps the spelling the pass encountered.
pub(super) fn name_file_failure(
    mut report: ScanError,
    frame: Frame<'_>,
    names: &mut FailureNames,
) -> ScanError {
    let spelled = report.path.clone();
    let file_names = frame.run.spellings.names_of(&spelled);
    if let Some(first) = file_names.first() {
        report.path = first.clone();
    }
    for name in std::iter::once(spelled).chain(file_names) {
        names.keep(&report.path, &report.reason, &name);
    }
    report
}

/// Publish a case or ignore-file failure through the same name selection as the file pass
/// (§FS-check.2.4.2). An earlier report wins; retain this pass's names under that survivor.
pub(super) fn publish_file_failure(
    report: ScanError,
    frame: Frame<'_>,
    names: &mut FailureNames,
    reported: &mut Vec<ScanError>,
) {
    let report = name_file_failure(report, frame, names);
    let already = reported.iter_mut().find(|held| {
        held.reason
            .strip_prefix(UNREADABLE_LINK)
            .unwrap_or(&held.reason)
            == report.reason
            && (held.path == report.path || paths_same_location(&held.path, &report.path))
    });
    if let Some(held) = already {
        let introduced: Vec<_> = names
            .of(&report.path, &report.reason)
            .map(Path::to_path_buf)
            .collect();
        for name in introduced {
            names.keep(&held.path, &held.reason, &name);
        }
    } else {
        reported.push(report);
    }
}

/// A value source the value pass could not read, published in place of the walk's report of the
/// same directory entry for the same bare reason (§FS-check.2.4.2, §AR-scanner.1.relisted): a link
/// in a value home whose target lookup is refused is reported by the walk too, whatever its name,
/// and the value pass's line, with its value-home remedy, is
/// the one that stands (§FS-values.5.3.1), since `exclude` does not reach a value source.
pub(super) fn publish_value_source_failure(
    report: ScanError,
    error: &Error,
    reported: &mut Vec<ScanError>,
) {
    let reason = io_reason(error);
    let entry = link_identity(&report.path);
    let walk_line = reported.iter_mut().find(|held| {
        held.reason
            .strip_prefix(UNREADABLE_LINK)
            .unwrap_or(&held.reason)
            == reason
            && (held.path == report.path || entry.is_some() && link_identity(&held.path) == entry)
    });
    match walk_line {
        Some(held) => *held = report,
        None => reported.push(report),
    }
}

fn publish_unless_reported(
    spelled: PathBuf,
    error: &Error,
    context: Option<&str>,
    failure: ScanFailure,
    reported: &mut Vec<ScanError>,
    same_location: impl Fn(&Path, &str) -> bool,
) {
    let reason = io_reason(error);
    let already = reported.iter().any(|held| {
        held.reason
            .strip_prefix(UNREADABLE_LINK)
            .unwrap_or(&held.reason)
            == reason
            && same_location(&held.path, &held.reason)
    });
    if !already {
        let message = match context {
            Some(context) => format!("{context}: {reason}"),
            None => reason,
        };
        reported.push(ScanError::new(spelled, message, failure));
    }
}

/// Keep a scan failure's context and non-I/O descriptions while deriving its
/// typed I/O causes' public reasons in one place (§FS-check.2.4.1). This builds
/// only the published text; it never replaces the original error chain.
pub(crate) fn scan_error_reason(error: &anyhow::Error) -> String {
    error
        .chain()
        .map(|cause| match cause.downcast_ref::<Error>() {
            Some(io) => io_reason(io),
            None => cause.to_string(),
        })
        .collect::<Vec<_>>()
        .join(": ")
}
