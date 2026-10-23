//! The published `check` contract (§AR-system.2.9): the options a caller passes,
//! the output it gets back, and the three entry points that return data rather
//! than printing a report or mapping a process exit (§FS-distribution.3.1,
//! §AR-bindings.2).
//!
//! The run behind them is `run.rs` and the conversion into the published shape
//! is `report.rs`, which is the boundary §AR-core-module-layout.2 asks for: this
//! file is the surface an embedder reads, and neither the pipeline nor the
//! adapters are in it.

use anyhow::Result;
use std::path::{Path, PathBuf};

use super::report::public_report;
use super::run::run_check_with_run_warnings;
use crate::checker::ArmedChecks;
use crate::model::{Catalog, Finding, Report, Switch};
use crate::scanner::scan_tree_strict;
use crate::workspace::resolve_workspace_config;

#[derive(Clone)]
#[non_exhaustive]
pub struct CheckOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub require_grounding: bool,
    /// Surface the citation-direction suggestions channel (§FS-check.2.3) —
    /// the `grund check --suggestions` flag at the library level.
    pub include_suggestions: bool,
    /// Walk the whole config root past `[scan] include`, adding the out-of-scope
    /// reference tier and scanner-invariant outside-section findings
    /// (§FS-check.1.3, §FS-check.3.14, §FS-declarations.checks.section-outside-declaration) — the `grund check --full`
    /// flag at the library level. Purely additive: the findings inside the
    /// configured scope are unchanged by it.
    pub full: bool,
    /// One additive controlled-English constraint (§FS-rules.8).
    pub rule: Option<String>,
}

impl Default for CheckOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
            require_grounding: false,
            include_suggestions: false,
            full: false,
            rule: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CheckOutput {
    pub output_format: String,
    pub report: Report,
    pub had_scan_errors: bool,
    /// The run's warning channel (§FS-distribution.3.1): the `[workspace]`
    /// cautions this run settled before any report existed — §FS-check.4.10's
    /// unread opted-out block and §FS-workspace.6.1.7.6's undecidable ancestor
    /// claim, each anchored at the
    /// `grund.toml` line its message names. Not report findings — the report
    /// carries none of them, and none enters the §FS-errors.5.5 selector
    /// vocabulary — but a caller renders each as a CLI-level `warning:`
    /// (§FS-check.2.1.1) and a run that earns one prints no `success`
    /// (§FS-check.2.1.3). §FS-check.3.29 is one of the report's errors
    /// (§FS-check.3.29.13) and is not here, and §FS-check.3.30's absorbed scan
    /// refuses the run as a config error.
    pub warnings: Vec<Finding>,
}

/// Scan one project tree and return the raw scanner findings. This is the
/// embedding surface later frontends share instead of re-reading files.
pub fn scan(path: &Path) -> Result<Catalog> {
    let config = resolve_workspace_config(path)?;
    scan_tree_strict(config.schema(), config.frame(), Some(path), true)
}

/// Programmatic `check`: load config, scan, and return structured findings
/// without CLI argument parsing, stdout/stderr rendering, or exit-code mapping
/// (§FS-distribution.3.1, §AR-bindings.2).
pub fn check(path: &Path) -> Result<Report> {
    Ok(check_with_opts(CheckOpts {
        path: path.to_path_buf(),
        path_provided: true,
        require_grounding: false,
        include_suggestions: false,
        full: false,
        rule: None,
    })?
    .report)
}

/// Programmatic `check` with the same scope and grounding options as the CLI,
/// returning data instead of printing a report or mapping a process exit.
pub fn check_with_opts(opts: CheckOpts) -> Result<CheckOutput> {
    check_with_run_warnings(opts).1.map(|(output, _)| output)
}

/// [`check_with_opts`] with the run-root warnings preserved beside a later
/// refusal (§FS-check.4.10.8), and with what the run armed, which only a
/// frontend's `--only` reads (§FS-check.1.4.1): it is never a field of the
/// public [`CheckOutput`]. Frontends render the returned data; the engine
/// writes no stream (§FS-distribution.3.1).
#[doc(hidden)]
pub fn check_with_run_warnings(
    opts: CheckOpts,
) -> (Vec<Finding>, Result<(CheckOutput, ArmedChecks)>) {
    let mut warnings = Vec::new();
    let output = check_run(opts, &mut warnings).map(|(output, armed, _)| (output, armed));
    (warnings, output)
}

/// [`check_with_opts`] with the root of each project the run checked,
/// which `config migrate`'s report gate compares (§FS-config-migrate.5.2).
pub(super) fn check_with_projects(
    opts: CheckOpts,
) -> (Vec<Finding>, Result<(CheckOutput, Vec<PathBuf>)>) {
    let mut warnings = Vec::new();
    let output = check_run(opts, &mut warnings).map(|(output, _, projects)| (output, projects));
    (warnings, output)
}

fn check_run(
    opts: CheckOpts,
    warnings: &mut Vec<Finding>,
) -> Result<(CheckOutput, ArmedChecks, Vec<PathBuf>)> {
    let mut run = run_check_with_run_warnings(
        &opts.path,
        opts.path_provided,
        opts.require_grounding,
        opts.full,
        opts.rule.as_deref(),
        warnings,
    )?;
    let mut armed = std::mem::take(&mut run.report.armed);
    // §FS-check.1.4.1: the suggestions channel's gate is the flag, decided here.
    if opts.include_suggestions {
        armed.arm(Switch::Suggestions);
    }
    let armed = ArmedChecks::new(armed, run.config.frame().version);
    let output = CheckOutput {
        output_format: run.config.output_format.clone(),
        warnings: warnings.clone(),
        report: public_report(&run.config, run.report, opts.include_suggestions),
        had_scan_errors: run.had_scan_errors,
    };
    Ok((output, armed, run.projects))
}
