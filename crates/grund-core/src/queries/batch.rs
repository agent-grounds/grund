use anyhow::{Result, anyhow};

use super::batch_generation::exhaustive_batch_queries;
use super::glob_scope::{GlobSelection, read_globs};
use super::show::{render_show_output_json, show_declaration_with_home};
use super::show_query::{ShowFormat, ShowOpts, ShowQueryError};
use crate::config::{display_path, run_warning_findings};
use crate::grammar::flatten_cross_ref_links;
use crate::model::{Finding, FindingSite, ShowOutput};
use crate::resolver::{
    WorkspaceContext, load_reported_workspace_context, with_member_id_candidates,
};
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

/// A batch's records and the notes of the globs that generated it
/// (§FS-coordinate-globs.errors.note).
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct BatchShowRun {
    pub records: Vec<BatchShowRecord>,
    pub note: Option<String>,
}

/// Rich batch records preserve all query diagnostics and ShowOutput fields
/// without changing the existing JSON batch API (§FS-distribution.3.1).
pub(crate) struct BatchDataRecord {
    pub(crate) query: BatchShowQuery,
    pub(crate) result: Result<ShowOutput>,
    pub(crate) metadata: serde_json::Value,
}

/// A batch's records, and the note its globs earned (§FS-coordinate-globs.bindings.note).
pub(crate) type BatchData = (Vec<BatchDataRecord>, Option<String>);

/// The same shared context and query implementation, with source-typed failures
/// retained rather than projected to CLI JSON (§FS-distribution.3.3.2).
pub(crate) fn show_batch_data(
    queries: Option<Vec<BatchShowQuery>>,
    globs: &[String],
    mut opts: ShowOpts,
    path_provided: bool,
) -> (Vec<Finding>, Result<BatchData>) {
    let mut cautions = Vec::new();
    let result = (|| {
        let globs = batch_globs(queries.as_ref(), globs)?;
        if queries.as_ref().is_some_and(Vec::is_empty) {
            return Ok((Vec::new(), None));
        }
        let context = load_reported_workspace_context(&opts.path, path_provided)?;
        cautions = run_warning_findings(context.render_config(), context.run_warnings.clone());
        let selection = selection(&context, &globs)?;
        if let Some((file, message)) = context.projects.iter().find_map(|p| p.scan_errors.first()) {
            let mut diagnostic = crate::model::OperationDiagnostic::new(
                "filesystem",
                "io",
                crate::scanner::scan_error_message(message),
            );
            diagnostic.path = Some(display_path(context.render_config(), file));
            return Err(diagnostic.into());
        }
        let queries = match queries {
            Some(queries) => queries,
            None => {
                exhaustive_batch_queries(&context, &opts.path, path_provided, selection.as_ref())?
            }
        };
        let note = selection.and_then(|selection| selection.note());
        opts.format = ShowFormat::Json;
        let records: Result<Vec<_>> = queries
            .into_iter()
            .map(|query| {
                let mut metadata = serde_json::Value::Null;
                let result = show_batch_query_with_metadata(
                    &context,
                    &query.id,
                    ShowOpts {
                        section: query.section.clone(),
                        ..opts.clone()
                    },
                    &mut metadata,
                );
                match result {
                    Ok(mut output) => {
                        output.path = display_path(context.render_config(), &output.path).into();
                        Ok(BatchDataRecord {
                            query,
                            result: Ok(output),
                            metadata,
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
                            metadata,
                        })
                    }
                    Err(error) => Err(error),
                }
            })
            .collect();
        Ok((records?, note))
    })();
    (cautions, result)
}

/// The batch's globs read before discovery; explicit queries take none
/// (§FS-coordinate-globs.bindings.options).
fn batch_globs(
    queries: Option<&Vec<BatchShowQuery>>,
    globs: &[String],
) -> Result<Vec<crate::grammar::CoordinateGlob>> {
    if queries.is_some() && !globs.is_empty() {
        return Err(crate::model::OperationDiagnostic::new(
            "input",
            "conflicting-options",
            "explicit queries cannot be combined with globs",
        )
        .into());
    }
    read_globs(globs)
}

fn selection<'g>(
    context: &WorkspaceContext,
    globs: &'g [crate::grammar::CoordinateGlob],
) -> Result<Option<GlobSelection<'g>>> {
    match globs.is_empty() {
        true => Ok(None),
        false => GlobSelection::new(context, globs).map(Some),
    }
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
    let (run_warnings, run) = show_batch_with_globs(queries, &[], opts, path_provided);
    (run_warnings, run.map(|run| run.records))
}

