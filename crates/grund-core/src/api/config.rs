//! The published config contract (§AR-system.2.9): the effective `Config` for a
//! path, the validation `grund config validate` is (§FS-config.4.1), and the
//! non-fatal `warning:` texts a loaded config carries — all as data, with no
//! CLI TOML rendered and no stream written (§AR-bindings.2).

/// Load the effective config for a path without rendering it as CLI TOML.
use anyhow::Result;
use std::path::Path;

use super::config_findings::config_diagnostics;
use super::report::public_run_warnings;
use crate::config::{Config, load_config};
use crate::model::Finding;
use crate::resolver::settled_run_warnings;
use crate::workspace::{expand_workspace_tree, resolve_workspace_config};

/// The run's `[workspace]` warnings off a `Config` a caller already holds
/// (§FS-check.4.7, §FS-check.4.10, §FS-workspace.6.1): what the workspace pass
/// settled on it, plus the walk §FS-check.4.10 needs, published as the
/// `Finding`s a frontend renders (§FS-distribution.3.1).
///
/// The channel rides on the config every walking command resolves, so a command
/// that hands one back — [`validate_config`], which loads every member's
/// (§FS-config.4.1.1) — needs no channel of its own.
#[doc(hidden)]
pub fn config_run_warnings(config: &Config) -> Vec<Finding> {
    public_run_warnings(config, settled_run_warnings(config))
}

pub fn effective_config(path: &Path) -> Result<Config> {
    load_config(path)
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
/// discovery pair (§FS-config.1.1, §FS-check.4.3) and the deprecated `.agents/`
/// location (§FS-config.1.2, §FS-check.4.11). Message text only, so
/// `grund config validate` and `grund config show` print the same sentences
/// `grund check` does without depending on the checker's report type.
pub fn config_warnings(config: &Config) -> Vec<String> {
    config_diagnostics(config)
        .map(|diagnostic| diagnostic.message)
        .collect()
}

/// Reference marker and typing trigger resolved for one path. §FS-lsp.1.4
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceStyle {
    pub marker: String,
    pub trigger: String,
}

/// Resolve the marker/trigger pair for a specific document. Workspace member
/// files use the member config, matching `grund fmt` and `grund check`
/// (§FS-lsp.1.4, §FS-workspace.5).
pub fn reference_style(path: &Path) -> Result<ReferenceStyle> {
    let config = resolve_workspace_config(path)?;
    Ok(ReferenceStyle {
        marker: config.marker,
        trigger: config.trigger,
    })
}
