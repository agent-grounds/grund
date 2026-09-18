use std::path::PathBuf;
use std::process::ExitCode;

use crate::api::FmtOpts;
use crate::api::format_references;
use crate::writers::FmtScanAbort;

pub(crate) fn command_fmt(args: &[String]) -> ExitCode {
    let mut path = PathBuf::from(".");
    let mut path_provided = false;
    let mut write = false;
    let mut check_flag = false;
    let mut marker = false;
    let mut cross_refs = false;
    for arg in args {
        match arg.as_str() {
            "--check" => check_flag = true,
            "--write" => write = true,
            "--marker" => marker = true,
            "--cross-refs" => cross_refs = true,
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
            other => {
                if path_provided {
                    eprintln!("error: fmt takes at most one path argument");
                    return ExitCode::from(2);
                }
                path = PathBuf::from(other);
                path_provided = true;
            }
        }
    }
    if write && check_flag {
        eprintln!("error: --check and --write cannot be used together");
        return ExitCode::from(2);
    }
    // §FS-fmt.3 / §AR-bindings.2: the compatibility adapter uses the same
    // workspace-wide strict preflight as the public API. Keeping a second
    // project loop here could let its mutation and aggregation ordering drift.
    let output = match format_references(FmtOpts {
        path,
        path_provided,
        write,
        add_marker: marker,
        cross_refs,
    }) {
        Ok(output) => output,
        Err(err) => {
            print_fmt_error(&err);
            return ExitCode::from(2);
        }
    };
    for path in &output.refused_writes {
        eprintln!("warning: {path}: not rewritten: the symlink target is outside the config root");
    }
    // §FS-fmt.3 / §FS-errors.1: the report is `fmt`'s output — on stdout, the
    // same stream `grund check`'s findings use, not the stderr transcript shape
    // `grund init` uses (§FS-errors.6). Only CLI-level `error:` lines go to stderr.
    if write {
        let mut files = output
            .changes
            .iter()
            .map(|change| change.path.clone())
            .collect::<Vec<_>>();
        files.sort();
        files.dedup();
        println!(
            "rewrote {} line{}{}",
            output.changes.len(),
            if output.changes.len() == 1 { "" } else { "s" },
            if files.is_empty() { "" } else { ":" }
        );
        for path in &files {
            let count = output
                .changes
                .iter()
                .filter(|change| &change.path == path)
                .count();
            println!("  {path} ({count})");
        }
    } else {
        for change in &output.changes {
            println!("{}:{}: {}", change.path, change.line, change.label);
        }
    }
    if !output.scan_errors.is_empty() {
        // Partial-scan semantics (§FS-fmt.3 / §FS-check.2): what was rewritten is
        // real — and, under `--write`, already on disk — but the tree the rewrite
        // ran over was not the whole tree.
        for error in &output.scan_errors {
            eprintln!("error: {}: {}", error.path, error.message);
        }
        return ExitCode::from(2);
    }
    if write || output.changes.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Render the compatibility CLI's fatal formatter error. Strict scan aborts
/// stay structured through the API boundary, so each path gets its own CLI
/// prefix and the same unmistakable refusal as the published CLI (§FS-fmt.3).
fn print_fmt_error(err: &anyhow::Error) {
    if let Some(abort) = err.downcast_ref::<FmtScanAbort>() {
        for error in &abort.scan_errors {
            eprintln!(
                "error: nothing was rewritten: {}: {}",
                error.path, error.message
            );
        }
    } else {
        eprintln!("error: {err:#}");
    }
}
