use anyhow::{Result, anyhow};
use std::collections::BTreeSet;

use super::show::{render_show_output_json, show_declaration_with_overlays};
use super::show_query::{ShowFormat, ShowOpts, ShowQueryError};
use crate::config::{display_path, run_warning_findings};
use crate::grammar::{flatten_cross_ref_links, render_id};
use crate::model::{Finding, FindingSite, ShowOutput, TextOverlays};
use crate::resolver::{WorkspaceContext, load_workspace_context, with_member_id_candidates};
use crate::scanner::resolve_id_arg;
use crate::workspace::split_qualified_id_arg;

/// One input coordinate for the CLI-only batch-show adapter
/// (§FS-show.1.8, §FS-show.2.6).
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchShowQuery {
    pub id: String,
    pub section: Option<String>,
}

/// A query failure carried inside a batch envelope so one bad coordinate does
/// not stop the rest (§FS-output-shapes.4.1).
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchShowFailure {
    pub code: &'static str,
    pub message: String,
    pub sites: Vec<FindingSite>,
}

/// The rendered single-show JSON object or its per-query failure, paired with
/// the spelling batch output must preserve (§FS-output-shapes.4.1).
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct BatchShowRecord {
    pub query: BatchShowQuery,
    pub result: std::result::Result<String, BatchShowFailure>,
}

/// Rich batch records preserve all query diagnostics and ShowOutput fields
/// without changing the existing JSON batch API (§FS-distribution.3.1).
pub(crate) struct BatchDataRecord {
    pub(crate) query: BatchShowQuery,
    pub(crate) result: Result<ShowOutput>,
}

/// The same shared context and query implementation, with source-typed failures
/// retained rather than projected to CLI JSON (§FS-distribution.3.3.2).
pub(crate) fn show_batch_data(
    queries: Option<Vec<BatchShowQuery>>,
    mut opts: ShowOpts,
    path_provided: bool,
) -> (Vec<Finding>, Result<Vec<BatchDataRecord>>) {
    let mut cautions = Vec::new();
    let result = (|| {
        if queries.as_ref().is_some_and(Vec::is_empty) {
            return Ok(Vec::new());
        }
        let context = load_workspace_context(&opts.path, path_provided)?;
        cautions = run_warning_findings(context.render_config(), context.run_warnings.clone());
        if let Some((file, message)) = context.projects.iter().find_map(|p| p.scan_errors.first()) {
            let mut diagnostic =
                crate::model::OperationDiagnostic::new("filesystem", "io", message);
            diagnostic.path = Some(display_path(context.render_config(), file));
            return Err(diagnostic.into());
        }
        let queries = queries.unwrap_or_else(|| exhaustive_batch_queries(&context));
        opts.format = ShowFormat::Json;
        queries
            .into_iter()
            .map(|query| {
                let result = show_batch_query_in_context(
                    &context,
                    &query.id,
                    ShowOpts {
                        section: query.section.clone(),
                        ..opts.clone()
                    },
                    &TextOverlays::new(),
                );
                match result {
                    Ok(mut output) => {
                        output.path = display_path(context.render_config(), &output.path).into();
                        Ok(BatchDataRecord {
                            query,
                            result: Ok(output),
                        })
                    }
                    Err(error)
                        if error.downcast_ref::<ShowQueryError>().is_some()
                            || error
                                .downcast_ref::<crate::model::OperationDiagnostic>()
                                .is_some_and(|e| e.class == "query") =>
                    {
                        Ok(BatchDataRecord {
                            query,
                            result: Err(error),
                        })
                    }
                    Err(error) => Err(error),
                }
            })
            .collect()
    })();
    (cautions, result)
}

/// Run an explicit query list (`Some`) or exhaustive discovery (`None`) against
/// one shared workspace context. This is intentionally a CLI adapter rather
/// than a replacement for the stable one-query core API (§FS-show.2.6).
#[doc(hidden)]
pub fn show_batch_with_scope(
    queries: Option<Vec<BatchShowQuery>>,
    opts: ShowOpts,
    path_provided: bool,
) -> (Vec<Finding>, Result<Vec<BatchShowRecord>>) {
    let mut run_warnings = Vec::new();
    let records = show_batch_run(queries, opts, path_provided, &mut run_warnings);
    (run_warnings, records)
}

fn show_batch_run(
    queries: Option<Vec<BatchShowQuery>>,
    mut opts: ShowOpts,
    path_provided: bool,
    run_warnings: &mut Vec<Finding>,
) -> Result<Vec<BatchShowRecord>> {
    if queries.as_ref().is_some_and(Vec::is_empty) {
        return Ok(Vec::new());
    }

    // §AR-resolver.3.1: this is the batch's only loader entry. Exhaustive
    // discovery reads the returned catalog and never starts a preliminary scan.
    let context = load_workspace_context(&opts.path, path_provided)?;
    // §FS-check.4.10.11, §FS-workspace.6.1.7: the run's `[workspace]`
    // warnings come back beside the records, because the batch refuses after the
    // workspace pass has already settled them.
    *run_warnings = run_warning_findings(context.render_config(), context.run_warnings.clone());
    if let Some((file, message)) = context
        .projects
        .iter()
        .find_map(|project| project.scan_errors.first())
    {
        return Err(anyhow!(
            "{}: {}",
            display_path(context.render_config(), file),
            message
        ));
    }
    let queries = queries.unwrap_or_else(|| exhaustive_batch_queries(&context));
    opts.format = ShowFormat::Json;

    queries
        .into_iter()
        .map(|query| {
            let query_opts = ShowOpts {
                section: query.section.clone(),
                ..opts.clone()
            };
            let result = match show_batch_query_in_context(
                &context,
                &query.id,
                query_opts,
                &TextOverlays::new(),
            ) {
                Ok(output) => Ok(output.json.unwrap_or_default()),
                Err(error) => match batch_query_failure(&error) {
                    Some(failure) => Err(failure),
                    None => return Err(error),
                },
            };
            Ok(BatchShowRecord { query, result })
        })
        .collect()
}

