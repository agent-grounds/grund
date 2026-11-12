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
use super::walk_errors::{BROKEN_LINK, link_identity, loop_reason, unresolved_link_reason};
use crate::config::{Display, Frame, Schema, report_path};
use crate::model::{Code, Diagnostic, FailureNames, OperationDiagnostic, ValueFormat, remedies};
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
    /// The file whose parse failed, which the message names ahead of the reason. A path, not
    /// text, so it is spelled when the report is, by the spelling of the location
    /// (§FS-errors.2.2.5, §AR-scanner.1.failure).
    parsed: Option<PathBuf>,
    /// The directory a symlink loop's link reaches back into, which the message names after
    /// the reason: a path, spelled when the report is, and left unnamed where it is the base the
    /// report spells from (§FS-check.2.4.2, §AR-scanner.1.failure). One link reaches one
    /// directory, so the bare reason beside it is what failures are compared by.
    ancestor: Option<PathBuf>,
    failure: ScanFailure,
}

/// How a report spells the paths a scan failure names: the one spelling of a reported path, and
/// whether a path is the base that spelling starts from, asked of the path and never of its
/// spelling (§AR-config.1.7, §FS-cli.3.4).
pub(crate) trait ReportSpelling {
    fn path(&self, path: &Path) -> String;
    fn is_base(&self, path: &Path) -> bool;
}

impl ReportSpelling for Display<'_> {
    fn path(&self, path: &Path) -> String {
        Display::path(self, path)
    }

    fn is_base(&self, path: &Path) -> bool {
        Display::is_base(self, path)
    }
}

/// The spelling of a reader that holds only the report base (§AR-config.1.7).
pub(crate) struct FromBase<'a>(pub(crate) &'a Path);

impl ReportSpelling for FromBase<'_> {
    fn path(&self, path: &Path) -> String {
        report_path(self.0, path)
    }

    fn is_base(&self, path: &Path) -> bool {
        path == self.0
    }
}

/// Every path absolute, as `grund init` still locates a scan failure: no path is a base it is
/// spelled from.
pub(crate) struct Absolute;

impl ReportSpelling for Absolute {
    fn path(&self, path: &Path) -> String {
        path.display().to_string()
    }

    fn is_base(&self, _: &Path) -> bool {
        false
    }
}

impl ScanError {
    pub(crate) fn new(path: PathBuf, reason: String, failure: ScanFailure) -> Self {
        Self {
            path,
            reason,
            parsed: None,
            ancestor: None,
            failure,
        }
    }

    /// This failure as the parse of `file` that failed for its reason, the file named when the
    /// report is spelled rather than while scanning (§FS-errors.2.2.5).
    pub(super) fn parsing(self, file: PathBuf) -> Self {
        Self {
            parsed: Some(file),
            ..self
        }
    }

    /// This failure as a symlink loop whose link reaches back into `directory`, the directory
    /// named when the report is spelled rather than while scanning (§FS-check.2.4.2).
    pub(super) fn looping_into(self, directory: PathBuf) -> Self {
        Self {
            ancestor: Some(directory),
            ..self
        }
    }

    /// The reason with the file it parsed, or the directory a loop reaches back into, named by
    /// `spell`, the report's spelling of a path (§FS-errors.2.2.5, §FS-check.2.4.2): what a reader
    /// that prints the reason without its *Fix* prints.
    pub(crate) fn spelled_reason(&self, spell: &impl ReportSpelling) -> String {
        match (&self.parsed, &self.ancestor) {
            (Some(file), _) => format!("parse {}: {}", spell.path(file), self.reason),
            (None, Some(directory)) => loop_reason(
                &self.reason,
                (!spell.is_base(directory)).then(|| spell.path(directory)),
            ),
            (None, None) => self.reason.clone(),
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

impl ApiScanError {
    /// The `error: ` line's text, `<location>: <message>`, which every command that prints the
    /// failure prints (§FS-errors.2.2.5, §FS-workspace.8.7.3).
    #[doc(hidden)]
    pub fn line(&self) -> String {
        format!("{}: {}", self.path, self.message)
    }

    /// The record an operation failed on, when `error` is the refusal [`scan_operation_failure`]
    /// built: a command prints its [`Self::line`], while a binding reads the refusal's own fields
    /// (§FS-errors.2.2.5).
    #[doc(hidden)]
    pub fn failed_on(error: &anyhow::Error) -> Option<&Self> {
        let refusal = error.downcast_ref::<OperationDiagnostic>()?;
        refusal.origin::<RefusedOn>().map(|origin| &origin.0)
    }
}

/// The record a scan failure's operation refusal copies its fields from, carried on it whole
/// (§FS-errors.2.2.5).
#[derive(Debug)]
struct RefusedOn(ApiScanError);

impl std::fmt::Display for RefusedOn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.line())
    }
}

/// The whole reported form of one failure for a run, built here once and read by every
/// surface that publishes it (§FS-errors.2.2.5, §AR-scanner.1.failure): its location and the
/// file it parsed spelled by `spell`, the run's one spelling of a reported path
/// (§AR-config.1.7), so an embedder and the terminal name one file the same way
/// (§FS-errors.4); its message; and its code.
pub(crate) fn reported_scan_failure(
    failure: &ScanError,
    spell: &impl ReportSpelling,
) -> ApiScanError {
    ApiScanError {
        path: spell.path(&failure.path),
        message: scan_error_message(failure, spell),
        code: failure.code().as_str().to_string(),
    }
}

/// [`reported_scan_failure`] as `frame`'s report spells it, the report base `.`
/// (§FS-cli.3.4).
pub(crate) fn api_scan_error(frame: Frame<'_>, failure: &ScanError) -> ApiScanError {
    reported_scan_failure(failure, &frame.display)
}

/// The failure an operation fails with, as `frame`'s report spells it: located at the record's
/// `path`, so its message is the record's and never repeats the location at its head
/// (§FS-errors.2.2.5, §FS-distribution.3.3.2). It carries the record, which a command prints
/// as [`ApiScanError::line`].
pub(crate) fn scan_operation_failure(frame: Frame<'_>, failure: &ScanError) -> OperationDiagnostic {
    let record = api_scan_error(frame, failure);
    let mut diagnostic = OperationDiagnostic::new(
        "filesystem",
        failure.code().as_str(),
        record.message.clone(),
    );
    diagnostic.path = Some(record.path.clone());
    diagnostic.with_origin(RefusedOn(record))
}

/// The failure as one of `check`'s findings, its location left a path for the report to spell
/// and narrow and its message spelled by `display`, the report's (§FS-errors.2.2.5,
/// §FS-check.2.4).
pub(crate) fn scan_failure_diagnostic(failure: ScanError, display: Display<'_>) -> Diagnostic {
    Diagnostic {
        code: failure.code(),
        message: scan_error_message(&failure, &display),
        path: Some(failure.path),
        line: None,
        column: None,
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// The text after `error: <path>: ` for a scan failure: the reason, the file it parsed spelled
/// by `spell`, then the *Fix* of its failure and place, built here once so every command that
/// prints the line, and `check`'s own report, print the same bytes (§FS-check.2.4.3,
/// §FS-workspace.8.7.3, §FS-errors.2.2.5).
pub(crate) fn scan_error_message(failure: &ScanError, spell: &impl ReportSpelling) -> String {
    format!("{}; {}", failure.spelled_reason(spell), failure.fix())
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
