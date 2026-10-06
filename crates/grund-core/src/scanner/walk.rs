use anyhow::{Result, anyhow};
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};

use super::e2e::e2e_id_from_case_dir_name;
use super::walk_boundaries::{
    is_directory_symlink, is_scannable, outward_directory_link_root, owned_by_another_project,
};
pub(super) use super::walk_reporting::walk_scannable_files_reporting;
use crate::config::{Config, canonical_config_root, root_scope_roots, unwalked_homes};
use crate::model::{
    configured_home_path_key, is_hidden, normalize_path_lexically, scanned_decl_relative_path,
};

/// The walker one scan root is traversed with (§AR-scanner.1): the hidden-name and
/// ignore-file rules the builder carries, and the workspace boundary, `[scan]
/// exclude`, canonical project-root boundary, unwalked-home and E2E-case prunes
/// [`WalkDirFilter`] carries.
///
/// Built apart from the reporting pass so [`walk_reads_any_file`] uses the
/// same one. That is not a tidiness point: §FS-check.4.10 reports a tree *because*
/// no scan reads it, so a probe that pruned differently from the scan would caution
/// a repository about content the scan would have skipped anyway — the one outcome
/// that finding cannot afford (§DF-unread-opted-out-block).
///
/// `link_roots` and `looping_links` are the filter's two outputs and belong to the
/// caller, because the reporting walk reads both after the traversal.
pub(super) fn scannable_walker(
    config: &Config,
    scan_root: &Path,
    canonical_scan_root: &Path,
    physical_root: &Path,
    link_roots: &std::sync::Arc<std::sync::Mutex<Vec<PathBuf>>>,
    looping_links: &std::sync::Arc<std::sync::Mutex<Vec<(PathBuf, PathBuf)>>>,
) -> Result<ignore::Walk> {
    // §FS-check.6.1.3: include ignore discovery outside the source roots.
    if config.respect_gitignore && !crate::config::observe_ignore_inputs(scan_root) {
        return Err(anyhow!("watch ignore coverage failed"));
    }
    let mut builder = WalkBuilder::new(scan_root);
    builder.hidden(false);
    // §FS-config.3.5.1: a symlink is part of the tree at the path it occupies, so
    // the walk reads through it — a linked file as the file, a linked directory by
    // descending; the entry keeps its in-tree path either way.
    builder.follow_links(true);
    if !config.respect_gitignore {
        builder
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false)
            .parents(false);
    }
    // §AR-workspace.6: precompute the boundary path components once, expressed
    // relative to the canonical scan root, so the walker filter is a single
    // component-suffix compare — no per-entry `canonicalize`, no allocation.
    let boundary_suffixes: Vec<PathBuf> = config
        .workspace_boundary_roots
        .iter()
        .filter_map(|root| root.strip_prefix(canonical_scan_root).ok())
        .map(Path::to_path_buf)
        .collect();
    let filter = WalkDirFilter {
        scan_root: scan_root.to_path_buf(),
        canonical_scan_root: canonical_scan_root.to_path_buf(),
        boundary_suffixes,
        boundary_roots: config.workspace_boundary_roots.clone(),
        excluded: config.exclude.clone(),
        e2e_cases_root: config
            .kinds
            .iter()
            .find(|kind| kind.kind == "E2E" && kind.citable)
            .and_then(|kind| kind.folder.as_deref())
            .map(|folder| config.root.join(folder)),
        physical_root: physical_root.to_path_buf(),
        unwalked_homes: walk_pruned_home_keys(config, scan_root, physical_root),
        config: config.clone(),
        link_roots: std::sync::Arc::clone(link_roots),
        looping_links: std::sync::Arc::clone(looping_links),
    };
    builder.filter_entry(move |entry| filter.keep(entry));
    Ok(builder.build())
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
pub(crate) fn walk_reads_any_file(config: &Config, scan_root: &Path) -> bool {
    if !scan_root.exists() {
        return false;
    }
    let canonical_scan_root =
        fs::canonicalize(scan_root).unwrap_or_else(|_| scan_root.to_path_buf());
    let physical_root = canonical_config_root(config);
    // §FS-config.3.5.1, §FS-workspace.6: the reporting walk's own scan-root
    // gates, ahead of the branch below, because neither a file root nor a walk
    // root ever reaches the filter.
    if outward_directory_link_root(scan_root, &canonical_scan_root, &physical_root)
        || config
            .workspace_boundary_roots
            .iter()
            .any(|root| canonical_scan_root.starts_with(root))
        || owned_by_another_project(config, &physical_root, &canonical_scan_root)
    {
        return false;
    }
    if scan_root.is_file() {
        return is_scannable(scan_root, config);
    }
    let link_roots = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let looping_links = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let Ok(walker) = scannable_walker(
        config,
        scan_root,
        &canonical_scan_root,
        &physical_root,
        &link_roots,
        &looping_links,
    ) else {
        return false;
    };
    walker.filter_map(Result::ok).any(|entry| {
        entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
            && is_scannable(entry.path(), config)
    })
}

