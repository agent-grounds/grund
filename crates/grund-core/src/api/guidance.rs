//! Query repair hints shared with process frontends (§FS-show.3.5.1).

use anyhow::Result;
use std::path::Path;

use super::unread_configs::UnreadConfigs;
use crate::config::{Config, format_guidance};
use crate::resolver::{WorkspaceContext, names_member_id_candidate};
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

/// The `hint:` lines a refused `show` query earns below its message (§FS-show.3.5),
/// chosen once for the single read and the batch, so the two print the same lines
/// for the same failure (§FS-show.3.3.2). Every input is read from `context`, the
/// run's one load: the project `id_arg` names, its alias's or the current one,
/// supplies the ID format an invalid ID is hinted with and the separator a missing
/// section is cut at, and no configuration is resolved again (§AR-resolver.3.1.1).
/// `id_arg` is the query's `id` as written; a section passed beside it is not part
/// of it, so such a query is hinted as the `--section` read is.
pub(crate) fn query_failure_hints(
    context: &WorkspaceContext,
    id_arg: &str,
    message: &str,
) -> Vec<String> {
    let named = split_qualified_id_arg(id_arg).ok();
    let project = match named.as_ref().map(|(alias, _)| alias.as_deref()) {
        Some(Some(alias)) => context
            .project_by_alias(alias)
            .map(|project| &project.config),
        _ => context.current_project().map(|project| &project.config),
    };
    let config: &Config = project.unwrap_or_else(|| context.render_config());
    if message.starts_with("invalid ID") {
        // §FS-show.3.5.1: an alias that names no project leaves the run's own template.
        let selected = match &named {
            Some((Some(_), raw)) if project.is_some() => Some(*raw),
            Some((None, raw)) => Some(*raw),
            _ => None,
        };
        vec![invalid_id_hint(config, id_arg, selected)]
    } else if message.starts_with("ID not found:") {
        // §FS-show.3.5: the hint gives way where the line already names the
        // project that declares the ID (§FS-workspace.8.1.1.7).
        match names_member_id_candidate(message) {
            true => Vec::new(),
            false => vec![
                "hint: run `grund list` to see every declared ID, or `grund id <KIND> \"<title>\"` \
                 to propose a new one"
                    .to_string(),
            ],
        }
    } else if message.starts_with("section not found:") {
        // §FS-show.3.5: cut at the separator of the project the query names.
        let base_id = id_arg
            // coordinate-lexer: hint text for a refusal already made
            .rsplit_once(config.section_separator.as_str())
            .map_or(id_arg, |(base, _)| base);
        vec![format!(
            "hint: run `grund {base_id} --toc` to print the lead with the section map"
        )]
    } else {
        Vec::new()
    }
}
