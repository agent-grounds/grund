//! Bounded host path protection, before alias derivation (§FS-distribution.3.3.3).

use super::members::expand_workspace_member_list;
use crate::config::{Config, load_config, load_config_at};
use crate::model::OperationDiagnostic;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

/// Use the actual workspace member discovery; never invent a second glob walker.
/// Existing process entrypoints do not opt into this preflight (§FS-distribution.3.1).
pub(crate) fn preflight_embedding_paths(path: &Path) -> Result<()> {
    let config = load_config(path)?;
    let mut seen = BTreeSet::new();
    visit(config.clone(), &mut seen)?;
    // §FS-distribution.3.3.3: narrowed calls can derive aliases from claiming ancestors.
    let mut claims = super::members::AncestorWorkspaces::quiet_for_run_at(&config.root);
    let mut child = config.root.clone();
    for ancestor in config.root.ancestors().skip(1) {
        let canonical = super::members::canonical_workspace_path(&child);
        if let Ok(Some(parent)) =
            claims.claiming_block(ancestor, &child, &canonical, &config.cli_base)
        {
            let parent = parent.clone();
            child = parent.root.clone();
            visit(parent, &mut seen)?;
        }
    }
    Ok(())
}

fn visit(config: Config, seen: &mut BTreeSet<std::path::PathBuf>) -> Result<()> {
    if !config.workspace_declared || !seen.insert(config.root.clone()) {
        return Ok(());
    }
    // Ordinary discovery/validation refusals remain the operation's responsibility:
    // they stop before alias derivation and must keep that operation's cautions.
    let Ok(expanded) = expand_workspace_member_list(&config) else {
        return Ok(());
    };
    for member in expanded.members {
        let Ok(child) = load_config_at(&member.root, &config.cli_base) else {
            continue;
        };
        if !member.optional
            && child.project_name.is_none()
            && member.root.file_name().and_then(|n| n.to_str()).is_none()
        {
            return Err(OperationDiagnostic::new(
                "path-encoding",
                "path-encoding",
                "unnamed workspace member has a non-Unicode basename; configure project_name",
            )
            .into());
        }
        visit(child, seen)?;
    }
    Ok(())
}
