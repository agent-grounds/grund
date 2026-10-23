//! The breadcrumb an empty ID-mode `refs` answer earns (§FS-refs.2.1): the typo
//! note for an ID the scanned tree declares nowhere, or the pointer to the
//! `list --unused` call that answers for every declaration at once
//! (§FS-refs.2.1.1). Decided once, as the `note` of the result, so the CLI, the
//! Rust API and both bindings carry the same text (§FS-distribution.3.2.2.2).

use std::collections::BTreeSet;
use std::path::Path;

use crate::grammar::{render_catalog_id, render_input_id};
use crate::model::Id;
use crate::queries::unused_leaves_out;
use crate::resolver::WorkspaceProject;

/// What an ID-mode query asked, beside the answer it got.
pub(super) struct NoteQuery<'a> {
    /// The explicit scan path, as supplied, when one was given (§FS-scan.path.refs).
    pub(super) path: Option<&'a Path>,
    /// The selected project, whose declarations decide which note applies.
    pub(super) target: &'a WorkspaceProject,
    /// The ID as the operand spelled it, alias removed.
    pub(super) raw_id: &'a str,
    pub(super) id: &'a Id,
    /// An alias-qualified query in a loaded workspace (§FS-workspace.8.2).
    pub(super) qualified: bool,
    /// Whether a section is in play, inline or by `--section`.
    pub(super) section: bool,
    /// Whether the scan read every file it walked (§FS-refs.4).
    pub(super) complete: bool,
}

/// The `note` of a `refs` answer that lists no site: the typo note when the
/// target project declares the ID nowhere (§FS-refs.2.1), else the
/// `list --unused` pointer where that listing answers the question asked, which
/// is a whole-ID query with no path over a complete scan (§FS-refs.2.1.1).
pub(super) fn empty_answer_note(query: &NoteQuery<'_>) -> Option<String> {
    if !query.target.findings.declarations.contains_key(query.id) {
        return Some(typo_note(query));
    }
    (!query.section && query.path.is_none() && query.complete).then(|| unused_note(query))
}

/// §FS-refs.2.1: the ID declared nowhere, and where to see what is declared.
fn typo_note(query: &NoteQuery<'_>) -> String {
    let alias = query.target.alias.as_str();
    // §FS-config.3.2.8: a missing query reports its own authored digits.
    let rendered_id = render_input_id(&query.target.config.grammar, query.raw_id, query.id);
    // §FS-refs.2.1: explicit paths qualify both the denial and the list advice.
    let (scan_scope, outside_scan) = match query.path {
        Some(path) => (
            format!(" in scan path `{}`", path.display()),
            ", including outside this scan",
        ),
        None => (String::new(), ""),
    };
    let (query_id, list_command, declarations) = if query.qualified {
        (
            format!("{alias}/{rendered_id}"),
            format!("grund list --project {alias}"),
            format!("{alias}'s declared IDs"),
        )
    } else {
        let declarations = if query.path.is_some() {
            "project declarations"
        } else {
            "every declared ID"
        };
        (
            rendered_id,
            "grund list".to_string(),
            declarations.to_string(),
        )
    };
    format!(
        "{query_id} is neither declared nor cited{scan_scope} — run `{list_command}` to see {declarations}{outside_scan}"
    )
}

/// §FS-refs.2.1.1: the declaration in its catalog spelling, and the
/// `list --unused` call whose listing holds it — its project's under an alias,
/// and with the case kind selected where the plain listing leaves it out.
fn unused_note(query: &NoteQuery<'_>) -> String {
    let alias = query.target.alias.as_str();
    let declarations = &query.target.findings.declarations;
    let catalog_id = render_catalog_id(&query.target.config.grammar, declarations, query.id);
    let (query_id, project) = if query.qualified {
        (
            format!("{alias}/{catalog_id}"),
            format!(" --project {alias}"),
        )
    } else {
        (catalog_id, String::new())
    };
    let kind = &query.id.kind;
    let kind_filter = if unused_leaves_out(kind, &BTreeSet::new()) {
        format!(" --kind {kind}")
    } else {
        String::new()
    };
    format!(
        "{query_id} is declared but nothing cites it; `grund list{project} --unused{kind_filter}` lists every such declaration"
    )
}
