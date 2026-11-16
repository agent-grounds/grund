//! The exhaustive form of `show --batch` (§FS-show.2.6.2): every coordinate the loaded
//! context declares, generated once and in one order, which `--all` reads whole and
//! `--glob` filters (§FS-show.1.9).

use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

use super::batch::BatchShowQuery;
use super::glob_scope::GlobSelection;
use super::report_scope::{reported_scope, reports};
use crate::grammar::render_catalog_id;
use crate::model::{Id, paired_declarations};
use crate::resolver::{WorkspaceContext, WorkspaceProject};

/// `id` as the exhaustive form spells it (§FS-show.2.6.2): the catalog's spelling
/// in `project`, unqualified when `project` is the current one and prefixed with its
/// alias and `/` otherwise, whatever alias a query reached it by.
pub(crate) fn generated_id(
    context: &WorkspaceContext,
    project: &WorkspaceProject,
    id: &Id,
) -> String {
    let local_id = render_catalog_id(&project.config.grammar, &project.findings.declarations, id);
    match context.current_project() {
        Some(current) if std::ptr::eq(current, project) => local_id,
        _ => format!("{}/{local_id}", project.alias),
    }
}

/// The header a succeeded query is named by in a `text` or `md` batch
/// (§FS-show.3.3.1): its [`generated_id`], then the section read, joined by the
/// separator of the project that declares the ID, so the header is a coordinate
/// a reader pastes back into `grund show`.
pub(crate) fn answered_coordinate(
    context: &WorkspaceContext,
    project: &WorkspaceProject,
    id: &Id,
    section: Option<&str>,
) -> String {
    let id = generated_id(context, project, id);
    match section {
        Some(section) => format!("{id}{}{section}", project.config.section_separator),
        None => id,
    }
}

/// Generate stable coordinates from the declarations and sections already in
/// the loaded context (§FS-show.2.6.2). BTree sets collapse duplicate claimants;
/// the final bytewise sort is over the public qualified spelling. A path narrower
/// than its config root bounds which declarations generate coordinates, as it
/// bounds the entries `list` prints, and every query still resolves against the
/// whole project (§FS-show.2.6.2, §AR-resolver.3.3). Globs keep the coordinates
/// they select, matched against the project-local ID (§FS-show.1.9).
pub(crate) fn exhaustive_batch_queries(
    context: &WorkspaceContext,
    path: &Path,
    path_provided: bool,
    globs: Option<&GlobSelection<'_>>,
) -> Result<Vec<BatchShowQuery>> {
    let selects = |project: usize, id: &str, section: Option<&str>| {
        globs.is_none_or(|globs| globs.selects(project, id, section))
    };
    let scope = reported_scope(context, path, path_provided)?;
    let mut queries = Vec::new();
    for (project_index, project) in context.projects.iter().enumerate() {
        for (id, declarations) in &project.findings.declarations {
            let records = paired_declarations(declarations)
                .records()
                .filter(|record| reports(scope.as_ref(), declarations, record))
                .collect::<Vec<_>>();
            if records.is_empty() {
                continue;
            }
            let local_id =
                render_catalog_id(&project.config.grammar, &project.findings.declarations, id);
            // §FS-show.3.3.1: the spelling a succeeded query's header reads too.
            let qualified_id = generated_id(context, project, id);
            if selects(project_index, &local_id, None) {
                queries.push(BatchShowQuery {
                    id: qualified_id.clone(),
                    section: None,
                });
            }
            // §FS-declarations.stubs.target.5: a paired declaration's sections are its
            // target's, scanned or not.
            let sections: BTreeSet<&str> = records
                .into_iter()
                .flat_map(|declaration| {
                    declaration.sections.keys().map(String::as_str).chain(
                        declaration
                            .duplicate_sections
                            .iter()
                            .map(|(path, _)| path.as_str()),
                    )
                })
                .collect();
            queries.extend(
                sections
                    .into_iter()
                    .filter(|section| selects(project_index, &local_id, Some(section)))
                    .map(|section| BatchShowQuery {
                        id: qualified_id.clone(),
                        section: Some(section.to_string()),
                    }),
            );
        }
    }
    queries.sort_by(|left, right| {
        left.id
            .as_bytes()
            .cmp(right.id.as_bytes())
            .then_with(|| left.section.as_deref().cmp(&right.section.as_deref()))
    });
    Ok(queries)
}
