//! The install fetch ends in (§FS-fetch.4, §FS-fetch.5): admission's bytes through the shared
//! writer in `atomic_install.rs`, against what the scan read before the integration ran
//! (§FS-fetch.6.1, §REQ-no-data-loss.2).

use std::path::Path;

use super::atomic_install::{Holds, Install, InstallFailure, atomic_install};
use super::fetch::{FetchFailure, fetch_io, fetch_operational};

/// §FS-fetch.6.1: install complete bytes, each of the writer's failures as fetch's refusal; a
/// target that no longer holds what the scan read is named from the report base.
pub(super) fn install(
    path: &Path,
    shown: &str,
    bytes: &[u8],
    expected: Holds<'_>,
    diagnostic: &mut Option<anyhow::Error>,
) -> Result<(), FetchFailure> {
    let install = Install {
        bytes,
        tag: "fetch",
        expected,
    };
    atomic_install(path, install).map_err(|failure| match failure {
        InstallFailure::Io { error, context } => fetch_io(diagnostic, path, error, context),
        InstallFailure::Operational(message) => fetch_operational(message),
        InstallFailure::Changed => fetch_operational(format!(
            "{shown} changed while the integration ran; fetch again"
        )),
    })
}
