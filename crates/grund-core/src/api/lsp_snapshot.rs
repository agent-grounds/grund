//! The walk that fills one editor snapshot (§AR-system.2.9, §AR-lsp.5): every
//! declaration, section, stub, citation and finding range in the loaded
//! workspace, with the navigation target each resolves to (§FS-lsp.1).
//!
//! The records it returns are `queries/editor_snapshot.rs`'s, because the two
//! editor answers of §FS-lsp read them; the builder is here because it consumes
//! the whole pipeline — config, workspace, scanner and checker — and only the api
//! sits above all of it (§AR-system.4). The per-range span and target helpers are
//! `lsp_ranges.rs`.

use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::lsp_ranges::{
    absolutize_path, declaration_range_parts, heading_span_parts, lsp_query_id,
    lsp_target_for_citation, lsp_target_for_stub, section_range_parts,
};
use super::report::public_lsp_report;
use super::scope_cautions::scan_scope_caution;
use crate::checker::{
    WorkspaceCheckTarget, check_with_workspace_and_overlays, is_stub_for_inline_decl,
    sort_diagnostics,
};
use crate::config::display_path;
use crate::grammar::render_id;
use crate::model::{
    CheckReport, Declaration, Diagnostic, TextOverlays, canonical_snapshot_path, sort_path_key,
};
use crate::queries::{
    LspCitation, LspDeclaration, LspFindingRange, LspSnapshot, LspSnapshotOpts, LspStub,
};
use crate::scanner::api_scan_error;
use crate::workspace::{
    WorkspaceContext, absent_only_workspace_caution, absent_optional_member_warnings,
    load_resolved_workspace_context, resolve_workspace_config,
};

/// Programmatic snapshot for `grund-lsp`: all scanner-derived declaration and
/// citation ranges plus their resolved navigation targets. This keeps the LSP
/// transport from re-implementing the reference grammar (§AR-lsp.placement).

