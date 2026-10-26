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

use std::path::Path;

use crate::config::Frame;
use crate::model::remedies;

/// One file the walk could not read, as a caller of the embedding API sees it:
/// the path rendered against the run's report base (§FS-config.3.6) and the
/// reason verbatim (§FS-check.2.4, §AR-bindings.2).
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ApiScanError {
    pub path: String,
    pub message: String,
}

/// Render one [`ScanError`](super::tree::ScanError) for a caller: the same path
/// spelling `check` prints, so an embedder and the terminal name one file the
/// same way (§FS-errors.4).
pub(crate) fn api_scan_error(frame: Frame<'_>, path: &Path, message: &str) -> ApiScanError {
    ApiScanError {
        path: frame.display_path(path),
        message: scan_error_message(message),
    }
}

/// The text after `error: <path>: ` for a path the walk could not read: the reason,
/// then `io`'s *Fix*, built here once so every command that prints the line, and
/// `check`'s own report, print the same bytes (§FS-check.2.4, §FS-workspace.8.7.3).
pub(crate) fn scan_error_message(reason: &str) -> String {
    format!("{reason}; {}", remedies::io())
}
