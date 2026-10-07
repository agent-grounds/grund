use anyhow::{Result, anyhow};
use std::collections::{BTreeMap, BTreeSet};

use super::citation_counts::ListCitationCounts;
use crate::config::{
    Config, display_path, measure_point_text, non_citable_kind_error, run_warning_findings,
};
use crate::grammar::render_id;
use crate::model::{
    Declaration, Finding, Id, SectionInfo, TextOverlays, format_path, is_stub_for_inline_decl,
    sort_path_key,
};
use crate::resolver::{PointBodyCache, WorkspaceContext, load_workspace_context, point_body_pair};
use crate::rules::sentence::{RuleSubject, RuleVocabulary, parse_selector};
use crate::scanner::api_scan_error;

pub use super::size_output::{ListSizeEntry, ListSizeMeasurement, ListSizeOpts, ListSizeOutput};

/// Programmatic point-size catalog. It selects the same declaration set as
/// [`list`](crate::list), adds scanner-recorded section sites, and measures show-identical
/// lead/full bodies through one per-file cache (§FS-list.2, §FS-list.3.4.1,
/// §FS-workspace.8.3.3).
pub fn list_sizes(opts: ListSizeOpts) -> Result<ListSizeOutput> {
    list_sizes_with_run_warnings(opts).1
}

/// Additive cautions survive a later refusal (§FS-distribution.3.1).
pub fn list_sizes_with_run_warnings(opts: ListSizeOpts) -> (Vec<Finding>, Result<ListSizeOutput>) {
    let mut cautions = Vec::new();
    let result = list_sizes_run(opts, &mut cautions);
    (cautions, result)
}

