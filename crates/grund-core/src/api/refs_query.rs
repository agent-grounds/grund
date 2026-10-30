//! The walk behind the published `refs` contract (§AR-system.2.9): scan once,
//! select the target grammar, and return either citation data or the typed
//! resolver rejection (§FS-refs.2, §FS-refs.4, §AR-bindings.2).
//!
//! The adapter §AR-core-module-layout.2 keeps out of `refs.rs`, which is the
//! published surface: what is here is the project selection, the hit walk and
//! the sort, none of them a signature a caller sees. The breadcrumb an empty
//! answer earns is `refs_note.rs`'s (§FS-refs.2.1).

/// Data-producing implementation behind `refs_outcome`: scan once, select the target
/// grammar, and return either citation data or the typed resolver rejection
/// (§AR-bindings.2, §FS-refs.2, §FS-refs.4).
use anyhow::{Result, anyhow};
use std::collections::BTreeSet;
use std::path::Path;

use super::guidance::invalid_id_hint;
use super::refs::{RefHit, RefsOpts, RefsOutcome, RefsOutput, RefsQueryFailure, RefsWithMetadata};
use super::refs_note::{NoteQuery, empty_answer_note};
use super::report::context_run_warnings;
use crate::config::display_path;
use crate::grammar::{path_at_or_under, render_catalog_id, render_citation_id};
use crate::model::{Citation, sort_path_key};
use crate::queries::{declaration_ambiguity_refusal, reported_scope, reports_site};
use crate::resolver::{WorkspaceContext, WorkspaceProject, load_classifying_workspace_context};
use crate::scanner::{ApiScanError, api_scan_error, resolve_id_arg};
use crate::workspace::split_qualified_id_arg;

/// Whether one citation's own section coordinate is inside the requested
/// scope: the exact path, or — under `--descendants` — that path and every
/// section beneath it (§FS-refs.2). A bare-ID citation carries no coordinate
/// and is in neither scope, as it already was not in an exact one.
fn section_in_scope(cited: Option<&str>, requested: &str, descendants: bool) -> bool {
    match cited {
        Some(cited) if descendants => path_at_or_under(cited, requested, "."),
        cited => cited == Some(requested),
    }
}

/// Every loaded project's scan errors, in project order, spelled against the run's
/// base (§FS-workspace.8.7): what every `refs` mode reports beside its hits.
pub(super) fn refs_scan_errors(context: &WorkspaceContext) -> Vec<ApiScanError> {
    context
        .projects
        .iter()
        .flat_map(|project| {
            project.scan_errors.iter().map(|(file, message)| {
                api_scan_error(context.render_config().frame(), file, message)
            })
        })
        .collect()
}

/// A citing file as a hit spells it: from the workspace root in a workspace, else
/// from its own project's root (§FS-workspace.8.2).
pub(super) fn hit_path(
    context: &WorkspaceContext,
    project: &WorkspaceProject,
    path: &Path,
) -> String {
    if context.workspace_loaded {
        display_path(context.render_config(), path)
    } else {
        display_path(&project.config, path)
    }
}

/// Resolve and refuse recorded target ambiguities before filtering any citations
/// (§FS-refs.4), retaining absent-target queries (§FS-refs.1).
pub(super) fn refs_impl(opts: RefsOpts) -> Result<RefsWithMetadata> {
    refs_impl_with_details(opts, &mut serde_json::json!({}), &mut Vec::new(), &mut None)
}

/// Additive source diagnostics without changing RefsOutput (§FS-distribution.3.1).
pub(super) fn refs_impl_with_details(
    opts: RefsOpts,
    details: &mut serde_json::Value,
    cautions: &mut Vec<crate::Finding>,
    hint: &mut Option<String>,
) -> Result<RefsWithMetadata> {
    refs_impl_with_annotations(opts, details, cautions, hint, None, &mut Vec::new())
}

