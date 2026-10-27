//! The published `list` contract and the catalog behind it (§AR-system.2.9): the
//! declaration rows and the per-kind summary rows, with no output format chosen
//! and no exit code mapped (§FS-list.2, §FS-list.3, §AR-bindings.2).
//!
//! Its own file since before the component was a module, so the list contract
//! could grow without the contract file growing with it
//! (§AR-core-module-layout.3). The citation counts each row's `refs` is read off
//! are the catalog query's, read downward (§FS-list.3.2.1).

use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::config::{Config, KindConfig, display_path};
use crate::grammar::{field_value, render_declaration_id};
use crate::model::{
    Declaration, FileRecords, Finding, Id, format_path, paired_declarations, sort_path_key,
};
use crate::queries::{
    GlobSelection, ListCitationCounts, check_list_scope, read_globs, reported_scope, reports,
    require_unique_literal, unused_leaves_out,
};
use crate::resolver::{WorkspaceProject, load_reported_workspace_context};

use super::report::context_run_warnings;
use crate::scanner::{api_scan_error, section_row_title};

pub use super::list_output::{ListEntry, ListOutput, ListSummary, ListValueRoot};

#[derive(Clone)]
#[non_exhaustive]
pub struct ListOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub kind_filter: BTreeSet<String>,
    pub project_filter: BTreeSet<String>,
    pub unused_only: bool,
    /// Optional declaration/chapter selector (§FS-rules.8).
    pub selector: Option<String>,
    /// Coordinate globs, unioned; empty lists declarations as before
    /// (§FS-coordinate-globs.matching.union).
    pub globs: Vec<String>,
}

impl Default for ListOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
            kind_filter: BTreeSet::new(),
            project_filter: BTreeSet::new(),
            unused_only: false,
            selector: None,
            globs: Vec::new(),
        }
    }
}

/// §FS-list.3.3: a kind's home as the summary prints it — every place of its row,
/// comma-separated in written order (§FS-config-v2.schema.places); one place
/// under v1.
fn list_summary_home(config: &Config, kind: &KindConfig) -> String {
    let row = config
        .schema()
        .rows
        .iter()
        .find(|row| row.name == kind.kind);
    let places: Vec<&str> = row
        .into_iter()
        .flat_map(|row| row.places.iter().filter_map(|place| place.path()))
        .collect();
    places.join(", ")
}

/// Programmatic `list`: return the catalog and per-kind summary rows without
/// selecting text/JSON rendering or an exit code (§AR-bindings.2).
pub fn list(opts: ListOpts) -> Result<ListOutput> {
    list_with_run_warnings(opts).1
}

/// [`list`] for a frontend that renders the run's `[workspace]` warnings even
/// when the query is refused (§FS-check.4.10.7, §FS-workspace.6.1.7).
///
/// A refusal `list` settles *after* the workspace pass — an unknown project alias,
/// an unknown kind — leaves an `Err` with nowhere to carry a caution the reader is
/// already owed, and the run said it before the query failed. On success the list
/// returned here is the one [`ListOutput::warnings`] carries.
#[doc(hidden)]
pub fn list_with_run_warnings(opts: ListOpts) -> (Vec<Finding>, Result<ListOutput>) {
    let mut warnings = Vec::new();
    let output = list_run(opts, &mut warnings);
    (warnings, output)
}

