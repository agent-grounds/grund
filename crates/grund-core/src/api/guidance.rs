//! Query repair hints shared with process frontends (§FS-show.3.5.1).

use anyhow::Result;
use std::path::Path;

use crate::config::format_guidance;
use crate::workspace::{expand_workspace_tree, resolve_workspace_config, split_qualified_id_arg};

/// §FS-show.3.5.1: select the query's project before describing its format.
/// This only reads configuration; it neither resolves nor reclassifies the query.
#[doc(hidden)]
pub fn query_format_hint(path: &Path, operand: &str) -> Result<String> {
    let mut config = resolve_workspace_config(path)?;
    let Ok((alias, raw)) = split_qualified_id_arg(operand) else {
        return Ok(format_guidance(config.schema(), config.frame(), None).hint());
    };
    if let Some(alias) = alias {
        let projects = expand_workspace_tree(&mut config)?;
        if let Some(project) = projects.iter().find(|project| project.alias == alias) {
            return Ok(
                format_guidance(project.config.schema(), project.config.frame(), Some(raw)).hint(),
            );
        }
        return Ok(format_guidance(config.schema(), config.frame(), None).hint());
    }
    Ok(format_guidance(config.schema(), config.frame(), Some(raw)).hint())
}
