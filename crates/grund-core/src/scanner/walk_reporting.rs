//! Reporting traversal and its source-retaining adapter (§AR-scanner.1,
//! §FS-distribution.3.3.2). Walk setup and filtering remain in `walk.rs`.

use anyhow::Result;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::tree::ScanError;
use super::walk::{scan_roots, scannable_walker};
use super::walk_boundaries::{is_scannable, outward_directory_link_root, owned_by_another_project};
use super::walk_errors::{symlink_loop_report, walk_error_report};
use crate::config::{Config, canonical_config_root};
use crate::model::{physical_path_key, sort_path_key};

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
    config: &Config,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<Vec<PathBuf>> {
    Ok(walk_scannable_files_reporting(config, scope, explicit_scope)?.files)
}

/// What one walk of the tree produced (§AR-scanner.1).
pub(crate) struct WalkedTree {
    /// The scannable files, sorted, one entry per physical file (§FS-errors.4.1).
    pub(crate) files: Vec<PathBuf>,
    /// The paths the walk could not read, for the caller to report (§FS-check.2.4).
    pub(crate) errors: Vec<ScanError>,
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
/// Why the error list is sorted and deduplicated here: the text report sorts before
/// printing while the API surface hands the list over as it stands, so one sort
/// serves both; and overlapping roots — plus `--full`, which walks every `include`
/// root beside the config root that contains it — meet the same broken link once per
/// root, so an undeduplicated list prints every such error twice.
///
/// Why `outside_root` compares physical paths on both sides: a config root reached
/// through a link contains none of the paths its own files resolve to, and
/// `fmt --write` would then refuse to rewrite the whole repository.
pub(crate) fn walk_scannable_files_reporting(
    config: &Config,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<WalkedTree> {
    walk_scannable_files_with_sources(config, scope, explicit_scope, &mut |_, _| {})
}

/// The same reporting walk, retaining each admitted error's original source
/// beside its unchanged scan record (§FS-distribution.3.3.2). The callback runs
/// before sorting/deduplication; callers match sources by the complete record.
pub(crate) fn walk_scannable_files_with_sources(
    config: &Config,
    scope: Option<&Path>,
    explicit_scope: bool,
    on_error: &mut dyn FnMut(&ScanError, ignore::Error),
) -> Result<WalkedTree> {
    let roots = scan_roots(config, scope, explicit_scope)?;
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
    let physical_root = canonical_config_root(config);
    for scan_root in roots {
        if !scan_root.exists() {
            continue;
        }
        let canonical_scan_root =
            fs::canonicalize(&scan_root).unwrap_or_else(|_| scan_root.to_path_buf());
        // §FS-config.3.5.1: a scan root reached through a directory link uses
        // the same canonical project-root boundary as a link met during descent.

        // §AR-workspace.6: a root scan starts outside member namespaces; an
        // included path at or below a member boundary belongs to the member scan,
        // and one in another project belongs there (§FS-workspace.6.2).
        if outward_directory_link_root(&scan_root, &canonical_scan_root, &physical_root)
            || config
                .workspace_boundary_roots
                .iter()
                .any(|root| canonical_scan_root.starts_with(root))
            || owned_by_another_project(config, &physical_root, &canonical_scan_root)
        {
            continue;
        }
        // A root that resolves elsewhere reaches every one of its files under a
        // spelling that is not the file's own, so all of them can alias (§FS-check.1.3.2).
        let root_is_aliased = canonical_scan_root != scan_root;
        if scan_root.is_file() {
            if is_scannable(&scan_root, config) {
                // §FS-check.1.3.6.1: a file handed as the path is walked beside roots
                // that reach it under its own name, so a link's spelling has to
                // resolve to the same one read (§FS-config.3.5.4).
                if root_is_aliased {
                    aliasable.insert(scan_root.clone());
                }
                files.push(scan_root);
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
            config,
            &scan_root,
            &canonical_scan_root,
            &physical_root,
            &link_roots,
            &looping_links,
        );
        let mut root_files = Vec::new();
        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
                // §FS-config.3.5.5: a link the walk cannot resolve is a file the scan
                // cannot read — reported at its own path, the walk continuing past it
                // (§FS-check.2.4). Failing the scan would let it take the whole report.
                Err(err) => {
                    if let Some(report) = walk_error_report(&err, config, &scan_root) {
                        on_error(&report, err);
                        errors.push(report);
                    }
                    continue;
                }
            };
            // §FS-check.3.29.11: a directory is not a scannable file, so it falls out
            // one line below. Its path is what the unlisted-`[workspace]` rule needs,
            // and the scan root itself — the entry at depth 0 — is one of them.
            if entry
                .file_type()
                .is_some_and(|file_type| file_type.is_dir())
            {
                dirs.push(entry.path().to_path_buf());
            }
            if !entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
                || !is_scannable(entry.path(), config)
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
        for (link, target) in looping_links
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .drain(..)
        {
            // The in-tree name of the directory the link reaches back into, or
            // nothing where the target reaches over the walk root.
            let ancestor = target
                .strip_prefix(&canonical_scan_root)
                .ok()
                .map(|rest| scan_root.join(rest));
            errors.extend(symlink_loop_report(&link, ancestor.as_deref(), config));
        }
        // §FS-errors.4.1: within one root the order is the filesystem's, and the
        // first-seen rule below turns that into a choice of *spelling*. Sorting each
        // root's list first makes the choice ours: earlier root wins, then lexicographic.
        root_files.sort_by_key(|path| sort_path_key(path));
        files.append(&mut root_files);
    }
    // One file, one read (§FS-check.1.3.2, §FS-config.3.5.4). Two spellings first,
    // while the list is still in walk order and first-seen wins; then the
    // byte-identical ones, which the sort has just brought together.
    let resolved = resolve_aliasable(&aliasable);
    // §FS-workspace.6.2: the directory filter stops a *directory* link at another
    // project's root, and a link straight onto one of its files is the same
    // crossing one entry lower down.
    if !config.workspace_project_roots.is_empty() {
        files.retain(|file| {
            !resolved
                .get(file.as_path())
                .is_some_and(|physical| owned_by_another_project(config, &physical_root, physical))
        });
    }
    if !resolved.is_empty() {
        dedup_by_file_identity(&mut files, &resolved);
    }
    files.sort_by_key(|path| sort_path_key(path));
    // The roots may overlap — `include = ["docs", "docs/api"]` names one subtree
    // twice, and under `--full` every `include` root is walked beside the config root
    // containing it (§FS-check.1.3.2). A file read twice duplicates its own declaration.
    files.dedup();
    // §FS-errors.4.1: the walk meets its unreadable paths in readdir order, so they
    // are sorted once here for both surfaces, then deduplicated — printing a scan
    // error twice is what the additivity rule of §FS-check.1.3.4 forbids.
    errors.sort_by_key(|(path, message)| (sort_path_key(path), message.clone()));
    errors.dedup();
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
    dirs.sort_by_key(|path| sort_path_key(path));
    dirs.dedup();
    Ok(WalkedTree {
        files,
        errors,
        outside_root,
        dirs,
    })
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
