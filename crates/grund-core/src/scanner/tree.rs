use anyhow::{Result, anyhow};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use super::e2e::scan_e2e_cases;
use super::file_pass::scan_file_text;
use super::legacy::promote_local_legacy_citations;
use super::merge::merge_findings;
use super::read_predicate::unread;
use super::section_locks::read_section_locks;
use super::stub_homes::resolve_stubs;
use super::value_json::{scan_value_sources, value_sources};
use super::walk::{scan_roots, walk_scannable_files_reporting};
use crate::config::{Frame, Schema};
use crate::grammar::resolve_shorthand_citations;
use crate::model::{Catalog, normalize_path_lexically, sort_path_key};
use crate::sources::{canonicalize_existing_prefix, paths_same_location};
use crate::workspace::WorkspaceCitationTarget;

/// The file count above which one tree scan is worth splitting across threads
/// (§AR-scanner.1.12, §GOAL-fast-feedback): below it the rayon fan-out costs more
/// than the pass it parallelizes.
const PARALLEL_SCAN_MIN_FILES: usize = 256;

/// Depth of a heading line — count of leading `#` — used to decide whether a
/// section heading nests under the current declaration (§AR-scanner.2.2).
pub(super) fn heading_level_for_line(
    line: &str,
    markdown_heading: bool,
    caps: &regex::Captures,
) -> usize {
    if markdown_heading {
        return line
            .trim_start()
            .chars()
            .take_while(|ch| *ch == '#')
            .count()
            .max(1);
    }
    // Code-form declarations (§DF-code-declarations-drop-hash) match the branch
    // that has no `#+`, so no heading group is set; default to depth 1.
    caps.name("hashes")
        .or_else(|| caps.name("mdhashes"))
        .map(|m| m.as_str().len())
        .unwrap_or(1)
}

/// A file that could not be read or decoded during the walk. The walk continues
/// past it (§FS-check.2.4); callers that are point queries treat any entry here as
/// fatal, `check` and `refs` report it and exit 2 with a still-printed report.
pub(crate) type ScanError = (PathBuf, String);

type FileScanResult = (PathBuf, std::result::Result<Catalog, String>);

/// One file's pass on the text a save would write: the editor's unsaved text where
/// the file is open in one, the disk only where it is not
/// (§AR-system.2.13).
fn scan_one_file(
    file: &Path,
    schema: &Schema,
    frame: Frame<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
) -> FileScanResult {
    let mut findings = Catalog::default();
    // §FS-check.6.1.1: `sources` observes this input before its shared read.
    let result = match frame.sources().shared_text(file) {
        Ok(text) => scan_file_text(file, text, schema, frame, &mut findings, workspace_targets),
        Err(err) => Err(err.into()),
    };
    match result {
        Ok(()) => {
            findings.scanned_files.push(file.to_path_buf());
            (file.to_path_buf(), Ok(findings))
        }
        Err(err) => (file.to_path_buf(), Err(format!("{err:#}"))),
    }
}

/// One file's pass outside any walk (§AR-scanner.2), its overlay first: the
/// records of a file the walk did not reach, for a stub whose target lies
/// outside scan scope and still reads as the scanned file would (§FS-show.2.3.7),
/// its qualified citations recognized against the workspace's projects as the
/// walk's are (§FS-declarations.stubs.target.5).
pub(super) fn scan_unwalked_file(
    file: &Path,
    schema: &Schema,
    frame: Frame<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
) -> Result<Catalog> {
    let (_, result) = scan_one_file(file, schema, frame, workspace_targets);
    let mut findings = result.map_err(|message| anyhow!(message))?;
    // §FS-workspace.1.2: the two passes' sites, back in the single-project scan's order.
    if !workspace_targets.is_empty() {
        findings
            .citations
            .sort_by_key(|cite| (cite.line, cite.column));
    }
    Ok(findings)
}

fn scan_file_results(
    files: &[PathBuf],
    schema: &Schema,
    frame: Frame<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
) -> Vec<FileScanResult> {
    // §AR-bindings.3: scoped observation follows the shared parallel scan.
    let observer = crate::config::check_input_observer();
    files
        .par_iter()
        .map(|file| {
            crate::config::with_check_input_observer(observer.clone(), || {
                scan_one_file(file, schema, frame, workspace_targets)
            })
        })
        .collect::<Vec<_>>()
}

