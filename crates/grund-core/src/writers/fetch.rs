//! The explicit external-snapshot materializer (§FS-fetch). This module owns
//! integration execution and output validation; every scanner and query remains
//! a reader under §REQ-runs-offline. The snapshot-home discovery and the atomic
//! install it ends in are `fetch_write.rs`.

use std::path::{Path, PathBuf};

use super::fetch_write::{write_file_home, write_folder_home};
use crate::config::run_warning_findings;
use crate::grammar::{MarkdownBlocks, parse_id_arg};
use crate::model::Finding;
use crate::resolver::settled_run_warnings;
use crate::workspace::{expand_workspace_tree, resolve_workspace_config};

/// Which public CLI exit class a fetch refusal belongs to (§FS-fetch.7).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FetchFailureKind {
    Query,
    Operational,
}

/// A located fetch refusal with its fixed exit class (§FS-fetch.7).
#[derive(Debug)]
pub struct FetchFailure {
    pub kind: FetchFailureKind,
    pub message: String,
}

impl std::fmt::Display for FetchFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FetchFailure {}

fn fetch_query(message: impl Into<String>) -> FetchFailure {
    FetchFailure {
        kind: FetchFailureKind::Query,
        message: message.into(),
    }
}

pub(super) fn fetch_operational(message: impl Into<String>) -> FetchFailure {
    FetchFailure {
        kind: FetchFailureKind::Operational,
        message: message.into(),
    }
}

/// Retain the source beside the unchanged fetch refusal (§FS-distribution.3.3.2).
pub(super) fn fetch_io(
    diagnostic: &mut Option<anyhow::Error>,
    path: &Path,
    error: std::io::Error,
    context: String,
) -> FetchFailure {
    let failure = fetch_operational(format!("{context}: {error}"));
    let source = crate::model::OperationDiagnostic::filesystem(path, &error, context);
    *diagnostic = Some(anyhow::Error::new(error).context(source));
    failure
}

/// Materialize exactly one local or qualified external ID (§FS-fetch.1).
pub fn fetch_snapshot(raw: &str, path: &Path) -> std::result::Result<(), FetchFailure> {
    fetch_snapshot_with_run_warnings(raw, path).1
}

/// [`fetch_snapshot`] for a frontend that also renders the run's `[workspace]`
/// warnings (§FS-check.4.10.11, §FS-workspace.6.1.7): `fetch` resolves
/// a block's member boundary like every other walking command, and its failure
/// is a typed refusal with nowhere to carry a caution.
#[doc(hidden)]
pub fn fetch_snapshot_with_run_warnings(
    raw: &str,
    path: &Path,
) -> (Vec<Finding>, std::result::Result<(), FetchFailure>) {
    let mut run_warnings = Vec::new();
    let result = fetch_run(raw, path, &mut run_warnings, &mut None);
    (run_warnings, result)
}

/// Structured source errors without changing FetchFailure (§FS-distribution.3.1).
pub(crate) fn fetch_with_diagnostics(raw: &str, path: &Path) -> (Vec<Finding>, anyhow::Result<()>) {
    let mut warnings = Vec::new();
    let mut diagnostic = None;
    let result = fetch_run(raw, path, &mut warnings, &mut diagnostic).map_err(|failure| {
        diagnostic.unwrap_or_else(|| {
            crate::model::OperationDiagnostic::new(
                match failure.kind {
                    FetchFailureKind::Query => "query",
                    FetchFailureKind::Operational => "operation",
                },
                "fetch",
                failure.message,
            )
            .into()
        })
    });
    (warnings, result)
}