fn list_run(opts: ListOpts, run_warnings: &mut Vec<Finding>) -> Result<ListOutput> {
    // §FS-coordinate-globs.errors.shape: what the grammar alone refuses, before discovery.
    let globs = read_globs(&opts.globs)?;
    // §AR-resolver.3.3: resolved against the whole owning project, and the path
    // bounds only which entries print.
    let context = load_reported_workspace_context(&opts.path, opts.path_provided)?;
    *run_warnings = context_run_warnings(&context);
    // §FS-list.1.2: aliases, kinds and selector are checked against the selection.
    let selector = check_list_scope(
        &context,
        &opts.project_filter,
        &opts.kind_filter,
        opts.selector.as_deref(),
    )?;
    let globs = match globs.is_empty() {
        true => None,
        false => Some(GlobSelection::new(&context, &globs)?),
    };
    let reported = reported_scope(&context, &opts.path, opts.path_provided)?;

    struct Entry<'a> {
        project_index: usize,
        project_alias: &'a str,
        project_config: &'a Config,
        files: &'a FileRecords,
        id: &'a Id,
        home: &'a Declaration,
        duplicate: bool,
        refs: usize,
    }

    let counts = ListCitationCounts::new(&context);
    let mut entries: Vec<Entry<'_>> = Vec::new();
    let mut scan_errors = Vec::new();
    for (project_index, project) in context.projects.iter().enumerate() {
        if !opts.project_filter.is_empty() && !opts.project_filter.contains(&project.alias) {
            continue;
        }
        // §FS-workspace.8.7.3: rendered against the run's config, like the entries
        // below, not the scanning project's — the same spelling `check` uses.
        scan_errors.extend(
            project.scan_errors.iter().map(|(file, message)| {
                api_scan_error(context.render_config().frame(), file, message)
            }),
        );
        let ref_counts = counts.refs_for(project.alias.as_str());
        let used_counts = counts.used_for(project.alias.as_str());
        for (id, decls) in &project.findings.declarations {
            if !opts.kind_filter.is_empty() && !opts.kind_filter.contains(&id.kind) {
                continue;
            }
            let refs = ref_counts.get(id).copied().unwrap_or(0);
            if opts.unused_only && used_counts.get(id).copied().unwrap_or(0) > 0 {
                continue;
            }
            // §FS-list.1.3.1: a configured `E2E` kind is left out unless `--kind` selects it.
            if opts.unused_only && unused_leaves_out(&id.kind, &opts.kind_filter) {
                continue;
            }
            // §AR-scanner.4.6: the declarations every command counts, each at its record,
            // so a paired declaration is its target's, one row per declaration the target
            // holds, wherever it sits (§FS-declarations.stubs.pairing.4, §FS-list.2.5).
            let paired = paired_declarations(decls);
            let duplicate = paired.len() > 1;
            // §AR-resolver.3.3: the path's entries, each with the project's answers.
            for home in paired
                .records()
                .filter(|home| reports(reported.as_ref(), decls, home))
            {
                entries.push(Entry {
                    project_index,
                    project_alias: project.alias.as_str(),
                    project_config: &project.config,
                    files: &project.findings.files,
                    // §FS-config.3.2.8: report the selected home, including a stub's target.
                    id: &home.id,
                    home,
                    duplicate,
                    refs,
                });
            }
        }
    }
    if context.workspace_loaded {
        entries.sort_by(|a, b| {
            (
                a.project_alias,
                a.id,
                sort_path_key(&a.home.file),
                a.home.line,
            )
                .cmp(&(
                    b.project_alias,
                    b.id,
                    sort_path_key(&b.home.file),
                    b.home.line,
                ))
        });
    }

    // Validate the selector before producing rows: kind selectors may span members; an exact
    // literal must resolve once, as a rule subject does (§FS-rules.8), counted by the one
    // subject matcher (§AR-rules.2).
    if let Some(subject) = &selector
        && let Some(literal) = subject.literal()
    {
        let exact_matches = entries
            .iter()
            .map(|entry| {
                let label = render_declaration_id(&entry.project_config.grammar, entry.home);
                let kind = &entry.id.kind;
                // A literal carries no whose clause, so it reads no field.
                usize::from(subject.selects(kind, &label, None, None))
                    + subject.chapter_path(kind, &label, None).map_or(0, |path| {
                        usize::from(entry.home.sections.contains_key(path))
                            + entry
                                .home
                                .duplicate_sections
                                .iter()
                                .filter(|(section, _)| section == path)
                                .count()
                    })
            })
            .sum();
        require_unique_literal(&literal, exact_matches)?;
    }

    let render_qualified = |entry: &Entry<'_>| -> String {
        if context.workspace_loaded {
            format!(
                "{}/{}",
                entry.project_alias,
                render_declaration_id(&entry.project_config.grammar, entry.home)
            )
        } else {
            render_declaration_id(&entry.project_config.grammar, entry.home)
        }
    };
    let render_config = context.render_config();
    let public_entries = entries
        .iter()
        .flat_map(|entry| {
            let rendered_id = render_qualified(entry);
            let base = || ListEntry {
                project: context
                    .workspace_loaded
                    .then(|| entry.project_alias.to_string()),
                id: rendered_id.clone(),
                section: None,
                section_separator: entry.project_config.section_separator.clone(),
                kind: entry.id.kind.clone(),
                path: display_path(render_config, &entry.home.file),
                line: entry.home.line,
                title: entry.home.title.clone(),
                stub: entry.home.is_stub,
                defines: entry
                    .home
                    .defined_in
                    .as_ref()
                    .map(|target| format_path(target)),
                refs: entry.refs,
                duplicate: entry.duplicate,
                value_roots: entry
                    .home
                    .sections
                    .iter()
                    .filter_map(|(section, info)| {
                        info.value_root.as_ref().map(|root| ListValueRoot {
                            id: format!(
                                "{}{}{}",
                                render_qualified(entry),
                                entry.project_config.section_separator,
                                section
                            ),
                            valid: root.valid,
                        })
                    })
                    .collect(),
            };
            let label = render_declaration_id(&entry.project_config.grammar, entry.home);
            // §FS-rules.8: a whose clause is matched against the field's display name.
            let value = selector
                .as_ref()
                .and_then(|subject| subject.whose_field())
                .and_then(|field| field_value(entry.home, field));
            let value = value.as_deref();

            // §FS-rules.8: the one subject matcher (§AR-rules.2) says what a row is, and a
            // chapter subject names one whole path, so each declaration contributes the
            // declaration or at most the one chapter at it.
            let declaration_selected = selector
                .as_ref()
                .is_none_or(|subject| subject.selects(&entry.id.kind, &label, None, value));
            let chapter = selector
                .as_ref()
                .filter(|_| !declaration_selected)
                .and_then(|subject| subject.chapter_path(&entry.id.kind, &label, value));
            let section_row = |(section, info): (&String, &crate::model::SectionInfo)| {
                let mut row = base();
                row.section = Some(section.clone());
                row.line = info.line;
                row.title = Some(section_row_title(entry.files, entry.home, section, info));
                row.stub = false;
                row.defines = None;
                row.refs = 0;
                row.duplicate = false;
                row.value_roots.clear();
                row
            };
            let Some(globs) = &globs else {
                return match chapter {
                    None if declaration_selected => vec![base()],
                    chapter => chapter
                        .and_then(|path| entry.home.sections.get_key_value(path))
                        .map(section_row)
                        .into_iter()
                        .collect(),
                };
            };
            // §FS-coordinate-globs.scope.filters: the glob intersects the selector, and a
            // declaration's rows are the declaration, then its sections in path order.
            let glob_selects =
                |section: Option<&str>| globs.selects(entry.project_index, &label, section);
            let declaration = (declaration_selected && glob_selects(None)).then(base);
            let sections = entry
                .home
                .sections
                .iter()
                .filter(|(section, _)| {
                    (selector.is_none() || chapter == Some(section.as_str()))
                        && glob_selects(Some(section))
                })
                .map(section_row);
            declaration.into_iter().chain(sections).collect()
        })
        .collect::<Vec<_>>();

    let mut summaries = Vec::new();
    if context.workspace_loaded {
        let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
        for entry in &public_entries {
            if let Some(project) = &entry.project {
                *counts
                    .entry((project.clone(), entry.kind.clone()))
                    .or_insert(0) += 1;
            }
        }
        // §FS-workspace.8.3.4: rows sorted by alias — the same byte-wise `str`
        // order the catalog above sorts `entries` by — then by that
        // project's configured kind order.
        let mut projects: Vec<&WorkspaceProject> = context.projects.iter().collect();
        projects.sort_by(|a, b| a.alias.cmp(&b.alias));
        for project in projects {
            if !opts.project_filter.is_empty() && !opts.project_filter.contains(&project.alias) {
                continue;
            }
            for kind in &project.config.kinds {
                let count = counts
                    .get(&(project.alias.clone(), kind.kind.clone()))
                    .copied()
                    .unwrap_or(0);
                if count == 0 {
                    continue;
                }
                summaries.push(ListSummary {
                    project: Some(project.alias.clone()),
                    kind: kind.kind.clone(),
                    title: kind
                        .title
                        .clone()
                        .unwrap_or_else(|| "Declaration".to_string()),
                    home: list_summary_home(&project.config, kind),
                    count,
                });
            }
        }
    } else {
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for entry in &public_entries {
            *counts.entry(&entry.kind).or_insert(0) += 1;
        }
        for kind in &render_config.kinds {
            let count = counts.get(kind.kind.as_str()).copied().unwrap_or(0);
            if count == 0 {
                continue;
            }
            summaries.push(ListSummary {
                project: None,
                kind: kind.kind.clone(),
                title: kind
                    .title
                    .clone()
                    .unwrap_or_else(|| "Declaration".to_string()),
                home: list_summary_home(render_config, kind),
                count,
            });
        }
    }

    Ok(ListOutput {
        output_format: render_config.output_format.clone(),
        workspace: context.workspace_loaded,
        entries: public_entries,
        summaries,
        scan_errors,
        note: globs.and_then(|globs| globs.note()),
        warnings: run_warnings.clone(),
    })
}