/// The directory filter the walk runs on every entry it meets: the workspace
/// boundary (§AR-workspace.6), hidden names and `[scan] exclude`
/// (§FS-config.3.5.3), and the E2E case directories the manifest pass owns
/// (§AR-scanner.6.4).
///
/// The **name** tests read the in-tree path, so a followed link is pruned under
/// the name it wears in the tree — `docs/node_modules -> ../../node_modules` is
/// excluded exactly as a real directory of that name would be. The canonical
/// project-root fence and the two **ownership** tests read the canonical path as
/// well, because the physical root, a member root, and a case directory are
/// properties of the directory rather than of the name it is reached under
/// (§AR-scanner.1.3): reached through a link they match neither the in-tree
/// prefix, the precomputed suffix, nor the parent compare, and the walk would
/// descend into a namespace that is not its to read.
///
/// `link_roots` is how a link-reached directory is recognized without a syscall
/// per entry. The sequential walker filters a directory before its children, so
/// every directory link the walk meets is recorded here before anything below it
/// is asked about, and a prefix test then answers for the whole subtree; a link
/// already covered by one is not recorded again, so the list holds the disjoint
/// link subtrees and nothing more.
struct WalkDirFilter {
    scan_root: PathBuf,
    canonical_scan_root: PathBuf,
    physical_root: PathBuf,
    boundary_suffixes: Vec<PathBuf>,
    boundary_roots: Vec<PathBuf>,
    excluded: Vec<String>,
    /// The `scan = false` homes this walk prunes, as home path keys
    /// (§FS-config.3.4.7.2). Empty under `--full`, and empty in the tree that
    /// configures no such kind — which is every tree that never pays for the
    /// test below (§GOAL-fast-feedback).
    unwalked_homes: Vec<PathBuf>,
    e2e_cases_root: Option<PathBuf>,
    config: Config,
    link_roots: std::sync::Arc<std::sync::Mutex<Vec<PathBuf>>>,
    looping_links: std::sync::Arc<std::sync::Mutex<Vec<(PathBuf, PathBuf)>>>,
}

