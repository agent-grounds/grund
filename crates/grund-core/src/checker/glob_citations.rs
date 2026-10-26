//! The glob-citation rule (§FS-check.checks.glob-citation): a marked candidate
//! the scanner read as a pattern (§FS-check.1.1.11) is reported once at its
//! marker. The scanner recorded no citation for it, so this is the only trace
//! the pattern leaves in a run.

use crate::config::Schema;
use crate::model::remedies;
use crate::model::{Catalog, CheckReport, Code, Diagnostic};

/// §FS-check.checks.glob-citation.1: an error since grund 0.19.0, and the
/// clause says so in the landed half of the release guard's vocabulary
/// (§FS-distribution.4.2.3, §DF-glob-citation-lands-in-0-19).
const LANDED_CLAUSE: &str = "; this became an error in grund 0.19.0";

/// §FS-check.checks.glob-citation: one error per pattern, at its marker's line
/// and column, with both markers spelled as configured. Every surface reads its
/// severity off the report's error channel (§REQ-one-answer.text.parity).
pub(super) fn check_glob_citations(findings: &Catalog, schema: &Schema, report: &mut CheckReport) {
    let marker = &schema.citation.marker;
    report
        .errors
        .extend(findings.glob_citations.iter().map(|pattern| Diagnostic {
            code: Code::GLOB_CITATION,
            path: Some(pattern.file.clone()),
            line: Some(pattern.line),
            column: Some(pattern.column),
            message: format!(
                "glob citation {marker}{token}: a citation names one exact point, not a \
                 pattern; {}{LANDED_CLAUSE}",
                remedies::glob_citation(format!("<{marker}>{}", pattern.token)),
                token = pattern.token,
            ),
            sites: Vec::new(),
            authority: Vec::new(),
        }));
}
