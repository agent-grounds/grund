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
use crate::grammar::{render_id, section_display_name};
use crate::model::{Declaration, Finding, Id, format_path, is_stub_for_inline_decl, sort_path_key};
use crate::queries::{ListCitationCounts, check_list_scope, require_unique_literal};
use crate::resolver::{WorkspaceProject, load_workspace_context};
use crate::rules::sentence::RuleSubject;

use super::report::context_run_warnings;
use crate::scanner::api_scan_error;

pub use super::list_output::{ListEntry, ListOutput, ListSummary, ListValueRoot};

#[derive(Clone)]
pub struct ListOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub kind_filter: BTreeSet<String>,
    pub project_filter: BTreeSet<String>,
    pub unused_only: bool,
    /// Optional declaration/chapter selector (§FS-rules.8).
    pub selector: Option<String>,
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
        }
    }
}

fn list_summary_home(kind: &KindConfig) -> String {
    kind.file
        .as_deref()
        .or(kind.folder.as_deref())
        .unwrap_or_default()
        .to_string()
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
    let context = load_workspace_context(&opts.path, opts.path_provided)?;
    *run_warnings = context_run_warnings(&context);
    // §FS-list.1.2: aliases, kinds and selector are checked against the selection.
    let selector = check_list_scope(
        &context,
        &opts.project_filter,
        &opts.kind_filter,
        opts.selector.as_deref(),
    )?;

    struct Entry<'a> {
        project_alias: &'a str,
        project_config: &'a Config,
        id: &'a Id,
        home: &'a Declaration,
        duplicate: bool,
        refs: usize,
    }

    let counts = ListCitationCounts::new(&context);
    let mut entries: Vec<Entry<'_>> = Vec::new();
    let mut scan_errors = Vec::new();
    for project in &context.projects {
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
            if opts.unused_only && id.kind == "E2E" && !opts.kind_filter.contains("E2E") {
                continue;
            }
            let mut homes: Vec<&Declaration> = decls
                .iter()
                .filter(|decl| !is_stub_for_inline_decl(&project.config.root, decl, decls))
                .collect();
            homes.sort_by(|a, b| {
                (sort_path_key(&a.file), a.line).cmp(&(sort_path_key(&b.file), b.line))
            });
            // §FS-declarations.checks.duplicate.2: a stub whose target declares the ID
            // twice is one row, and two homes.
            let duplicate = homes.iter().flat_map(|home| home.home_sites()).count() > 1;
            for home in homes {
                entries.push(Entry {
                    project_alias: project.alias.as_str(),
                    project_config: &project.config,
                    id,
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

    // Validate the selector against the selected catalog before producing any
    // rows. Kind selectors may span workspace members; an exact literal must
    // resolve once, just as a rule subject does (§FS-rules.2, §FS-rules.8).
    if let Some(subject) = &selector {
        let resolution = match subject {
            RuleSubject::ExactDeclaration(literal) => Some((
                literal.clone(),
                entries
                    .iter()
                    .filter(|entry| render_id(&entry.project_config.grammar, entry.id) == *literal)
                    .count(),
            )),
            RuleSubject::ExactChapter {
                declaration,
                path,
                separator,
            } => Some((
                format!("{declaration}{separator}{path}"),
                entries
                    .iter()
                    .filter(|entry| {
                        render_id(&entry.project_config.grammar, entry.id) == *declaration
                    })
                    .map(|entry| {
                        usize::from(entry.home.sections.contains_key(path))
                            + entry
                                .home
                                .duplicate_sections
                                .iter()
                                .filter(|(section, _)| section == path)
                                .count()
                    })
                    .sum(),
            )),
            _ => None,
        };
        if let Some((literal, exact_matches)) = resolution {
            require_unique_literal(&literal, exact_matches)?;
        }
    }

    let render_qualified = |entry: &Entry<'_>| -> String {
        if context.workspace_loaded {
            format!(
                "{}/{}",
                entry.project_alias,
                render_id(&entry.project_config.grammar, entry.id)
            )
        } else {
            render_id(&entry.project_config.grammar, entry.id)
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
            match &selector {
                None => vec![base()],
                Some(RuleSubject::Kind(kind)) if kind == &entry.id.kind => {
                    vec![base()]
                }
                Some(RuleSubject::ExactDeclaration(subject))
                    if subject == &render_id(&entry.project_config.grammar, entry.id) =>
                {
                    vec![base()]
                }
                // §FS-rules.2.1: a chapter subject names one whole path, so each
                // declaration contributes at most the one chapter at it.
                Some(subject) => subject
                    .chapter_path(
                        &entry.id.kind,
                        &render_id(&entry.project_config.grammar, entry.id),
                    )
                    .and_then(|path| entry.home.sections.get_key_value(path))
                    .map(|(section, info)| {
                        let mut row = base();
                        row.section = Some(section.clone());
                        row.line = info.line;
                        row.title = Some(section_display_name(&info.title, section).to_string());
                        row.stub = false;
                        row.defines = None;
                        row.refs = 0;
                        row.duplicate = false;
                        row.value_roots.clear();
                        row
                    })
                    .into_iter()
                    .collect(),
            }
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
                    home: list_summary_home(kind),
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
                home: list_summary_home(kind),
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
        warnings: run_warnings.clone(),
    })
}
