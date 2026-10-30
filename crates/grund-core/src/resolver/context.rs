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
use super::mount::{MountLoader, Mounted, mark_canonical};
use super::shorthand::resolve_qualified_shorthand_citations;
use super::unread_block::settled_run_warnings;
use crate::config::{Invocation, ProjectRecords, Selection};
use crate::model::{Catalog, Citation, Diagnostic};
use crate::scanner::{ScanError, scan_tree_with_workspace};
use crate::sources::record_test_workspace_load;
use crate::workspace::{
    WorkspaceCitationTarget, expand_workspace_tree_with_report_base, open, open_narrowable,
    scope_is_config_root, unlisted_workspace_block_run_warnings,
};

/// One project in scope for a query command — an alias, the loaded project's
/// records (§AR-config.5), and the scanner's findings + scan errors for that
/// project's tree.
/// Mirrors `ProjectScan` in `api/run.rs`; kept here as the shared shape
/// every query command consumes (§AR-resolver.3).
#[derive(Clone)]
pub(crate) struct WorkspaceProject {
    pub(crate) alias: String,
    pub(crate) config: ProjectRecords,
    pub(crate) findings: Catalog,
    pub(crate) scan_errors: Vec<ScanError>,
    /// Set where a mount grafted this project into the run, `None` for one of the
    /// run's own (§FS-remote-projects.mount.rule): its findings are suppressed and
    /// its sites are reported once, under the canonical alias path
    /// (§FS-imports.findings, §FS-imports.graph.diamond).
    pub(crate) mount: Option<Mounted>,
}

impl WorkspaceProject {
    /// Whether this project's own findings are the run's to report: every project
    /// but a mounted one (§FS-imports.findings).
    pub(crate) fn reports_findings(&self) -> bool {
        self.mount.is_none()
    }

    /// Whether this copy is the one a site of its tree is reported under: an
    /// ordinary project always, a mounted copy only at its canonical alias path
    /// (§FS-imports.graph.diamond).
    pub(crate) fn reports_sites(&self) -> bool {
        self.mount.as_ref().is_none_or(|mounted| mounted.canonical)
    }
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
    /// alias) resolves against (§FS-workspace.8.9). `None` only for a
    /// workspace-root run with `include_root = false`, where there is no root
    /// project for unqualified lookups (§FS-workspace.8.9).
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
    pub(crate) render_config: ProjectRecords,
    /// The run's warning channel (§FS-distribution.3.1): the three `[workspace]`
    /// cautions of §FS-check.3.29.15, §FS-check.4.10.11 and
    /// §FS-workspace.6.1.7, settled and in the order the run earned them, for
    /// whichever frontend asked to render. Every command that walks passes
    /// through this loader, which is what puts all three on `list`, `refs`,
    /// `cover`, `fmt` and the ID read rather than on `check` alone.
    pub(crate) run_warnings: Vec<Diagnostic>,
}

impl WorkspaceContext {
    pub(crate) fn current_project(&self) -> Option<&WorkspaceProject> {
        self.current.map(|current| &self.projects[current])
    }

    pub(crate) fn render_config(&self) -> &ProjectRecords {
        &self.render_config
    }

    pub(crate) fn project_by_alias(&self, alias: &str) -> Option<&WorkspaceProject> {
        self.projects.iter().find(|project| project.alias == alias)
    }

    /// The index of the loaded project an alias names; `None` outside a workspace,
    /// where no alias names a project (§FS-workspace.8.1).
    pub(crate) fn loaded_alias(&self, alias: &str) -> Option<usize> {
        self.projects
            .iter()
            .position(|project| self.workspace_loaded && project.alias == alias)
    }

    /// The index of the project a citation in the project at `citing` targets: the
    /// loaded project its alias names, or else the one it sits in; `None` for an alias
    /// that names no loaded project (§FS-coordinate-globs.scope.targets,
    /// §FS-workspace.8.2.4). `refs` in both modes and `cover` all read it.
    pub(crate) fn citation_target(&self, citing: usize, citation: &Citation) -> Option<usize> {
        match citation.namespace.as_deref() {
            None => Some(citing),
            Some(alias) => self.loaded_alias(alias),
        }
    }

    /// Whether two citation targets are one tree: the same loaded project, or two
    /// alias paths a mount gave one tree (§FS-imports.graph.diamond).
    pub(crate) fn same_target(&self, left: Option<usize>, right: Option<usize>) -> bool {
        match (left, right) {
            (Some(left), Some(right)) => {
                left == right
                    || (self.projects[left].mount.is_some() || self.projects[right].mount.is_some())
                        && self.projects[left].config.root == self.projects[right].config.root
            }
            (left, right) => left == right,
        }
    }

