//! What the deprecated `main_entry()` path prints (§AR-system.2.9): the text and
//! JSON report shapes of §FS-errors.1 and §FS-errors.5, the bare query refusal,
//! and the CLI-level `warning:` lines that have no report to ride on — the
//! config warnings of §FS-config.4.2 and the run's `[workspace]` channel, which
//! this frontend's own workspace loader renders before a command says anything.
//!
//! The `output` category was this file, and it held two things §AR-system.4 does
//! not allow in one place: the spellings every component below needs — the path
//! renderers, the sort key, the JSON escape, the English list — and the printing.
//! The spellings went down into `model/paths.rs`, `model/text.rs` and
//! `config/report_paths.rs`, each to the lowest component that reads it; the
//! `Diagnostic` builders the run folds in went up into `api/scope_cautions.rs`
//! and `api/config_findings.rs`. What is left is the printing, which is the
//! deprecated path's and nobody else's: the run-level `[workspace]` warnings the
//! live path once reached in here are a data channel now, and this frontend
//! renders them like any other (§FS-distribution.3.1, §DA-engine-renders-nothing).

/// Print the compatibility CLI's text report in the fixed output shapes
/// (§FS-errors.1, §FS-errors.2.1, §FS-errors.2.4): channel-bearing located
/// findings, run-level diagnostics on stderr, and `success` for a clean text
/// check (§FS-check.2.1). Diagnostic lines stay in the fixed order
/// (§FS-errors.4).
use anyhow::Result;
use std::path::Path;

use crate::api::{config_warnings, render_finding_sites_json};
use crate::checker::diagnostic_cmp;
use crate::config::{Config, display_path};
use crate::model::{CheckReport, Diagnostic, Finding, FindingSite, json_escape};
use crate::resolver::{WorkspaceContext, load_workspace_context};

/// The run's `[workspace]` warnings in §FS-errors.2.2's CLI-level shape — one
/// `warning: ` line each on stderr, ahead of anything the command itself prints
/// and with the exit code untouched (§FS-check.4.7, §FS-check.4.8,
/// §FS-check.4.10, §FS-workspace.6.1).
///
/// The deprecated frontend renders the same channel the published one renders,
/// from the same place, so the two cannot drift on a byte (§FS-lsp.4).
pub(super) fn print_run_warnings(warnings: &[Diagnostic]) {
    for warning in warnings {
        eprintln!("warning: {}", warning.message);
    }
}

/// The same channel once it has been published for a caller (§FS-distribution.3.1):
/// what the api surfaces hand back is a `Finding`, and the line it renders as is
/// the one above.
pub(super) fn print_published_run_warnings(warnings: &[Finding]) {
    for warning in warnings {
        eprintln!("warning: {}", warning.message);
    }
}

/// This frontend's workspace loader: the run's `[workspace]` warnings rendered
/// before the command prints anything of its own, then the loaded context.
///
/// One door rather than one call per adapter, for the reason every other shared
/// printer here is shared: five commands carry these four findings, and five
/// copies of the same three lines are five places for them to drift
/// (§DF-unlisted-workspace-block.2.4).
pub(super) fn workspace_context(path: &Path, path_provided: bool) -> Result<WorkspaceContext> {
    let context = load_workspace_context(path, path_provided)?;
    print_run_warnings(&context.run_warnings);
    Ok(context)
}

pub(super) fn print_report(
    config: &Config,
    report: &CheckReport,
    include_suggestions: bool,
    run_warnings: usize,
) {
    // §FS-check.2.3: the `success` marker keys off errors and warnings only — a
    // suggestion is not a finding about well-formedness, so it never suppresses
    // `success`, and without `--suggestions` it is not printed at all.

    // §FS-check.4.7 / §FS-check.4.10 / §FS-workspace.6.1: a warning this frontend
    // printed before the report existed, so the marker asks the run's warning
    // channel too — else stderr says unchecked and stdout says `success`.
    if run_warnings == 0
        && report.errors.is_empty()
        && report.warnings.is_empty()
        && (!include_suggestions || report.suggestions.is_empty())
    {
        println!("success");
        return;
    }
    // §FS-errors.4: each report partition is already bytewise sorted; joining
    // errors, warnings, then suggestions realizes text's fixed channel groups.
    let mut diagnostics = report
        .errors
        .iter()
        .map(|diagnostic| ("error", diagnostic))
        .chain(
            report
                .warnings
                .iter()
                .map(|diagnostic| ("warning", diagnostic)),
        )
        .collect::<Vec<_>>();
    if include_suggestions {
        diagnostics.extend(
            report
                .suggestions
                .iter()
                .map(|diagnostic| ("suggestion", diagnostic)),
        );
    }
    for (severity, diagnostic) in diagnostics {
        let line = render_diagnostic_text(config, severity, diagnostic);
        // §FS-errors.1 / §FS-check.2.1: a located finding is `check`'s output →
        // stdout. A `line`-less diagnostic — a mid-walk read failure (§FS-check.2)
        // or the empty-scan caution (§FS-check.2.2) — is about the run → stderr.
        if diagnostic.line.is_some() {
            println!("{line}");
        } else {
            eprintln!("{line}");
        }
    }
}

