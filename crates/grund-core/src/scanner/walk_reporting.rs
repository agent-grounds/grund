//! Reporting traversal and its source-retaining adapter (§AR-scanner.1,
//! §FS-distribution.3.3.2). Walk setup and filtering remain in `walk.rs`.

use anyhow::{Result, anyhow};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::read_predicate::{Unread, extension_refusal, name_refusal};
use super::scan_error::FailureRoots;
use super::tree::ScanError;
use super::value_json::value_sources;
use super::walk::{MetCases, WalkDirFilter, scan_roots, scannable_walker};
use super::walk_boundaries::{PlacedRoot, is_scannable, owned_by_another_project, place_root};
use super::walk_errors::{
    LinkRecovery, root_lookup_failure, symlink_loop_report, walk_error_reports,
};
use super::walk_failures::{sort_failures, whole_run_failures};
use super::walk_names::settle_names;
use super::walk_order::sort_paths;
use crate::config::{Frame, Schema, canonical_config_root, file_root_rows};
use crate::model::{FailureNames, ReadingConflict, ReadingMove, Refused, RootRow};
use crate::sources::{
    is_directory, is_regular_file, path_is_file, paths_same_location, physical_path_key,
};

/// The tree walk for the callers that ask a yes/no question about the tree and
/// nothing else — today the `--cross-refs` auto-enable probe, which wants to know
/// whether the scope holds any Markdown (§FS-fmt.6.6). Every caller that *reports*
/// takes `walk_scannable_files_reporting`, so an unresolvable link reaches the
/// report rather than being dropped here (§FS-config.3.5.5, §FS-check.2.4): this one
/// is walking a tree that the reporting walk is about to walk again and account
/// for, so repeating its errors would print each of them twice.
///
/// The tree walk: which roots a scan starts from, which files it yields, and what
/// it does with a path it cannot read (§AR-scanner.1). It sits beside
/// `file_pass.rs` rather than inside it because the two are different machines —
/// that one is a line-by-line pass over a file's text, this one is a directory
/// traversal — and they meet only at the file list one hands the other.
pub(crate) fn walk_scannable_files(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<Vec<PathBuf>> {
    Ok(walk_scannable_files_reporting(schema, frame, scope, explicit_scope)?.files)
}

/// What one walk of the tree produced (§AR-scanner.1).
pub(crate) struct WalkedTree {
    /// The scannable files, sorted, one entry per physical file (§FS-errors.4.1).
    pub(crate) files: Vec<PathBuf>,
    /// The paths the walk could not read, for the caller to report (§FS-check.2.4).
    pub(crate) errors: Vec<ScanError>,
    /// The other names each of those failures was reported under (§AR-scanner.1.entry).
    pub(crate) failure_names: FailureNames,
    /// The files that are in this tree only by a link: their physical path is
    /// outside the config root. Read like any other file (§FS-config.3.5.1) —
    /// `fmt --write` is the caller that treats them differently, because
    /// rewriting one edits a file the project does not own (§FS-fmt.2.3.2).
    pub(crate) outside_root: BTreeSet<PathBuf>,
    /// Every directory the walk descended into, scan roots included, sorted and
    /// deduplicated (§FS-errors.4.1). Carried out rather than asked about here: the
    /// scanner never asks "am I in a workspace?" (§AR-workspace.1), and the one
    /// caller of this list — the unlisted-`[workspace]` rule of §FS-check.3.29 —
    /// probes each directory for a config and answers the claim above the walk.
    pub(crate) dirs: Vec<PathBuf>,
    /// Whether the scope the walk resolved is the whole project, as `scan_roots`
    /// answered it: only such a `fmt --write` writes a section lock
    /// (§FS-section-lock.write).
    pub(crate) root_scope: bool,
    /// The configured file roots a name rule refused, each with the row that made it
    /// a root and the rules that refused it, for `check`'s report (§AR-scanner.1.6,
    /// §FS-scan.checks.unread-scan-root).
    pub(crate) unread_roots: Vec<RootRow>,
    /// The files read under neither name, gone from `files` (§FS-scan.checks.reading-conflict).
    pub(crate) reading_conflicts: Vec<ReadingConflict>,
    /// The links whose reading moved under the rule (§FS-scan.checks.reading-moved).
    pub(crate) reading_moves: Vec<ReadingMove>,
    /// The E2E case directories the walk met and pruned for the manifest pass, as the walk
    /// spelled them, sorted and deduplicated: the one listing the pass reads its cases from
    /// (§FS-declarations.e2e-case.excluded, §FS-check.2.4.2).
    pub(crate) e2e_cases: Vec<PathBuf>,
}

/// The tree walk (§AR-scanner.1): from each scan root, descend skipping hidden and
/// `[scan] exclude` directories, honouring `.gitignore` and friends unless
/// `respect_gitignore = false` (§AR-scanner.1.1, §FS-config.3.5.15), following
/// symlinks, keeping only scannable files, in a sorted order so findings are
/// deterministic (§FS-errors.4.1). Returns the paths it could not read beside the
/// files, for the caller to report (§FS-check.2.4).
///
/// Why `aliasable` is a list and not a flag: the identity pass resolves what is in
/// it and compares everything else by path, so one link in a repository costs one
/// `realpath` and not one per file.
///
/// Paths stay in-tree. Following a symlink does not change the entry's path, and
/// that spelling is what the directory filter below and every finding are expressed
/// in. The boundary filter can therefore compare precomputed suffixes: `strip_prefix`
/// only removes the root, so the descendant suffix is invariant under symlink
/// resolution — the compare works even if `scan_root` itself is a symlink.
///
/// Why the other-project test after resolution is cheap: the alias resolution is
/// already paid for, so the boundary costs a prefix test over the files that can
/// wear a second name, and nothing at all for a run that loaded no workspace.
///
/// Why the error list is sorted and compared here: the text report sorts before
/// printing while the API surface hands the list over as it stands, so one sort
/// serves both; and overlapping roots — plus `--full`, which walks every `include`
/// root beside the config root that contains it — meet the same broken link once per
/// root, and a directory link meets one entry under two names, so an uncompared list
/// prints each such error twice (§FS-check.2.4.2, §AR-scanner.1.entry).
///
/// Why `outside_root` compares physical paths on both sides: a config root reached
/// through a link contains none of the paths its own files resolve to, and
/// `fmt --write` would then refuse to rewrite the whole repository.
pub(crate) fn walk_scannable_files_reporting(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<WalkedTree> {
    walk_scannable_files_with_sources(schema, frame, scope, explicit_scope, &mut |_, _| {})
}

/// The same reporting walk, retaining each admitted error's original source
/// beside its unchanged scan record (§FS-distribution.3.3.2). The callback runs
/// before sorting/deduplication; callers match sources by the complete record.
pub(crate) fn walk_scannable_files_with_sources(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
    on_error: &mut dyn FnMut(&ScanError, ignore::Error),
) -> Result<WalkedTree> {
    let (roots, root_scope) = scan_roots(schema, frame, scope, explicit_scope)?;
    let widened = explicit_scope && !root_scope && frame.run.scope.resolution_wide;
    let mut handed = 0;
    let mut handed_failures = 0;
    // §FS-config.3.5.4: an aliased root, or a symlink met on the way down, is what
    // hands the same file to the walk under two spellings — nothing else does, so
    // a tree with neither never pays for the identity pass (§GOAL-fast-feedback).
    let mut aliasable = BTreeSet::new();
    let mut files = Vec::new();
    let mut errors = Vec::new();
    // §FS-check.3.29.11: the directories the walk met, for the rule that asks which of
    // them carries a `[workspace]` block nothing claims. Collected here because the
    // entries are already being enumerated — no second traversal (§GOAL-fast-feedback).
    let mut dirs = Vec::new();
    // Where this project physically is, for every comparison below that reads a
    // resolved path. Equal to `config.root` for the roots `grund` discovers, which
    // are canonical already (§FS-config.1) — one `stat` per run either way.
    let physical_root = canonical_config_root(frame.root());
    // §AR-scanner.1.6: the file roots a name rule refused, with their rows,
    // and the rows themselves, read only once a root is refused.
    let mut unread_roots = Vec::new();
    let mut file_rows = None;
    let mut value_homes: Option<Option<Vec<PathBuf>>> = None;
    // §FS-declarations.e2e-case.excluded: the case directories every root's walk met.
    let met_cases = MetCases::default();
    for (position, scan_root) in roots.iter().cloned().enumerate() {
        // §FS-check.1.3.6.1: a widened walk starts at the path, so what the list holds at the
        // second root is what the run was handed: a file, a directory's files, or nothing.
        if widened && position == 1 {
            handed = files.len();
            handed_failures = errors.len();
        }
        // §FS-check.6.1.1: the actual walk roots are covered before traversal.
        if !crate::config::observe_input(&scan_root, true) {
            return Err(anyhow!("watch input coverage failed"));
        }
        // §FS-scan.scope.homes.lookup: absent only when the lookup says so; the rest is reported
        // below, after the boundaries, so a root another scan owns stays that scan's.
        let Some(PlacedRoot {
            looked_up,
            canonical: canonical_scan_root,
            outward_link,
        }) = place_root(&scan_root, &physical_root, schema)
        else {
            continue;
        };
        // §FS-config.3.5.1: a scan root reached through a directory link uses
        // the same canonical project-root boundary as a link met during descent.

        // §AR-scanner.1.6: the path this walk was handed, unlooked-up, has no other reader, so
        // no member boundary or owner takes it (§FS-scan.scope.homes.lookup.path).
        let unread_passed = looked_up.is_err() && explicit_scope && !root_scope && position == 0;
        // §AR-workspace.6: a root scan starts outside member namespaces; an
        // included path at or below a member boundary belongs to the member scan,
        // and one in another project belongs there (§FS-workspace.6.2).
        if outward_link
            || (!unread_passed
                && (frame
                    .run
                    .workspace
                    .boundary_roots
                    .iter()
                    .any(|root| canonical_scan_root.starts_with(root))
                    || owned_by_another_project(
                        &frame.run.workspace.project_roots,
                        &physical_root,
                        &canonical_scan_root,
                    )))
        {
            continue;
        }
        if let Err(error) = looked_up {
            let (report, source) = root_lookup_failure(&scan_root, error, frame.version);
            on_error(&report, source);
            errors.push(report);
            continue;
        }
        // A root that resolves elsewhere reaches every one of its files under a
        // spelling that is not the file's own, so all of them can alias (§FS-check.1.3.2).
        let root_is_aliased = canonical_scan_root != scan_root;
        if path_is_file(&scan_root) {
            match name_refusal(&scan_root, schema) {
                None => {
                    // §FS-check.1.3.6.1: a file handed as the path is walked beside roots
                    // that reach it under its own name, so a link's spelling has to
                    // resolve to the same one read (§FS-config.3.5.4).
                    if root_is_aliased {
                        aliasable.insert(scan_root.clone());
                    }
                    files.push(scan_root);
                }
                // §AR-scanner.1.13: the predicate's own answer, kept with the row
                // that made the file a root (§FS-scan.checks.unread-scan-root.location).
                Some(refusal) => {
                    // §AR-scanner.1.6: the hidden rule is asked first, so a hidden root
                    // also asks the one extension test, and the record names both.
                    let hidden = refusal == Unread::HiddenName;
                    let refused = Refused {
                        hidden,
                        extension: !hidden || extension_refusal(&scan_root, schema).is_some(),
                    };
                    let rows =
                        file_rows.get_or_insert_with(|| file_root_rows(schema, frame.root()));
                    // §FS-scan.checks.unread-scan-root.silent: a value row's value-source
                    // home is read by the value pass; when that pass cannot list them,
                    // record none.
                    let value_homes = value_homes.get_or_insert_with(|| {
                        value_sources(schema, frame)
                            .ok()
                            .map(|sources| sources.into_iter().map(|source| source.0).collect())
                    });
                    if let Some(value_homes) = value_homes
                        && !value_homes
                            .iter()
                            .any(|home| paths_same_location(home, &scan_root))
                    {
                        record_unread_root(&mut unread_roots, rows, &scan_root, refused);
                    }
                }
            }
            continue;
        }
        // The directory links the filter met, shared with the loop below: a file
        // under one of them is reached under a spelling that is not its own, the
        // same as a file that is a link itself (§AR-scanner.1.8).
        let link_roots = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        // §FS-config.3.5.5: the directory links the filter pruned as loops, for the
        // report to be raised from without the descent the walker would need to
        // notice them (§AR-scanner.1.9).
        let looping_links = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let walker = scannable_walker(
            schema,
            frame,
            &scan_root,
            &canonical_scan_root,
            &physical_root,
            &link_roots,
            &looping_links,
            Some(&met_cases),
        )?;
        let mut root_files = Vec::new();
        let mut root_errors = Vec::new();
        // §AR-scanner.1.9: the last of this root's `dirs` is the directory the walk is inside when
        // an error names no entry. The walk's own filter and the once-per-directory recovery are
        // both built on the first error, so the success path pays for neither.
        let first_dir = dirs.len();
        let mut error_state = None;
        let new_error_state = || {
            let filter = WalkDirFilter::new(
                schema,
                frame,
                &scan_root,
                &canonical_scan_root,
                &physical_root,
                &link_roots,
                &looping_links,
            );
            (filter, LinkRecovery::default())
        };
        let mut report_walk_error = |err, listing: Option<&Path>| {
            let (filter, recovery) = error_state.get_or_insert_with(new_error_state);
            for (report, source) in
                walk_error_reports(err, filter, recovery, listing, schema, frame, &scan_root)
            {
                on_error(&report, source);
                root_errors.push(report);
            }
        };
        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
                // §FS-config.3.5.5: a link the walk cannot resolve is a file the scan
                // cannot read — reported at its own path, the walk continuing past it
                // (§FS-check.2.4). Failing the scan would let it take the whole report.
                Err(err) => {
                    report_walk_error(err, dirs[first_dir..].last().map(PathBuf::as_path));
                    continue;
                }
            };
            // §FS-check.2.4.2: a directory's unparsable ignore file rides on its entry, and the
            // directory is still walked (§AR-scanner.1.9).
            if let Some(err) = entry.error() {
                report_walk_error(err.clone(), dirs[first_dir..].last().map(PathBuf::as_path));
            }
            // §FS-check.3.29.11: a directory is not a scannable file, so it falls out
            // one line below. Its path is what the unlisted-`[workspace]` rule needs,
            // and the scan root itself — the entry at depth 0 — is one of them.
            if entry.file_type().is_some_and(is_directory) {
                dirs.push(entry.path().to_path_buf());
            }
            if !entry
                .file_type()
                .is_some_and(|file_type| is_regular_file(file_type))
                || !is_scannable(entry.path(), schema)
            {
                continue;
            }
            if root_is_aliased || entry.path_is_symlink() || under_link(&link_roots, entry.path()) {
                aliasable.insert(entry.path().to_path_buf());
            }
            root_files.push(entry.path().to_path_buf());
        }
        // §FS-config.3.5.5: the loops the filter pruned. The link is owed the same
        // report a loop the walker found earns, and by the same gates — it is the
        // descent, not the report, that pruning removes.
        let pruned_loops = std::mem::take(
            &mut *looping_links
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        if !pruned_loops.is_empty() {
            let (filter, _) = error_state.get_or_insert_with(new_error_state);
            for (link, target) in pruned_loops {
                root_errors.extend(symlink_loop_report(&link, &target, filter, schema, frame));
            }
        }
        // §FS-errors.4.1: within one root the order is the filesystem's, and the
        // first-seen rule below turns that into a choice of *spelling*. Sorting each
        // root's list first makes the choice ours: earlier root wins, then lexicographic.
        sort_paths(&mut root_files);
        files.append(&mut root_files);
        // §AR-scanner.1.entry: the failures' name is chosen as the files' is, so each root sorts
        // its own and the joined list stays in root order until the entry step has run.
        sort_failures(&mut root_errors);
        errors.append(&mut root_errors);
    }
    // One file, one read (§FS-check.1.3.2, §FS-config.3.5.4). Two spellings first,
    // while the list is still in walk order and first-seen wins; then the
    // byte-identical ones, which the sort has just brought together.
    let resolved = resolve_aliasable(&aliasable);
    // §FS-workspace.6.2: the directory filter stops a *directory* link at another
    // project's root, and a link straight onto one of its files is the same
    // crossing one entry lower down.
    if !frame.run.workspace.project_roots.is_empty() {
        let kept = |file: &PathBuf| {
            !resolved.get(file.as_path()).is_some_and(|physical| {
                owned_by_another_project(
                    &frame.run.workspace.project_roots,
                    &physical_root,
                    physical,
                )
            })
        };
        // What was handed is counted again, since the count slices this list below.
        let rest = files.split_off(handed);
        files.retain(&kept);
        handed = files.len();
        files.extend(rest.into_iter().filter(&kept));
    }
    // §AR-scanner.1.8: while the list is in walk order, so the first name is first-seen's.
    let whole_project = !explicit_scope || root_scope || frame.run.scope.resolution_wide;
    let verdicts = settle_names(schema, frame, &files, handed, &resolved, whole_project)?;
    if !resolved.is_empty() {
        dedup_by_file_identity(&mut files, &resolved);
    }
    // §FS-scan.checks.reading-conflict: read under neither name.
    if !verdicts.conflicts.is_empty() {
        files.retain(|file| !frame.run.spellings.is_refused(file));
    }
    sort_paths(&mut files);
    // The roots may overlap — `include = ["docs", "docs/api"]` names one subtree
    // twice, and under `--full` every `include` root is walked beside the config root
    // containing it (§FS-check.1.3.2). A file read twice duplicates its own declaration.
    files.dedup();
    // §FS-check.2.4.2: choose ordinary-root names while keeping encounter-time classification.
    let failure_names = if errors.is_empty() {
        FailureNames::default()
    } else {
        let at = FailureRoots::new(roots, root_scope, explicit_scope, frame.root());
        whole_run_failures(&mut errors, handed_failures, &at)
    };
    sort_failures(&mut errors);
    // §FS-fmt.2.3.2: a file whose physical path is not under the config root is in
    // this tree only by the link that reaches it. The resolution is already paid
    // for above, so this is a prefix test over the links and nothing more.
    let outside_root = files
        .iter()
        .filter(|file| {
            resolved
                .get(file.as_path())
                .is_some_and(|physical| !physical.starts_with(&physical_root))
        })
        .cloned()
        .collect();
    // §FS-errors.4.1: overlapping roots — and `--full`, which walks every `include`
    // root beside the config root containing it — meet the same directory once per
    // root, so one sort and one dedup make the candidate list a set.
    sort_paths(&mut dirs);
    dirs.dedup();
    let mut e2e_cases = std::mem::take(
        &mut *met_cases
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    );
    sort_paths(&mut e2e_cases);
    e2e_cases.dedup();
    Ok(WalkedTree {
        files,
        errors,
        failure_names,
        outside_root,
        dirs,
        root_scope,
        unread_roots,
        reading_conflicts: verdicts.conflicts,
        reading_moves: verdicts.moves,
        e2e_cases,
    })
}

/// Record the row that made `scan_root` a root, once however many roots spell it:
/// under version 2 a `files` entry is an `include` root and a home at once
/// (§FS-scan.checks.unread-scan-root.location). A root no configured key names, a
/// path typed on the command line, has no row to report at. The record carries the
/// rules that refused the root, so one root is one record (§DF-unread-hidden-root.2.1).
fn record_unread_root(
    unread: &mut Vec<RootRow>,
    rows: &[(PathBuf, RootRow)],
    scan_root: &Path,
    refused: Refused,
) {
    let Some((_, row)) = rows.iter().find(|(path, _)| path == scan_root) else {
        return;
    };
    let row = RootRow {
        refused,
        ..row.clone()
    };
    if !unread.contains(&row) {
        unread.push(row);
    }
}

/// Whether the walk reached this path *through* one of the directory links it
/// recorded — which makes the file's spelling not its own, exactly as being a
/// link itself would (§AR-scanner.1.8).
fn under_link(link_roots: &std::sync::Mutex<Vec<PathBuf>>, path: &Path) -> bool {
    link_roots
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .any(|root| path.starts_with(root))
}

/// Where each file that can wear a second name physically is: the in-tree path it
/// was walked under, mapped to the path `canonicalize` resolves it to.
type AliasTargets = std::collections::HashMap<PathBuf, PathBuf>;

/// Resolve the files the walk saw arrive under a spelling that is not their own —
/// a link, a file below a directory link, or any file of an aliased root. This is
/// the only `canonicalize` the walk spends: everything else is answered by
/// comparing a path against what these resolved to (§AR-scanner.1.8,
/// §GOAL-fast-feedback).
fn resolve_aliasable(aliasable: &BTreeSet<PathBuf>) -> AliasTargets {
    aliasable
        .iter()
        .map(|file| (file.clone(), physical_path_key(file)))
        .collect()
}

/// Collapse the files reached under two spellings, keeping the **first**
/// (§FS-check.1.3.2, §FS-config.3.5.4). `root_scope_roots` walks the `include` roots
/// before the config root `--full` adds, so the surviving spelling is the one the
/// plain run reports, and `--full` stays purely additive: it appends out-of-scope
/// lines and never restates an in-scope one under a second name. Within a single
/// root the caller has already sorted, so "first" there is the lexicographically
/// first path rather than whatever readdir happened to say (§FS-errors.4.1).
///
/// `resolved` covers the files that can wear a second name, and the paths they
/// resolve to are the only ones another file can turn out to be — so every other
/// file is answered by a lookup in that small target set and is never resolved at
/// all. A repository with one symlink pays one `realpath` and not one per file,
/// which is what a flag saying "this tree has a link in it" could not do
/// (§GOAL-fast-feedback, §AR-scanner.1.8).
fn dedup_by_file_identity(files: &mut Vec<PathBuf>, resolved: &AliasTargets) {
    let targets: std::collections::HashSet<&Path> =
        resolved.values().map(PathBuf::as_path).collect();
    let mut seen = BTreeSet::new();
    files.retain(|file| {
        let key = resolved
            .get(file.as_path())
            .map_or(file.as_path(), PathBuf::as_path);
        !targets.contains(key) || seen.insert(key.to_path_buf())
    });
}
