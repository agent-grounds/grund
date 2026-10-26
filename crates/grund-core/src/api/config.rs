//! The published config contract (§AR-system.2.9): the effective `Project` for
//! a path and the `Config` façade over it (§AR-bindings.2), the validation `grund config validate` is (§FS-config.4.1), and the
//! non-fatal `warning:` texts a loaded config carries — all as data, with no
//! CLI TOML rendered and no stream written (§AR-bindings.2).

/// Load the effective config for a path without rendering it as CLI TOML.
use anyhow::Result;
use std::path::Path;

use super::config_findings::config_diagnostics;
use super::report::public_run_warnings;
use crate::config::explain::{ExplainAsk, Explained};
use crate::config::{
    Config, Invocation, Project, ScaffoldNote, SchemaView, Selection, config_file_read,
};
use crate::model::Finding;
use crate::resolver::settled_run_warnings;
use crate::workspace::{expand_workspace_tree, open, resolve_workspace_config};
use crate::writers::scaffold_project;

/// The run's `[workspace]` warnings off a `Config` a caller already holds
/// (§FS-check.4.10, §FS-workspace.6.1.7.6): what the workspace pass
/// settled on it, plus the walk §FS-check.4.10.2 needs, published as the
/// `Finding`s a frontend renders (§FS-distribution.3.1).
///
/// The channel rides on the config every walking command resolves, so a command
/// that hands one back — [`validate_config`], which loads every member's
/// (§FS-config.4.1.1) — needs no channel of its own.
#[doc(hidden)]
pub fn config_run_warnings(config: &Config) -> Vec<Finding> {
    public_run_warnings(config, settled_run_warnings(config.run()))
}

pub fn effective_config(path: &Path) -> Result<Config> {
    open(&Invocation::new(path, true, Selection::Discovery))
}

/// The project the config for `path` lowers to, every default applied and no
/// per-run fact in it (§AR-config.1.1, §AR-bindings.2): what an embedder reads
/// in place of the `Config` façade, which is on its way out (§AR-config.5).
pub fn effective_project(path: &Path) -> Result<Project> {
    Ok(effective_config(path)?.project().clone())
}

/// The schema view of the project the config for `path` describes
/// (§FS-schema, §AR-config.6.3): discovered as [`effective_config`] discovers it
/// (§FS-schema.2), lowered from its `Project` and nothing else — no scan, so a
/// tree whose citations are broken shows the same view (§FS-schema.3). The
/// header's config path is spelled from the config root, whatever the run's
/// `--path-base` (§FS-schema.1), and a nameless project is named by its root.
/// Where no config file was found, the view carries what `grund init --scaffold`
/// would write there beside it (§FS-schema.8.2).
pub fn schema_view(path: &Path) -> Result<SchemaView> {
    let config = effective_config(path)?;
    let mut view = SchemaView::of_config(&config);
    if config_file_read(&config).is_none() {
        let scaffold = scaffold_project(&config.root, &view.project)?;
        view.scaffold = Some(ScaffoldNote::of(&view, &scaffold));
    }
    Ok(view)
}

/// Validate config discovery/parsing for a path without printing CLI output
/// (§FS-config.4.1). Discovery *is* the validation for one project — a config
/// that loads is a config that is valid. At a workspace root the config the
/// run loads includes every member's, so validation loads them too
/// (§FS-config.4.1.1): every member config the run would load, and the
/// `members` list itself, is validated here exactly as `expand_workspace_tree`
/// validates it for `grund check` — no tree scan. Returns the resolved config
/// so a caller can also report the non-fatal findings [`config_warnings`]
/// carries.
pub fn validate_config(path: &Path) -> Result<Config> {
    let mut config = resolve_workspace_config(path)?;
    if config.workspace_declared {
        expand_workspace_tree(&mut config)?;
    }
    Ok(config)
}

/// The CLI-level `warning:` texts a loaded config carries: the redundant
/// discovery pair (§FS-config.1.1.1, §FS-check.4.3.2) and the deprecated `.agents/`
/// location (§FS-config.1.2, §FS-check.4.11.2). Message text only, so
/// `grund config validate` and `grund config show` print the same sentences
/// `grund check` does without depending on the checker's report type.
pub fn config_warnings(config: &Config) -> Vec<String> {
    config_diagnostics(config, config.display())
        .map(|diagnostic| diagnostic.message)
        .collect()
}

/// Reference marker and typing trigger resolved for one path. §FS-lsp.1.4.6
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceStyle {
    pub marker: String,
    pub trigger: String,
}

/// Resolve the marker/trigger pair for a specific document. Workspace member
/// files use the member config, matching `grund fmt` and `grund check`
/// (§FS-lsp.1.4.6, §FS-workspace.5.1).
pub fn reference_style(path: &Path) -> Result<ReferenceStyle> {
    let config = resolve_workspace_config(path)?;
    Ok(ReferenceStyle {
        marker: config.marker,
        trigger: config.trigger,
    })
}

/// `grund config explain` (§FS-config-explain): what a table, a key or a check
/// code is, in the version the config discovered from `path` is read as, or in
/// both with no config, version 2 first (§FS-config-explain.2). It reads that
/// file's version and nothing else, and runs nothing the repository configures
/// (§FS-config-explain.6); the answer is data, which a frontend renders.
#[doc(hidden)]
pub fn config_explain(path: &Path, ask: &ExplainAsk) -> Result<Explained> {
    crate::config::explain::explain(path, ask)
}

/// The tables and keys the reader of configuration `version` admits, `[header]`
/// or `""` for the top level, and the codes `check` can report: what `grund
/// config explain` answers for (§FS-config-explain.3, §FS-config-explain.4),
/// read by the coverage test that holds every one of them answered
/// (§REQ-discoverable-offline.3).
#[doc(hidden)]
pub fn explain_vocabulary(version: u32) -> (Vec<(String, Vec<String>)>, Vec<&'static str>) {
    let codes = crate::model::Code::ALL
        .iter()
        .map(|code| code.as_str())
        .collect();
    (crate::config::admitted(version), codes)
}
