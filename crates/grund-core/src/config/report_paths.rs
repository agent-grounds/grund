//! How a report spells a path, which is a question about one `grund.toml` key
//! (§AR-system.2.3): `[output] relative_paths` picks the base every reported
//! path is rendered against (§FS-config.3.6), unless the run's `--path-base`
//! outranks it (§FS-cli.3.4).
//!
//! It sat in the deprecated path's `output` category while workspace, the
//! scanner, the checker, the queries, the writers and the api all read it upward
//! (§AR-system.4, §AR-system.2.9.1). The spelling itself — forward slashes, the
//! sort key, the `..` walk out of a base — is a function of a `Path` alone and
//! went one level further down, into `model/paths.rs`; what is left here is the
//! part that needs a `Config` to answer, so it belongs to the component that
//! owns the key.

use std::path::Path;

use super::call_scope::PathBase;
use super::record::Config;
use crate::model::{Diagnostic, Finding, format_path, relative_from_base};

impl Config {
    /// Whether this run spells report paths from the config root: the run's
    /// `--path-base` when one was passed, else `[output] relative_paths`
    /// (§FS-cli.3.4 — flag, then key, then the `project` default).
    pub(crate) fn reports_from_root(&self) -> bool {
        match self.path_base {
            Some(PathBase::Project) => true,
            Some(PathBase::Invocation) => false,
            None => self.relative_paths,
        }
    }

    /// The directory every reported path is spelled from (§FS-cli.3.4).
    pub(crate) fn report_base(&self) -> &Path {
        if self.reports_from_root() {
            &self.root
        } else {
            &self.cli_base
        }
    }
}

/// Render a path the way reports show it: relative to the repo root by default,
/// or relative to the CLI base directory under `--path-base=invocation` or
/// `[output] relative_paths = false` (§FS-config.3.6.1 — an in-root target
/// outside that base uses bounded `..`).
pub(crate) fn display_path(config: &Config, path: &Path) -> String {
    let base = config.report_base();
    let relative = path
        .strip_prefix(base)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| {
            if !config.reports_from_root() && path.starts_with(&config.root) {
                relative_from_base(base, path)
            } else {
                path.to_path_buf()
            }
        });
    format_path(&relative)
}

/// The run's `[workspace]` warnings in the published shape (§FS-distribution.3.1):
/// one `Finding` each, with the anchor the engine gave it spelled the way this
/// run spells every other reported path (§FS-errors.4).
///
/// Here rather than beside the report conversion because a `Diagnostic` with no
/// `sites` is nothing but a message and a `<config>:<line>` anchor, and every
/// component that settles one — the walking commands' api, the `init` and `fetch`
/// writers — needs one spelling of it (§FS-check.4.10,
/// §FS-workspace.6.1.7).
pub(crate) fn run_warning_findings(config: &Config, warnings: Vec<Diagnostic>) -> Vec<Finding> {
    warnings
        .into_iter()
        .map(|warning| Finding {
            severity: "warning",
            code: warning.code,
            path: warning.path.map(|path| display_path(config, &path)),
            line: warning.line,
            column: None,
            message: warning.message,
            sites: Vec::new(),
            authority: Vec::new(),
        })
        .collect()
}
