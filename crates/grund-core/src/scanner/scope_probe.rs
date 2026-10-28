//! What a walk would read, asked without scanning (§AR-scanner.1.13): whether a
//! root holds one file the scan reads, which file that is, the `exclude`
//! pattern that kept a root from reading any, and the project configs below a
//! root no config applies to. Each walks with the reporting walk's own builder
//! and filter, so a probe and the scan cannot disagree.

use std::path::{Path, PathBuf};

use super::exclude_globs::ExcludeGlobs;
use super::walk::{scan_roots_for, scannable_walk_builder, scannable_walker};
use super::walk_boundaries::{is_scannable, outward_directory_link_root, owned_by_another_project};
use crate::config::{Frame, Schema, canonical_config_root, config_file_in, root_scope_roots};
use crate::model::sort_path_key;
use crate::sources::{canonical_path, is_directory, is_regular_file, path_exists, path_is_file};

/// §FS-check.2.2.2.5.2: how many levels below the root the config search asks.
const CONFIG_SEARCH_DEPTH: usize = 3;

/// Whether the effective configured scope contains a file the scanner would read
/// (§FS-config.3.5, §FS-init.2.2.2). Root selection stays beside the walk so init
/// cannot grow a second, path-existence approximation of scanner policy. Both
/// levels use lazy `any`, stopping at the first readable file in the first root
/// that contains one (§GOAL-fast-feedback).
pub(crate) fn effective_scope_reads_any_file(schema: &Schema, frame: Frame<'_>) -> bool {
    effective_scope_reads_any_file_with(schema, frame, |root| {
        walk_reads_any_file(schema, frame, root)
    })
}

/// Apply the effective root order lazily, separated only so the short-circuit
/// itself can be pinned without replacing scanner behavior (§FS-config.3.5).
pub(crate) fn effective_scope_reads_any_file_with(
    schema: &Schema,
    frame: Frame<'_>,
    mut root_reads_any_file: impl FnMut(&Path) -> bool,
) -> bool {
    root_scope_roots(schema, frame.root(), frame.run.scope.full)
        .iter()
        .any(|root| root_reads_any_file(root))
}

/// §FS-check.4.10.2: whether a walk of `scan_root` under this config would read a
/// file — the same builder, the same filter and the same `is_scannable` test the
/// reporting walk applies, stopped at the first hit.
///
/// The question a `[workspace]` block that opted out of being a project is asked
/// about its own tree, and it is asked of the scanner rather than of the spec text
/// so the two cannot drift apart. `Walk` is a lazy iterator, so `any` *is* the
/// early exit: the cost is "is there one file here", not the size of the tree, and
/// only a block that opted out ever pays it (§GOAL-fast-feedback).
///
/// Every root is **gated first**, exactly as the reporting walk gates it: an
/// outward directory link crosses the canonical project-root fence
/// (§FS-config.3.5.1), while a canonical root inside one of this config's
/// boundary roots, or one another project of the run owns, is that project's
/// tree rather than this block's (§FS-workspace.6). The gate has to be applied
/// to the root by hand, because the filter below is never asked about it — a walk
/// root is never pruned at depth zero (§FS-config.3.5.9), and a file root reaches
/// no filter at all.
///
/// A root that is not a directory takes `is_scannable` directly, which is the same
/// answer the walk above gives a file root — including that a hidden *file* is
/// skipped even as a root, while a walk root is never pruned by `exclude`, an
/// ignore file, or the hidden-directory rule (§FS-config.3.5.12).
///
/// A path the walk cannot read is not an error here the way it is in
/// [`walk_scannable_files_reporting`]: no run is scanning this tree, so there is no
/// report to raise it into, and the question — would a project have read something
/// — is answered by the files that can be read.
pub(crate) fn walk_reads_any_file(schema: &Schema, frame: Frame<'_>, scan_root: &Path) -> bool {
    first_read_file(schema, frame, scan_root).is_some()
}