/// One full tree walk: scan every file (§AR-scanner.2) plus the e2e case
/// directories (§AR-scanner.6), collecting unreadable files rather than aborting
/// so `check` can report them and keep going (§FS-check.2.4). The wrapper around
/// the workspace-aware variant with no targets — single-project scans and
/// member-local scans share this path.
pub(crate) fn scan_tree(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<(Catalog, Vec<ScanError>)> {
    scan_tree_with_workspace(schema, frame, scope, explicit_scope, &[])
}

/// Workspace-aware tree walk: `§<alias>/<ID>` citations parse with each
/// target's grammar inline, so the workspace layer (§FS-workspace.1.2,
/// §AR-workspace.2) never needs to re-read the files the initial scan
/// already read.
pub(crate) fn scan_tree_with_workspace(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
    workspace_targets: &[WorkspaceCitationTarget],
) -> Result<(Catalog, Vec<ScanError>)> {
    scan_tree_with_workspace_threshold(
        schema,
        frame,
        scope,
        explicit_scope,
        workspace_targets,
        PARALLEL_SCAN_MIN_FILES,
    )
}

pub(crate) fn scan_tree_with_workspace_threshold(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
    workspace_targets: &[WorkspaceCitationTarget],
    parallel_min_files: usize,
) -> Result<(Catalog, Vec<ScanError>)> {
    // §FS-config.3.5: a link the walk could not resolve is already a scan failure
    // before a single file is opened — it joins the per-file ones (§FS-check.2.4).
    let walked = walk_scannable_files_reporting(schema, frame, scope, explicit_scope)?;
    // §FS-check.3.29.11: the walk's directories travel with its files, for the rule that
    // asks which of them holds a `[workspace]` block nothing claims. Carried, not
    // judged: the scanner never asks that question itself (§AR-workspace.1).
    let mut findings = Catalog {
        walked_dirs: walked.dirs,
        unread_roots: walked.unread_roots,
        ..Catalog::default()
    };
    let (mut files, mut errors) = (walked.files, walked.errors);
    add_overlay_scan_files(schema, frame, scope, explicit_scope, &mut files)?;
    // §FS-values.2.2: a value source is a declaration input, never a general text
    // scan input even when a repository adds `json` to `[scan].extensions`; a
    // YAML file those extensions claim is no value source.
    if schema.value_rows().next().is_some() {
        let home_values = value_sources(schema, frame)
            .map(|sources| {
                sources
                    .into_iter()
                    .map(|(path, _, _)| path)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        files.retain(|file| {
            !home_values
                .iter()
                .any(|source| paths_same_location(source, file))
        });
    }
    // §AR-scanner.2.8: each file's record is set aside as its findings merge, and the map
    // is built once from all of them, not by one keyed insert per file.
    let mut records = Vec::with_capacity(files.len());
    let mut merge = |findings: &mut Catalog, mut file_findings: Catalog| {
        records.extend(std::mem::take(&mut file_findings.files));
        merge_findings(findings, file_findings);
    };
    if files.len() >= parallel_min_files {
        for (file, result) in scan_file_results(&files, schema, frame, workspace_targets) {
            match result {
                Ok(file_findings) => merge(&mut findings, file_findings),
                Err(message) => errors.push((file, message)),
            }
        }
    } else {
        for file in files {
            match scan_one_file(&file, schema, frame, workspace_targets) {
                (_, Ok(file_findings)) => merge(&mut findings, file_findings),
                (_, Err(message)) => errors.push((file, message)),
            }
        }
    }
    findings.files = records.into_iter().collect();
    if let Err(err) = scan_e2e_cases(schema, frame, scope, explicit_scope, &mut findings) {
        errors.push((frame.root().join("e2e/cases"), format!("{err:#}")));
    }
    scan_value_sources(schema, frame, &mut findings, &mut errors);
    read_section_locks(frame, &mut findings, &mut errors);
    // §FS-workspace.1.2: when the citing-grammar pass and the target-grammar pass both
    // fire on the same line they emit in source order *per pass*; one sort at the end
    // keeps a workspace scan's per-line order the single-project scan's left-to-right one.
    if !workspace_targets.is_empty() {
        findings.citations.sort_by(|a, b| {
            (sort_path_key(&a.file), a.line, a.column).cmp(&(
                sort_path_key(&b.file),
                b.line,
                b.column,
            ))
        });
        findings.value_bindings.sort_by(|a, b| {
            (sort_path_key(&a.file), a.line, a.column).cmp(&(
                sort_path_key(&b.file),
                b.line,
                b.column,
            ))
        });
    }
    // §AR-scanner.2.6.6: shorthand citations name a declaration that may live in
    // any file, so they can only be resolved once the whole walk (including the
    // E2E cases above) has produced the declaration set.
    promote_local_legacy_citations(schema, frame, &mut findings);
    resolve_shorthand_citations(frame.grammar(), &mut findings);
    // §AR-scanner.4.6: a stub's verdict is reached only once every declaration is in.
    let (roots, _) = scan_roots(schema, frame, scope, explicit_scope)?;
    resolve_stubs(schema, frame, &roots, workspace_targets, &mut findings);
    Ok((findings, errors))
}

/// The buffers with no file behind them that the walk would read were they
/// saved, asked of the one read predicate (§AR-system.2.13,
/// §AR-scanner.1.13): every ignore file, `exclude` and unwalked home reaches
/// them as it reaches the walk.
fn add_overlay_scan_files(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    let buffer_only = frame.sources().buffer_only_files();
    if buffer_only.is_empty() {
        return Ok(());
    }
    let (roots, _) = scan_roots(schema, frame, scope, explicit_scope)?;
    for path in buffer_only {
        if unread(schema, frame, &roots, &path).is_some() {
            continue;
        }
        let path = normalize_path_lexically(&path);
        if files.iter().any(|file| paths_same_location(file, &path)) {
            continue;
        }
        files.push(path);
    }
    files.sort_by_key(|path| sort_path_key(path));
    Ok(())
}

pub(super) fn path_starts_with(path: &Path, root: &Path) -> bool {
    let path = canonicalize_existing_prefix(path);
    let root = canonicalize_existing_prefix(root);
    path == root || path.starts_with(root)
}

/// Scan helper for point-query subcommands (`show`, `id`): any unreadable file
/// is fatal — a partial view of the tree could miss the declaration entirely or
/// allocate a colliding number (§FS-show.3, §FS-id.4).
pub(crate) fn scan_tree_strict(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<Catalog> {
    let (findings, errors) = scan_tree(schema, frame, scope, explicit_scope)?;
    if let Some((path, message)) = errors.into_iter().next() {
        return Err(anyhow!("{}: {}", frame.display_path(&path), message));
    }
    Ok(findings)
}
