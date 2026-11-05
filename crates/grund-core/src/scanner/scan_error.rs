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
use std::path::Path;

use crate::config::Frame;
use crate::model::remedies;

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