pub fn lsp_snapshot(opts: LspSnapshotOpts) -> Result<LspSnapshot> {
    let overlays = normalized_overlays(opts.open_documents);
    // §FS-lsp.1.1: classify citing sides so the citation-direction checks
    // (`missing-citation` / `forbidden-citation`) run and surface as editor
    // diagnostics, the same errors `grund check` reports.
    let mut config = resolve_workspace_config(&opts.path)?;
    // §FS-lsp.2.2: an editor's explicit zero-config folder is the project anchor
    // even when the server process started elsewhere; CLI discovery deliberately
    // roots defaults at cwd, so this API corrects that root before it builds.
    if opts.path_provided && opts.path.is_dir() && config.config_file.is_none() {
        config.root = canonical_snapshot_path(&opts.path);
    }
    let context =
        load_resolved_workspace_context(config, &opts.path, opts.path_provided, &overlays, true)?;
    let render_config = context.render_config().clone();
    // LSP routes findings back to project snapshots by filesystem identity.
    // Preserve absolute paths here instead of reconstructing them from rendered
    // `../` paths under Windows verbatim roots (§FS-lsp.1.1, §FS-lsp.2.2).
    let report = public_lsp_report(
        &render_config,
        check_workspace_context(&context, false, &overlays),
    );
    let mut declarations = Vec::new();
    let mut sections = Vec::new();
    let mut finding_ranges = Vec::new();
    let mut stubs = Vec::new();
    let mut citations = Vec::new();
    let mut scanned_files = BTreeSet::new();
    let mut scan_errors = Vec::new();

    for project in &context.projects {
        scanned_files.extend(
            project
                .findings
                .scanned_files
                .iter()
                .map(|file| absolutize_path(file)),
        );
        scan_errors.extend(
            project
                .scan_errors
                .iter()
                .map(|(file, message)| api_scan_error(&project.config, file, message)),
        );
        finding_ranges.extend(
            project
                .findings
                .section_headings_outside_declarations
                .iter()
                .map(|heading| {
                    let (column, text) =
                        heading_span_parts(&heading.file, heading.line, &heading.path, &overlays);
                    LspFindingRange {
                        code: "section-outside-declaration",
                        path: absolutize_path(&heading.file),
                        line: heading.line,
                        column,
                        text,
                    }
                }),
        );
        // §FS-check.4.14 / §FS-lsp.1.1: warnings select the complete authored
        // ATX heading without promoting it into the navigation catalog.
        finding_ranges.extend(project.findings.unmarked_headings.iter().map(|heading| {
            LspFindingRange {
                code: "unmarked-heading",
                path: absolutize_path(&heading.file),
                line: heading.line,
                column: heading.column,
                text: heading.heading.clone(),
            }
        }));
        for (id, decls) in &project.findings.declarations {
            let rendered = render_id(&project.config, id);
            let query_id = lsp_query_id(&context, project, &rendered, None);
            let mut homes: Vec<&Declaration> = decls
                .iter()
                .filter(|decl| !is_stub_for_inline_decl(&project.config.root, decl, decls))
                .collect();
            homes.sort_by(|a, b| {
                (sort_path_key(&a.file), a.line).cmp(&(sort_path_key(&b.file), b.line))
            });
            for home in homes {
                let display = if context.workspace_loaded {
                    display_path(context.render_config(), &home.file)
                } else {
                    display_path(&project.config, &home.file)
                };
                let (column, text) = declaration_range_parts(home, &rendered, &overlays);
                declarations.push(LspDeclaration {
                    project: context.workspace_loaded.then(|| project.alias.clone()),
                    path: absolutize_path(&home.file),
                    display_path: display.clone(),
                    line: home.line,
                    column,
                    text,
                    query_id: query_id.clone(),
                    section_separator: project.config.section_separator.clone(),
                });
                // Each citable section heading is its own declaration-side
                // title: editors navigate `<ID>.<section>` to that section's
                // citations, the same way the whole-ID title does (§FS-lsp.1.3.1).
                for (section, info) in &home.sections {
                    let (column, text) = section_range_parts(home, info, section, &overlays);
                    sections.push(LspDeclaration {
                        project: context.workspace_loaded.then(|| project.alias.clone()),
                        path: absolutize_path(&home.file),
                        display_path: display.clone(),
                        line: info.line,
                        column,
                        text,
                        query_id: lsp_query_id(&context, project, &rendered, Some(section)),
                        section_separator: project.config.section_separator.clone(),
                    });
                }
            }
            for stub in decls
                .iter()
                .filter(|decl| is_stub_for_inline_decl(&project.config.root, decl, decls))
            {
                let target = lsp_target_for_stub(project, stub, decls);
                if let Some((target_path, target_line)) = target {
                    let (column, text) = declaration_range_parts(stub, &rendered, &overlays);
                    stubs.push(LspStub {
                        project: context.workspace_loaded.then(|| project.alias.clone()),
                        path: absolutize_path(&stub.file),
                        display_path: if context.workspace_loaded {
                            display_path(context.render_config(), &stub.file)
                        } else {
                            display_path(&project.config, &stub.file)
                        },
                        line: stub.line,
                        column,
                        text,
                        query_id: query_id.clone(),
                        section_separator: project.config.section_separator.clone(),
                        target_path: absolutize_path(&target_path),
                        target_line,
                    });
                }
            }
        }
        for citation in &project.findings.citations {
            let target_project = match citation.namespace.as_deref() {
                Some(alias) => context.project_by_alias(alias),
                None => Some(project),
            };
            let rendered_id = target_project
                .map(|target| render_id(&target.config, &citation.id))
                .unwrap_or_else(|| {
                    citation
                        .text
                        .trim_start_matches(&render_config.marker)
                        .to_string()
                });
            let query_id = target_project
                .map(|target| {
                    lsp_query_id(&context, target, &rendered_id, citation.section.as_deref())
                })
                .unwrap_or_else(|| rendered_id.clone());
            let declaration_query_id = target_project
                .map(|target| lsp_query_id(&context, target, &rendered_id, None))
                .unwrap_or_else(|| rendered_id.clone());
            let section_separator = target_project
                .map(|target| target.config.section_separator.clone())
                .unwrap_or_else(|| render_config.section_separator.clone());
            let target =
                target_project.and_then(|target| lsp_target_for_citation(target, citation));
            citations.push(LspCitation {
                project: context.workspace_loaded.then(|| project.alias.clone()),
                path: absolutize_path(&citation.file),
                display_path: if context.workspace_loaded {
                    display_path(context.render_config(), &citation.file)
                } else {
                    display_path(&project.config, &citation.file)
                },
                line: citation.line,
                column: citation.column,
                text: citation.text.clone(),
                query_id,
                declaration_query_id,
                section_separator,
                target_path: target.as_ref().map(|(path, _)| absolutize_path(path)),
                target_line: target.map(|(_, line)| line),
            });
        }
    }

    let declaration_sort = |a: &LspDeclaration, b: &LspDeclaration| {
        (sort_path_key(&a.path), a.line, a.column, &a.text).cmp(&(
            sort_path_key(&b.path),
            b.line,
            b.column,
            &b.text,
        ))
    };
    declarations.sort_by(declaration_sort);
    sections.sort_by(declaration_sort);
    finding_ranges.sort_by(|a, b| {
        (sort_path_key(&a.path), a.line, a.column, &a.text).cmp(&(
            sort_path_key(&b.path),
            b.line,
            b.column,
            &b.text,
        ))
    });
    stubs.sort_by(|a, b| {
        (sort_path_key(&a.path), a.line, a.column, &a.text).cmp(&(
            sort_path_key(&b.path),
            b.line,
            b.column,
            &b.text,
        ))
    });
    citations.sort_by(|a, b| {
        (sort_path_key(&a.path), a.line, a.column, &a.text).cmp(&(
            sort_path_key(&b.path),
            b.line,
            b.column,
            &b.text,
        ))
    });

    Ok(LspSnapshot {
        root: absolutize_path(&render_config.root),
        marker: render_config.marker,
        trigger: render_config.trigger,
        workspace: context.workspace_loaded,
        report,
        declarations,
        sections,
        finding_ranges,
        stubs,
        citations,
        scanned_files,
        scan_errors,
    })
}

