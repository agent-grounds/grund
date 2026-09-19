//! What a workspace-aware command holds (§AR-system.2.10): the set of projects a
//! run operates on, loaded and scanned, plus the index that says which of them
//! an unqualified ID resolves against and the root every path renders from
//! (§FS-workspace.8 intro, §AR-resolver.3).
//!
//! The loaders here are the one seam every walking command enters the workspace
//! through, whichever of the three shapes a run turns out to be — standalone,
//! member-local, or the whole workspace — so no command carries a second opinion
//! about what "the current project" is (§AR-workspace.5.1).
//!
//! This is the file the resolver exists for: it takes the project map workspace
//! expanded out of configs and *scans* every project in it, which is what puts
//! this component above the scanner while the expansion stays below it
//! (§AR-resolver.placement).

use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use super::legacy_promotion::promote_qualified_legacy_citations;
use super::shorthand::resolve_qualified_shorthand_citations;
use super::unread_block::settled_run_warnings;
use crate::config::Config;
use crate::model::{Diagnostic, Findings, TextOverlays};
use crate::scanner::{ScanError, scan_tree_with_workspace_overlays};
use crate::workspace::{
    WorkspaceCitationTarget, expand_workspace_tree, resolve_workspace_config, scope_is_config_root,
    unlisted_workspace_block_run_warnings,
};

/// One project in scope for a query command — an alias, the loaded config,
/// and the scanner's findings + scan errors for that project's tree.
/// Mirrors `ProjectScan` in `api/run.rs`; kept here as the shared shape
/// every query command consumes (§AR-resolver.3).
pub(crate) struct WorkspaceProject {
    pub(crate) alias: String,
    pub(crate) config: Config,
    pub(crate) findings: Findings,
    pub(crate) scan_errors: Vec<ScanError>,
}

/// Everything a workspace-aware query command needs (§FS-workspace.8 intro,
/// §AR-resolver.3): every loaded project (the current one plus, when running
/// at the workspace root, every member configured under `[workspace]`), an
/// optional index naming the project unqualified IDs resolve against, and the
/// canonical render-root used for `[output] relative_paths`.
///
/// Member-local and standalone runs collapse to one project at index `0` with
/// `workspace_loaded == false` — every command can route through one struct.
pub(crate) struct WorkspaceContext {
    pub(crate) projects: Vec<WorkspaceProject>,
    /// Index into `projects` for the "current project" — what `<ID>` (no
    /// alias) resolves against (§FS-workspace.8.9 intro). `None` only for a
    /// workspace-root run with `include_root = false`, where there is no root
    /// project for unqualified lookups (§FS-workspace.8.9 intro).
    pub(crate) current: Option<usize>,
    /// `true` only when a `[workspace]` block was discovered AND the
    /// invocation actually loads the workspace (i.e. not pinned member-local
    /// by an explicit path inside a member). When `false`, `projects` is a
    /// single entry and qualified `alias/<ID>` lookups must fail with
    /// `unknown project alias <name>`.
    pub(crate) workspace_loaded: bool,
    /// The repository root used for path rendering in workspace mode (the
    /// `[output] relative_paths` base). For workspace mode this is the
    /// workspace root; for single-project mode it equals
    /// `projects[current].config.root`. Used by `fmt --cross-refs` to
    /// compute a relative URL that spans projects (§FS-workspace.8.5).
    pub(crate) render_root: PathBuf,
    /// The config that owns the render root. In workspace mode this is the
    /// root workspace config even when `include_root = false`; commands use it
    /// for output format and path rendering without pretending it is a loaded
    /// project.
    ///
    /// It is the config **after** the workspace walk, so it carries what the walk
    /// learned about the tree as well as how to spell it — `workspace_absent_optional`
    /// in particular, which is where `check_workspace_context` reads the
    /// §FS-check.4.9 announcement from and which no loaded project can supply when
    /// every project in the block was the absent one (§FS-lsp.4.1).
    pub(crate) render_config: Config,
    /// The run's warning channel (§FS-distribution.3.1): the four `[workspace]`
    /// cautions of §FS-check.4.7.7, §FS-check.4.8.15, §FS-check.4.10.11 and
    /// §FS-workspace.6.1.7, settled and in the order the run earned them, for
    /// whichever frontend asked to render. Every command that walks passes
    /// through this loader, which is what puts all four on `list`, `refs`,
    /// `cover`, `fmt` and the ID read rather than on `check` alone.
    pub(crate) run_warnings: Vec<Diagnostic>,
}

