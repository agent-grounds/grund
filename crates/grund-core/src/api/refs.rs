//! The published `refs` contract (§AR-system.2.9): the citation sites of one ID,
//! and the typed resolver rejection a process frontend applies §FS-refs.4's
//! staged exit policy from without parsing prose (§FS-refs, §AR-bindings.2).
//!
//! The walk that produces both is `refs_query.rs`, the adapter §AR-core-module-layout.2
//! keeps out of the contract.

use anyhow::{Result, anyhow};
use std::collections::BTreeSet;
use std::path::PathBuf;

use super::refs_query::refs_impl;
use crate::grammar::IdArgError;
use crate::model::Finding;
use crate::scanner::ApiScanError;

#[derive(Clone)]
#[non_exhaustive]
pub struct RefsOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub id: String,
    pub section: Option<String>,
    /// Widen the section filter from the exact coordinate to that coordinate
    /// and every section beneath it (§FS-refs.1). With no section in play
    /// there is nothing to widen and the field changes nothing (§FS-refs.4),
    /// so a caller holding an operand it did not build may set it either way.
    pub descendants: bool,
}

impl Default for RefsOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
            id: String::new(),
            section: None,
            descendants: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RefHit {
    pub project: Option<String>,
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub id: String,
    pub section: Option<String>,
    pub marker: bool,
    pub text: String,
    /// The nearest declaration whose body contains this **citing** site, or
    /// `None` where the site sits in no declaration body (§FS-refs.3.2). Bare —
    /// rendered under the *citing* project's `[id]` config, never
    /// alias-qualified, because the declaration it names sits in the citing file
    /// and `project` already says which project that is (§FS-workspace.8.2.4).
    /// Unlike `id`, which renders under the **target** project's config.
    pub enclosing_declaration: Option<String>,
    /// The nearest accepted section path containing this **citing** site,
    /// numbered and named alike, or `None` where no accepted section contains it
    /// (§FS-refs.3.2, §AR-scanner.2.4.4). The citing-side counterpart of
    /// `section`, which is the coordinate of the *cited* ID.
    pub enclosing_section: Option<String>,
}

/// The two ways a selected project's resolver can reject a `refs` operand
/// (§FS-refs.4). This is query-result data rather than an [`anyhow::Error`], so
/// every frontend can apply the same staged exit policy without parsing prose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RefsQueryFailureKind {
    InvalidId,
    Ambiguous,
}

impl RefsQueryFailureKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::InvalidId => "invalid-id",
            Self::Ambiguous => "ambiguous",
        }
    }
}

/// A `refs` operand rejected after workspace context and the target grammar
/// were selected (§FS-errors.2.3, §FS-workspace.8.7.2).
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RefsQueryFailure {
    pub kind: RefsQueryFailureKind,
    pub message: String,
    pub format_hint: Option<String>,
}

impl RefsQueryFailure {
    pub(crate) fn from_resolver_error(error: &IdArgError, id_format: &str) -> Self {
        let kind = match error {
            IdArgError::Unparsable(_) => RefsQueryFailureKind::InvalidId,
            IdArgError::Ambiguous(_) => RefsQueryFailureKind::Ambiguous,
        };
        Self {
            kind,
            message: error.to_string(),
            format_hint: error.wants_format_hint().then(|| id_format.to_string()),
        }
    }

