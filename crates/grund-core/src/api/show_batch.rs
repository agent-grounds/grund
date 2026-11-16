//! `grund show --batch` and the bindings' `showBatch` (§FS-show.1.8, §FS-show.2.6,
//! §FS-distribution.3.1): one loop that loads the workspace once and answers every
//! query through the single read's own [`ShowSource`], so the bytes under each
//! answer are the ones `grund <ID>` prints, in whichever format was asked for, and a
//! failure carries the hint the single read prints for it (§AR-resolver.3.1,
//! §AR-resolver.3.1.1, §AR-system.principle). The CLI adapter and the bindings'
//! adapter differ only in what they keep of each answer.

use anyhow::Result;

use super::guidance::query_failure_hints;
use super::show::ShowSource;
use crate::config::{display_path, run_warning_findings};
use crate::model::{Finding, OperationDiagnostic, ShowOutput};
use crate::queries::{
    BatchShowQuery, BatchShowRecord, BatchShowRun, ShowFormat, ShowOpts, ShowQueryError,
    batch_globs, batch_query_failure, batch_selection, exhaustive_batch_queries,
    written_coordinate,
};
use crate::resolver::{load_reported_workspace_context, stopping_failure};
use crate::scanner::{ApiScanError, scan_operation_failure};

/// One query's answer from the batch's one context, before an adapter keeps
/// what its surface prints of it.
struct BatchAnswer {
    query: BatchShowQuery,
    /// The `text`/`md` header: the answered coordinate, or the query as written
    /// when it failed (§FS-show.3.3.1).
    coordinate: String,
    result: Result<ShowOutput>,
    metadata: serde_json::Value,
    /// The failure's `hint:` lines, from the same context (§AR-resolver.3.1.1).
    hints: Vec<String>,
}

/// A batch's answers and the note its globs earned (§FS-coordinate-globs.errors.note).
type BatchAnswers = (Vec<BatchAnswer>, Option<String>);

/// The one batch loop. An empty explicit stream loads nothing; every other batch
/// loads once, refuses a scan failure as `show` does, and answers each query in
/// order against that context in `opts.format` (§FS-show.2.6, §FS-show.3.3).
fn batch_answers(
    queries: Option<Vec<BatchShowQuery>>,
    globs: &[String],
    opts: &ShowOpts,
    path_provided: bool,
    run_warnings: &mut Vec<Finding>,
) -> Result<BatchAnswers> {
    let globs = batch_globs(queries.as_ref(), globs)?;
    if queries.as_ref().is_some_and(Vec::is_empty) {
        return Ok((Vec::new(), None));
    }
    // §AR-resolver.3.1: this is the batch's only loader entry. Exhaustive
    // discovery reads the returned catalog and never starts a preliminary scan.
    // §AR-resolver.3.3: every query resolves against the whole owning project.
    let context = load_reported_workspace_context(&opts.path, path_provided)?;
    // §FS-check.4.10.11, §FS-workspace.6.1.7: the run's `[workspace]` warnings come
    // back beside the records, because the batch refuses after the workspace pass
    // has already settled them.
    *run_warnings = run_warning_findings(context.render_config(), context.run_warnings.clone());
    let selection = batch_selection(&context, &globs)?;
    // §AR-scanner.1.6: the failure `show` stops at, else the first any project met.
    if let Some((first, _)) = stopping_failure(&context.projects, &context.projects) {
        // §FS-errors.2.2.5: the one form, the report base `.` (§FS-cli.3.4).
        let frame = context.render_config().frame();
        return Err(scan_operation_failure(frame, first).into());
    }
    let queries = match queries {
        Some(queries) => queries,
        None => exhaustive_batch_queries(&context, &opts.path, path_provided, selection.as_ref())?,
    };
    let note = selection.and_then(|selection| selection.note());
    let answers = queries
        .into_iter()
        .map(|query| {
            let mut metadata = serde_json::Value::Null;
            let answered = ShowSource::resolve(&context, &query.id, query.section.clone())
                .and_then(|source| {
                    let output = source.answer(&context, opts.mode, opts.format, &mut metadata)?;
                    Ok((source.coordinate(&context), output))
                });
            let (coordinate, result, hints) = match answered {
                Ok((coordinate, mut output)) => {
                    // §FS-cli.3.4: a record at the report base reads `.`.
                    output.path = display_path(context.render_config(), &output.path).into();
                    (coordinate, Ok(output), Vec::new())
                }
                Err(error) => {
                    let message = ApiScanError::printed(&error);
                    let hints = query_failure_hints(&context, &query.id, &message);
                    (written_coordinate(&context, &query), Err(error), hints)
                }
            };
            BatchAnswer {
                query,
                coordinate,
                result,
                metadata,
                hints,
            }
        })
        .collect();
    Ok((answers, note))
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
/// (§FS-show.1.9): `queries` is `None` whenever `globs` is not empty. Each record
/// carries its answer in `opts.format`, or its failure with its hint, and the
/// coordinate a `text` or `md` header names it by (§FS-show.3.3). Only
/// coordinate-level refusals become records; an unexpected body read or other
/// operational error aborts the run (§FS-show.2.6.3).
#[doc(hidden)]
pub fn show_batch_with_globs(
    queries: Option<Vec<BatchShowQuery>>,
    globs: &[String],
    opts: ShowOpts,
    path_provided: bool,
) -> (Vec<Finding>, Result<BatchShowRun>) {
    let mut run_warnings = Vec::new();
    let answers = batch_answers(queries, globs, &opts, path_provided, &mut run_warnings);
    let run = answers.and_then(|(answers, note)| {
        let records = answers
            .into_iter()
            .map(|answer| {
                let result = match answer.result {
                    Ok(output) => Ok(output),
                    Err(error) => match batch_query_failure(&error) {
                        Some(failure) => Err(crate::queries::BatchShowFailure {
                            hints: answer.hints,
                            ..failure
                        }),
                        None => return Err(error),
                    },
                };
                Ok(BatchShowRecord {
                    query: answer.query,
                    coordinate: answer.coordinate,
                    result,
                })
            })
            .collect::<Result<_>>()?;
        Ok(BatchShowRun { records, note })
    });
    (run_warnings, run)
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
/// retained rather than projected to CLI JSON (§FS-distribution.3.3.2). The
/// bindings take no format: they read the CLI's JSON batch bytes
/// (§FS-distribution.3.2.1).
pub(crate) fn show_batch_data(
    queries: Option<Vec<BatchShowQuery>>,
    globs: &[String],
    mut opts: ShowOpts,
    path_provided: bool,
) -> (Vec<Finding>, Result<BatchData>) {
    let mut cautions = Vec::new();
    opts.format = ShowFormat::Json;
    let answers = batch_answers(queries, globs, &opts, path_provided, &mut cautions);
    let data = answers.and_then(|(answers, note)| {
        let records = answers
            .into_iter()
            .map(|answer| match answer.result {
                Err(error) if !is_query_failure(&error) => Err(error),
                result => Ok(BatchDataRecord {
                    query: answer.query,
                    result,
                    metadata: answer.metadata,
                }),
            })
            .collect::<Result<_>>()?;
        Ok((records, note))
    });
    (cautions, data)
}

/// A failure the bindings carry as one query's, by its typed carrier alone
/// (§FS-distribution.3.3.2).
fn is_query_failure(error: &anyhow::Error) -> bool {
    error.downcast_ref::<ShowQueryError>().is_some()
        || error
            .downcast_ref::<OperationDiagnostic>()
            .is_some_and(|error| error.class == "query")
}
