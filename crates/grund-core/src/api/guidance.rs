//! Query repair hints shared with process frontends (§FS-show.3.5.1).

use anyhow::Result;
use std::path::Path;

use super::unread_configs::UnreadConfigs;
use crate::config::{Config, format_guidance};
use crate::workspace::{expand_workspace_tree, resolve_workspace_config, split_qualified_id_arg};

/// §FS-show.3.5.1: select the query's project before describing its format.
/// This only reads configuration; it neither resolves nor reclassifies the query.
#[doc(hidden)]
pub fn query_format_hint(path: &Path, operand: &str) -> Result<String> {
    let mut config = resolve_workspace_config(path)?;
    let Ok((alias, raw)) = split_qualified_id_arg(operand) else {
        return Ok(invalid_id_hint(&config, operand, None));
    };
    if let Some(alias) = alias {
        let projects = expand_workspace_tree(&mut config)?;
        if let Some(project) = projects.iter().find(|project| project.alias == alias) {
            return Ok(invalid_id_hint(&project.config, operand, Some(raw)));
        }
        return Ok(invalid_id_hint(&config, operand, None));
    }
    Ok(invalid_id_hint(&config, operand, Some(raw)))
}

/// §FS-show.3.5.1: the one hint a grammar-rejected `operand` earns, for every
/// query refusal adapter. It names the effective template of `selected`'s kind
/// in `config`, unless the run loaded no config and a project config lies below
/// its root: then the template is not the cause, so it names that config and
/// drops the `grund list` tail, which would list nothing there.
pub(super) fn invalid_id_hint(config: &Config, operand: &str, selected: Option<&str>) -> String {
    match UnreadConfigs::below(config, config.display()) {
        Some(unread) => format!(
            "hint: no grund.toml applies here, so grund read `{operand}` against its \
             defaults. {}: {}",
            unread.names(),
            unread.query_routes()
        ),
        None => format_guidance(config.schema(), config.frame(), selected).hint(),
    }
}
