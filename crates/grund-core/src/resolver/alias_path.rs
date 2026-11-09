//! Which project an alias path names, and where the fetch of an ID in it runs
//! (§FS-imports.writes.answer): the reads select through
//! [`WorkspaceContext::project_at_alias`], and `grund fetch` loads what that selection
//! asks and no scan through [`load_alias_path`], so the two cannot answer differently.

use std::fmt;
use std::path::Path;

use anyhow::{Result, anyhow};

use super::context::{WorkspaceContext, WorkspaceProject, load_workspace_projects_through};
use super::mount::MountLoader;
use super::unread_block::settled_run_warnings;
use crate::config::{ProjectRecords, Run};
use crate::model::{Catalog, OperationDiagnostic, format_path, relative_from_base};

impl WorkspaceContext {
    /// The project an alias path names, or the refusal that says why none does: a
    /// missing import first, then an unknown alias (§FS-workspace.8.1,
    /// §FS-imports.checks.import-missing).
    pub(crate) fn project_at_alias(&self, name: &str) -> Result<&WorkspaceProject> {
        if let Some(project) = self.project_by_alias(name) {
            return Ok(project);
        }
        if let Some(refusal) = self.missing_import(name) {
            return Err(refusal);
        }
        let hint = if self.workspace_loaded {
            format!("known aliases: {}", self.aliases().join(", "))
        } else {
            "note: workspace aliases are defined in the root grund.toml under [workspace]"
                .to_string()
        };
        Err(anyhow!(OperationDiagnostic::new(
            "query",
            "unknown-project",
            format!("unknown project alias `{name}`\n{hint}")
        )))
    }
}

/// What [`WorkspaceContext::project_at_alias`] needs to answer for `alias`, unscanned
/// (§FS-imports.writes.answer): the run's own tree and only the imports on that alias
/// path, so an import it does not name cannot fail it. Only an alias nothing answers
/// reads every edge, because its refusal lists every alias the run knows.
pub(crate) fn load_alias_path(config: ProjectRecords, alias: &str) -> Result<WorkspaceContext> {
    let along = unscanned_context(config.clone(), Some(alias))?;
    if along.project_by_alias(alias).is_some() || along.missing_import(alias).is_some() {
        return Ok(along);
    }
    unscanned_context(config, None)
}

fn unscanned_context(mut root: ProjectRecords, along: Option<&str>) -> Result<WorkspaceContext> {
    let declared = root.project().workspace.declared;
    let current = if declared {
        root.project().workspace.include_root.then_some(0)
    } else {
        Some(0)
    };
    let projects = if declared {
        let mounts = MountLoader::unscanned(&root, along);
        load_workspace_projects_through(&mut root, mounts)?
    } else {
        vec![WorkspaceProject {
            alias: String::new(),
            config: root.clone(),
            findings: Catalog::default(),
            scan_errors: Vec::new(),
            mount: None,
        }]
    };
    Ok(WorkspaceContext {
        projects,
        current,
        workspace_loaded: declared,
        render_root: root.frame().root().to_path_buf(),
        run_warnings: settled_run_warnings(root.run()),
        render_config: root,
    })
}

/// Where the fetch of one ID runs (§FS-imports.writes.answer). Its `Display` is what
/// follows `grund fetch` in a remedy: the one spelling of that command's operands.
pub(crate) enum FetchPlace {
    /// From where the run started: a local or workspace member's ID.
    Run { id: String },
    /// In the target of the last import edge on the alias path, read-only here.
    Import {
        qualified: String,
        import: String,
        id: String,
        target: String,
    },
}

/// §FS-imports.writes.answer: the place to fetch `local`, an ID of the project whose
/// run is `run`, cited through `namespace`, with the target spelled from `base`
/// (§FS-errors.4).
pub(crate) fn fetch_place(
    run: &Run,
    base: &Path,
    namespace: Option<&str>,
    local: &str,
) -> FetchPlace {
    let qualified = namespace.map_or_else(|| local.to_owned(), |name| format!("{name}/{local}"));
    let workspace = &run.workspace;
    let (Some(namespace), Some(root), Some(below)) = (
        namespace,
        workspace.mounted_root.as_deref(),
        workspace.import_below.as_deref(),
    ) else {
        return FetchPlace::Run { id: qualified };
    };
    let (import, id) = if below.is_empty() {
        (namespace, local.to_owned())
    } else {
        let import = namespace
            .strip_suffix(below)
            .map_or(namespace, |rest| rest.trim_end_matches('/'));
        (import, format!("{below}/{local}"))
    };
    FetchPlace::Import {
        qualified,
        import: import.to_owned(),
        id,
        target: format_path(&relative_from_base(base, root)),
    }
}

impl FetchPlace {
    /// §FS-imports.writes: `grund fetch`'s refusal of an ID in an import; `None` for
    /// one this run may fetch.
    pub(crate) fn import_refusal(&self) -> Option<String> {
        let Self::Import {
            qualified, import, ..
        } = self
        else {
            return None;
        };
        Some(format!(
            "`{qualified}` is in import `{import}`, read-only here — run grund fetch {self}"
        ))
    }
}

impl fmt::Display for FetchPlace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Run { id } => f.write_str(id),
            Self::Import { id, target, .. } => write!(f, "{id} in {target}"),
        }
    }
}
