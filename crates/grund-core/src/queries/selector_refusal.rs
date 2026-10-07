//! The typed refusals `list` and the size catalog raise for a selector
//! (§FS-distribution.3.2.1, §FS-distribution.3.3.2): the text is the one the CLI
//! prints, and the class lets an embedding host tell a refused query from a
//! failed run.

use crate::model::OperationDiagnostic;

/// A selector the sentence grammar refused, already rendered (§FS-rules.8.1).
pub(crate) fn selector_refusal(rendered: String) -> anyhow::Error {
    anyhow::Error::new(OperationDiagnostic::new("query", "query-failed", rendered))
}

/// A literal subject must name exactly one site (§FS-rules.8).
pub(crate) fn require_unique_literal(literal: &str, matches: usize) -> anyhow::Result<()> {
    let (code, verb) = match matches {
        1 => return Ok(()),
        0 => ("not-found", "does not resolve"),
        _ => ("ambiguous", "is ambiguous"),
    };
    let message = format!("literal subject {literal} {verb}");
    Err(anyhow::Error::new(OperationDiagnostic::new(
        "query", code, message,
    )))
}
