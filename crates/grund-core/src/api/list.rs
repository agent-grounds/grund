//! The published `list` contract and the catalog behind it (§AR-system.2.9): the
//! declaration rows and the per-kind summary rows, with no output format chosen
//! and no exit code mapped (§FS-list.2, §FS-list.3, §AR-bindings.2).
//!
//! Its own file since before the component was a module, so the list contract
//! could grow without the contract file growing with it
//! (§AR-core-module-layout.3). The citation counts each row's `refs` is read off
//! are the catalog query's, read downward (§FS-list.3.2.1).

use anyhow::{Result, anyhow};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::config::{Config, KindConfig, display_path, non_citable_kind_error};
use crate::grammar::{render_id, section_display_name};
use crate::model::{Declaration, Finding, Id, format_path, is_stub_for_inline_decl, sort_path_key};
use crate::queries::ListCitationCounts;
use crate::resolver::{WorkspaceProject, load_workspace_context};
use crate::rules::sentence::{RuleSubject, RuleVocabulary, parse_selector};

use super::report::context_run_warnings;
use crate::scanner::{ApiScanError, api_scan_error};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListEntry {
    pub project: Option<String>,
    pub id: String,
    /// Exact named-section path for a chapter row; declaration rows omit it.
    pub section: Option<String>,
    pub kind: String,
    pub path: String,
    pub line: usize,
    pub title: Option<String>,
    pub stub: bool,
    pub defines: Option<String>,
    pub refs: usize,
    pub duplicate: bool,
    /// Embedded value authorities owned by this declaration's existing
    /// sections, omitted by serializers when empty (§FS-values.6.1,
    /// §FS-list.3.1.1).
    pub value_roots: Vec<ListValueRoot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListValueRoot {
    pub id: String,
    pub valid: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListSummary {
    pub project: Option<String>,
    pub kind: String,
    pub title: String,
    pub home: String,
    pub count: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ListOutput {
    pub output_format: String,
    pub workspace: bool,
    pub entries: Vec<ListEntry>,
    pub summaries: Vec<ListSummary>,
    pub scan_errors: Vec<ApiScanError>,
    /// The run's warning channel (§FS-distribution.3.1): the four `[workspace]`
    /// cautions of §FS-check.4.7.7, §FS-check.4.8.15, §FS-check.4.10.11 and
    /// §FS-workspace.6.1.7, each anchored at the `grund.toml` line its own message
    /// names. A frontend renders each as one CLI-level `warning:` on stderr
    /// (§FS-check.2.1.1); an editor publishes it on that line (§FS-lsp.1.1.3).
    pub warnings: Vec<Finding>,
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
/// when the query is refused (§FS-check.4.7.2, §FS-check.4.10.7, §FS-workspace.6.1.7).
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
    if !opts.project_filter.is_empty() && !context.workspace_loaded {
        return Err(anyhow!(
            "--project requires workspace mode (no [workspace] block discovered)"
        ));
    }
    for alias in &opts.project_filter {
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
    for kind in &opts.kind_filter {
        // §FS-list.1.1, as in the CLI frontend: a configured but non-citable kind
        // is refused with its reason rather than selected into an empty list.
        let matched = context
            .projects
            .iter()
            .filter(|project| {
                opts.project_filter.is_empty() || opts.project_filter.contains(&project.alias)
            })
            .find_map(|project| {
                project
                    .config
                    .kinds
                    .iter()
                    .find(|candidate| &candidate.kind == kind)
            });
        if !matches!(matched, Some(candidate) if candidate.citable) {
            let headline = match matched {
                Some(candidate) => non_citable_kind_error(candidate),
                None => format!("unknown kind `{kind}`"),
            };
            let mut known: Vec<String> = Vec::new();
            let mut seen: BTreeSet<String> = BTreeSet::new();
            for project in &context.projects {
                for k in &project.config.kinds {
                    if k.citable && seen.insert(k.kind.clone()) {
                        known.push(k.kind.clone());
                    }
                }
            }
            return Err(anyhow!("{headline}\nknown kinds: {}", known.join(", ")));
        }
    }
    let selected_projects = || {
        context.projects.iter().filter(|project| {
            opts.project_filter.is_empty() || opts.project_filter.contains(&project.alias)
        })
    };
    let kinds = selected_projects()
        .flat_map(|project| project.config.kinds.iter())
        .filter(|kind| kind.citable)
        .map(|kind| kind.kind.clone())
        .collect::<BTreeSet<_>>();
    let vocabulary = RuleVocabulary {
        kinds: kinds.clone(),
        target_kinds: kinds,
        named_sections: selected_projects().all(|project| project.config.named_sections),
        id_grammars: selected_projects()
            .map(|project| project.config.grammar.clone())
            .collect(),
    };
    let selector = opts
        .selector
        .as_deref()
        .map(|raw| parse_selector(raw, &vocabulary).map_err(|error| anyhow!(error.message)))
        .transpose()?;

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
            project
                .scan_errors
                .iter()
                .map(|(file, message)| api_scan_error(context.render_config(), file, message)),
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
            let duplicate = homes.len() > 1;
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
        let literal = match subject {
            RuleSubject::ExactDeclaration(literal) => Some(literal.as_str()),
            RuleSubject::ExactChapter { declaration, .. } => Some(declaration.as_str()),
            _ => None,
        };
        if let Some(literal) = literal {
            let exact_matches = entries
                .iter()
                .filter(|entry| render_id(&entry.project_config.grammar, entry.id) == literal)
                .collect::<Vec<_>>();
            if exact_matches.is_empty() {
                return Err(anyhow!("literal subject {literal} does not resolve"));
            }
            if exact_matches.len() > 1 {
                return Err(anyhow!("literal subject {literal} is ambiguous"));
            }
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
                Some(RuleSubject::ChapterOfKind { kind, name }) if kind == &entry.id.kind => entry
                    .home
                    .sections
                    .iter()
                    .filter(|(section, info)| {
                        section.rsplit('.').next() == Some(name.as_str())
                            || section_display_name(&info.title, section).eq_ignore_ascii_case(name)
                    })
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
                    .collect(),
                Some(RuleSubject::ExactChapter { declaration, path })
                    if declaration == &render_id(&entry.project_config.grammar, entry.id) =>
                {
                    entry
                        .home
                        .sections
                        .get(path)
                        .map(|info| {
                            let mut row = base();
                            row.section = Some(path.clone());
                            row.line = info.line;
                            row.title = Some(section_display_name(&info.title, path).to_string());
                            row.stub = false;
                            row.defines = None;
                            row.refs = 0;
                            row.duplicate = false;
                            row.value_roots.clear();
                            row
                        })
                        .into_iter()
                        .collect()
                }
                _ => Vec::new(),
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
