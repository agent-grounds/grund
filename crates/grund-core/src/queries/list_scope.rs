//! The scope check `list` and `list --size` both run before they read a row
//! (§FS-list.1, §FS-workspace.8.3.2): the `--project` aliases, the `--kind`
//! lookup and its refusal, and the `--selector` parsed against the selected
//! projects' vocabulary. One copy, so the two modes cannot answer one selection
//! two ways (§FS-list.1.2). It lives here rather than beside `api/list.rs`
//! because the size catalog in this component reads it too (§AR-system.4).

use anyhow::{Result, anyhow};
use std::collections::{BTreeMap, BTreeSet};

use super::selector_refusal::selector_refusal;
use crate::config::non_citable_kind_error;
use crate::resolver::WorkspaceContext;
use crate::rules::sentence::{RuleSubject, RuleVocabulary, parse_selector};

/// Check a `list` run's `--project`, `--kind` and `--selector` against the
/// projects it selects, and return the parsed selector. An empty
/// `project_filter` selects every loaded project, and every refusal that names
/// kinds ends in that selection's `known kinds:` line (§FS-list.1.2).
pub(crate) fn check_list_scope(
    context: &WorkspaceContext,
    project_filter: &BTreeSet<String>,
    kind_filter: &BTreeSet<String>,
    selector: Option<&str>,
) -> Result<Option<RuleSubject>> {
    check_project_aliases(context, project_filter)?;
    let known_kinds = || context.known_kinds_line(project_filter);
    let selected_projects = || {
        context
            .projects
            .iter()
            .filter(|project| project_filter.is_empty() || project_filter.contains(&project.alias))
    };
    for kind in kind_filter {
        // §FS-list.1.1: accepted when any selected project has the kind citable.
        let entries = || {
            selected_projects()
                .flat_map(|project| &project.config.kinds)
                .filter(|entry| &entry.kind == kind)
        };
        if entries().any(|entry| entry.citable) {
            continue;
        }
        // Configured but citable nowhere selected: the reason, named by the
        // first such entry in load order, rather than "unknown" (§FS-list.1.1).
        let headline = match entries().next() {
            Some(entry) => non_citable_kind_error(entry),
            None => format!("unknown kind `{kind}`"),
        };
        return Err(anyhow!("{headline}\n{}", known_kinds()));
    }
    let Some(raw) = selector else {
        return Ok(None);
    };
    let kinds = selected_projects()
        .flat_map(|project| project.config.kinds.iter())
        .filter(|kind| kind.citable)
        .map(|kind| kind.kind.clone())
        .collect::<BTreeSet<_>>();
    let vocabulary = RuleVocabulary {
        kinds: kinds.clone(),
        target_kinds: kinds,
        target_namespaces: BTreeMap::new(),
        named_sections: selected_projects().all(|project| project.config.named_sections),
        id_grammars: selected_projects()
            .map(|project| project.config.grammar.clone())
            .collect(),
        section_separators: selected_projects()
            .map(|project| project.config.section_separator.clone())
            .collect(),
    };
    // §FS-rules.8.1: a refusal recovering no kind ends in `known kinds:`.
    parse_selector(raw, &vocabulary)
        .map(Some)
        .map_err(|refusal| selector_refusal(refusal.render(&known_kinds())))
}

/// `--project` needs workspace mode and names only loaded aliases
/// (§FS-list.1.2).
fn check_project_aliases(
    context: &WorkspaceContext,
    project_filter: &BTreeSet<String>,
) -> Result<()> {
    if !project_filter.is_empty() && !context.workspace_loaded {
        return Err(anyhow!(
            "--project requires workspace mode (no [workspace] block discovered)"
        ));
    }
    for alias in project_filter {
        if context.project_by_alias(alias).is_none() {
            let known = context.aliases().join(", ");
            return if known.is_empty() {
                Err(anyhow!("unknown project alias `{alias}`"))
            } else {
                Err(anyhow!(
                    "unknown project alias `{alias}`\nknown aliases: {known}"
                ))
            };
        }
    }
    Ok(())
}
