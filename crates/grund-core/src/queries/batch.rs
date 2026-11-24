//! The records `show --batch` answers with and the steps of its run that need no
//! answer yet: the globs, their selection, a failed query's header, and which
//! failures are a query's rather than the run's (§FS-show.1.8, §FS-show.2.6). The
//! one loop that answers every query is `api/show_batch.rs`, beside the single
//! read it shares its answer with (§AR-system.principle).

use anyhow::Result;

use super::glob_scope::{GlobSelection, read_globs};
use super::show_query::ShowQueryError;
use crate::model::{FindingSite, ShowOutput};
use crate::resolver::WorkspaceContext;

/// One input coordinate for the CLI-only batch-show adapter
/// (§FS-show.1.8, §FS-show.2.6).
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchShowQuery {
    pub id: String,
    pub section: Option<String>,
}

/// A query failure carried in its place among the answers so one bad coordinate
/// does not stop the rest (§FS-output-shapes.4.1, §FS-show.3.3.2). `hints` are the
/// `hint:` lines the single read prints below the same failure, built from the
/// batch's one loaded context; a `json` envelope does not print them
/// (§AR-resolver.3.1.1, §FS-output-shapes.6.1).
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchShowFailure {
    pub code: &'static str,
    pub message: String,
    pub sites: Vec<FindingSite>,
    pub hints: Vec<String>,
}

/// One query's answer, in the format the batch was asked for, or its failure,
/// paired with the spelling the `json` envelope preserves and the coordinate a
/// `text` or `md` header names it by (§FS-output-shapes.4.1, §FS-show.3.3.1).
#[doc(hidden)]
pub struct BatchShowRecord {
    pub query: BatchShowQuery,
    pub coordinate: String,
    pub result: std::result::Result<ShowOutput, BatchShowFailure>,
}

/// A batch's records and the notes of the globs that generated it
/// (§FS-coordinate-globs.errors.note).
#[doc(hidden)]
pub struct BatchShowRun {
    pub records: Vec<BatchShowRecord>,
    pub note: Option<String>,
}

/// The batch's globs read before discovery; explicit queries take none
/// (§FS-coordinate-globs.bindings.options).
pub(crate) fn batch_globs(
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

/// The selection the batch's globs make over the loaded context, or `None`
/// where there are no globs (§FS-show.1.9).
pub(crate) fn batch_selection<'g>(
    context: &WorkspaceContext,
    globs: &'g [crate::grammar::CoordinateGlob],
) -> Result<Option<GlobSelection<'g>>> {
    match globs.is_empty() {
        true => Ok(None),
        false => GlobSelection::new(context, globs).map(Some),
    }
}

/// The header a failed query is named by in a `text` or `md` batch: the query as
/// written, a separate `section` joined to its `id` by the run's own section
/// separator (§FS-show.3.3.1).
pub(crate) fn written_coordinate(context: &WorkspaceContext, query: &BatchShowQuery) -> String {
    match &query.section {
        Some(section) => format!(
            "{}{}{section}",
            query.id,
            context.render_config().section_separator
        ),
        None => query.id.clone(),
    }
}

/// Convert only coordinate-level refusals into envelopes. An unexpected body
/// read or other operational error remains a run-level abort (§FS-show.2.6).
pub(crate) fn batch_query_failure(error: &anyhow::Error) -> Option<BatchShowFailure> {
    // §FS-distribution.3.3.2: new source carriers need no message classification.
    if let Some(carrier) = error.downcast_ref::<crate::model::OperationDiagnostic>()
        && carrier.class == "query"
    {
        return Some(BatchShowFailure {
            code: carrier.code,
            message: carrier.message.clone(),
            sites: Vec::new(),
            hints: Vec::new(),
        });
    }
    if let Some(carrier) = error.downcast_ref::<ShowQueryError>() {
        return Some(BatchShowFailure {
            code: carrier.code,
            message: carrier.message.clone(),
            sites: carrier.sites.clone(),
            hints: Vec::new(),
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
        hints: Vec::new(),
    })
}