impl WorkspaceContext {
    pub(crate) fn current_project(&self) -> Option<&WorkspaceProject> {
        self.current.map(|current| &self.projects[current])
    }

    pub(crate) fn render_config(&self) -> &Config {
        &self.render_config
    }

    pub(crate) fn project_by_alias(&self, alias: &str) -> Option<&WorkspaceProject> {
        self.projects.iter().find(|project| project.alias == alias)
    }

    /// Every known alias in the workspace, in `projects` order. An empty list
    /// when `workspace_loaded == false`. Used by completions and by the
    /// "neither declared nor cited" hint in `refs` to suggest the right
    /// `--project` slug.
    pub(crate) fn aliases(&self) -> Vec<&str> {
        if !self.workspace_loaded {
            return Vec::new();
        }
        self.projects
            .iter()
            .map(|project| project.alias.as_str())
            .collect()
    }
}

/// Load every project a query command should see, given the same `(path,
/// path_provided)` pair every entry point already accepts. The three cases:
///
/// - **Standalone** (no `[workspace]` discovered) → one project, the
///   discovered config.
/// - **Member-local** (path resolves inside a workspace member, or the
///   discovered config is a member's own) → one project, with the member
///   config. `workspace_loaded == false`; qualified citations cannot resolve.
/// - **Workspace** (path is at the workspace root or a non-member subdir of
///   it) → root (when `include_root = true`) plus every configured member.
///   `current` is the root when it is included, otherwise `None` so
///   unqualified lookups cannot silently resolve against a member.
///
/// Discovery itself is delegated to the existing `resolve_workspace_config`
/// — this helper is strictly the "load every project that's in scope" layer
/// on top of it (§AR-workspace.5.1).
pub(crate) fn load_workspace_context(path: &Path, path_provided: bool) -> Result<WorkspaceContext> {
    record_test_workspace_load();
    load_workspace_context_with_overlays(path, path_provided, &TextOverlays::new(), false)
}

/// Test-only observation seam for §AR-resolver.3: an opted-in black-box test
/// can count public CLI loader entries without changing the loader's result.
#[cfg(feature = "test-workspace-load-count")]
fn record_test_workspace_load() {
    use std::io::Write;

    let Some(path) = std::env::var_os("GRUND_TEST_WORKSPACE_LOAD_LOG") else {
        return;
    };
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open GRUND_TEST_WORKSPACE_LOAD_LOG");
    writeln!(log, "load").expect("write GRUND_TEST_WORKSPACE_LOAD_LOG");
}

#[cfg(not(feature = "test-workspace-load-count"))]
fn record_test_workspace_load() {}

pub(crate) fn load_workspace_context_with_overlays(
    path: &Path,
    path_provided: bool,
    overlays: &TextOverlays,
    classify_citation_sources: bool,
) -> Result<WorkspaceContext> {
    let config = resolve_workspace_config(path)?;
    load_resolved_workspace_context(
        config,
        path,
        path_provided,
        overlays,
        classify_citation_sources,
    )
}

