use anyhow::{Result, anyhow};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use super::e2e::scan_e2e_cases;
use super::file_pass::scan_file_text;
use super::legacy::promote_local_legacy_citations;
use super::merge::{drop_file_findings, merge_findings};
use super::read_predicate::unread;
pub(crate) use super::scan_error::ScanError;
use super::scan_error::{
    FailureRoots, ScanFailure, failure_place, name_file_failure, scan_error_reason,
};
use super::section_locks::read_section_locks;
use super::stub_homes::{resolve_stubs, resolve_stubs_through};
use super::value_json::{scan_value_sources, value_sources};
use super::walk::{scan_roots, walk_scannable_files_reporting};
use crate::config::{Frame, Schema};
use crate::grammar::resolve_shorthand_citations;
use crate::model::{Catalog, normalize_path_lexically, scanned_path_key, sort_path_key};
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

type FileScanResult = (PathBuf, std::result::Result<Catalog, String>);

/// One file's pass on the text a save would write: the editor's unsaved text where
/// the file is open in one, the disk only where it is not
/// (§AR-system.2.13), with the shared scan I/O reason (§FS-check.2.4.1).
fn scan_one_file(
    file: &Path,
    schema: &Schema,
    frame: Frame<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
) -> FileScanResult {
    let result = pass_over_file(file, schema, frame, workspace_targets);
    (
        file.to_path_buf(),
        result.map_err(|err| scan_error_reason(&err)),
    )
}

/// [`scan_one_file`]'s pass, its failure still holding the I/O error a caller outside
/// the walk classifies by (§FS-distribution.3.3.2).
fn pass_over_file(
    file: &Path,
    schema: &Schema,
    frame: Frame<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
) -> Result<Catalog> {
    let mut findings = Catalog::default();
    // §FS-check.6.1.1: `sources` observes this input before its shared read.
    let text = frame.sources().shared_text(file)?;
    scan_file_text(file, text, schema, frame, &mut findings, workspace_targets)?;
    findings.scanned_files.push(file.to_path_buf());
    Ok(findings)
}

/// One file's pass outside any walk (§AR-scanner.2), its overlay first: the
/// records of a file the walk did not reach, for a stub whose target lies
/// outside scan scope and still reads as the scanned file would (§FS-show.2.3.7),
/// its qualified citations recognized against the workspace's projects as the
/// walk's are (§FS-declarations.stubs.target.5). Fetch asks it of the files its
/// own walk of a home does not replace (§FS-fetch.5.2).
pub(crate) fn scan_unwalked_file(
    file: &Path,
    schema: &Schema,
    frame: Frame<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
) -> Result<Catalog> {
    // §AR-scanner.1.8: no walk met it, and the rule still reads it.
    frame.run.spellings.resolve(file);
    let mut findings = pass_over_file(file, schema, frame, workspace_targets)?;
    // §FS-workspace.1.2: the two passes' sites, back in the single-project scan's order.
    if !workspace_targets.is_empty() {
        findings
            .citations
            .sort_by_key(|cite| (cite.line, cite.column));
    }
    Ok(findings)
}

