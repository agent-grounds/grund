//! The explicit external-snapshot materializer (§FS-fetch). This module owns
//! integration execution and output validation; every scanner and query remains
//! a reader under §REQ-runs-offline. The snapshot-home discovery and the atomic
//! install it ends in are `fetch_write.rs`.

use std::path::{Path, PathBuf};

use super::fetch_write::{write_file_home, write_folder_home};
use crate::config::{
    Config, KindConfig, ProjectRecords, RefusedHome, fetch_integration_path,
    fetch_names_nothing_reason, fetch_runs_nothing, run_warning_findings,
};
use crate::grammar::{Grammar, MarkdownBlocks, markdown_line_records, parse_id_arg};
use crate::model::{Finding, format_path, relative_from_base};
use crate::resolver::{fetch_place, load_alias_path, settled_run_warnings};
use crate::scanner::{Place, ScanFailure, unmarked_below_declaration};
use crate::workspace::{resolve_workspace_config, split_qualified_id_arg};

/// Which public CLI exit class a fetch refusal belongs to (§FS-fetch.7).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FetchFailureKind {
    Query,
    Operational,
}

/// A located fetch refusal with its fixed exit class (§FS-fetch.7).
#[derive(Debug)]
#[non_exhaustive]
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
    let root_config = resolve_workspace_config(path).map_err(|err| {
        let message = format!("{err:#}");
        *diagnostic = Some(err);
        fetch_operational(message)
    })?;
    let report_base = root_config.display().base().to_path_buf();
    // §FS-fetch.1, §FS-workspace.8.11: the one alias-path splitter, so `fetch` refuses
    // a malformed path with the words `show` prints, still at the operational exit.
    let (namespace, local) =
        split_qualified_id_arg(raw).map_err(|err| fetch_operational(format!("{err:#}")))?;
    // §FS-imports.writes.answer: the ID query's selection, over the alias path alone.
    let selected = if let Some(namespace) = namespace.as_deref() {
        let context =
            load_alias_path(ProjectRecords::of(root_config), namespace).map_err(|err| {
                let message = format!("{err:#}");
                *diagnostic = Some(err);
                fetch_operational(message)
            })?;
        let config = context.render_config();
        *run_warnings = run_warning_findings(config, context.run_warnings.clone());
        context
            .project_at_alias(namespace)
            .map_err(|err| fetch_operational(format!("{err:#}")))?
            .config
            .clone()
    } else {
        *run_warnings = run_warning_findings(&root_config, settled_run_warnings(root_config.run()));
        ProjectRecords::of(root_config)
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
    refuse_home_without_markdown(&selected, kind, &report_base)?;
    // §FS-imports.writes: asked last, so only a command that can run is named.
    let place = fetch_place(selected.run(), &report_base, namespace.as_deref(), local);
    if let Some(refusal) = place.import_refusal() {
        return Err(fetch_operational(refusal));
    }
    let integration = fetch_integration_path(&selected.root, fetch);

    // §FS-fetch.2.1: immediately before the spawn, behind every earlier refusal.
    if fetch_runs_nothing(fetch, &integration) {
        let reason = fetch_names_nothing_reason(&selected, &kind.kind, &report_base);
        return Err(fetch_operational(format!(
            "cannot run fetch integration `{fetch}` for {raw}: {reason}"
        )));
    }

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

    // The snapshot goes to the kind's first place on purpose: a fetched row's
    // several places are unspecified (§FS-config-v2.schema.places), one is the home.
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
    validate_fetched_declaration(snapshot, local, depth, &selected.grammar)?;
    match home {
        FetchHome::File(path) => write_file_home(
            &path,
            &selected.grammar,
            &id,
            // §FS-check.2.4.3: a file home is the row's own root, so `exclude` cannot prune it.
            ScanFailure::Unread {
                place: Place::Root,
                version: selected.frame().version,
            },
            &report_base,
            snapshot.as_bytes(),
            diagnostic,
        ),
        // §FS-fetch.5.3: a member's refusal still names the file from the run's report base.
        FetchHome::Folder(path) => write_folder_home(
            &path,
            &selected,
            &id,
            local,
            &report_base,
            snapshot.as_bytes(),
            diagnostic,
        ),
    }
}

