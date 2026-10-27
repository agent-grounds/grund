//! The scope check `list` and `list --size` both run before they read a row
//! (§FS-list.1, §FS-workspace.8.3.2): the `--project` aliases, the `--kind`
//! lookup and its refusal, and the `--selector` parsed against the selected
//! projects' vocabulary. One copy, so the two modes cannot answer one selection
//! two ways (§FS-list.1.2). It lives here rather than beside `api/list.rs`
//! because the size catalog in this component reads it too (§AR-system.4).

use anyhow::{Result, anyhow};
use std::collections::BTreeSet;

use super::selector_refusal::selector_refusal;
use crate::config::{KindScope, KindTable, KindVerdict, Quote};
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
    // §AR-config.1.5: every kind name below is answered from the run's
    // one table, in the projects this run selects.
    let table = context.kind_table();
    let scope = KindScope::Selected(project_filter);
    for kind in kind_filter {
        // §FS-list.1.1: accepted when any selected project has the kind citable;
        // configured but citable nowhere selected is refused for that reason,
        // named by the first such entry in load order (§FS-errors.2.5.1).
        let headline = match table.verdict(scope, kind) {
            KindVerdict::Citable => continue,
            KindVerdict::NonCitable(entry) => entry.non_citable_reason(Quote::Backtick),
            // §FS-list.1.1: `code` with no row is refused for its reason alone (§FS-errors.3.8.1).
            _ => table.unknown_or_complement(
                scope,
                kind,
                format!("unknown kind `{kind}`"),
                Quote::Backtick,
            ),
        };
        return Err(anyhow!("{headline}\n{}", table.known_kinds_line(scope)));
    }
    let Some(raw) = selector else {
        return Ok(None);
    };
    let selected_projects = || {
        context
            .projects
            .iter()
            .filter(|project| project_filter.is_empty() || project_filter.contains(&project.alias))
    };
    let kinds = table
        .citable(scope)
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let vocabulary = RuleVocabulary {
        kinds,
        // A selector names no object (§AR-rules.1).
        objects: KindTable::default(),
        named_sections: table.named_sections(scope),
        // §AR-rules.1: `KIND.NAME` reads named sections under the
        // project that configures `KIND`.
        named_kinds: table
            .named_section_kinds(scope)
            .into_iter()
            .map(str::to_string)
            .collect(),
        // §FS-rules.2.2: a whose clause is judged under the project that configures its kind.
        fields: table.subject_fields(scope),
        non_citable: table.non_citable_entries(scope).into(),
        id_grammars: selected_projects()
            .map(|project| project.config.grammar.clone())
            .collect(),
        section_separators: selected_projects()
            .map(|project| project.config.section_separator.clone())
            .collect(),
    };
    // §AR-rules.1: a refusal recovering no kind ends in `known kinds:`.
    parse_selector(raw, &vocabulary)
        .map(Some)
        .map_err(|refusal| selector_refusal(refusal.render(&table.known_kinds_line(scope))))
}

/// Whether `--unused` under the `--kind` selection `kind_filter` leaves a
/// declaration of `kind` out though nothing cites it: an `E2E` case, unless
/// that kind is selected (§FS-list.1.3.1). `list`, `list --size` and the `refs`
/// note that names the listing (§FS-refs.2.1.1) all read this one rule.
pub(crate) fn unused_leaves_out(kind: &str, kind_filter: &BTreeSet<String>) -> bool {
    kind == "E2E" && !kind_filter.contains(kind)
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
