use anyhow::{Result, anyhow};
use std::collections::BTreeMap;

use super::citation_counts::ListCitationCounts;
use super::glob_scope::{GlobSelection, read_globs};
use super::list_scope::{check_list_scope, unused_leaves_out};
use super::report_scope::{reported_scope, reports};
use super::selector_refusal::require_unique_literal;
use crate::config::{Config, display_path, measure_point_text, run_warning_findings};
use crate::grammar::{field_value, render_declaration_id};
use crate::model::{
    Declaration, FileRecords, Finding, Id, SectionInfo, format_path, paired_declarations,
    sort_path_key,
};
use crate::resolver::{load_reported_workspace_context, point_body_pair};
use crate::scanner::api_scan_error;

pub use super::size_output::{ListSizeEntry, ListSizeMeasurement, ListSizeOpts, ListSizeOutput};

/// Programmatic point-size catalog. It selects the same declaration set as
/// [`list`](crate::list), adds scanner-recorded section sites, and measures show-identical
/// lead/full bodies out of the file record each scan kept (§FS-list.2, §FS-list.3.4.1,
/// §FS-workspace.8.3.3). A healthy stub whose target the walk did not reach is measured
/// at the home `show` reads there, with that home's sections (§FS-list.3.4.6).
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
    // §FS-coordinate-globs.errors.shape: what the grammar alone refuses, before discovery.
    let globs = read_globs(&opts.globs)?;
    // §AR-resolver.3.3: the same rows as `list`, with the project's answers.
    let context = load_reported_workspace_context(&opts.path, opts.path_provided)?;
    *cautions = run_warning_findings(context.render_config(), context.run_warnings.clone());
    // §FS-workspace.8.3.2: the same scope check as `list`, before any row is read.
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

    struct Pending<'a> {
        project_index: usize,
        project_alias: &'a str,
        project_config: &'a Config,
        files: &'a FileRecords,
        id: &'a Id,
        declaration: &'a Declaration,
        section: Option<(&'a str, &'a SectionInfo)>,
        duplicate: bool,
        broken_stub: bool,
    }

    let counts = ListCitationCounts::new(&context);
    let mut pending = Vec::new();
    let mut scan_errors = Vec::new();
    for (project_index, project) in context.projects.iter().enumerate() {
        if !opts.project_filter.is_empty() && !opts.project_filter.contains(&project.alias) {
            continue;
        }
        scan_errors.extend(
            project.scan_errors.iter().map(|(file, message)| {
                api_scan_error(context.render_config().frame(), file, message)
            }),
        );
        let used_counts = counts.used_for(project.alias.as_str());
        for (id, declarations) in &project.findings.declarations {
            if !opts.kind_filter.is_empty() && !opts.kind_filter.contains(&id.kind) {
                continue;
            }
            if opts.unused_only && used_counts.get(id).copied().unwrap_or(0) > 0 {
                continue;
            }
            // §FS-list.1.3.1: a configured `E2E` kind is left out unless `--kind` selects it.
            if opts.unused_only && unused_leaves_out(&id.kind, &opts.kind_filter) {
                continue;
            }
            // §FS-list.3.4.6: one row per declaration, at its record, so a healthy stub
            // whose target is outside the walk is measured there, and a target declaring
            // the ID twice is two rows (§AR-scanner.4.6).
            let paired = paired_declarations(declarations);
            let duplicate_declaration = paired.len() > 1;
            for one in paired.iter() {
                let declaration = one.record;
                if !reports(reported.as_ref(), declarations, declaration) {
                    continue;
                }
                pending.push(Pending {
                    project_index,
                    project_alias: &project.alias,
                    project_config: &project.config,
                    files: &project.findings.files,
                    // §FS-config.3.2.8: each measured declaration keeps its own digits.
                    id: &declaration.id,
                    declaration,
                    section: None,
                    duplicate: duplicate_declaration,
                    broken_stub: one.is_broken_stub(),
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
                            project_index,
                            project_alias: &project.alias,
                            project_config: &project.config,
                            files: &project.findings.files,
                            // §FS-config.3.2.8: section rows name their owning home.
                            id: &declaration.id,
                            declaration,
                            section: Some((section, info)),
                            duplicate: duplicate_section,
                            broken_stub: false,
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
        // §FS-rules.8: the one subject matcher (§AR-rules.2) says which rows a
        // selector keeps, read as `list` reads it, and a literal counts the same.
        let selected = |row: &Pending<'_>| {
            let label = render_declaration_id(&row.project_config.grammar, row.declaration);
            // §FS-rules.2.2: a whose clause is matched against the field's display name.
            let value = subject
                .whose_field()
                .and_then(|field| field_value(row.declaration, field));
            subject.selects(
                &row.id.kind,
                &label,
                row.section.map(|(section, _)| section),
                value.as_deref(),
            )
        };
        if let Some(literal) = subject.literal() {
            require_unique_literal(&literal, pending.iter().filter(|row| selected(row)).count())?;
        }
        pending.retain(selected);
    }
    // §FS-coordinate-globs.scope.filters: the glob intersects, as `list` reads it.
    if let Some(globs) = &globs {
        pending.retain(|row| {
            let label = render_declaration_id(&row.project_config.grammar, row.declaration);
            globs.selects(
                row.project_index,
                &label,
                row.section.map(|(section, _)| section),
            )
        });
    }

    let render_config = context.render_config();
    let mut entries = Vec::with_capacity(pending.len());
    for row in pending {
        // §AR-scanner.2.8: the file record its project's scan kept, buffers in front of the disk.
        let bodies = point_body_pair(
            row.files,
            row.project_config.schema(),
            row.project_config.frame(),
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
        let rendered = render_declaration_id(&row.project_config.grammar, row.declaration);
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
            stub: row.broken_stub,
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
        note: globs.and_then(|globs| globs.note()),
        warnings: run_warning_findings(render_config, context.run_warnings.clone()),
    })
}