enum FetchHome {
    File(PathBuf),
    Folder(PathBuf),
}

/// §FS-fetch.4, §FS-fetch.4.2: a value source and a file read as source hold no
/// Markdown declaration, and the answer is spelled from the run's report base
/// (§FS-errors.4), as `FetchPlace` spells its target. A value source's way out
/// names a folder without it, since one kept beside a fetched snapshot of its own ID
/// is a duplicate (§FS-values.2.3); a source file's moves its declarations too, since
/// one left there is what the scan goes on reading.
fn refuse_home_without_markdown(
    selected: &Config,
    kind: &KindConfig,
    report_base: &Path,
) -> std::result::Result<(), FetchFailure> {
    let Some((home, reason)) =
        selected
            .frame()
            .fetch_refused_home(selected.schema(), &selected.root, &kind.kind)
    else {
        return Ok(());
    };
    let shown = format_path(&relative_from_base(report_base, &home));
    let why = match reason {
        RefusedHome::ValueSource => {
            "a value source holds no Markdown; \
             move the kind to a Markdown file or a folder without it, or remove its `fetch`"
        }
        RefusedHome::SourceFile | RefusedHome::SourceFileValue => {
            "a file read as source holds no Markdown; \
             move the kind and its declarations to a Markdown file or a folder, \
             or remove its `fetch`"
        }
    };
    Err(fetch_operational(format!(
        "kind `{}` cannot fetch into {shown}: {why}",
        kind.kind
    )))
}

/// §FS-fetch.3: validate the complete stdout before any mutation.
pub(super) fn validate_fetched_declaration(
    text: &str,
    requested: &str,
    native_depth: usize,
    grammar: &Grammar,
) -> std::result::Result<(), FetchFailure> {
    // §FS-check.1.1.5.1: the output is read by the one fence and raw-text block
    // machine the scanner reads its snapshot with, so a heading shown inside
    // either is content, never the declaration or one of its sections.
    let mut blocks = MarkdownBlocks::default();
    let mut declaration_count = 0usize;
    let mut first_content = true;
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        let block = blocks.line(line);
        if block.in_fence() {
            first_content = false;
            continue;
        }
        let hashes = trimmed.bytes().take_while(|byte| *byte == b'#').count();
        // §FS-fetch.3.1: a line below the requested declaration that the scanner would record
        // as a declaration, conforming or near miss, is refused at any depth before the depth
        // limit, by the scanner's own reading rather than this validator's heading test.
        if declaration_count == 1
            && block.may_be_heading()
            && hashes > native_depth
            && let Some(nested) = markdown_line_records(grammar, line)
        {
            return Err(fetch_operational(format!(
                "fetch output nests declaration `{nested}` inside requested `{requested}`; \
                 fetch output must hold only the requested declaration"
            )));
        }
        // §FS-fetch.3.2: at subsection depth, a heading the next scan would call unmarked.
        if declaration_count == 1
            && block.may_be_heading()
            && unmarked_below_declaration(grammar, line, native_depth + 1)
        {
            return Err(fetch_operational(format!(
                "fetch output line {}: heading `{}` is not a section of `{requested}`; \
                 number it, fence it, or make it a bold label",
                index + 1,
                line.trim_end()
            )));
        }
        // An ATX heading's `#`s end at a space or a tab, as the scanner reads them
        // (§FS-declarations.checks.unmarked-heading.1).
        if block.may_be_heading()
            && hashes > 0
            && matches!(trimmed.as_bytes().get(hashes), Some(b' ' | b'\t'))
        {
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
