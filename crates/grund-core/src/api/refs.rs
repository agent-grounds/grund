//! The published `refs` contract (§AR-system.2.9): the citation sites of one ID,
//! and the typed resolver rejection a process frontend applies §FS-refs.4's
//! staged exit policy from without parsing prose (§FS-refs, §AR-bindings.2).
//!
//! The walk that produces both is `refs_query.rs`, the adapter §AR-core-module-layout.2
//! keeps out of the contract.

use anyhow::{Result, anyhow};
use std::path::PathBuf;

use super::refs_query::refs_impl;
use crate::grammar::IdArgError;
use crate::model::Finding;
use crate::scanner::ApiScanError;

#[derive(Clone)]
pub struct RefsOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub id: String,
    pub section: Option<String>,
}

impl Default for RefsOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
            id: String::new(),
            section: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefHit {
    pub project: Option<String>,
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub id: String,
    pub section: Option<String>,
    pub marker: bool,
    pub text: String,
}

/// The two ways a selected project's resolver can reject a `refs` operand
/// (§FS-refs.4). This is query-result data rather than an [`anyhow::Error`], so
/// every frontend can apply the same staged exit policy without parsing prose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
/// were selected (§FS-errors.2.3, §FS-workspace.8.7).
#[derive(Clone, Debug, Eq, PartialEq)]
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

    pub(crate) fn into_refs_error(self) -> anyhow::Error {
        if let Some(format) = self.format_hint {
            anyhow!(
                "{}\nhint: this repo's [id] format is `{format}` (run `grund config show`); `grund list` shows the IDs that exist",
                self.message
            )
        } else {
            anyhow!(self.message)
        }
    }
}

/// The compatibility warning is shared by both CLI entry points so the public
/// and deprecated adapters cannot drift during the §FS-refs.4 release ramp.
pub const REFS_QUERY_FAILURE_WARNING: &str = "warning: `grund refs` invalid IDs and ambiguous number-only shorthands currently exit 2; they will exit 1 (failed query) in grund 0.15.0";

/// Whether §FS-refs.4's resolver-rejection mapping has reached its exit-`1`
/// phase. Frontends own process exit codes, but consume this one core policy.
pub fn refs_query_failure_is_exit_one() -> bool {
    let mut parts = env!("CARGO_PKG_VERSION")
        .trim_end_matches("-dev")
        .split('.')
        .filter_map(|part| part.parse::<u32>().ok());
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0)) >= (0, 15)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RefsOutput {
    pub output_format: String,
    pub workspace: bool,
    pub hits: Vec<RefHit>,
    pub note: Option<String>,
    pub scan_errors: Vec<ApiScanError>,
    /// The run's warning channel (§FS-distribution.3.1): the four `[workspace]`
    /// cautions of §FS-check.4.7, §FS-check.4.8, §FS-check.4.10 and
    /// §FS-workspace.6.1, each anchored at the `grund.toml` line its own message
    /// names. A frontend renders each as one CLI-level `warning:` on stderr
    /// (§FS-check.2.1.1); an editor publishes it on that line (§FS-lsp.1.1).
    pub warnings: Vec<Finding>,
}

/// The additive classified `refs` result used by process frontends during the
/// §FS-refs.4 release ramp. Keeping the carrier outside [`RefsOutput`] preserves
/// the exhaustively constructible embedding API required by §AR-bindings.2.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RefsOutcome {
    pub output: RefsOutput,
    pub query_failure: Option<RefsQueryFailure>,
}

/// Programmatic `refs` with the typed resolver-rejection outcome process
/// frontends need to apply §FS-refs.4 without parsing an error string.
pub fn refs_outcome(opts: RefsOpts) -> Result<RefsOutcome> {
    refs_impl(opts)
}

/// Programmatic `refs`: resolve an ID query and return all citation sites
/// without selecting text/summary/JSON rendering (§AR-bindings.2). This keeps
/// the established API: resolver and setup failures are returned as `Err`.
pub fn refs(opts: RefsOpts) -> Result<RefsOutput> {
    let outcome = refs_outcome(opts)?;
    match outcome.query_failure {
        Some(failure) => Err(failure.into_refs_error()),
        None => Ok(outcome.output),
    }
}
