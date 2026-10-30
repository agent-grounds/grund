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
use crate::config::{ImportRead, MountEdge, ProjectRecords, config_at, config_file_in};
use crate::model::{Catalog, OperationDiagnostic, format_path};
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
    /// The `project_name` the tree's own config declares (§FS-imports.declaration.identity).
    name: Option<String>,
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
    /// Each root on the chain, by the name its importer gave it and the alias
    /// path its import spelled in the importer's run.
    chain: Vec<(PathBuf, String, String)>,
    memo: BTreeMap<PathBuf, Native>,
    cli_base: PathBuf,
    report_base: PathBuf,
    /// Every edge reached so far, in the order reached (§FS-imports.checks).
    pub(super) read: Vec<ImportRead>,
}

impl MountLoader {
    /// The loader for a run rooted at `consumer`, whose name opens every cycle's
    /// chain (§FS-imports.graph.cycle) and whose root every mounted path and
    /// config location is spelled from (§FS-imports.paths, §FS-errors.4).
    pub(crate) fn for_consumer(consumer: &ProjectRecords) -> Self {
        let name = consumer.project().name.clone().unwrap_or_default();
        Self {
            chain: vec![(consumer.root.clone(), name, String::new())],
            memo: BTreeMap::new(),
            cli_base: consumer.cli_base.clone(),
            report_base: consumer.root.clone(),
            read: Vec::new(),
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
            // §FS-imports.checks: every edge the run reaches is announced, a missing one
            // instead of being mounted.
            self.read.push(ImportRead {
                alias: self.spelled(edge),
                written: edge.written.clone(),
                source: edge.source.clone(),
                missing: edge.missing,
            });
            if edge.missing {
                continue;
            }
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
        if self.chain.iter().any(|(root, ..)| *root == edge.root) {
            // §FS-imports.graph.cycle: the chain from the consumer, by the names
            // each importer gave, closed by the import that reaches back.
            let chain = self
                .chain
                .iter()
                .map(|(_, name, _)| name.as_str())
                .chain(std::iter::once(edge.segment.as_str()))
                .collect::<Vec<_>>()
                .join(" → ");
            return Err(config_location_error(
                declaring,
                edge.source.as_ref(),
                format!("{CYCLE}{chain}"),
            ));
        }
        if let Some(native) = self.memo.get(&edge.root) {
            let native = native.clone();
            identity(edge, declaring, native.name.as_deref())?;
            return Ok(native);
        }
        let mut invocation = declaring.run().members();
        invocation.path = edge.root.clone();
        // §FS-imports.failures: the target's own refusal, then the import that reached it.
        let spelled = self.spelled(edge);
        let config = config_at(
            &edge.root,
            &self.cli_base,
            Some(&self.report_base),
            &invocation,
        );
        let mut config = config.map_err(|err| imported(err, &spelled, edge, declaring))?;
        identity(edge, declaring, config.project_name.as_deref())?;
        config.set_mounted_root(edge.root.clone());
        let mut records = ProjectRecords::of(config);
        self.chain
            .push((edge.root.clone(), edge.segment.clone(), edge.alias.clone()));
        let native = self.load_native(&mut records);
        self.chain.pop();
        let native = native.map_err(|err| imported(err, &spelled, edge, declaring))?;
        self.memo.insert(edge.root.clone(), native.clone());
        Ok(native)
    }

    /// The alias path the run spells `edge`'s target with: every mount the chain
    /// passed through, then the edge's own (§FS-imports.graph.transitive).
    fn spelled(&self, edge: &MountEdge) -> String {
        let outer = self
            .chain
            .iter()
            .skip(1)
            .map(|(_, _, alias)| alias.as_str());
        outer
            .chain(std::iter::once(edge.alias.as_str()))
            .collect::<Vec<_>>()
            .join("/")
    }

    /// The projects a mounted root's own run builds (§FS-remote-projects.mount.rule):
    /// its workspace, members and mounts beneath it, or the one project it is.
    fn load_native(&mut self, records: &mut ProjectRecords) -> Result<Native> {
        let mut roots = vec![records.root.clone()];
        let name = records.project_name.clone();
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
                name,
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
            name,
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

/// What opens the message of a cycle (§FS-imports.graph.cycle), whose chain already
/// names every import that reached its closing table.
const CYCLE: &str = "import cycle: ";

/// §FS-imports.declaration.identity: the target names itself what the import
/// expects, refused at the `project` line of the config that declares the import.
fn identity(edge: &MountEdge, declaring: &ProjectRecords, found: Option<&str>) -> Result<()> {
    let expected = &edge.project;
    if found == Some(expected.as_str()) {
        return Ok(());
    }
    let declares = found.map_or("declares no project_name".to_string(), |found| {
        format!("declares `{found}`")
    });
    let (alias, written) = (&edge.segment, &edge.written);
    Err(config_location_error(
        declaring,
        edge.project_source.as_ref(),
        format!("import `{alias}` expects project `{expected}`, but `{written}` {declares}"),
    ))
}

/// §FS-imports.failures: a refusal inside the target's own tree, followed by one
/// `note:` line naming the import that reached it, as a recovery breadcrumb
/// (§FS-errors.1.2); a cycle, or a refusal a deeper import already noted, is
/// passed on as it is. Each import it passes through marks itself as the site,
/// so the outermost one, in the consumer's own config, is the last to.
fn imported(
    err: anyhow::Error,
    spelled: &str,
    edge: &MountEdge,
    declaring: &ProjectRecords,
) -> anyhow::Error {
    let from = edge.source.as_ref().map_or_else(String::new, |source| {
        format!(" from {}:{}", format_path(&source.path), source.line)
    });
    let note = format!("\nnote: imported as `{spelled}`{from}");
    let site = edge
        .source
        .as_ref()
        .and_then(|source| Some((config_file_in(&declaring.root)?, source.line)));
    match err.downcast::<OperationDiagnostic>() {
        Ok(mut failure) => {
            if !failure.message.contains(CYCLE) && !failure.message.contains("\nnote: ") {
                failure.message.push_str(&note);
            }
            failure.import_site = site.or(failure.import_site.take());
            failure.into()
        }
        Err(err) => anyhow::anyhow!("{err:#}{note}"),
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