/// Resolve one batch coordinate through the shared context and the same lower
/// level resolver, body extractor, flattening, and JSON renderer as single show
/// (§FS-show.2.6.1, §AR-resolver.3.1).
fn show_batch_query_in_context(
    context: &WorkspaceContext,
    id_arg: &str,
    opts: ShowOpts,
    overlays: &TextOverlays,
) -> Result<ShowOutput> {
    let (alias, raw_id) = split_qualified_id_arg(id_arg)?;
    let project = match alias.as_deref() {
        Some(name) => context.project_by_alias(name).ok_or_else(|| {
            if !context.workspace_loaded {
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
    let config = &project.config;
    let (id, inline_section) = resolve_id_arg(raw_id, config, &project.findings)
        .map_err(|error| anyhow::Error::new(error.diagnostic()))?;
    if opts.section.is_some() && inline_section.is_some() {
        return Err(anyhow!(crate::model::OperationDiagnostic::new(
            "query",
            "query-failed",
            format!("--section cannot be combined with an inline section")
        )));
    }
    let section = opts.section.or(inline_section);
    let mut output = show_declaration_with_overlays(
        config,
        context.render_config(),
        &project.findings,
        &id,
        section.as_deref(),
        opts.mode.render_mode(),
        false,
        overlays,
    )
    .map_err(|error| with_member_id_candidates(error, context, alias.as_deref(), raw_id))?;
    let markdown_body = output.path.extension().and_then(|ext| ext.to_str()) == Some("md");
    // §FS-show.2.5, §FS-show.3.2.1: batch reads use the same Markdown fence
    // precedence and ordinary-prose inverse as single reads.
    output.body = flatten_cross_ref_links(&output.body, config.lexical(), markdown_body);
    output.json = Some(render_show_output_json(
        config,
        context.render_config(),
        &id,
        section.as_deref(),
        opts.mode.render_mode(),
        &output,
    ));
    Ok(output)
}

/// Generate stable coordinates from the declarations and sections already in
/// the loaded context (§FS-show.2.6.2). BTree sets collapse duplicate claimants;
/// the final bytewise sort is over the public qualified spelling.
fn exhaustive_batch_queries(context: &WorkspaceContext) -> Vec<BatchShowQuery> {
    let mut queries = Vec::new();
    for (project_index, project) in context.projects.iter().enumerate() {
        for (id, declarations) in &project.findings.declarations {
            let local_id = render_id(&project.config.grammar, id);
            let qualified_id = if context.current == Some(project_index) {
                local_id
            } else {
                format!("{}/{}", project.alias, local_id)
            };
            queries.push(BatchShowQuery {
                id: qualified_id.clone(),
                section: None,
            });
            let sections: BTreeSet<&str> = declarations
                .iter()
                .flat_map(|declaration| {
                    declaration.sections.keys().map(String::as_str).chain(
                        declaration
                            .duplicate_sections
                            .iter()
                            .map(|(path, _)| path.as_str()),
                    )
                })
                .collect();
            queries.extend(sections.into_iter().map(|section| BatchShowQuery {
                id: qualified_id.clone(),
                section: Some(section.to_string()),
            }));
        }
    }
    queries.sort_by(|left, right| {
        left.id
            .as_bytes()
            .cmp(right.id.as_bytes())
            .then_with(|| left.section.as_deref().cmp(&right.section.as_deref()))
    });
    queries
}

/// Convert only coordinate-level refusals into envelopes. An unexpected body
/// read or other operational error remains a run-level abort (§FS-show.2.6.3).
fn batch_query_failure(error: &anyhow::Error) -> Option<BatchShowFailure> {
    // §FS-distribution.3.3.2: new source carriers need no message classification.
    if let Some(carrier) = error.downcast_ref::<crate::model::OperationDiagnostic>()
        && carrier.class == "query"
    {
        return Some(BatchShowFailure {
            code: carrier.code,
            message: carrier.message.clone(),
            sites: Vec::new(),
        });
    }
    if let Some(carrier) = error.downcast_ref::<ShowQueryError>() {
        return Some(BatchShowFailure {
            code: carrier.code,
            message: carrier.message.clone(),
            sites: carrier.sites.clone(),
        });
    }
    let message = format!("{error:#}");
    let code = if message.starts_with("ID not found:") {
        "not-found"
    } else if message.starts_with("ambiguous ID:") {
        "ambiguous"
    } else if message.starts_with("section not found:") {
        "missing-section"
    } else if message.starts_with("invalid ID") {
        "invalid-id"
    } else if message.starts_with("broken stub:") {
        "broken-stub"
    } else if message.starts_with("unknown project alias") {
        "unknown-project"
    } else if message.starts_with("invalid project alias")
        || message.starts_with("unqualified ID requires a project alias")
        || message == "--section cannot be combined with an inline section"
    {
        "query-failed"
    } else {
        return None;
    };
    Some(BatchShowFailure {
        code,
        message,
        sites: Vec::new(),
    })
}
