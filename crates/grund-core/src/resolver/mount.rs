//! The mount layer's resolver half (§AR-resolver.mount): every mounted root a run
//! reaches is loaded once as its own outermost root, scanned natively, and grafted
//! into the run under its alias path, so each reader downstream sees one more
//! loaded project and asks nothing about mounts (§FS-remote-projects.mount.rule).
//!
//! The graft is the one place an identity is mapped: the native root alias becomes
//! the mount alias, every other alias path gains the mount alias in front, and the
//! literal text of a citation is never rewritten, which is why each grafted project
//! keeps the inverse of that map for the one reader that prints what was written.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::context::{WorkspaceProject, load_projects_with};
use crate::config::{MountEdge, ProjectRecords, config_at};
use crate::model::Catalog;
use crate::scanner::scan_tree_with_workspace;
use crate::workspace::config_location_error;

/// What a mount put into the run about one of its projects
/// (§FS-remote-projects.mount.rule): each mount it was grafted through, outermost
/// first, with the alias that mount is shown under and the native root alias that
/// alias replaces, and whether this alias path is the canonical one of the
/// project's tree (§FS-imports.graph.diamond).
#[derive(Clone, Debug)]
pub(crate) struct Mounted {
    layers: Vec<(String, Option<String>)>,
    pub(crate) canonical: bool,
}

impl Mounted {
    /// The namespace a citation of this project was written with, given the one the
    /// graft mapped it to: the inverse of every [`graft_namespace`] it went through
    /// (§FS-remote-projects.mount.rule, the literal text is never rewritten).
    pub(crate) fn written_namespace(&self, mapped: &str) -> String {
        let mut namespace = mapped.to_string();
        for (alias, root) in &self.layers {
            namespace = if namespace == *alias {
                root.clone().unwrap_or(namespace)
            } else {
                namespace
                    .strip_prefix(alias.as_str())
                    .and_then(|rest| rest.strip_prefix('/'))
                    .map_or(namespace.clone(), str::to_string)
            };
        }
        namespace
    }
}

/// One tree's native load: the projects its own standalone run builds, and the
/// alias its root project carries there, `None` where it loads no workspace or
/// leaves its root out (§FS-remote-projects.mount.rule).
#[derive(Clone)]
struct Native {
    projects: Vec<WorkspaceProject>,
    root_alias: Option<String>,
    /// Every mounted root the native load reached, its own included, so the
    /// consumer's report can spell each file below one (§FS-imports.paths).
    roots: Vec<PathBuf>,
}

/// The mounts one run loads (§AR-resolver.mount): the chain of canonical roots the
/// current import was reached through, which closes a cycle
/// (§FS-imports.graph.cycle), and each tree's native load, kept by canonical root so
/// a diamond loads a tree once (§FS-imports.graph.diamond).
pub(crate) struct MountLoader {
    chain: Vec<(PathBuf, String)>,
    memo: BTreeMap<PathBuf, Native>,
    cli_base: PathBuf,
    report_base: PathBuf,
}

impl MountLoader {
    /// The loader for a run rooted at `consumer`, whose name opens every cycle's
    /// chain (§FS-imports.graph.cycle) and whose root every mounted path and
    /// config location is spelled from (§FS-imports.paths, §FS-errors.4).
    pub(crate) fn for_consumer(consumer: &ProjectRecords) -> Self {
        let name = consumer.project().name.clone().unwrap_or_default();
        Self {
            chain: vec![(consumer.root.clone(), name)],
            memo: BTreeMap::new(),
            cli_base: consumer.cli_base.clone(),
            report_base: consumer.root.clone(),
        }
    }

    /// The base a mounted config's locations and paths are spelled from: the
    /// consumer's root (§FS-imports.paths).
    pub(super) fn report_base(&self) -> &Path {
        &self.report_base
    }

    /// Load every mount `config` declares and graft each under its alias path
    /// (§FS-remote-projects.mount.rule), in alias order, with the mounted roots the
    /// run now reports from (§FS-imports.paths).
    pub(super) fn graft_all(
        &mut self,
        config: &ProjectRecords,
    ) -> Result<(Vec<WorkspaceProject>, Vec<PathBuf>)> {
        let mut edges = config.run().workspace.mounts.clone();
        edges.sort_by(|a, b| a.alias.cmp(&b.alias));
        let mut grafted = Vec::new();
        let mut roots = Vec::new();
        for edge in &edges {
            let native = self.native(edge, config)?;
            roots.extend(native.roots.iter().cloned());
            grafted.extend(graft(&native, &edge.alias));
        }
        roots.sort();
        roots.dedup();
        Ok((grafted, roots))
    }

