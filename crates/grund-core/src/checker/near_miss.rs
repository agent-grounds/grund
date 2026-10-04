//! The declaration near-miss rule (§FS-declarations.checks.declaration-near-miss), in a file of its own beside
//! the other rule families (§AR-core-module-layout.1): a heading that opens like
//! a declaration and does not parse as one, reported per heading at the line a
//! contributor has to edit.
//!
//! The rule is one function because it is one question asked of a list the
//! scanner already built. What it must not become is a guess: it names the token,
//! the format it missed, and the shape that format reads — never a corrected ID,
//! which would be an opinion about what the author meant (§FS-non-goals.3).

use crate::model::{CheckReport, Diagnostic, Findings, LANDED_CLAUSE};

/// §FS-declarations.checks.declaration-near-miss: one error per heading that came close, since
/// `0.16.0` (§FS-declarations.checks.declaration-near-miss.5). The severity never changes catalog
/// recognition or citation promotion, so the read commands still resolve the ID. Sorted with the
/// rest of the report by the shared comparator, so a run over one tree prints them in the same
/// order every time (§FS-errors.4.1).
pub(super) fn check_declaration_near_misses(findings: &Findings, report: &mut CheckReport) {
    report.errors.extend(
        findings
            .near_miss_headings
            .iter()
            .map(|heading| Diagnostic {
                code: "declaration-near-miss",
                path: Some(heading.file.clone()),
                line: Some(heading.line),
                column: None,
                message: near_miss_message(&heading.format, &heading.text),
                sites: Vec::new(),
                authority: Vec::new(),
            }),
    );
}

/// The sentence: the token as written, the configured template, and the shape
/// that template reads. Three facts, no proposal — `check` reports facts about
/// the tree and the config (§FS-check.3 vs §FS-check.4), and the corrected ID is the one
/// thing here that would be a guess. It ends in the landed clause
/// (§FS-declarations.checks.declaration-near-miss.5).
fn near_miss_message(format: &str, text: &str) -> String {
    format!(
        "`{text}` resolves for compatibility but does not match [id] format = \
         \"{format}\" — rename it or change the effective format{LANDED_CLAUSE}",
    )
}
