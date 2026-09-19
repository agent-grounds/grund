//! The deprecated `grund check` adapter (§AR-system.2.9.1): argv, the exact-code
//! selectors, the report format, the printing and the exit code (§FS-check.1,
//! §FS-check.2.1, §FS-cli.5).
//!
//! The renderer half of the flat `check` adapter. The run it prints — the
//! config resolution, the scan, the check and the cautions and config findings
//! folded in — is `api/run.rs`, read downward, so this surface cannot report
//! something the embedding API does not (§FS-lsp.4, §AR-system.4).

/// `grund check [path]`: validate repeatable exact-code selectors before config
/// discovery (§FS-check.1.4), scan the whole tree, select the completed report,
/// and exit `0` clean / `1` on a retained error / `2` on a CLI or I/O failure
/// (§FS-check.2.1, §FS-cli.5).
use std::path::PathBuf;
use std::process::ExitCode;

use super::output::{print_json_report, print_report, print_run_warnings};
use crate::api::run_check;
use crate::checker::CheckFindingSelection;
use crate::resolver::settled_run_warnings;

pub(crate) fn command_check(args: &[String]) -> ExitCode {
    let mut path = PathBuf::from(".");
    let mut path_provided = false;
    let mut format_override = None;
    let mut require_grounding = false;
    let mut include_suggestions = false;
    let mut full = false;
    let mut selection = CheckFindingSelection::default();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            other if other.starts_with("--format=") => {
                format_override = Some(other.trim_start_matches("--format=").to_string());
            }
            "--format" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --format requires a value");
                    return ExitCode::from(2);
                }
                format_override = Some(args[idx].clone());
            }
            "--require-grounding" => require_grounding = true,
            "--full" => full = true,
            "--suggestions" => include_suggestions = true,
            other if other.starts_with("--only=") => {
                let value = other
                    .strip_prefix("--only=")
                    .expect("guarded by starts_with");
                if let Err(err) = selection.add_only(value) {
                    eprintln!("error: {err}");
                    return ExitCode::from(2);
                }
            }
            "--only" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --only requires a finding code");
                    return ExitCode::from(2);
                }
                if let Err(err) = selection.add_only(&args[idx]) {
                    eprintln!("error: {err}");
                    return ExitCode::from(2);
                }
            }
            other if other.starts_with("--ignore=") => {
                let value = other
                    .strip_prefix("--ignore=")
                    .expect("guarded by starts_with");
                if let Err(err) = selection.add_ignore(value) {
                    eprintln!("error: {err}");
                    return ExitCode::from(2);
                }
            }
            "--ignore" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --ignore requires a finding code");
                    return ExitCode::from(2);
                }
                if let Err(err) = selection.add_ignore(&args[idx]) {
                    eprintln!("error: {err}");
                    return ExitCode::from(2);
                }
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
            other => {
                // §FS-cli.3.2: a path-taking subcommand accepts at most one path;
                // a second positional is a CLI error, never a silent drop.
                if path_provided {
                    eprintln!("error: check takes at most one path argument");
                    return ExitCode::from(2);
                }
                path = PathBuf::from(other);
                path_provided = true;
            }
        }
        idx += 1;
    }
    if let Some(format) = &format_override
        && !matches!(format.as_str(), "text" | "json")
    {
        eprintln!("error: unsupported check format `{format}`");
        return ExitCode::from(2);
    }
    let mut run = match run_check(&path, path_provided, require_grounding, full) {
        Ok(run) => run,
        Err(err) => {
            eprintln!("error: {err:#}");
            return ExitCode::from(2);
        }
    };
    let format = format_override.unwrap_or_else(|| run.config.output_format.clone());
    if !matches!(format.as_str(), "text" | "json") {
        eprintln!("error: unsupported check format `{format}`");
        return ExitCode::from(2);
    }
    // §FS-check.2.1.2: filter only after the ordinary checker completed, before
    // the existing sort, rendering, and selected-report exit decision.
    run.report
        .errors
        .retain(|diagnostic| selection.retains(diagnostic.code));
    run.report
        .warnings
        .retain(|diagnostic| selection.retains(diagnostic.code));
    run.report
        .suggestions
        .retain(|diagnostic| selection.retains(diagnostic.code));
    // §FS-check.4.7.7, §FS-check.4.10.11, §FS-workspace.6.1: the run's warning channel,
    // rendered ahead of the report exactly where the engine used to print it, in
    // §FS-check.2.1.1's shape on both formats (§FS-errors.5.2).
    let run_warnings = settled_run_warnings(&run.config);
    print_run_warnings(&run_warnings);
    if format == "json" {
        print_json_report(&run.config, &run.report, include_suggestions);
    } else {
        print_report(
            &run.config,
            &run.report,
            include_suggestions,
            run_warnings.len(),
        );
    }
    if run.had_scan_errors {
        ExitCode::from(2)
    } else if run.report.errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