/// §FS-refs.3.1.1: frontend annotations from this scan, outside result records.
/// `set_aside` receives the text of each warning a broken stub set aside beside a
/// real home earns, which a text frontend alone prints (§FS-refs.4.1.1).
pub(super) fn refs_impl_with_annotations(
    opts: RefsOpts,
    details: &mut serde_json::Value,
    cautions: &mut Vec<crate::Finding>,
    hint: &mut Option<String>,
    index_entries: Option<&mut BTreeSet<(String, usize, usize)>>,
    set_aside: &mut Vec<String>,
) -> Result<RefsWithMetadata> {
    // §AR-scanner.2.4.2: the classifying loader, not the plain one — `refs`
    // publishes each hit's enclosing declaration and section (§FS-refs.3.2), and
    // the plain wrapper is shared by the five queries that do not.
    let context = load_classifying_workspace_context(&opts.path, opts.path_provided)?;
    *cautions = context_run_warnings(&context);
    let current_config = context
        .current_project()
        .map(|project| &project.config)
        .unwrap_or_else(|| context.render_config());
    let (alias, raw_id) = split_qualified_id_arg(&opts.id).map_err(|err| {
        anyhow!(
            "{err:#}\n{}",
            invalid_id_hint(current_config, &opts.id, None)
        )
    })?;
    let target_project = match alias.as_deref() {
        Some(name) => context.project_by_alias(name).ok_or_else(|| {
            if let Some(refusal) = context.missing_import(name) {
                refusal
            } else if !context.workspace_loaded {
                anyhow!(crate::model::OperationDiagnostic::new("query", "unknown-project", format!(
                    "unknown project alias `{name}`\nnote: workspace aliases are defined in the root grund.toml under [workspace]"
                )))
            } else {
                anyhow!(crate::model::OperationDiagnostic::new("query", "unknown-project", format!(
                    "unknown project alias `{name}`\nknown aliases: {}",
                    context.aliases().join(", ")
                )))
            }
        })?,
        None => context.current_project().ok_or_else(|| {
            let known = context.aliases().join(", ");
            if known.is_empty() {
                anyhow!(crate::model::OperationDiagnostic::new("query", "query-failed", format!("unqualified ID requires a project alias when include_root = false")))
            } else {
                anyhow!(crate::model::OperationDiagnostic::new("query", "query-failed", format!(
                    "unqualified ID requires a project alias when include_root = false\nknown aliases: {known}"
                )))
            }
        })?,
    };
    let render_config = &target_project.config;
    let scan_errors = refs_scan_errors(&context);
    // §FS-show.3.5.1: the format hint is for an argument that does not match
    // it; an ambiguous shorthand did match and lists its candidates instead.
    let (id, inline_section) = match resolve_id_arg(
        raw_id,
        render_config.schema(),
        render_config.frame(),
        &target_project.findings,
    ) {
        Ok(resolved) => resolved,
        Err(error) => {
            // §FS-show.3.5.1: carry prose separately; structured formats stay templates.
            *hint = error
                .wants_format_hint()
                .then(|| invalid_id_hint(render_config, &opts.id, Some(raw_id)));
            // §FS-distribution.3.3.2: retain original resolver data.
            *details = error.diagnostic().details;
            return Ok(RefsWithMetadata {
                kind_title: None,
                outcome: RefsOutcome {
                    output: RefsOutput {
                        output_format: render_config.output_format.clone(),
                        workspace: context.workspace_loaded,
                        hits: Vec::new(),
                        note: None,
                        scan_errors,
                        warnings: context_run_warnings(&context),
                    },
                    query_failure: Some(RefsQueryFailure::from_resolver_error(
                        &error,
                        &render_config.id_format,
                    )),
                },
            });
        }
    };
    if opts.section.is_some() && inline_section.is_some() {
        return Err(anyhow!(crate::model::OperationDiagnostic::new(
            "query",
            "query-failed",
            format!("--section cannot be combined with an inline section")
        )));
    }
    let section = opts.section.or(inline_section);

    // §FS-refs.4: only the selected project's homes and requested coordinate
    // can refuse; descendants, summary and total never union claimants.
    match declaration_ambiguity_refusal(
        render_config,
        context.render_config(),
        &target_project.findings,
        &id,
        section.as_deref(),
    ) {
        // §FS-refs.4.1.1: a broken stub beside a real home is set aside and named.
        Ok(warnings) => *set_aside = warnings,
        Err(refusal) => {
            // §FS-refs.4: keep format, warnings and partial-scan precedence beside
            // the typed refusal, without changing the public result records.
            return Err(anyhow::Error::msg(RefsOutput {
                output_format: render_config.output_format.clone(),
                workspace: context.workspace_loaded,
                hits: Vec::new(),
                note: None,
                scan_errors,
                warnings: context_run_warnings(&context),
            })
            .context(refusal));
        }
    }

    struct Hit<'a> {
        project: &'a WorkspaceProject,
        citation: &'a Citation,
    }
    let mut hits = Vec::new();
    let target = context
        .projects
        .iter()
        .position(|project| std::ptr::eq(project, target_project));
    // §AR-resolver.3.3: the report scope's sites, in a workspace as in one project.
    let scope = reported_scope(&context, &opts.path, opts.path_provided)?;
    for (index, project) in context.projects.iter().enumerate() {
        // §FS-refs.2: the set holds what a stub's target lends, resolved.
        for citation in project.findings.edges() {
            // §FS-imports.graph.diamond: either alias path of one tree is that tree.
            let cited = context.citation_target(index, citation);
            if !context.same_target(cited, target) || citation.id != id {
                continue;
            }
            if !reports_site(scope.as_ref(), &project.findings.declarations, citation) {
                continue;
            }
            // §FS-refs.2: `--descendants` widens the exact coordinate to its
            // subtree, on `.` components — a recorded section path is dotted at
            // every depth whatever the outer separator is (§FS-config.3.3.4).
            if let Some(expected) = section.as_deref()
                && !section_in_scope(citation.section.as_deref(), expected, opts.descendants)
            {
                continue;
            }
            // §FS-imports.graph.diamond: each site once, under its canonical alias path.
            let (index, citation) = context.canonical_site(index, citation);
            hits.push(Hit {
                project: &context.projects[index],
                citation,
            });
        }
    }
    hits.sort_by(|a, b| {
        (
            sort_path_key(&a.citation.file),
            a.citation.line,
            a.citation.column,
        )
            .cmp(&(
                sort_path_key(&b.citation.file),
                b.citation.line,
                b.citation.column,
            ))
    });
    hits.dedup_by(|a, b| {
        a.citation.file == b.citation.file
            && a.citation.line == b.citation.line
            && a.citation.column == b.citation.column
    });
    let public_hits = hits
        .iter()
        .map(|hit| RefHit {
            project: context.workspace_loaded.then(|| hit.project.alias.clone()),
            path: hit_path(&context, hit.project, &hit.citation.file),
            line: hit.citation.line,
            column: hit.citation.column,
            id: render_citation_id(
                &render_config.grammar,
                &target_project.findings.declarations,
                hit.citation,
                &hit.project.config.marker,
            ),
            section: hit.citation.section.clone(),
            marker: hit.citation.has_marker,
            text: hit.citation.text.clone(),
            // §FS-refs.3.2: the citing site's own unit, after `text` and before
            // `kind_title`. §FS-workspace.8.2.4: bare, under the **citing**
            // project's grammar — `id` above renders under the target's.
            enclosing_declaration: hit.citation.enclosing_declaration.as_ref().map(|id| {
                render_catalog_id(
                    &hit.project.config.grammar,
                    &hit.project.findings.declarations,
                    id,
                )
            }),
            enclosing_section: hit.citation.enclosing_section.clone(),
        })
        .collect::<Vec<_>>();
    // §FS-refs.2.1: the breadcrumb an empty answer earns, decided in one place.
    let note = public_hits
        .is_empty()
        .then(|| {
            empty_answer_note(&NoteQuery {
                path: opts.path_provided.then_some(opts.path.as_path()),
                target: target_project,
                raw_id,
                id: &id,
                qualified: context.workspace_loaded && alias.is_some(),
                section: section.is_some(),
                complete: scan_errors.is_empty(),
            })
        })
        .flatten();
    // §FS-refs.3.2: kind metadata does not require an existing declaration.
    let kind_title = render_config
        .kinds
        .iter()
        .find(|kind| kind.kind == id.kind)
        .and_then(|kind| kind.title.clone());
    if let Some(entries) = index_entries {
        *entries = super::refs_annotations::index_entry_sites(
            &context,
            hits.iter().map(|hit| (hit.project, hit.citation)),
        );
    }
    Ok(RefsWithMetadata {
        kind_title,
        outcome: RefsOutcome {
            output: RefsOutput {
                output_format: render_config.output_format.clone(),
                workspace: context.workspace_loaded,
                hits: public_hits,
                note,
                scan_errors,
                warnings: context_run_warnings(&context),
            },
            query_failure: None,
        },
    })
}