/// The first file a walk of `scan_root` reads, in walk order: `walk_reads_any_file`'s
/// hit, named.
fn first_read_file(schema: &Schema, frame: Frame<'_>, scan_root: &Path) -> Option<PathBuf> {
    if !path_exists(&scan_root) {
        return None;
    }
    let canonical_scan_root = canonical_path(scan_root).unwrap_or_else(|_| scan_root.to_path_buf());
    let physical_root = canonical_config_root(frame.root());
    // §FS-config.3.5.1, §FS-workspace.6: the reporting walk's own scan-root
    // gates, ahead of the branch below, because neither a file root nor a walk
    // root ever reaches the filter.
    if outward_directory_link_root(scan_root, &canonical_scan_root, &physical_root)
        || frame
            .run
            .workspace
            .boundary_roots
            .iter()
            .any(|root| canonical_scan_root.starts_with(root))
        || owned_by_another_project(
            &frame.run.workspace.project_roots,
            &physical_root,
            &canonical_scan_root,
        )
    {
        return None;
    }
    if path_is_file(&scan_root) {
        return is_scannable(scan_root, schema).then(|| scan_root.to_path_buf());
    }
    let link_roots = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let looping_links = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let Ok(walker) = scannable_walker(
        schema,
        frame,
        scan_root,
        &canonical_scan_root,
        &physical_root,
        &link_roots,
        &looping_links,
    ) else {
        return None;
    };
    walker
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_type()
                .is_some_and(|file_type| is_regular_file(file_type))
                && is_scannable(entry.path(), schema)
        })
        .map(ignore::DirEntry::into_path)
}

/// §FS-check.2.2.2.2: the `[schema.sources] exclude` pattern that kept a walk of
/// the handed `path` from reading the first file it would read were the list empty,
/// met on the way down from the root the path resolves to. `None` where no pattern
/// did, under v1, and where the path reads nothing either way. Asked only of a scan
/// that read nothing, so no run that read a file walks twice (§GOAL-fast-feedback).
pub(crate) fn pruning_pattern(schema: &Schema, frame: Frame<'_>, path: &Path) -> Option<String> {
    let globs = ExcludeGlobs::of(schema)?;
    let roots = scan_roots_for(schema, frame, Some(path), true, false, false).ok()?;
    let mut unexcluded = schema.clone();
    unexcluded.sources.model.as_mut()?.exclude.clear();
    roots.iter().find_map(|root| {
        let file = first_read_file(&unexcluded, frame, root)?;
        globs
            .excluding_below(frame.root(), root, &file)
            .map(str::to_string)
    })
}

/// §FS-check.2.2.2.5.2: the project configs below `frame`'s root, each as the
/// directory it governs and the file discovery would load there
/// ([`config_file_in`], §FS-config.1), in path order. The directories asked are
/// the ones the scan's own walker enters — never a hidden one, one `[scan]
/// exclude` names, or one an ignore file excludes — at most
/// [`CONFIG_SEARCH_DEPTH`] levels down, and none below a directory that holds a
/// config, since what lies there is that project's own.
///
/// It answers for the empty-scan warning and the invalid-ID hint of a run that
/// loaded no config (§FS-check.2.2.2.5, §FS-show.3.5.1); they ask only on those
/// failure paths, so no other run pays for the walk (§GOAL-fast-feedback). The
/// walk still lists a found directory's first levels; it only never asks them.
pub(crate) fn project_configs_below(schema: &Schema, frame: Frame<'_>) -> Vec<(PathBuf, PathBuf)> {
    let root = frame.root();
    let canonical_root = canonical_path(root).unwrap_or_else(|_| root.to_path_buf());
    let link_roots = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let looping_links = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let Ok(mut builder) = scannable_walk_builder(
        schema,
        frame,
        root,
        &canonical_root,
        &canonical_config_root(root),
        &link_roots,
        &looping_links,
    ) else {
        return Vec::new();
    };
    let mut found: Vec<(PathBuf, PathBuf)> = Vec::new();
    for entry in builder
        .max_depth(Some(CONFIG_SEARCH_DEPTH))
        .build()
        .filter_map(Result::ok)
    {
        let dir = entry.path();
        // The walk meets a directory before anything below it, so the last config
        // found is the only one a later directory can lie under.
        if entry.depth() == 0
            || !entry.file_type().is_some_and(is_directory)
            || found
                .last()
                .is_some_and(|(above, _)| dir.starts_with(above))
        {
            continue;
        }
        if let Some(config) = config_file_in(dir) {
            found.push((dir.to_path_buf(), config));
        }
    }
    found.sort_by_cached_key(|(dir, _)| sort_path_key(dir));
    found
}
