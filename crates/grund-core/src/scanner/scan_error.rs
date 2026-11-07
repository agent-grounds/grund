//! The published form of a scan failure (§AR-system.2.5): the `(path, message)`
//! pair the walk collects (§FS-check.2.4), with its path already spelled the way
//! the report spells it.
//!
//! It sat in the api's contract file while the queries and the writers read it
//! upward — the size catalog, the formatter's walk, its completeness proof and
//! its refusal all carry one (§AR-system.4, §FS-fmt.3, §FS-list.3.4). A scan
//! error belongs to the component that produces it, which is the walk, and both
//! readers sit above the scanner, so the record and the one line that renders it
//! live here beside the walk that raises it (§AR-scanner.1).

use std::io::Error;
use std::path::{Path, PathBuf};

use super::tree::ScanError;
use crate::config::Frame;
use crate::model::{FailureNames, remedies};
use crate::sources::{io_reason, paths_same_location};

/// One file the walk could not read, as a caller of the embedding API sees it:
/// the path rendered against the run's report base (§FS-config.3.6) and the
/// reason followed by `; ` and `io`'s *Fix* (§FS-check.2.4, §AR-bindings.2).
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ApiScanError {
    pub path: String,
    pub message: String,
}

/// Render one [`ScanError`](super::tree::ScanError) for a caller: the same path
/// spelling `check` prints, so an embedder and the terminal name one file the
/// same way (§FS-errors.4), and the report base `.` (§FS-cli.3.4).
pub(crate) fn api_scan_error(frame: Frame<'_>, path: &Path, message: &str) -> ApiScanError {
    ApiScanError {
        path: frame.display_location(path),
        message: scan_error_message(message),
    }
}

/// The text after `error: <path>: ` for a path the walk could not read: the reason,
/// then `io`'s *Fix*, built here once so every command that prints the line, and
/// `check`'s own report, print the same bytes (§FS-check.2.4, §FS-workspace.8.7.3).
pub(crate) fn scan_error_message(reason: &str) -> String {
    format!("{reason}; {}", remedies::io())
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
    reported: &mut Vec<ScanError>,
) {
    publish_unless_reported(spelled, error, context, reported, |path, _| {
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
    reported: &mut Vec<ScanError>,
) {
    let at = spelled.clone();
    publish_unless_reported(spelled, error, None, reported, |path, said| {
        names.of(path, said).any(|name| name == at)
    });
}

/// A failure to read, decode or parse one file, published at `spelled` unless the scan's reports
/// so far already hold it: the same file, under any of its spellings, and the same bare reason
/// (§FS-check.2.4.2, §FS-declarations.e2e-case.failure). Two passes raise it, for a case's file and
/// for an ignore file the walk may have met too. One file is one failure, under any spelling, so
/// the line already there stands, named as it was, the one listed first, unless only `spelled`
/// lies inside `run_path`, the path the run was handed: then that is the spelling it is named by,
/// so the run that names it keeps its exit.
pub(super) fn publish_file_failure(
    spelled: PathBuf,
    reason: String,
    run_path: Option<&Path>,
    reported: &mut Vec<ScanError>,
) {
    let already = reported.iter_mut().find(|(path, said)| {
        said.strip_prefix(UNREADABLE_LINK).unwrap_or(said) == reason
            && (*path == spelled || paths_same_location(path, &spelled))
    });
    match already {
        Some((path, _)) => {
            if run_path.is_some_and(|run| spelled.starts_with(run) && !path.starts_with(run)) {
                *path = spelled;
            }
        }
        None => reported.push((spelled, reason)),
    }
}

fn publish_unless_reported(
    spelled: PathBuf,
    error: &Error,
    context: Option<&str>,
    reported: &mut Vec<ScanError>,
    same_location: impl Fn(&Path, &str) -> bool,
) {
    let reason = io_reason(error);
    let already = reported.iter().any(|(path, said)| {
        said.strip_prefix(UNREADABLE_LINK).unwrap_or(said) == reason && same_location(path, said)
    });
    if !already {
        let message = match context {
            Some(context) => format!("{context}: {reason}"),
            None => reason,
        };
        reported.push((spelled, message));
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