fn list_sizes_run(opts: ListSizeOpts, cautions: &mut Vec<Finding>) -> Result<ListSizeOutput> {
    if opts.units.is_empty() {
        return Err(anyhow!("at least one size unit is required"));
    }
    if matches!(opts.top, Some(0)) {
        return Err(anyhow!("top must be positive"));
    }
    let context = load_workspace_context(&opts.path, opts.path_provided)?;
    *cautions = run_warning_findings(context.render_config(), context.run_warnings.clone());
    validate_list_scope_filters(&context, &opts.project_filter, &opts.kind_filter)?;
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
        target_namespaces: BTreeMap::new(),
        named_sections: selected_projects().all(|project| project.config.named_sections),
        id_grammars: selected_projects()
            .map(|project| project.config.grammar.clone())
            .collect(),
        section_separators: selected_projects()
            .map(|project| project.config.section_separator.clone())
            .collect(),
    };
    let selector = opts
        .selector
        .as_deref()
        .map(|raw| {
            // §FS-rules.8.1: a refusal recovering no kind ends in `known kinds:`.
            parse_selector(raw, &vocabulary)
                .map_err(|refusal| anyhow!(refusal.render(&context.known_kinds_line())))
        })
        .transpose()?;

    struct Pending<'a> {
        project_alias: &'a str,
        project_config: &'a Config,
        id: &'a Id,
        declaration: &'a Declaration,
        section: Option<(&'a str, &'a SectionInfo)>,
        duplicate: bool,
    }

    let counts = ListCitationCounts::new(&context);
    let mut pending = Vec::new();
    let mut scan_errors = Vec::new();
    for project in &context.projects {
        if !opts.project_filter.is_empty() && !opts.project_filter.contains(&project.alias) {
            continue;
        }
        scan_errors.extend(
            project
                .scan_errors
                .iter()
                .map(|(file, message)| api_scan_error(context.render_config(), file, message)),
        );
        let used_counts = counts.used_for(project.alias.as_str());
        for (id, declarations) in &project.findings.declarations {
            if !opts.kind_filter.is_empty() && !opts.kind_filter.contains(&id.kind) {
                continue;
            }
            if opts.unused_only && used_counts.get(id).copied().unwrap_or(0) > 0 {
                continue;
            }
            if opts.unused_only && id.kind == "E2E" && !opts.kind_filter.contains("E2E") {
                continue;
            }
            let mut homes: Vec<&Declaration> = declarations
                .iter()
                .filter(|decl| !is_stub_for_inline_decl(&project.config.root, decl, declarations))
                .collect();
            homes.sort_by(|a, b| {
                (sort_path_key(&a.file), a.line).cmp(&(sort_path_key(&b.file), b.line))
            });
            let duplicate_declaration = homes.len() > 1;
            for declaration in homes {
                pending.push(Pending {
                    project_alias: &project.alias,
                    project_config: &project.config,
                    id,
                    declaration,
                    section: None,
                    duplicate: duplicate_declaration,
                });

                let mut section_sites: BTreeMap<&str, Vec<&SectionInfo>> = BTreeMap::new();
                for (section, info) in &declaration.sections {
                    section_sites.entry(section).or_default().push(info);
                }
                for (section, info) in &declaration.duplicate_sections {
                    section_sites.entry(section).or_default().push(info);
                }
                for (section, mut sites) in section_sites {
                    sites.sort_by_key(|info| info.line);
                    let duplicate_section = duplicate_declaration || sites.len() > 1;
                    for info in sites {
                        pending.push(Pending {
                            project_alias: &project.alias,
                            project_config: &project.config,
                            id,
                            declaration,
                            section: Some((section, info)),
                            duplicate: duplicate_section,
                        });
                    }
                }
            }
        }
    }

    // The normal order is also the complete tie-break for `--top`.
    pending.sort_by(|a, b| {
        (
            a.project_alias,
            a.id,
            a.section.map(|(section, _)| section),
            sort_path_key(&a.declaration.file),
            a.section
                .map(|(_, info)| info.line)
                .unwrap_or(a.declaration.line),
        )
            .cmp(&(
                b.project_alias,
                b.id,
                b.section.map(|(section, _)| section),
                sort_path_key(&b.declaration.file),
                b.section
                    .map(|(_, info)| info.line)
                    .unwrap_or(b.declaration.line),
            ))
    });
    if let Some(subject) = &selector {
        let resolution = match subject {
            RuleSubject::ExactDeclaration(literal) => Some((
                literal.clone(),
                pending
                    .iter()
                    .filter(|row| {
                        row.section.is_none()
                            && render_id(&row.project_config.grammar, row.id) == *literal
                    })
                    .count(),
            )),
            RuleSubject::ExactChapter {
                declaration,
                path,
                separator,
            } => Some((
                format!("{declaration}{separator}{path}"),
                pending
                    .iter()
                    .filter(|row| {
                        row.section.is_some_and(|(section, _)| section == path)
                            && render_id(&row.project_config.grammar, row.id) == *declaration
                    })
                    .count(),
            )),
            _ => None,
        };
        if let Some((literal, matches)) = resolution {
            if matches == 0 {
                return Err(anyhow!("literal subject {literal} does not resolve"));
            }
            if matches > 1 {
                return Err(anyhow!("literal subject {literal} is ambiguous"));
            }
        }
        pending.retain(|row| {
            let rendered = render_id(&row.project_config.grammar, row.id);
            match (subject, row.section) {
                (RuleSubject::Kind(kind), None) => kind == &row.id.kind,
                (RuleSubject::ExactDeclaration(literal), None) => literal == &rendered,
                (RuleSubject::ChapterOfKind { kind, name }, Some((section, _))) => {
                    kind == &row.id.kind && section.rsplit('.').next() == Some(name.as_str())
                }
                (
                    RuleSubject::ExactChapter {
                        declaration, path, ..
                    },
                    Some((section, _)),
                ) => declaration == &rendered && path == section,
                _ => false,
            }
        });
    }

    let overlays = TextOverlays::new();
    let mut cache = PointBodyCache::new(&overlays);
    let render_config = context.render_config();
    let mut entries = Vec::with_capacity(pending.len());
    for row in pending {
        let bodies = point_body_pair(
            &mut cache,
            row.project_config,
            row.id,
            row.declaration,
            row.section,
        )?;
        let measurements = opts
            .units
            .iter()
            .map(|unit| ListSizeMeasurement {
                unit: *unit,
                lead: bodies
                    .as_ref()
                    .map(|(lead, _)| measure_point_text(lead, *unit)),
                full: bodies
                    .as_ref()
                    .map(|(_, full)| measure_point_text(full, *unit)),
            })
            .collect();
        let rendered = render_id(&row.project_config.grammar, row.id);
        entries.push(ListSizeEntry {
            project: context
                .workspace_loaded
                .then(|| row.project_alias.to_string()),
            id: if context.workspace_loaded {
                format!("{}/{rendered}", row.project_alias)
            } else {
                rendered
            },
            section: row.section.map(|(section, _)| section.to_string()),
            section_separator: row.project_config.section_separator.clone(),
            kind: row.id.kind.clone(),
            path: display_path(render_config, &row.declaration.file),
            line: row
                .section
                .map(|(_, info)| info.line)
                .unwrap_or(row.declaration.line),
            stub: row.declaration.is_stub,
            defines: row
                .declaration
                .defined_in
                .as_ref()
                .map(|target| format_path(target)),
            duplicate: row.duplicate,
            measurements,
        });
    }

    if let Some(top) = opts.top {
        entries.retain(|entry| entry.measurements[0].lead.is_some());
        entries.sort_by(|a, b| {
            b.measurements[0]
                .lead
                .cmp(&a.measurements[0].lead)
                // `entries` entered this stable sort in normal order.
                .then(std::cmp::Ordering::Equal)
        });
        entries.truncate(top);
    }

    Ok(ListSizeOutput {
        output_format: render_config.output_format.clone(),
        workspace: context.workspace_loaded,
        entries,
        scan_errors,
        warnings: run_warning_findings(render_config, context.run_warnings.clone()),
    })
}

/// Shared validation for size mode's pre-measurement catalog selectors
/// (§FS-list.1, §FS-workspace.8.3.2).
fn validate_list_scope_filters(
    context: &WorkspaceContext,
    project_filter: &BTreeSet<String>,
    kind_filter: &BTreeSet<String>,
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
    for kind in kind_filter {
        let matched = context
            .projects
            .iter()
            .filter(|project| project_filter.is_empty() || project_filter.contains(&project.alias))
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
            return Err(anyhow!("{headline}\n{}", context.known_kinds_line()));
        }
    }
    Ok(())
}
