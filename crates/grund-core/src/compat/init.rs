use std::path::PathBuf;
use std::process::ExitCode;

use super::output::print_published_run_warnings;
use crate::writers::InitOpts;
use crate::writers::{InitAgentEntrypointSelection, InitNext, InitOutput, init};

pub(super) fn command_init(args: &[String]) -> ExitCode {
    let mut path: Option<PathBuf> = None;
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut docs = false;
    let mut force = false;
    let mut dry_run = false;
    let mut check = false;
    let mut no_vcs = false;
    let mut agent_selection = InitAgentEntrypointSelection::default();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--docs" => docs = true,
            "--force" => force = true,
            "--dry-run" => dry_run = true,
            "--check" => check = true,
            "--no-vcs" => no_vcs = true,
            "--agents-md" => agent_selection.canonical = true,
            "--claude" => agent_selection.claude = true,
            "--gemini" => agent_selection.gemini = true,
            "--copilot" => agent_selection.copilot = true,
            "--cursor" => agent_selection.cursor = true,
            "--windsurf" => agent_selection.windsurf = true,
            "--zed" => agent_selection.zed = true,
            "--name" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --name requires a value");
                    return ExitCode::from(2);
                }
                name = Some(args[idx].clone());
            }
            other if other.starts_with("--name=") => {
                name = Some(other.trim_start_matches("--name=").to_string());
            }
            "--description" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --description requires a value");
                    return ExitCode::from(2);
                }
                description = Some(args[idx].clone());
            }
            other if other.starts_with("--description=") => {
                description = Some(other.trim_start_matches("--description=").to_string());
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
            other => {
                if path.is_some() {
                    eprintln!("error: init takes at most one path argument");
                    return ExitCode::from(2);
                }
                path = Some(PathBuf::from(other));
            }
        }
        idx += 1;
    }

    let output = match init(InitOpts {
        target: path.unwrap_or_else(|| PathBuf::from(".")),
        name,
        description,
        docs,
        force,
        dry_run,
        check,
        no_vcs,
        agent_selection,
    }) {
        Ok(output) => output,
        Err(err) => {
            print_init_output(&err.output);
            eprintln!("error: {err}");
            return ExitCode::from(2);
        }
    };
    print_init_output(&output);
    if output.has_errors() {
        return ExitCode::from(1);
    }
    // §FS-init.4: the verdict is drawn from the report that was just printed,
    // and only `--check` asks for one — `--dry-run` alone keeps its `0`.
    if check && output.has_pending_changes() {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn print_init_output(output: &InitOutput) {
    print_published_run_warnings(&output.warnings);
    for finding in &output.errors {
        match (finding.path.as_deref(), finding.line) {
            (Some(path), Some(line)) => {
                println!("{path}:{line}: {}: {}", finding.severity, finding.message)
            }
            _ => println!("{}: {}", finding.severity, finding.message),
        }
    }
    for event in &output.events {
        eprintln!("{} {}", event.verb, event.path);
    }
    for note in &output.notes {
        eprintln!("note: {note}");
    }
    if let Some(next) = &output.next {
        print_next_block(next);
    }
}

/// Compatibility output consumes the same structured guidance as the shipped
/// CLI (§FS-init.2.2.2).
fn print_next_block(next: &InitNext) {
    eprint!("{}", next.render());
}