/// [`show_batch_with_scope`] with coordinate globs filtering the generated batch
/// (§FS-show.1.9): `queries` is `None` whenever `globs` is not empty.
#[doc(hidden)]
pub fn show_batch_with_globs(
    queries: Option<Vec<BatchShowQuery>>,
    globs: &[String],
    opts: ShowOpts,
    path_provided: bool,
) -> (Vec<Finding>, Result<BatchShowRun>) {
    let mut run_warnings = Vec::new();
    let run = show_batch_run(queries, globs, opts, path_provided, &mut run_warnings);
    (run_warnings, run)
}

fn show_batch_run(
    queries: Option<Vec<BatchShowQuery>>,
    globs: &[String],
    mut opts: ShowOpts,
    path_provided: bool,
    run_warnings: &mut Vec<Finding>,
) -> Result<BatchShowRun> {
    let globs = batch_globs(queries.as_ref(), globs)?;
    if queries.as_ref().is_some_and(Vec::is_empty) {
        return Ok(BatchShowRun {
            records: Vec::new(),
            note: None,
        });
    }

    // §AR-resolver.3.1: this is the batch's only loader entry. Exhaustive
    // discovery reads the returned catalog and never starts a preliminary scan.
    // §AR-resolver.3.3: every query resolves against the whole owning project.
    let context = load_reported_workspace_context(&opts.path, path_provided)?;
    // §FS-check.4.10.11, §FS-workspace.6.1.7: the run's `[workspace]`
    // warnings come back beside the records, because the batch refuses after the
    // workspace pass has already settled them.
    *run_warnings = run_warning_findings(context.render_config(), context.run_warnings.clone());
    let selection = selection(&context, &globs)?;
    if let Some((file, message)) = context
        .projects
        .iter()
        .find_map(|project| project.scan_errors.first())
    {
        return Err(anyhow!(
            "{}: {}",
            display_path(context.render_config(), file),
            crate::scanner::scan_error_message(message)
        ));
    }
    let queries = match queries {
        Some(queries) => queries,
        None => exhaustive_batch_queries(&context, &opts.path, path_provided, selection.as_ref())?,
    };
    let note = selection.and_then(|selection| selection.note());
    opts.format = ShowFormat::Json;

    let records = queries
        .into_iter()
        .map(|query| {
            let query_opts = ShowOpts {
                section: query.section.clone(),
                ..opts.clone()
            };
            let result = match show_batch_query_in_context(&context, &query.id, query_opts) {
                Ok(output) => Ok(output.json.unwrap_or_default()),
                Err(error) => match batch_query_failure(&error) {
                    Some(failure) => Err(failure),
                    None => return Err(error),
                },
            };
            Ok(BatchShowRecord { query, result })
        })
        .collect::<Result<_>>()?;
    Ok(BatchShowRun { records, note })
}

/// Resolve one batch coordinate through the shared context and the same lower
/// level resolver, body extractor, flattening, and JSON renderer as single show
/// (§FS-show.2.6.1, §AR-resolver.3.1).
fn show_batch_query_in_context(
    context: &WorkspaceContext,
    id_arg: &str,
    opts: ShowOpts,
) -> Result<ShowOutput> {
    show_batch_query_with_metadata(context, id_arg, opts, &mut serde_json::Value::Null)
}

fn show_batch_query_with_metadata(
    context: &WorkspaceContext,
    id_arg: &str,
    opts: ShowOpts,
    metadata: &mut serde_json::Value,
) -> Result<ShowOutput> {
    let (alias, raw_id) = split_qualified_id_arg(id_arg)?;
    let project = match alias.as_deref() {
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
    let config = &project.config;
    let (id, inline_section) =
        resolve_id_arg(raw_id, config.schema(), config.frame(), &project.findings)
            .map_err(|error| anyhow::Error::new(error.diagnostic()))?;
    if opts.section.is_some() && inline_section.is_some() {
        return Err(anyhow!(crate::model::OperationDiagnostic::new(
            "query",
            "query-failed",
            format!("--section cannot be combined with an inline section")
        )));
    }
    let section = opts.section.or(inline_section);
    *metadata = super::show_metadata(config, &project.findings, &id, section.as_deref());
    let (mut output, home) = show_declaration_with_home(
        config,
        context.render_config(),
        &project.findings,
        &id,
        Some(raw_id),
        section.as_deref(),
        opts.mode.render_mode(),
        false,
    )
    .map_err(|error| with_member_id_candidates(error, context, alias.as_deref(), raw_id))?;
    // §FS-show.2.5, §FS-show.3.2.1: batch reads use the same Markdown fence
    // precedence and ordinary-prose inverse as single reads.
    let reading = config.reading(&output.path);
    output.body = flatten_cross_ref_links(&output.body, reading.lexical, reading.markdown);
    output.json = Some(render_show_output_json(
        config,
        &project.findings.files,
        context.render_config(),
        &home,
        &id,
        section.as_deref(),
        opts.mode.render_mode(),
        &output,
    ));
    Ok(output)
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