fn fetch_run(
    raw: &str,
    path: &Path,
    run_warnings: &mut Vec<Finding>,
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<(), FetchFailure> {
    let mut root_config = resolve_workspace_config(path).map_err(|err| {
        let message = format!("{err:#}");
        *diagnostic = Some(err);
        fetch_operational(message)
    })?;
    let (namespace, local) = raw
        .rsplit_once('/')
        .map_or((None, raw), |(ns, id)| (Some(ns), id));
    let selected = if let Some(namespace) = namespace {
        if !root_config.workspace_declared {
            return Err(fetch_operational(format!(
                "unknown project alias `{namespace}` for fetch"
            )));
        }
        let projects = expand_workspace_tree(&mut root_config).map_err(|err| {
            let message = format!("{err:#}");
            *diagnostic = Some(err);
            fetch_operational(message)
        })?;
        *run_warnings = run_warning_findings(&root_config, settled_run_warnings(root_config.run()));
        projects
            .into_iter()
            .find(|project| project.alias == namespace)
            .map(|project| project.config)
            .ok_or_else(|| {
                fetch_operational(format!("unknown project alias `{namespace}` for fetch"))
            })?
    } else {
        *run_warnings = run_warning_findings(&root_config, settled_run_warnings(root_config.run()));
        root_config
    };

    let (id, section) =
        parse_id_arg(local, &selected.grammar).map_err(|err| fetch_query(format!("{err:#}")))?;
    if section.is_some() {
        return Err(fetch_query(format!("invalid ID `{raw}`")));
    }
    let kind = selected
        .kinds
        .iter()
        .find(|kind| kind.kind == id.kind)
        .ok_or_else(|| fetch_query(format!("invalid ID `{raw}`")))?;
    let fetch = kind.fetch.as_deref().ok_or_else(|| {
        fetch_operational(format!("kind `{}` has no fetch integration", kind.kind))
    })?;
    let integration = {
        let configured = Path::new(fetch);
        if configured.is_absolute() {
            configured.to_path_buf()
        } else {
            selected.root.join(configured)
        }
    };

    // §FS-fetch.2: direct argv invocation, inherited environment, no shell and
    // no implicit stdin. Stdout is data and is never forwarded to the caller.
    let output = std::process::Command::new(&integration)
        .arg(local)
        .current_dir(&selected.root)
        .output()
        .map_err(|err| {
            fetch_io(
                diagnostic,
                &integration,
                err,
                format!("cannot run fetch integration `{fetch}` for {raw}"),
            )
        })?;
    if !output.status.success() {
        let status = output
            .status
            .code()
            .map_or_else(|| "signal".to_string(), |code| code.to_string());
        return Err(fetch_operational(format!(
            "fetch integration `{fetch}` for {raw} exited {status}"
        )));
    }
    let snapshot = std::str::from_utf8(&output.stdout).map_err(|_| {
        fetch_operational(format!(
            "fetch integration `{fetch}` for {raw} emitted non-UTF-8 output"
        ))
    })?;

    let (home, depth) = match (&kind.file, &kind.folder) {
        (Some(file), None) => (FetchHome::File(selected.root.join(file)), 2),
        (None, Some(folder)) => (FetchHome::Folder(selected.root.join(folder)), 1),
        _ => {
            return Err(fetch_operational(format!(
                "kind `{}` cannot fetch without exactly one snapshot home",
                kind.kind
            )));
        }
    };
    validate_fetched_declaration(snapshot, local, depth)?;
    match home {
        FetchHome::File(path) => write_file_home(
            &path,
            &selected.grammar,
            &id,
            snapshot.as_bytes(),
            diagnostic,
        ),
        FetchHome::Folder(path) => write_folder_home(
            &path,
            &selected,
            &id,
            local,
            snapshot.as_bytes(),
            diagnostic,
        ),
    }
}

enum FetchHome {
    File(PathBuf),
    Folder(PathBuf),
}

/// §FS-fetch.3: validate the complete stdout before any mutation.
fn validate_fetched_declaration(
    text: &str,
    requested: &str,
    native_depth: usize,
) -> std::result::Result<(), FetchFailure> {
    // §FS-check.1.1.5.1: the output is read by the one fence and raw-text block
    // machine the scanner reads its snapshot with, so a heading shown inside
    // either is content, never the declaration or one of its sections.
    let mut blocks = MarkdownBlocks::default();
    let mut declaration_count = 0usize;
    let mut first_content = true;
    for line in text.lines() {
        let trimmed = line.trim_start();
        let block = blocks.line(line);
        if block.in_fence() {
            first_content = false;
            continue;
        }
        let hashes = trimmed.bytes().take_while(|byte| *byte == b'#').count();
        if block.may_be_heading() && hashes > 0 && trimmed.as_bytes().get(hashes) == Some(&b' ') {
            if hashes == native_depth {
                let tail = &trimmed[hashes + 1..];
                let Some((id, title)) = tail.split_once(':') else {
                    return Err(fetch_operational(
                        "fetch output declaration needs `: <title>`",
                    ));
                };
                declaration_count += 1;
                if declaration_count != 1 || !first_content {
                    return Err(fetch_operational(
                        "fetch output must contain exactly one declaration and no leading content",
                    ));
                }
                if id.trim() != requested {
                    return Err(fetch_operational(format!(
                        "fetch output declares `{}` instead of requested `{requested}`",
                        id.trim()
                    )));
                }
                if title.trim().is_empty() {
                    return Err(fetch_operational("fetch output declaration title is empty"));
                }
            } else if declaration_count == 0 || hashes != native_depth + 1 {
                return Err(fetch_operational(format!(
                    "fetch output heading depth {hashes} is invalid for a depth-{native_depth} declaration"
                )));
            }
        } else if first_content && !line.trim().is_empty() {
            return Err(fetch_operational(
                "fetch output must begin with one Markdown declaration",
            ));
        }
        if !line.trim().is_empty() {
            first_content = false;
        }
    }
    if declaration_count != 1 {
        return Err(fetch_operational(
            "fetch output must contain exactly one Markdown declaration",
        ));
    }
    Ok(())
}
