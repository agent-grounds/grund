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

use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use super::tree::ScanError;
use crate::config::Frame;
use crate::model::remedies;
use crate::sources::paths_same_location;

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

/// One scan I/O kind, one reason without an OS number (§FS-check.2.4.1).
/// Decode and input-observation failures use I/O errors to carry non-I/O
/// descriptions; keep those details. Borrow the error so its original source
/// and OS number remain available to callers that publish raw causes.
pub(super) fn scan_io_reason(error: &Error) -> String {
    match error.kind() {
        ErrorKind::InvalidData | ErrorKind::Other if error.raw_os_error().is_none() => {
            error.to_string()
        }
        kind => Error::from(kind).to_string(),
    }
}

/// A pass's failure to list `listed`, a directory the walk lists too, published at `spelled`
/// unless the scan's reports so far already hold the same failure (§FS-check.2.4.2,
/// §AR-scanner.1.relisted): the same physical location and the bare reason derived from the
/// error's kind, as the walk derives its own. `context`, the pass's words around that reason,
/// is published with it and never compared (§FS-check.2.4.1). Only this one failure is
/// compared: nothing here deduplicates the list.
pub(super) fn publish_unlisted(
    listed: &Path,
    spelled: PathBuf,
    error: &Error,
    context: Option<&str>,
    reported: &mut Vec<ScanError>,
) {
    publish_unless_reported(spelled, error, context, reported, |path| {
        paths_same_location(path, listed)
    });
}

/// A case's failure to read a file or list a fixture directory, published at `spelled` unless a
/// report already says it there: the same path as the tree spells it and the same bare reason. The
/// walk spells below the canonical root as the pass does, so a plain tree meets here, and a report
/// at another spelling of the file answers no run holding this one (§AR-scanner.1.relisted).
pub(super) fn publish_case_failure(spelled: PathBuf, error: &Error, reported: &mut Vec<ScanError>) {
    let at = spelled.clone();
    publish_unless_reported(spelled, error, None, reported, |path| path == at);
}

fn publish_unless_reported(
    spelled: PathBuf,
    error: &Error,
    context: Option<&str>,
    reported: &mut Vec<ScanError>,
    same_location: impl Fn(&Path) -> bool,
) {
    let reason = scan_io_reason(error);
    let already = reported
        .iter()
        .any(|(path, said)| *said == reason && same_location(path));
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
            Some(io) => scan_io_reason(io),
            None => cause.to_string(),
        })
        .collect::<Vec<_>>()
        .join(": ")
}