/// The same load, entered from an already-resolved config. `resolve_workspace_config`
/// walks the member globs to apply the workspace boundary (§AR-workspace.6), so a
/// caller that had to resolve the config to make a routing decision must not pay
/// for it twice (§GOAL-fast-feedback).
///
/// Who asks for citing-side classification: the LSP snapshot passes `true` so
/// `grund check`'s citation-direction errors (`missing-citation` /
/// `forbidden-citation`, §FS-lsp.1.1) surface in the editor; `grund check` itself
/// uses `load_workspace_projects` directly and keeps the default (on).
///
/// Why `workspace_declared` is the canonical "is this a workspace run?": a path that
/// resolves member-local has already been rewritten by `config_for_member_scope` to
/// drop the flag, so the answer does not depend on where in the workspace the user
/// invoked the command — `grund alias/FS-x docs/`, `grund refs FS-y .`, and
/// `grund fmt --cross-refs subdir/` all see the same workspace.
pub(crate) fn load_resolved_workspace_context(
    mut config: Config,
    path: &Path,
    path_provided: bool,
    overlays: &TextOverlays,
    classify_citation_sources: bool,
) -> Result<WorkspaceContext> {
    // §AR-scanner.2.4.2 / §AR-benchmarks: the read-only commands (`list`, `show`,
    // `refs`, `fmt`) never read citing-side classification, so they pass `false` to
    // skip the scan post-pass. Workspace members inherit this below.
    config.classify_citation_sources = classify_citation_sources;
    // §FS-workspace.5 / §AR-workspace.6: workspace mode applies whenever the
    // discovered config carries `[workspace]` after member-scope rewriting, so this
    // flag is the single canonical "is this a workspace run?".
    if !config.workspace_declared {
        return single_project_context(config, path, path_provided, overlays);
    }

    let mut root_config = config;
    let render_root = root_config.root.clone();
    // §FS-workspace.8.9 intro: the current project is the root iff
    // `include_root = true` (the helper always emits the root first).
    let current = root_config.workspace_include_root.then_some(0);
    let projects = load_workspace_projects_with_overlays(&mut root_config, overlays)?;
    // Cloned *after* the expansion, not before: what the walk learns about the
    // tree is what the report is rendered from (§FS-check.4.9).
    let render_config = root_config.clone();
    // §FS-check.4.8.15: the query surfaces have no report to carry the finding, so it
    // joins the run's warning channel here (§DF-unlisted-workspace-block.2.3),
    // after the three the workspace pass settled — the order they were emitted in.
    let mut run_warnings = settled_run_warnings(&render_config);
    for project in &projects {
        run_warnings.extend(unlisted_workspace_block_run_warnings(
            &project.config,
            &render_config,
            Some(&project.alias),
            &project.findings.walked_dirs,
        ));
    }
    Ok(WorkspaceContext {
        projects,
        current,
        workspace_loaded: true,
        render_root,
        render_config,
        run_warnings,
    })
}

/// The one-project context every non-workspace run collapses to: the discovered
/// config, scanned under the caller's scope, with `workspace_loaded == false` so
/// qualified `<alias>/<ID>` lookups fail loud (§FS-workspace.8 intro).
///
/// Shared by the two ways a run gets here — no `[workspace]` block at all, and a
/// scope that narrows inside one (`load_narrowable_workspace_context`) — so the
/// two cannot drift on what "single project" means.
fn single_project_context(
    config: Config,
    path: &Path,
    path_provided: bool,
    overlays: &TextOverlays,
) -> Result<WorkspaceContext> {
    let (findings, scan_errors) =
        scan_tree_with_workspace_overlays(&config, Some(path), path_provided, &[], overlays)?;
    let render_root = config.root.clone();
    let render_config = config.clone();
    // §FS-check.4.8: the same finding for the runs that loaded one project — a
    // narrowed scope inside a workspace, or a repository with no `[workspace]` block
    // of its own that still walks into one.
    let mut run_warnings = settled_run_warnings(&config);
    run_warnings.extend(unlisted_workspace_block_run_warnings(
        &config,
        &render_config,
        None,
        &findings.walked_dirs,
    ));
    Ok(WorkspaceContext {
        projects: vec![WorkspaceProject {
            alias: String::new(),
            config,
            findings,
            scan_errors,
        }],
        current: Some(0),
        workspace_loaded: false,
        render_root,
        render_config,
        run_warnings,
    })
}

/// The loader for a command whose `<path>` argument narrows the **walk** rather
/// than only choosing which config answers — today that is `grund cover`
/// ([§FS-cover.1](FS-cover.md)).
///
/// `grund check` draws this line already: it takes the workspace-aggregate path
/// only when the scope *is* the config root, and otherwise runs one narrowed scan
/// (`scope_is_config_root`, §FS-check.1.3.6). A scope narrower than the root has to
/// stay narrow — an explicit path bypasses `[scan] include` (§AR-scanner.1.6), so
/// widening it back to every project would both answer a question the caller did
/// not ask and lose the files the narrowing was for.
///
/// `list`, `refs`, `show`, completions, and `fmt` keep [`load_workspace_context`]:
/// their `<path>` selects a project, it does not bound a walk (§FS-workspace.8.8).
pub(crate) fn load_narrowable_workspace_context(
    path: &Path,
    path_provided: bool,
) -> Result<WorkspaceContext> {
    let mut config = resolve_workspace_config(path)?;
    if !config.workspace_declared || scope_is_config_root(&config, path, path_provided) {
        // The resolved config is handed on rather than re-derived:
        // `load_workspace_context` would resolve it a second time, glob walk
        // included, on every aggregate run (§GOAL-fast-feedback).
        return load_resolved_workspace_context(
            config,
            path,
            path_provided,
            &TextOverlays::new(),
            false,
        );
    }
    // §AR-scanner.2.4.2: the caller is a read-only query — skip the classification
    // post-pass, exactly as `load_workspace_context` does (§AR-benchmarks).
    config.classify_citation_sources = false;
    single_project_context(config, path, path_provided, &TextOverlays::new())
}