    /// `edge`'s tree as its own standalone run loads it: from the memo when the run
    /// already reached that canonical root, else from its own config at exactly that
    /// root, climbing nothing (§FS-imports.declaration.root).
    fn native(&mut self, edge: &MountEdge, declaring: &ProjectRecords) -> Result<Native> {
        if self.chain.iter().any(|(root, _)| *root == edge.root) {
            // §FS-imports.graph.cycle: the chain from the consumer, by the names
            // each importer gave, closed by the import that reaches back.
            let chain = self
                .chain
                .iter()
                .map(|(_, name)| name.as_str())
                .chain(std::iter::once(edge.segment.as_str()))
                .collect::<Vec<_>>()
                .join(" → ");
            return Err(config_location_error(
                declaring,
                edge.source.as_ref(),
                format!("import cycle: {chain}"),
            ));
        }
        if let Some(native) = self.memo.get(&edge.root) {
            return Ok(native.clone());
        }
        let mut invocation = declaring.run().members();
        invocation.path = edge.root.clone();
        let mut config = config_at(
            &edge.root,
            &self.cli_base,
            Some(&self.report_base),
            &invocation,
        )?;
        config.set_mounted_root(edge.root.clone());
        let mut records = ProjectRecords::of(config);
        self.chain.push((edge.root.clone(), edge.segment.clone()));
        let native = self.load_native(&mut records);
        self.chain.pop();
        let native = native?;
        self.memo.insert(edge.root.clone(), native.clone());
        Ok(native)
    }

    /// The projects a mounted root's own run builds (§FS-remote-projects.mount.rule):
    /// its workspace, members and mounts beneath it, or the one project it is.
    fn load_native(&mut self, records: &mut ProjectRecords) -> Result<Native> {
        let mut roots = vec![records.root.clone()];
        if records.project().workspace.declared || !records.run().workspace.mounts.is_empty() {
            let include_root = records.project().workspace.include_root;
            let (projects, below) = load_projects_with(records, self)?;
            roots.extend(below);
            // The root is the first project its own run loads, where it loads it.
            let root_alias = projects
                .first()
                .filter(|project| include_root && project.mount.is_none())
                .map(|project| project.alias.clone());
            return Ok(Native {
                projects,
                root_alias,
                roots,
            });
        }
        let (findings, scan_errors) = scan_tree_with_workspace(
            records.schema(),
            records.frame(),
            Some(records.frame().root()),
            true,
            &[],
        )?;
        Ok(Native {
            projects: vec![WorkspaceProject {
                alias: String::new(),
                config: records.clone(),
                findings,
                scan_errors,
                mount: None,
            }],
            root_alias: None,
            roots,
        })
    }
}

/// `native`'s projects as the run shows them under `alias`
/// (§FS-remote-projects.mount.rule): each alias path and every namespace a citation
/// or binding names mapped once, here, and nothing else touched.
fn graft(native: &Native, alias: &str) -> Vec<WorkspaceProject> {
    let root = native.root_alias.as_deref();
    native
        .projects
        .iter()
        .map(|project| {
            let mut project = project.clone();
            project.alias = graft_namespace(root, alias, &project.alias);
            graft_catalog(&mut project.findings, root, alias);
            let layer = (alias.to_string(), root.map(str::to_string));
            let mounted = project.mount.get_or_insert_with(|| Mounted {
                layers: Vec::new(),
                canonical: true,
            });
            mounted.layers.insert(0, layer);
            project
        })
        .collect()
}

/// One alias path as the mount shows it (§FS-remote-projects.mount.rule): the native
/// root alias, or a standalone project's empty one, is the mount alias, and any
/// other gains the mount alias in front, so a namespace the native run could not
/// resolve stays one the consumer run cannot either.
fn graft_namespace(root: Option<&str>, alias: &str, native: &str) -> String {
    if native.is_empty() || Some(native) == root {
        alias.to_string()
    } else {
        format!("{alias}/{native}")
    }
}

/// Every target-naming namespace in `catalog`, mapped (§FS-remote-projects.mount.rule).
fn graft_catalog(catalog: &mut Catalog, root: Option<&str>, alias: &str) {
    let map = |namespace: &mut Option<String>| {
        if let Some(native) = namespace.as_deref() {
            *namespace = Some(graft_namespace(root, alias, native));
        }
    };
    catalog
        .edges_mut()
        .for_each(|cite| map(&mut cite.namespace));
    catalog
        .escaped_citations
        .iter_mut()
        .for_each(|cite| map(&mut cite.namespace));
    catalog
        .value_bindings
        .iter_mut()
        .for_each(|binding| map(&mut binding.namespace));
    catalog
        .legacy_citation_candidates
        .iter_mut()
        .for_each(|candidate| map(&mut candidate.namespace));
}

/// Name each tree's canonical alias path (§FS-imports.graph.diamond): an ordinary
/// project of the run owns its tree outright, and among mounted copies alone the one
/// with the fewest segments, then the first in byte order, is canonical.
pub(super) fn mark_canonical(projects: &mut [WorkspaceProject]) {
    let mut owner: BTreeMap<PathBuf, (bool, usize, String)> = BTreeMap::new();
    for project in projects.iter() {
        let key = (
            project.mount.is_some(),
            project.alias.split('/').count(),
            project.alias.clone(),
        );
        owner
            .entry(project.config.root.clone())
            .and_modify(|best| {
                if key < *best {
                    *best = key.clone();
                }
            })
            .or_insert(key);
    }
    for project in projects.iter_mut() {
        if let Some(mounted) = &mut project.mount {
            mounted.canonical = owner
                .get(&project.config.root)
                .is_some_and(|(_, _, alias)| *alias == project.alias);
        }
    }
}
