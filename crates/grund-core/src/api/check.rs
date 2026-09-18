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
use super::run::run_check;
use crate::model::{Findings, Report};
use crate::scanner::scan_tree_strict;
use crate::workspace::resolve_workspace_config;

#[derive(Clone)]
pub struct CheckOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub require_grounding: bool,
    /// Surface the citation-direction suggestions channel (§FS-check.2.3) —
    /// the `grund check --suggestions` flag at the library level.
    pub include_suggestions: bool,
    /// Walk the whole config root past `[scan] include`, adding the out-of-scope
    /// reference tier and scanner-invariant outside-section findings
    /// (§FS-check.1.3, §FS-check.3.14, §FS-check.3.23) — the `grund check --full`
    /// flag at the library level. Purely additive: the findings inside the
    /// configured scope are unchanged by it.
    pub full: bool,
}

impl Default for CheckOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
            require_grounding: false,
            include_suggestions: false,
            full: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckOutput {
    pub output_format: String,
    pub report: Report,
    pub had_scan_errors: bool,
    /// §FS-check.4.10: how many `[workspace]` blocks this run already told the
    /// reader no project scans, on stderr, before the report existed. Not a
    /// finding — the report carries none for it — but a caller that prints the
    /// `success` marker has to know stderr is not empty (§FS-check.2.1).
    pub unread_opted_out_blocks: usize,
}

/// Scan one project tree and return the raw scanner findings. This is the
/// embedding surface later frontends share instead of re-reading files.
pub fn scan(path: &Path) -> Result<Findings> {
    let config = resolve_workspace_config(path)?;
    scan_tree_strict(&config, Some(path), true)
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
    })?
    .report)
}

/// Programmatic `check` with the same scope and grounding options as the CLI,
/// returning data instead of printing a report or mapping a process exit.
pub fn check_with_opts(opts: CheckOpts) -> Result<CheckOutput> {
    let run = run_check(
        &opts.path,
        opts.path_provided,
        opts.require_grounding,
        opts.full,
    )?;
    Ok(CheckOutput {
        output_format: run.config.output_format.clone(),
        unread_opted_out_blocks: run.config.unread_opted_out_blocks,
        report: public_report(&run.config, run.report, opts.include_suggestions),
        had_scan_errors: run.had_scan_errors,
    })
}