/// Load every workspace project a workspace-mode command operates on:
/// expand the configured members, derive each alias, scan each project, and
/// reparse qualified citations against the full target list (§AR-workspace.5.1).
///
/// Returns one [`WorkspaceProject`] per project in the canonical order:
/// the root first when `include_root = true`, then members in member-glob
/// order. Mutates `root_config.workspace_boundary_roots` so any subsequent
/// root scan respects the member boundary (§AR-workspace.6).
pub(crate) fn load_workspace_projects(root_config: &mut Config) -> Result<Vec<WorkspaceProject>> {
    load_workspace_projects_with_overlays(root_config, &TextOverlays::new())
}

fn load_workspace_projects_with_overlays(
    root_config: &mut Config,
    overlays: &TextOverlays,
) -> Result<Vec<WorkspaceProject>> {
    // Stage 1: build the (alias, config) list, recursing into any member that
    // is itself a workspace root. Failing fast on alias errors, empty
    // workspaces, duplicates, member cycles, and missing members before any
    // scan keeps misconfiguration cheap to diagnose.
    let mut entries = expand_workspace_tree(root_config)?;

    // §AR-scanner.2.4.2: members inherit the root's classification intent, so a
    // read-only run skips the post-pass workspace-wide. §FS-check.1.3.8: `--full` is a
    // property of the run, so every member walks past its own `[scan] include` too.
    for entry in &mut entries {
        entry.config.classify_citation_sources = root_config.classify_citation_sources;
        entry.config.scan_full = root_config.scan_full;
    }

    // Stage 2: build the target list up-front so each project's scan can
    // parse `§<alias>/<ID>` citations with the target's grammar inline —
    // no second disk pass (§FS-workspace.1.2, §AR-workspace.2).
    let targets = entries
        .iter()
        .map(|entry| WorkspaceCitationTarget {
            alias: entry.alias.clone(),
            config: entry.config.clone(),
        })
        .collect::<Vec<_>>();

    // Stage 3: scan every project under its own config, with the workspace
    // targets in scope. Project scans are independent once aliases and target
    // grammars are validated; sort by the original entry index before returning
    // so root/member ordering stays byte-deterministic.
    let mut indexed = if entries.len() >= 2 {
        entries
            .into_par_iter()
            .enumerate()
            .map(|(index, entry)| {
                (
                    index,
                    load_workspace_project(entry.alias, entry.config, &targets, overlays),
                )
            })
            .collect::<Vec<_>>()
    } else {
        entries
            .into_iter()
            .enumerate()
            .map(|(index, entry)| {
                (
                    index,
                    load_workspace_project(entry.alias, entry.config, &targets, overlays),
                )
            })
            .collect::<Vec<_>>()
    };
    indexed.sort_by_key(|(index, _)| *index);
    let mut projects = indexed
        .into_iter()
        .map(|(_, project)| project)
        .collect::<Result<Vec<_>>>()?;
    promote_qualified_legacy_citations(&mut projects);
    resolve_qualified_shorthand_citations(&mut projects);
    Ok(projects)
}

fn load_workspace_project(
    alias: String,
    config: Config,
    targets: &[WorkspaceCitationTarget],
    overlays: &TextOverlays,
) -> Result<WorkspaceProject> {
    let (findings, scan_errors) =
        scan_tree_with_workspace_overlays(&config, Some(&config.root), true, targets, overlays)?;
    Ok(WorkspaceProject {
        alias,
        config,
        findings,
        scan_errors,
    })
}
