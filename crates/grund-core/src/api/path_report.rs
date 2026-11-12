//! The narrowing a `check <path>` report goes through once every rule has run over
//! the resolution scope (§FS-check.1.3.6.1, §AR-resolver.3.3), shared by the
//! single-project arm and the workspace arm so the two cannot disagree about what a
//! path keeps.

use std::path::Path;

use crate::scanner::ScanError;

use super::scope_cautions::full_scope_ignored_warning;
use crate::checker::{ScanScope, retain_diagnostics_in_report_scope};
use crate::config::{Config, Display};
use crate::model::{CheckReport, Diagnostic};

/// What the caller typed that the report answers to: the path, whether it was typed
/// at all, `--full`, and the report scope the path names (`None` over the root).
pub(super) struct PathReport<'a> {
    pub(super) path: &'a Path,
    pub(super) path_provided: bool,
    pub(super) full: bool,
    pub(super) scope: Option<&'a ScanScope>,
}

impl PathReport<'_> {
    /// Narrow `report` to the path and add the cautions the path itself earns.
    ///
    /// In this order, so caution insertion preserves the silence decision:
    /// the three channels are narrowed; the scope caution is asked whether the
    /// *narrowed* report is silent (§FS-check.2.2), through `scope_caution`, because
    /// each arm knows what its walk read; the files the resolution scope could not
    /// read outside the path become cautions (§FS-check.1.3.6.3), after the silence
    /// question so they cannot suppress it (§FS-check.2.2.3.1); `config_warnings`
    /// follow unnarrowed (§FS-check.4.3); and a `--full` the path made redundant says
    /// so last (§FS-check.1.3.7).
    /// The caller then completes channel ordering (§FS-errors.4.1).
    pub(super) fn narrow(
        &self,
        report: &mut CheckReport,
        config: &Config,
        unread_outside: Vec<ScanError>,
        scope_caution: impl FnOnce(bool) -> Option<Diagnostic>,
        config_warnings: impl IntoIterator<Item = Diagnostic>,
    ) {
        let frame = config.frame();
        retain_diagnostics_in_report_scope(&mut report.errors, frame, self.scope);
        retain_diagnostics_in_report_scope(&mut report.warnings, frame, self.scope);
        retain_diagnostics_in_report_scope(&mut report.suggestions, frame, self.scope);
        let report_is_silent = report.errors.is_empty() && report.warnings.is_empty();
        report.warnings.extend(scope_caution(report_is_silent));
        report.warnings.extend(unread_resolution_source_cautions(
            unread_outside,
            config.display(),
        ));
        report.warnings.extend(config_warnings);
        report.warnings.extend(full_scope_ignored_warning(
            config,
            self.path,
            self.path_provided,
            self.full,
        ));
    }
}

/// §FS-check.1.3.6.3: one caution per file the resolution scope could not read and
/// the report scope does not own. It carries the failure's own reason and the code
/// the §FS-check.2.4 error carries (§FS-check.2.4.3), because it is the same knowledge
/// on a softer channel rather than a new finding. `line` is `None`, which is what puts a CLI-level message on stderr
/// (§FS-check.2.1.1), and the path stays the file's so the reader can go and chmod
/// it. Silence is the thing this exists to prevent: the citation whose declaration
/// lay in that file is reported as unresolved, and §REQ-no-wrong-citation.2 asks
/// that a false alarm stay legible as one. The code's one cost is that
/// `--ignore` cannot turn the line off, which §FS-check.1.3.6.3 states rather than
/// leaves to be found.
pub(super) fn unread_resolution_source_cautions(
    unread: impl IntoIterator<Item = ScanError>,
    display: Display<'_>,
) -> Vec<Diagnostic> {
    unread
        .into_iter()
        .map(|failure| Diagnostic {
            code: failure.code(),
            message: format!(
                "{} — outside the report scope, so a citation declared there \
                 may be reported as unresolved",
                // §FS-errors.2.2.5: a file the reason names, spelled as the report spells.
                failure.spelled_reason(|path| display.path(path))
            ),
            path: Some(failure.path),
            line: None,
            column: None,
            sites: Vec::new(),
            authority: Vec::new(),
        })
        .collect()
}
