//! The conversion from the checker's report to the published one
//! (§AR-system.2.9): every `Diagnostic` becomes a `Finding` with its paths
//! spelled, in the two path modes the two consumers need (§FS-errors.2.1,
//! §FS-lsp.1.1).
//!
//! This is the adapter §AR-core-module-layout.2 keeps out of the contract: the
//! records are `model/report.rs`'s and the signatures an embedder calls are
//! `check.rs`'s, and what is here is the private machinery between them. The
//! query-refusal `sites` renderer sits beside it for the same reason — it is one
//! `pub` spelling of a `FindingSite`, shared by the `grund` CLI and the
//! deprecated mirror so the two printers cannot drift (§FS-errors.5).

use std::path::Path;

use super::lsp_ranges::absolutize_path;
use crate::config::{Config, display_path, run_warning_findings};
use crate::model::{CheckReport, Diagnostic, Finding, FindingSite, Report, json_escape};
use crate::resolver::WorkspaceContext;

pub(super) fn public_report(
    config: &Config,
    report: CheckReport,
    include_suggestions: bool,
) -> Report {
    public_report_with_path_mode(config, report, include_suggestions, false)
}

pub(super) fn public_lsp_report(config: &Config, report: CheckReport) -> Report {
    public_report_with_path_mode(config, report, false, true)
}

fn public_report_with_path_mode(
    config: &Config,
    report: CheckReport,
    include_suggestions: bool,
    absolute_paths: bool,
) -> Report {
    Report {
        errors: report
            .errors
            .into_iter()
            .map(|diagnostic| public_finding(config, "error", diagnostic, absolute_paths))
            .collect(),
        warnings: report
            .warnings
            .into_iter()
            .map(|diagnostic| public_finding(config, "warning", diagnostic, absolute_paths))
            .collect(),
        // §FS-check.2.3: suggestions are surfaced only on demand. The public
        // severity tag stays `"suggestion"` so a consumer can tell them apart.
        suggestions: if include_suggestions {
            report
                .suggestions
                .into_iter()
                .map(|diagnostic| public_finding(config, "suggestion", diagnostic, absolute_paths))
                .collect()
        } else {
            Vec::new()
        },
    }
}

fn public_finding(
    config: &Config,
    severity: &'static str,
    diagnostic: Diagnostic,
    absolute_paths: bool,
) -> Finding {
    let render_path = |path: &Path| {
        if absolute_paths {
            // Private LSP transport data, not user-facing report text: keep the
            // platform-native spelling so parsing it back into a `Path` preserves
            // Windows verbatim-prefix absolute paths (§FS-lsp.1.1).
            absolutize_path(path).to_string_lossy().into_owned()
        } else {
            public_path(config, path)
        }
    };
    Finding {
        severity,
        code: diagnostic.code,
        path: diagnostic.path.map(|path| render_path(&path)),
        line: diagnostic.line,
        column: diagnostic.column,
        message: diagnostic.message,
        sites: diagnostic
            .sites
            .into_iter()
            .map(|site| FindingSite {
                path: render_path(&site.path),
                line: site.line,
            })
            .collect(),
    }
}

/// The run's warning channel, published (§FS-distribution.3.1): every
/// `[workspace]` caution the run settled, as the `Finding`s a frontend renders
/// (§FS-check.4.7.7, §FS-check.4.8.15, §FS-check.4.10.11, §FS-workspace.6.1.7).
///
/// Each keeps the anchor the engine gave it — the `grund.toml` line its own
/// message already names — so an editor publishes it without reading a location
/// back out of the text (§FS-lsp.1.1.3, §AR-bindings.2). A terminal renders the
/// message alone, in §FS-check.2.1.1's CLI-level shape: the render shape is a
/// property of the finding rather than a consequence of whether a location is
/// known, which is why the anchor rides along unused there.
pub(super) fn public_run_warnings(config: &Config, warnings: Vec<Diagnostic>) -> Vec<Finding> {
    run_warning_findings(config, warnings)
}

/// The run's warnings off the context every walking command loads, published
/// for that command's own output (§FS-distribution.3.1). Rendered against the
/// run's report base, like every other path a walking command prints
/// (§FS-errors.4).
pub(super) fn context_run_warnings(context: &WorkspaceContext) -> Vec<Finding> {
    public_run_warnings(context.render_config(), context.run_warnings.clone())
}

/// [`public_run_warnings`] with the absolute paths the editor transport needs,
/// for the same reason [`public_lsp_report`] takes them (§FS-lsp.1.1).
pub(super) fn public_lsp_run_warnings(config: &Config, warnings: Vec<Diagnostic>) -> Vec<Finding> {
    warnings
        .into_iter()
        .map(|diagnostic| public_finding(config, "warning", diagnostic, true))
        .collect()
}

fn public_path(config: &Config, path: &Path) -> String {
    display_path(config, path)
}

/// The `sites` value of a query-refusal JSON diagnostic (§FS-errors.5.2): `null`
/// when empty, else `[{ path, line }]` in the caller's order. Shared by the
/// `grund` CLI and the deprecated `grund_core::main_entry()` mirror so the two
/// printers cannot drift on the same bytes.
pub fn render_finding_sites_json(sites: &[FindingSite]) -> String {
    if sites.is_empty() {
        return "null".to_string();
    }
    let entries: Vec<String> = sites
        .iter()
        .map(|site| {
            format!(
                "{{\"path\":\"{}\",\"line\":{}}}",
                json_escape(&site.path),
                site.line
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}