/// The tree `before` read, with `file` read again by the walk's own pass on the text the frame
/// holds for it (§AR-system.2.8.1): every other file's records are the ones `before` holds, so
/// nothing is walked and no file but `file`, or a target only a stub in it names, is read. The
/// stubs reading `file` again can move are judged again (§AR-scanner.4.6). Fetch asks it of the
/// candidate it would install, held in front of the disk. A `file` the walk did not read and
/// would not read is not read here either, as `scan_tree` skips a buffer the walk would not
/// read, so the reading holds no record of it (§AR-scanner.1.13). `workspace_targets` are the ones `before`
/// was scanned with (§FS-workspace.1.2).
pub(crate) fn scan_file_again(
    before: &Catalog,
    file: &Path,
    schema: &Schema,
    frame: Frame<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
) -> Result<Catalog> {
    let mut after = before.clone();
    drop_file_findings(&mut after, file);
    let (roots, _) = scan_roots(schema, frame, None, false)?;
    let key = scanned_path_key(file);
    let walked = before
        .scanned_files
        .iter()
        .any(|path| scanned_path_key(path) == key);
    if walked || unread(schema, frame, &roots, file).is_none() {
        let read = scan_unwalked_file(file, schema, frame, workspace_targets)?;
        merge_findings(&mut after, read);
    }
    resolve_stubs_through(schema, frame, &roots, workspace_targets, file, &mut after);
    Ok(after)
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
        outside_root: walked.outside_root,
        reading_conflicts: walked.reading_conflicts,
        reading_moves: walked.reading_moves,
        failure_names: walked.failure_names,
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
                // §AR-scanner.1.8: a file the rule reads as no value source is
                // the text scan's still, whichever name a value home took it by.
                sources
                    .into_iter()
                    .filter(|(_, _, format)| format.is_some())
                    .map(|(path, _, _)| path)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        files.retain(|file| {
            !home_values
                .iter()
                .any(|source| paths_same_location(source, file))
        });
        // §FS-values.2.2: a text-named link to a value source is the value reader's, which never
        // read it by that name, so nothing moved.
        findings
            .reading_moves
            .retain(|moved| files.contains(&moved.file));
    }
    // §AR-scanner.2.8: each file's record is set aside as its findings merge, and the map
    // is built once from all of them, not by one keyed insert per file.
    let mut records = Vec::with_capacity(files.len());
    let mut merge = |findings: &mut Catalog, mut file_findings: Catalog| {
        records.extend(std::mem::take(&mut file_findings.files));
        merge_findings(findings, file_findings);
    };
    // §FS-check.2.4.3: judged together once the pass is over, so a clean tree resolves no roots.
    let mut unread_files = Vec::new();
    if files.len() >= parallel_min_files {
        for (file, result) in scan_file_results(&files, schema, frame, workspace_targets) {
            match result {
                Ok(file_findings) => merge(&mut findings, file_findings),
                Err(message) => unread_files.push((file, message)),
            }
        }
    } else {
        for file in files {
            match scan_one_file(&file, schema, frame, workspace_targets) {
                (_, Ok(file_findings)) => merge(&mut findings, file_findings),
                (_, Err(message)) => unread_files.push((file, message)),
            }
        }
    }
    if !unread_files.is_empty() {
        let at = FailureRoots::resolve(schema, frame, scope, explicit_scope);
        for (file, reason) in unread_files {
            let place = failure_place(&file, &at);
            let failure = ScanFailure::Unread {
                place,
                version: frame.version,
            };
            errors.push(name_file_failure(
                ScanError::new(file, reason, failure),
                frame,
                &mut findings.failure_names,
            ));
        }
    }
    findings.files = records.into_iter().collect();
    // §AR-scanner.1.relisted: the E2E pass reads the cases the walk met, and both passes read the
    // reports so far, the walk's among them, so one failure both meet is one line.
    // §AR-scanner.6.6: each case failure at its path; two that read alike stay two.
    scan_e2e_cases(
        schema,
        frame,
        scope,
        explicit_scope,
        frame.run.scope.resolution_wide,
        &walked.e2e_cases,
        &mut errors,
        &mut findings,
    );
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
        // §FS-scan.identity.buffers: no walk met it, and the rule reads it as its file.
        frame.run.spellings.resolve(&path);
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
/// allocate a colliding number (§FS-show.3, §FS-id.4). The failure names its path
/// as a report does, the report base `.` (§FS-cli.3.4). It is the one at the path the
/// run was handed where there is one, before a directory that sorts ahead of it, as
/// `show` stops at it (§AR-scanner.1.6).
pub(crate) fn scan_tree_strict(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<Catalog> {
    let (findings, errors) = scan_tree(schema, frame, scope, explicit_scope)?;
    let stop = errors.iter().find(|error| error.at_passed_path());
    if let Some(error) = stop.or(errors.first()) {
        return Err(anyhow!(
            "{}: {}",
            frame.display_location(&error.path),
            error.reason
        ));
    }
    Ok(findings)
}