impl WalkDirFilter {
    /// Whether the walk keeps this entry — and, for a directory, descends into
    /// it. Files are otherwise not filtered here; the extension test is the
    /// caller's.
    ///
    /// Why the unwalked-home prune runs first, and on files too: keeping such a home
    /// out of the roots is only half the rule — the walk meets the same home again as
    /// a descendant of any root above it, the config root or an `include` entry it
    /// sits under, and read there it would be walked after all. It is the one rule
    /// asked of files as well as directories, which is why it is asked before the
    /// directory gate: a single-file home (`file = "docs/template.md"`) is never a
    /// directory to prune, and pruning its parent is not on offer — the parent is
    /// `docs`. Testing the in-tree path rather than the name prunes *this* home and
    /// nothing that merely shares its last component. The list is empty under `--full`
    /// and in every tree that configures no such kind, which is where the cost of
    /// asking per entry would otherwise fall.
    ///
    /// Why the walk stops at a link whose target is at or above the walk root: the
    /// walker compares a target against the directories it is *inside*, and this one
    /// is not one of them, so `docs/up -> ..` sends it down a complete second copy of
    /// the tree before it notices at `docs/up/docs/up`. That descent reports findings
    /// from a tree the run has just called unreadable and reads past `[scan] include`,
    /// so the link is pruned here and the report raised afterwards.
    fn keep(&self, entry: &ignore::DirEntry) -> bool {
        // §FS-check.6.1.1: linked files/dirs expose targets before descent/read.
        if entry.path_is_symlink() {
            if !crate::config::observe_input(
                entry.path(),
                entry.file_type().is_some_and(|t| t.is_dir()),
            ) {
                return false;
            }
        }
        if entry.depth() == 0 {
            return true;
        }
        // §FS-config.3.4.7.2: a home the config lists without walking is pruned here as
        // well as left out of `kind_home_roots`, on the in-tree path the way
        // §AR-scanner.2.4.2 decides which home a file is in (§GOAL-fast-feedback).
        if !self.unwalked_homes.is_empty()
            && let Some(relative) =
                scanned_decl_relative_path(entry.path(), &self.config.root, &self.physical_root)
            && self
                .unwalked_homes
                .iter()
                .any(|home| relative.starts_with(home))
        {
            return false;
        }
        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_dir())
        {
            return true;
        }
        let path = entry.path();
        let resolved = self.resolved_link_dir(entry);
        let resolved = resolved.as_deref();
        if self.crosses_a_scan_boundary(path, resolved) || is_hidden(path) {
            return false;
        }
        if self.is_e2e_case_dir(path) || resolved.is_some_and(|path| self.is_e2e_case_dir(path)) {
            return false;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return true;
        };
        if self.excluded.iter().any(|item| item == name) {
            return false;
        }
        // §FS-config.3.5.5: a directory link whose target is at or above the walk root
        // is a loop, and the one kind the walker cannot see, so it is pruned here and
        // the report raised afterwards from `looping_links` (§AR-scanner.1.9).
        if entry.path_is_symlink()
            && let Some(resolved) = resolved
            && self.canonical_scan_root.starts_with(resolved)
        {
            self.looping_links
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push((path.to_path_buf(), resolved.to_path_buf()));
            return false;
        }
        true
    }

    /// The canonical path of a directory the walk reached **through** a symlink
    /// — the link itself, or anything below one — and `None` for a directory
    /// reached under its own name, which is the ordinary case and pays no
    /// syscall (§AR-scanner.1.3, §GOAL-fast-feedback).
    fn resolved_link_dir(&self, entry: &ignore::DirEntry) -> Option<PathBuf> {
        let path = entry.path();
        let mut link_roots = self
            .link_roots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let is_link = entry.path_is_symlink();
        let covered = link_roots.iter().any(|root| path.starts_with(root));
        if is_link && !covered {
            link_roots.push(path.to_path_buf());
        }
        drop(link_roots);
        (is_link || covered)
            .then(|| fs::canonicalize(path).ok())
            .flatten()
    }

    /// A link-reached directory outside the canonical project root is out of
    /// bounds (§FS-config.3.5.1, §AR-scanner.1.2). Loaded-workspace ownership is
    /// stronger: another project remains out of bounds even when it lies inside
    /// this project's physical root (§FS-workspace.6.2, §AR-workspace.6.2).
    fn crosses_a_scan_boundary(&self, path: &Path, resolved: Option<&Path>) -> bool {
        if let Ok(relative) = path.strip_prefix(&self.scan_root)
            && self
                .boundary_suffixes
                .iter()
                .any(|suffix| relative == suffix.as_path())
        {
            return true;
        }
        resolved.is_some_and(|resolved| {
            !resolved.starts_with(&self.physical_root)
                || self
                    .boundary_roots
                    .iter()
                    .any(|root| resolved.starts_with(root))
                || owned_by_another_project(&self.config, &self.physical_root, resolved)
        })
    }

    fn is_e2e_case_dir(&self, path: &Path) -> bool {
        is_direct_e2e_case_dir(path, self.e2e_cases_root.as_deref(), &self.config)
    }
}

/// Direct `e2e/cases/<name>/` directories are E2E manifest declarations
/// (§AR-scanner.6.1), so the ordinary file walk must not scan their fixture repos.
pub(super) fn is_direct_e2e_case_dir(
    path: &Path,
    cases_root: Option<&Path>,
    config: &Config,
) -> bool {
    let Some(cases_root) = cases_root else {
        return false;
    };
    if path.parent() != Some(cases_root) || !path.join("expected.exit").is_file() {
        return false;
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| e2e_id_from_case_dir_name(config, name))
        .is_some()
}

