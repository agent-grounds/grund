//! The walk that fills one editor snapshot (§AR-system.2.9, §AR-lsp.5.1): every
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
use super::lsp_report::{editor_report, editor_run_warnings, widen_for_path_anchor};
use super::report::{public_lsp_report, public_lsp_run_warnings};
use crate::config::{ProjectRecords, display_path};
use crate::grammar::render_id;
use crate::model::{
    Declaration, TextOverlays, canonical_snapshot_path, paired_declarations, sort_path_key,
};
use crate::queries::{
    LspCitation, LspCompletionContext, LspDeclaration, LspFindingRange, LspSnapshot,
    LspSnapshotOpts, LspSnapshotWithCompletion, LspSnapshotWithMetadata, LspStub,
};
use crate::resolver::load_resolved_workspace_context;
use crate::scanner::api_scan_error;
use crate::workspace::resolve_workspace_config;

/// Programmatic snapshot for `grund-lsp`: all scanner-derived declaration and
/// citation ranges plus their resolved navigation targets. This keeps the LSP
/// transport from re-implementing the reference grammar (§AR-lsp.placement).

pub fn lsp_snapshot(opts: LspSnapshotOpts) -> Result<LspSnapshot> {
    Ok(lsp_snapshot_with_metadata(opts)?.snapshot)
}

/// Build hover metadata in the same scan as the editor snapshot (§FS-lsp.1.2).
/// Existing snapshot carriers and the original entry point remain unchanged.
pub fn lsp_snapshot_with_metadata(opts: LspSnapshotOpts) -> Result<LspSnapshotWithMetadata> {
    Ok(lsp_snapshot_with_completion(opts)?.metadata)
}

/// Load completion configuration and candidates with the editor snapshot
/// (§FS-lsp.1.6.2, §FS-lsp.1.6.4), never separately per request.
pub fn lsp_snapshot_with_completion(opts: LspSnapshotOpts) -> Result<LspSnapshotWithCompletion> {
    let overlays = normalized_overlays(opts.open_documents);
    // §FS-lsp.1.1: classify citing sides so the citation-direction checks
    // (`missing-citation` / `forbidden-citation`) run and surface as editor
    // diagnostics, the same errors `grund check` reports.
    let mut config = resolve_workspace_config(&opts.path)?;
    // §FS-lsp.2.2.1: an editor's explicit zero-config folder is the project anchor
    // even when the server process started elsewhere; CLI discovery deliberately
    // roots defaults at cwd, so this API corrects that root before it builds.
    if opts.path_provided && opts.path.is_dir() && config.config_file.is_none() {
        config.set_root(canonical_snapshot_path(&opts.path));
    }
    let report_scope = widen_for_path_anchor(&mut config, &opts.path, opts.path_provided)?;
    let context = load_resolved_workspace_context(
        ProjectRecords::of(config),
        &opts.path,
        opts.path_provided,
        &overlays,
        true,
    )?;
    let render_config = context.render_config().clone();
    let completion = LspCompletionContext::from_workspace(&context);
    let report = editor_report(&context, &overlays, &opts.path, report_scope.as_ref());
    // LSP routes findings back to project snapshots by filesystem identity.
    // Preserve absolute paths here instead of reconstructing them from rendered
    // `../` paths under Windows verbatim roots (§FS-lsp.1.1, §FS-lsp.2.2.2).
    let report = public_lsp_report(&render_config, report);
    let mut kind_titles = BTreeMap::new();
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
                .map(|(file, message)| api_scan_error(project.config.frame(), file, message)),
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
        // §FS-declarations.checks.unmarked-heading.4 / §FS-lsp.1.1: the errors select the complete
        // authored ATX heading without promoting it into the navigation catalog.
        finding_ranges.extend(project.findings.unmarked_headings.iter().map(|heading| {
            LspFindingRange {
                code: "unmarked-heading",
                path: absolutize_path(&heading.file),
                line: heading.line,
                column: heading.column,
                text: heading.heading.clone(),
            }
        }));
        // §FS-lsp.1.1 / §FS-check.3.24: unresolved local-section forms
        // remain outside the navigation graph, but retain their exact authored
        // spans for the shared check diagnostic.
        finding_ranges.extend(
            project
                .findings
                .local_section_citation_candidates
                .iter()
                .map(|candidate| LspFindingRange {
                    code: "local-section-citation",
                    path: absolutize_path(&candidate.file),
                    line: candidate.line,
                    column: candidate.column,
                    text: candidate.text.clone(),
                }),
        );
        for (id, decls) in &project.findings.declarations {
            let rendered = render_id(&project.config.grammar, id);
            let query_id = lsp_query_id(&context, project, &rendered, None);
            // §FS-config.3.4.3: metadata follows the declaration's owning project.
            if let Some(title) = project
                .config
                .kinds
                .iter()
                .find(|kind| kind.kind == id.kind)
                .and_then(|kind| kind.title.as_ref())
            {
                kind_titles.insert(query_id.clone(), title.clone());
                for section in decls.iter().flat_map(|decl| decl.sections.keys()) {
                    kind_titles.insert(
                        lsp_query_id(&context, project, &rendered, Some(section)),
                        title.clone(),
                    );
                }
            }
            // §FS-declarations.stubs.verdict: a title for each stand-in of the ID's paired
            // declarations, and a stub title for each stub that only points at one.
            let paired = paired_declarations(decls);
            let mut stand_ins: Vec<&Declaration> = paired.stand_ins().collect();
            stand_ins.sort_by(|a, b| {
                (sort_path_key(&a.file), a.line).cmp(&(sort_path_key(&b.file), b.line))
            });
            for &home in &stand_ins {
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
            for stub in decls.iter().filter(|decl| {
                decl.is_stub
                    && !stand_ins
                        .iter()
                        .any(|stand_in| std::ptr::eq(*stand_in, *decl))
            }) {
                let target = lsp_target_for_stub(stub);
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
                .map(|target| render_id(&target.config.grammar, &citation.id))
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

    // §FS-lsp.1.1.3: the run-level `[workspace]` warnings, published on the `grund.toml`
    // each one anchors at — the same channel the CLI renders, from the same place
    // (§FS-lsp.4.1).
    let run_warnings = public_lsp_run_warnings(&render_config, editor_run_warnings(&context));
    Ok(LspSnapshotWithCompletion {
        completion,
        metadata: LspSnapshotWithMetadata {
            kind_titles,
            snapshot: LspSnapshot {
                root: absolutize_path(&render_config.root),
                marker: render_config.marker.clone(),
                trigger: render_config.trigger.clone(),
                workspace: context.workspace_loaded,
                report,
                run_warnings,
                declarations,
                sections,
                finding_ranges,
                stubs,
                citations,
                scanned_files,
                scan_errors,
            },
        },
    })
}

pub(super) fn normalized_overlays(overlays: BTreeMap<PathBuf, String>) -> TextOverlays {
    overlays
        .into_iter()
        .map(|(path, text)| (absolutize_path(&path), text))
        .collect()
}