pub(super) fn normalized_overlays(overlays: BTreeMap<PathBuf, String>) -> TextOverlays {
    overlays
        .into_iter()
        .map(|(path, text)| (absolutize_path(&path), text))
        .collect()
}

/// §FS-lsp.4: the report the editor shows, decided here so that an editor and a
/// terminal over one tree say the same thing — this is `grund check`'s workspace
/// arm (`run_workspace_check`) for a surface that has no CLI.
///
/// §FS-check.4.9: the announcement of every namespace the run did not read is a
/// **located** report finding, so it is one of the diagnostics the editor must
/// mirror. It belongs to the run rather than to a project, which is why it is read
/// off the render config outside the loop below and survives a block whose every
/// project was the absent one — and why that config is cloned after the workspace
/// walk (`load_resolved_workspace_context`). §FS-check.2.2: the caution for that
/// same block rides beside it.
fn check_workspace_context(
    context: &WorkspaceContext,
    force_require_grounding: bool,
    overlays: &TextOverlays,
) -> CheckReport {
    let workspace = context
        .projects
        .iter()
        .map(|project| {
            (
                project.alias.clone(),
                WorkspaceCheckTarget {
                    findings: &project.findings,
                    config: &project.config,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut report = CheckReport::default();
    for project in &context.projects {
        let mut config = project.config.clone();
        // §FS-check.1: the same global default the key sets, per member — an
        // explicit `false` on a `[[kinds]]` row still wins (§FS-config.3.4.8).
        if force_require_grounding {
            config.require_grounding = true;
        }
        let mut project_report = if context.workspace_loaded {
            check_with_workspace_and_overlays(
                &project.findings,
                &config,
                // §FS-workspace.8.1: paths inside a message are spelled from the
                // render root, like the anchors beside them.
                context.render_config(),
                Some(&project.alias),
                &workspace,
                overlays,
            )
        } else {
            check_with_workspace_and_overlays(
                &project.findings,
                &config,
                &config,
                None,
                &BTreeMap::new(),
                overlays,
            )
        };
        let project_has_findings =
            !project_report.errors.is_empty() || !project_report.warnings.is_empty();
        report.errors.append(&mut project_report.errors);
        report.warnings.append(&mut project_report.warnings);
        append_lsp_scan_errors(&mut report, project.scan_errors.iter().cloned());
        // §FS-lsp.4: the same decision `grund check` makes, from the same
        // function — an editor and a terminal over one tree report one set of
        // diagnostics (§FS-check.2.2, §FS-check.4.5).
        report.warnings.extend(scan_scope_caution(
            &config,
            &project.findings,
            &config.root,
            true,
            project.scan_errors.is_empty() && !project_has_findings,
        ));
    }
    // §FS-check.4.9, §FS-check.2.2: the announcements and the caution — see this
    // function's docs.
    report
        .warnings
        .extend(absent_optional_member_warnings(context.render_config()));
    report.warnings.extend(absent_only_workspace_caution(
        context.render_config(),
        context.projects.is_empty(),
    ));
    sort_diagnostics(&mut report.errors);
    sort_diagnostics(&mut report.warnings);
    report
}

fn append_lsp_scan_errors(
    report: &mut CheckReport,
    scan_errors: impl IntoIterator<Item = (PathBuf, String)>,
) {
    for (file, message) in scan_errors {
        report.errors.push(Diagnostic {
            code: "io",
            path: Some(file),
            line: None,
            column: None,
            message,
            sites: Vec::new(),
        });
    }
}
