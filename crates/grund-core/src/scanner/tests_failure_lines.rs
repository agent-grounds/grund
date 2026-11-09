//! Test support: a scan failure's lines and signature as tests compare them (§FS-check.2.4,
//! §FS-check.2.4.3).

#[cfg(unix)]
use crate::api::CheckRun;
#[cfg(unix)]
use crate::config::display_path;
use crate::model::sort_path_key;
use crate::scanner::ScanError;

pub(crate) fn scan_errors_signature(errors: Vec<ScanError>) -> Vec<String> {
    errors
        .into_iter()
        .map(|error| format!("{}|{}", sort_path_key(&error.path), error.reason))
        .collect()
}

/// The `error: <path>: <reason>` lines a file the scan could not read earns
/// (§FS-check.2.4, §FS-errors.2.2.1).
/// Unix only: every caller is a symlink case and so `#[cfg(unix)]` too.
#[cfg(unix)]
pub(crate) fn scan_errors(run: &CheckRun) -> Vec<String> {
    run.report
        .errors
        .iter()
        .filter(|diagnostic| matches!(diagnostic.code.as_str(), "io" | "unparsable-file"))
        .map(|diagnostic| {
            format!(
                "{}: {}",
                diagnostic
                    .path
                    .as_ref()
                    .map(|path| display_path(&run.config, path))
                    .unwrap_or_default(),
                diagnostic.message
            )
        })
        .collect()
}