/// The directories (or single file) the walk starts from: `[scan] include`
/// resolved against the repo root, otherwise the whole root — and, for a run
/// given a `[path]` argument below that root, the path beside them, because the
/// path narrows the report and not the walk (§FS-config.3.5.7, §AR-scanner.1.6,
/// §FS-check.1.3.6.1).
pub(super) fn scan_roots(
    config: &Config,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<Vec<PathBuf>> {
    scan_roots_for(
        config,
        scope,
        explicit_scope,
        config.scan_full,
        config.scan_resolution_wide,
    )
}

/// §FS-check.1.3.1: `full` cancels `[scan] include` for this walk and nothing else
/// — an explicit path argument still narrows the report, and `exclude`, the ignore
/// files, and `extensions` are untouched. `check --full` asks both ways: once with
/// `true` to walk the whole root, and once with `false` to learn which of what it
/// read was inside the configured scope (§FS-check.3.14).
///
/// §FS-check.1.3.6.1: `widen` is the other axis, and the walk is the one caller
/// that passes it. It says the explicit path is this run's *report* scope, so the
/// roots returned are the ordinary ones union the path; `full` is then irrelevant,
/// since a path leaves the flag nothing to cancel (§FS-check.1.3.6). The two scope
/// layers ask with `widen = false`, because each wants the path as the *bound* it
/// is to them rather than as the walk's extra root (§AR-resolver.3.3).
pub(crate) fn scan_roots_for(
    config: &Config,
    scope: Option<&Path>,
    explicit_scope: bool,
    full: bool,
    widen: bool,
) -> Result<Vec<PathBuf>> {
    if explicit_scope {
        let scope = scope.unwrap_or(Path::new("."));
        if !scope.exists() {
            return Err(anyhow!("path does not exist: {}", scope.display()));
        }
        let lexical_scope = if scope.is_absolute() {
            normalize_path_lexically(scope)
        } else {
            std::env::current_dir()
                .map(|cwd| normalize_path_lexically(&cwd.join(scope)))
                .unwrap_or_else(|_| normalize_path_lexically(scope))
        };
        let resolved = fs::canonicalize(scope).unwrap_or_else(|_| lexical_scope.clone());
        // §FS-config.3.5.1, §FS-config.3.5.2.1: keep the lexical spelling for an
        // in-tree link and an external directory-link root; resolve other roots.
        let scope =
            if lexical_scope.starts_with(&config.root) || is_directory_symlink(&lexical_scope) {
                lexical_scope
            } else {
                walk_root_under_config_root(config, &resolved)
            };
        if resolved == canonical_config_root(config) {
            return Ok(root_scope_roots(config, full));
        }
        // §FS-check.1.3.6.1: the ordinary roots union the path, the path first so its
        // spelling wins for the files it names (§FS-check.1.3.2). `full` is no part of
        // it — an explicit path leaves the flag nothing to cancel (§FS-check.1.3.6).
        if widen {
            let mut roots = vec![scope];
            roots.extend(root_scope_roots(config, false));
            return Ok(roots);
        }
        return Ok(vec![scope]);
    }
    Ok(root_scope_roots(config, full))
}

/// A resolved scope re-expressed under the spelling `config.root` wears
/// (§FS-config.3.5.2.1). Resolving the scope above answers "which directory is
/// this", and on a root that is itself reached through a link — a symlinked
/// `~/work`, macOS resolving `/var` to `/private/var` — it throws away the
/// answer to "what is it called here": the walk would start at the physical
/// path, every finding would be spelled physically, and `relative_paths` could
/// not strip a root those paths no longer begin with (§FS-config.3.6). A root
/// that is already canonical — every root `grund` discovers for itself — takes
/// the first branch and the whole question costs one `stat` per run.
fn walk_root_under_config_root(config: &Config, resolved: &Path) -> PathBuf {
    let canonical_root = canonical_config_root(config);
    if canonical_root == config.root {
        return resolved.to_path_buf();
    }
    match resolved.strip_prefix(&canonical_root) {
        Ok(rest) => config.root.join(rest),
        Err(_) => resolved.to_path_buf(),
    }
}

/// The same homes as home *path keys*, for the walk's per-entry test: it
/// compares an in-tree path stripped to the config root, the way the scanner
/// decides which home a file is in (§AR-scanner.2.4.2), so a root reached through
/// a symlink still recognizes them.
///
/// Empty for the two walks that are meant to read such a home. `--full` reaches
/// it like any directory nobody configured (§FS-check.1.3). And a walk whose own
/// root is at or inside one is a path a user typed — `grund check docs/templates`
/// — which reads the directory it names the way an explicit argument already
/// reads past `[scan] include` (§FS-config.3.4.7.3); the key describes the default
/// scope, and `grund check .` resolves to that scope rather than to this branch.
fn walk_pruned_home_keys(config: &Config, scan_root: &Path, physical_root: &Path) -> Vec<PathBuf> {
    if config.scan_full {
        return Vec::new();
    }
    let homes = unwalked_homes(config)
        .map(configured_home_path_key)
        .collect::<Vec<_>>();
    let asked_for_one = scanned_decl_relative_path(scan_root, &config.root, physical_root)
        .is_some_and(|relative| homes.iter().any(|home| relative.starts_with(home)));
    if asked_for_one { Vec::new() } else { homes }
}
