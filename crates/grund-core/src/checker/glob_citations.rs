//! The glob-citation rule (§FS-check.checks.glob-citation): a marked candidate
//! the scanner read as a pattern (§FS-check.1.1.11) is reported once at its
//! marker. The scanner recorded no citation for it, so this is the only trace
//! the pattern leaves in a run.

use crate::config::Schema;
use crate::model::{Catalog, CheckReport, Diagnostic};

/// §FS-check.checks.glob-citation.1: a warning until grund 0.19.0. The clause
/// is the pending half of the release guard's vocabulary (§FS-distribution.4.2.3).
const RAMP_CLAUSE: &str = "; this warning becomes an error in grund 0.19.0";

/// §FS-check.checks.glob-citation: one warning per pattern, at its marker's line
/// and column, with both markers spelled as configured.
pub(super) fn check_glob_citations(findings: &Catalog, schema: &Schema, report: &mut CheckReport) {
    let marker = &schema.citation.marker;
    report
        .warnings
        .extend(findings.glob_citations.iter().map(|pattern| Diagnostic {
            code: "glob-citation",
            path: Some(pattern.file.clone()),
            line: Some(pattern.line),
            column: Some(pattern.column),
            message: format!(
                "glob citation {marker}{token}: a citation names one exact point, not a \
                 pattern; cite the point, or escape it as <{marker}>{token}{RAMP_CLAUSE}",
                token = pattern.token,
            ),
            sites: Vec::new(),
            authority: Vec::new(),
        }));
}