fn render_diagnostic_text(config: &Config, severity: &str, diagnostic: &Diagnostic) -> String {
    match (&diagnostic.path, diagnostic.line) {
        (Some(path), Some(line)) => {
            format!(
                "{}:{}: {severity}: {}",
                display_path(config, path),
                line,
                diagnostic.message
            )
        }
        // A file-level finding with no line to point at (e.g. an unreadable file
        // discovered mid-walk) uses the CLI-level shape — §FS-check.2, §FS-errors.2.2.
        (Some(path), None) => format!(
            "{severity}: {}: {}",
            display_path(config, path),
            diagnostic.message
        ),
        _ => format!("{severity}: {}", diagnostic.message),
    }
}

fn sorted_json_diagnostics(
    report: &CheckReport,
    include_suggestions: bool,
) -> Vec<(&'static str, &Diagnostic)> {
    let mut diagnostics = report
        .warnings
        .iter()
        .map(|diagnostic| ("warning", diagnostic))
        .chain(report.errors.iter().map(|diagnostic| ("error", diagnostic)))
        .collect::<Vec<_>>();
    if include_suggestions {
        diagnostics.extend(
            report
                .suggestions
                .iter()
                .map(|diagnostic| ("suggestion", diagnostic)),
        );
    }
    diagnostics.sort_by(|(_, a), (_, b)| diagnostic_cmp(a, b));
    diagnostics
}

/// Print the report as newline-delimited JSON objects — the `--format json` /
/// `[output] format = "json"` shape (§FS-errors.5): one object per finding with
/// `severity`, `path`, `line`, `code`, `message`, `sites`. Located findings go to
/// stdout (`check`'s output, §FS-errors.1); a `line`-less diagnostic (mid-walk read
/// failure, empty-scan caution) goes to stderr, mirroring the text form.
pub(super) fn print_json_report(config: &Config, report: &CheckReport, include_suggestions: bool) {
    for (channel, diagnostic) in sorted_json_diagnostics(report, include_suggestions) {
        let object = render_diagnostic_json(config, channel, diagnostic);
        if diagnostic.line.is_some() {
            println!("{object}");
        } else {
            eprintln!("{object}");
        }
    }
}

/// Render one diagnostic as a JSON object (§FS-errors.5). An `error` / `warning`
/// carries a `"severity"`; a citation-direction `suggestion` carries
/// `"channel":"suggestion"` instead, keeping the frozen `{error, warning}`
/// severity set intact (§FS-config.6, §FS-check.2.3).
fn render_diagnostic_json(config: &Config, channel: &str, diagnostic: &Diagnostic) -> String {
    let path = diagnostic
        .path
        .as_ref()
        .map(|path| format!("\"{}\"", json_escape(&display_path(config, path))))
        .unwrap_or_else(|| "null".to_string());
    let line = diagnostic
        .line
        .map(|line| line.to_string())
        .unwrap_or_else(|| "null".to_string());
    let sites = if diagnostic.sites.is_empty() {
        "null".to_string()
    } else {
        let values = diagnostic
            .sites
            .iter()
            .map(|site| {
                format!(
                    "{{\"path\":\"{}\",\"line\":{}}}",
                    json_escape(&display_path(config, &site.path)),
                    site.line
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("[{}]", values)
    };
    let tag = if channel == "suggestion" {
        "\"channel\":\"suggestion\"".to_string()
    } else {
        format!("\"severity\":\"{channel}\"")
    };
    format!(
        "{{{},\"path\":{},\"line\":{},\"code\":\"{}\",\"message\":\"{}\",\"sites\":{}}}",
        tag,
        path,
        line,
        diagnostic.code,
        json_escape(&diagnostic.message),
        sites
    )
}

/// §FS-errors.5: `sites` here are already display strings from the raise site
/// in `queries/show.rs` (rendered against `path_config`, which may be a
/// workspace root this printer's own `Config` is not), so they are rendered
/// through the shared [`render_finding_sites_json`] rather than re-derived
/// from a `Config` the way `render_diagnostic_json` renders `check`'s sites.
pub(super) fn print_bare_query_json(code: &'static str, message: &str, sites: &[FindingSite]) {
    eprintln!(
        "{{\"severity\":\"error\",\"path\":null,\"line\":null,\"code\":\"{}\",\"message\":\"{}\",\"sites\":{}}}",
        code,
        json_escape(message),
        render_finding_sites_json(sites)
    );
}

pub(super) fn show_query_error_code(message: &str) -> &'static str {
    if message.starts_with("ID not found:") {
        "not-found"
    } else if message.starts_with("section not found:") {
        "missing-section"
    } else if message.starts_with("invalid ID") {
        "invalid-id"
    } else if message.starts_with("ambiguous ID:") {
        "ambiguous"
    // §FS-show.2.2.2: the section-level twin of the ambiguous-ID refusal, under
    // its own code — the two need different edits, and a JSON consumer should not
    // have to read the prose to tell them apart (§DF-duplicate-section-path.2.5).
    } else if message.starts_with("ambiguous section:") {
        "ambiguous-section"
    } else if message.starts_with("broken stub:") {
        "broken-stub"
    } else {
        "query-failed"
    }
}

/// Print [`config_warnings`] in the CLI-level shape (§FS-errors.2.2): one
/// `warning: ` line each, on stderr, exit code untouched. Rendering, so it
/// lives with the other report printers rather than in `api/`, which is the
/// embedding surface *without* stdout/stderr rendering
/// (§AR-core-module-layout.1) — and in one place rather than in each `config`
/// frontend, so the published CLI and the deprecated `grund_core` adapter
/// cannot drift on the prefix or the stream (§FS-config.4.1, §FS-config.4.2).
pub fn print_config_warnings(config: &Config) {
    for warning in config_warnings(config) {
        eprintln!("warning: {warning}");
    }
}