    /// The copy a site of `project`'s tree is reported under, and the site as that
    /// copy holds it: the project itself unless a mount put a second alias path of
    /// the same tree in the run (§FS-imports.graph.diamond).
    pub(crate) fn canonical_site<'c>(
        &'c self,
        project: usize,
        citation: &'c Citation,
    ) -> (usize, &'c Citation) {
        if self.projects[project].reports_sites() {
            return (project, citation);
        }
        let root = &self.projects[project].config.root;
        let canonical = self
            .projects
            .iter()
            .position(|copy| copy.config.root == *root && copy.reports_sites());
        canonical
            .and_then(|index| {
                let same = |copy: &&Citation| {
                    copy.file == citation.file
                        && copy.line == citation.line
                        && copy.column == citation.column
                };
                let edges = self.projects[index].findings.edges();
                edges.into_iter().find(same).map(|copy| (index, copy))
            })
            .unwrap_or((project, citation))
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

    /// The run's kind table (§AR-config.1.5): the one the workspace
    /// walk built, held on the config that owns the render root, or the one
    /// project's where the run loaded no workspace.
    pub(crate) fn kind_table(&self) -> std::sync::Arc<crate::config::KindTable> {
        self.render_config.kind_table()
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
/// Discovery itself is delegated to `open()`
/// — this helper is strictly the "load every project that's in scope" layer
/// on top of it (§AR-workspace.5.1).
pub(crate) fn load_workspace_context(path: &Path, path_provided: bool) -> Result<WorkspaceContext> {
    record_test_workspace_load();
    load_invoked_workspace_context(
        &Invocation::new(path, path_provided, Selection::Project).classify(false),
    )
}

/// The load `show`, `list` and `--batch` answer from (§FS-scan.path): the projects
/// the path discovers, each read as `check`'s resolution scope is
/// (§FS-scan.path.check), so a path narrower than its config root bounds what the
/// command reports and never what it resolves against (§FS-scan.path.show,
/// §FS-scan.path.list). The `Check` selection is the one place that reading is
/// decided, so the four commands cannot answer one tree two ways
/// (§REQ-one-answer.scope.narrowed).
pub(crate) fn load_reported_workspace_context(
    path: &Path,
    path_provided: bool,
) -> Result<WorkspaceContext> {
    record_test_workspace_load();
    load_invoked_workspace_context(
        &Invocation::new(path, path_provided, Selection::Check).classify(false),
    )
}

/// The same load for a query that **publishes** citing-side attribution rather
/// than only resolving targets — today `grund refs` alone (§FS-refs.3.2).
///
/// Separate from [`load_workspace_context`] on purpose, not folded into it:
/// that wrapper is shared by `list`, `fmt`, `complete_ids`, `sizes` and `batch`,
/// none of which emits `enclosing_declaration` or `enclosing_section`, and
/// turning the post-pass on there would make five queries pay for a fact they
/// never print (§AR-scanner.2.4.2, §GOAL-fast-feedback).
pub(crate) fn load_classifying_workspace_context(
    path: &Path,
    path_provided: bool,
) -> Result<WorkspaceContext> {
    record_test_workspace_load();
    load_invoked_workspace_context(&Invocation::new(path, path_provided, Selection::Project))
}

/// The load for what one invocation decided, its buffers and its
/// classification included, every project's `Run` built by `open()`
/// (§AR-config.1.5).
pub(crate) fn load_invoked_workspace_context(invocation: &Invocation) -> Result<WorkspaceContext> {
    let config = ProjectRecords::of(open(invocation)?);
    load_resolved_workspace_context(config, &invocation.path, invocation.path_provided)
}

/// The same load, entered from an already-opened config. `open()`
/// walks the member globs to apply the workspace boundary (§AR-workspace.6), so a
/// caller that had to resolve the config to make a routing decision must not pay
/// for it twice (§GOAL-fast-feedback).
///
/// Who asks for citing-side classification (§AR-scanner.2.4.2), on the invocation
/// the config was opened with: the LSP snapshot asks so `grund check`'s citation-direction errors (`missing-citation` /
/// `forbidden-citation`, §FS-lsp.1.1) surface in the editor; `grund check` itself
/// uses `load_workspace_projects` directly and keeps the default (on); and
/// `cover` and `refs` ask because their JSON records publish the
/// enclosing declaration and section of the citing site (§FS-cover.3.2,
/// §FS-refs.3.2). `list`, `show`, `fmt`, ID completion, `sizes` and `batch` read
/// none of the three fields the pass fills and turn it off.
///
/// Why `workspace_declared` is the canonical "is this a workspace run?": a path that
/// resolves member-local has already been rewritten by `config_for_member_scope` to
/// drop the flag, so the answer does not depend on where in the workspace the user
/// invoked the command — `grund alias/FS-x docs/`, `grund refs FS-y .`, and
/// `grund fmt --cross-refs subdir/` all see the same workspace.
pub(crate) fn load_resolved_workspace_context(
    config: ProjectRecords,
    path: &Path,
    path_provided: bool,
) -> Result<WorkspaceContext> {
    // §FS-workspace.5 / §AR-workspace.6: workspace mode applies whenever the
    // discovered config carries `[workspace]` after member-scope rewriting, so this
    // flag is the single canonical "is this a workspace run?".
    if !config.project().workspace.declared {
        return single_project_context(config, path, path_provided);
    }

    let mut root_config = config;
    let render_root = root_config.frame().root().to_path_buf();
    // §FS-workspace.8.9: the current project is the root iff
    // `include_root = true` (the helper always emits the root first).
    let current = root_config.project().workspace.include_root.then_some(0);
    let projects = load_workspace_projects(&mut root_config)?;
    // Cloned *after* the expansion, not before: what the walk learns about the
    // tree is what the report is rendered from (§FS-check.4.9).
    let render_config = root_config.clone();
    // §FS-check.3.29.15: the query surfaces have no report to carry the finding, so it
    // joins the run's warning channel here (§DF-unlisted-workspace-block.2.3),
    // after the three the workspace pass settled — the order they were emitted in.
    let mut run_warnings = settled_run_warnings(render_config.run());
    // §FS-imports.findings: a mounted project's cautions are its standalone run's.
    for project in projects.iter().filter(|project| project.reports_findings()) {
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
    config: ProjectRecords,
    path: &Path,
    path_provided: bool,
) -> Result<WorkspaceContext> {
    let (findings, scan_errors) = scan_tree_with_workspace(
        config.schema(),
        config.frame(),
        Some(path),
        path_provided,
        &[],
    )?;
    let render_root = config.frame().root().to_path_buf();
    let render_config = config.clone();
    // §FS-check.3.29: the same finding for the runs that loaded one project — a
    // narrowed scope inside a workspace, or a repository with no `[workspace]` block
    // of its own that still walks into one.
    let mut run_warnings = settled_run_warnings(config.run());
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
            mount: None,
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
/// A scope narrower than the root stays one narrowed scan (§FS-scan.path.cover):
/// §AR-resolver.3.3 is which narrowness that is — which *projects* a walk covers
/// and which *files* are reported, never which declarations resolution reads.
/// `cover` is outside the ruling that gave `check <dir>` the projects a run from
/// `<dir>` sees (§FS-scan.projects.narrowed), so it keeps this one.
///
/// `list` and `show` take [`load_reported_workspace_context`], completions and
/// `fmt` [`load_workspace_context`], and `refs`
/// [`load_classifying_workspace_context`]: their `<path>` selects a project, it
/// does not bound a walk (§FS-workspace.8.8).
///
/// Both branches ask for the citing-side classification post-pass, because
/// `cover --format json` publishes the enclosing declaration and section of every
/// site it lists (§FS-cover.3.2, §AR-scanner.2.4.2). This function has one
/// caller, so that is a statement about `cover` alone.
///
/// `owner_lines` are the ranges a `cover --lines` run asks about, empty on every
/// other run: they ride on the config into each file's pass, which resolves them
/// while its heading stack is held (§FS-cover.6.2, §AR-scanner.2.4.4).
pub(crate) fn load_narrowable_workspace_context(
    path: &Path,
    path_provided: bool,
    owner_lines: &[(usize, usize)],
) -> Result<WorkspaceContext> {
    // §AR-scanner.2.4.2: both branches classify, so one command does not answer
    // two ways depending on where it was invoked (§FS-cover.3.2).
    let invocation =
        Invocation::new(path, path_provided, Selection::Project).owner_lines(owner_lines.to_vec());
    // §AR-config.1.5: narrowed or not, it judges the `[citations]`
    // targets the whole run judges, as `check` does.
    let config = ProjectRecords::of(open_narrowable(&invocation)?);
    if !config.project().workspace.declared
        || scope_is_config_root(&config.root, path, path_provided)
    {
        // The resolved config is handed on rather than re-derived:
        // `load_workspace_context` would resolve it a second time, glob walk
        // included, on every aggregate run (§GOAL-fast-feedback).
        return load_resolved_workspace_context(config, path, path_provided);
    }
    single_project_context(config, path, path_provided)
}

/// Load every workspace project a workspace-mode command operates on:
/// expand the configured members, derive each alias, scan each project, and
/// reparse qualified citations against the full target list (§AR-workspace.5.1).
///
/// Returns one [`WorkspaceProject`] per project in the canonical order:
/// the root first when `include_root = true`, then members in member-glob
/// order. Sets `root_config`'s workspace boundary roots so any subsequent
/// root scan respects the member boundary (§AR-workspace.6).
pub(crate) fn load_workspace_projects(
    root_config: &mut ProjectRecords,
) -> Result<Vec<WorkspaceProject>> {
    let mut mounts = MountLoader::for_consumer(root_config);
    let (mut projects, mounted_roots) = load_projects_with(root_config, &mut mounts)?;
    // §FS-imports.graph.diamond: named once every project of the run is known.
    mark_canonical(&mut projects);
    // §FS-imports.paths: the report spells a file below a mounted root from its base.
    if !mounted_roots.is_empty() {
        root_config
            .facade_mut()
            .set_mounted_roots(mounted_roots.clone());
        for project in &mut projects {
            project
                .config
                .facade_mut()
                .set_mounted_roots(mounted_roots.clone());
        }
    }
    Ok(projects)
}

/// [`load_workspace_projects`] for one root, the run's own or a mounted one
/// (§AR-resolver.mount): its tree expanded, each mount it declares loaded and
/// grafted through `mounts` before any scan of its own, so the scans parse a
/// qualified citation into a mount with that mount's grammar, and the grafted
/// projects after its own in alias order. Returns the mounted roots it reached.
pub(super) fn load_projects_with(
    root_config: &mut ProjectRecords,
    mounts: &mut MountLoader,
) -> Result<(Vec<WorkspaceProject>, Vec<PathBuf>)> {
    // Stage 1: build the (alias, config) list, recursing into any member that
    // is itself a workspace root. Failing fast on alias errors, empty
    // workspaces, duplicates, member cycles, and missing members before any
    // scan keeps misconfiguration cheap to diagnose.
    let report_base = if root_config.run().workspace.mounted_root.is_some() {
        mounts.report_base().to_path_buf()
    } else {
        root_config.root.clone()
    };
    let entries = expand_workspace_tree_with_report_base(root_config.facade_mut(), &report_base)?
        .into_iter()
        .map(|entry| (entry.alias, ProjectRecords::of(entry.config)))
        .collect::<Vec<_>>();

    // §AR-scanner.2.4.2, §FS-check.1.3.8: each member's run was built carrying the
    // root's classification and `--full` (`Run::members`), so nothing is set here.

    // §FS-remote-projects.mount.rule: every mount this tree declares, loaded natively.
    let (grafted, mounted_roots) = mounts.graft_all(root_config)?;

    // Stage 2: build the target list up-front so each project's scan can
    // parse `§<alias>/<ID>` citations with the target's grammar inline —
    // no second disk pass (§FS-workspace.1.2, §AR-workspace.2).
    let targets = entries
        .iter()
        .map(|(alias, records)| (alias, records))
        .chain(
            grafted
                .iter()
                .map(|project| (&project.alias, &project.config)),
        )
        .map(|(alias, records)| WorkspaceCitationTarget {
            alias: alias.clone(),
            schema: records.schema().clone(),
            compiled: records.compiled().clone(),
        })
        .collect::<Vec<_>>();

    // Stage 3: scan every project under its own config, with the workspace
    // targets in scope. Project scans are independent once aliases and target
    // grammars are validated; sort by the original entry index before returning
    // so root/member ordering stays byte-deterministic.

    // §FS-check.6.1.1, §AR-bindings.3: project workers inherit scoped coverage
    // before tree discovery, then restore their previous observer on return.
    let observer = crate::config::check_input_observer();
    let mut indexed = if entries.len() >= 2 {
        entries
            .into_par_iter()
            .enumerate()
            .map(|(index, (alias, records))| {
                crate::config::with_check_input_observer(observer.clone(), || {
                    (index, load_workspace_project(alias, records, &targets))
                })
            })
            .collect::<Vec<_>>()
    } else {
        entries
            .into_iter()
            .enumerate()
            .map(|(index, (alias, records))| {
                (index, load_workspace_project(alias, records, &targets))
            })
            .collect::<Vec<_>>()
    };
    indexed.sort_by_key(|(index, _)| *index);
    let mut projects = indexed
        .into_iter()
        .map(|(_, project)| project)
        .collect::<Result<Vec<_>>>()?;
    projects.extend(grafted);
    promote_qualified_legacy_citations(&mut projects);
    resolve_qualified_shorthand_citations(&mut projects);
    Ok((projects, mounted_roots))
}

fn load_workspace_project(
    alias: String,
    config: ProjectRecords,
    targets: &[WorkspaceCitationTarget],
) -> Result<WorkspaceProject> {
    let (findings, scan_errors) = scan_tree_with_workspace(
        config.schema(),
        config.frame(),
        Some(config.frame().root()),
        true,
        targets,
    )?;
    Ok(WorkspaceProject {
        alias,
        config,
        findings,
        scan_errors,
        mount: None,
    })
}