    /// §FS-show.3.5.1: prose comes from the selected project's guidance.
    pub(crate) fn into_refs_error(self, hint: Option<String>) -> anyhow::Error {
        if let Some(hint) = hint {
            anyhow!("{}\n{hint}", self.message)
        } else {
            anyhow!(self.message)
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct RefsOutput {
    pub output_format: String,
    pub workspace: bool,
    pub hits: Vec<RefHit>,
    pub note: Option<String>,
    pub scan_errors: Vec<ApiScanError>,
    /// The run's warning channel (§FS-distribution.3.1): the three `[workspace]`
    /// cautions of §FS-check.3.29.15, §FS-check.4.10.11 and
    /// §FS-workspace.6.1.7. A frontend renders each as one CLI-level `warning:`
    /// on stderr (§FS-check.2.1.1).
    ///
    /// Three keep the anchor the engine gave them — the `grund.toml` line their
    /// own message already names — and an editor publishes those on that line
    /// (§FS-lsp.1.1.3). §FS-check.3.29.15's is the exception: it carries no
    /// location field at all, states its location inside its own text
    /// (§FS-check.3.29.7), and reaches an editor off `check`'s report instead,
    /// located and as an error (§FS-check.3.29.13).
    pub warnings: Vec<Finding>,
}

/// An ambiguity error retains the run metadata as downcastable context
/// (§FS-refs.4). The outer `ShowQueryError` owns the diagnostic and its sites
/// (§FS-errors.5.2.1); this label never replaces that query refusal's message.
impl std::fmt::Display for RefsOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("refs scan context")
    }
}

/// The additive classified `refs` result process frontends render a resolver
/// rejection from as a failed query (§FS-refs.4). Keeping the carrier outside [`RefsOutput`] preserves
/// the exhaustively constructible embedding API required by §AR-bindings.2.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct RefsOutcome {
    pub output: RefsOutput,
    pub query_failure: Option<RefsQueryFailure>,
}

/// Programmatic `refs` with the typed resolver-rejection outcome process
/// frontends need to apply §FS-refs.4 without parsing an error string.
/// Recorded target ambiguities return `Err` carrying `ShowQueryError` and the
/// run's `RefsOutput` metadata.
pub fn refs_outcome(opts: RefsOpts) -> Result<RefsOutcome> {
    Ok(refs_with_metadata(opts)?.outcome)
}

/// Programmatic `refs`: resolve an ID query and return all citation sites
/// without selecting text/summary/JSON rendering (§AR-bindings.2). This keeps
/// the established API: resolver and setup failures are returned as `Err`.
pub fn refs(opts: RefsOpts) -> Result<RefsOutput> {
    let (metadata, hint) = refs_with_metadata_and_hint(opts)?;
    let outcome = metadata.outcome;
    match outcome.query_failure {
        Some(failure) => Err(failure.into_refs_error(hint)),
        None => Ok(outcome.output),
    }
}

/// Additive query context for §FS-config.3.4.3 and §FS-refs.3.2. The title
/// belongs to the selected target kind, not to any hit's citing project.
/// The existing exhaustively constructible refs records stay unchanged.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RefsWithMetadata {
    pub outcome: RefsOutcome,
    pub kind_title: Option<String>,
}

/// Read refs and target metadata from one resolved context, including queries
/// for valid undeclared IDs (§FS-refs.3.2). File summaries ignore this metadata.
/// Duplicate-home and requested-section refusals carry their typed sites and
/// scan metadata in `Err` (§FS-refs.4, §FS-errors.5.2.1).
pub fn refs_with_metadata(opts: RefsOpts) -> Result<RefsWithMetadata> {
    refs_impl(opts)
}

/// §FS-show.3.5.1: frontend prose from the same context as the typed refusal,
/// without adding presentation fields to public result records or scanning again.
#[doc(hidden)]
pub fn refs_with_metadata_and_hint(opts: RefsOpts) -> Result<(RefsWithMetadata, Option<String>)> {
    let mut hint = None;
    let metadata = super::refs_query::refs_impl_with_details(
        opts,
        &mut serde_json::json!({}),
        &mut Vec::new(),
        &mut hint,
    )?;
    Ok((metadata, hint))
}

/// §FS-refs.3.1.1 / §FS-refs.3.3.1: frontend-only excluded sites alongside the
/// unchanged records and resolver hint, derived from the same loaded context.
#[doc(hidden)]
pub fn refs_with_text_annotations(
    opts: RefsOpts,
) -> Result<(
    RefsWithMetadata,
    Option<String>,
    BTreeSet<(String, usize, usize)>,
)> {
    let mut hint = None;
    let mut entries = BTreeSet::new();
    let metadata = super::refs_query::refs_impl_with_annotations(
        opts,
        &mut serde_json::json!({}),
        &mut Vec::new(),
        &mut hint,
        Some(&mut entries),
    )?;
    Ok((metadata, hint, entries))
}
