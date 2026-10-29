//! The off-format-citation rule (§FS-check.checks.off-format-citation): a marked
//! token the promotion step recorded as off-format (§FS-check.1.1.12) is
//! reported once at its marker. It cites nothing, so this is the only trace it
//! leaves in a run. The rule belongs to the reference family because it is a
//! verdict on a citing token reached from the token and a catalog, so `--full`
//! reports it outside the configured scope too (§FS-check.3.14.2).

use super::reference_scope::ScanScope;
use crate::config::Schema;
use crate::model::remedies;
use crate::model::{Catalog, CheckReport, Code, Diagnostic};

/// The last release that dropped such a token without a word — the "from" half
/// of the pair §FS-check.checks.off-format-citation.1 names, in the past tense
/// §FS-distribution.4.2 closes the vocabulary on.
pub(super) const OFF_FORMAT_RULE_PRIOR_RELEASE: &str = "0.20.0";

/// The release the verdict moves in — the "to" half of that pair
/// (§FS-check.checks.off-format-citation.1).
pub(super) const OFF_FORMAT_RULE_RELEASE: &str = "0.21.0";

/// §FS-check.checks.off-format-citation: one error per record, at its marker's
/// line and column, naming the token whole, the format its kind follows in the
/// project the token names, and both remedies, with both markers spelled as
/// configured. `outside`, when set, restricts the pass to records outside that
/// scope, as it restricts the rest of the family (§FS-check.3.14).
pub(super) fn check_off_format_citations(
    findings: &Catalog,
    schema: &Schema,
    outside: Option<&ScanScope>,
    report: &mut CheckReport,
) {
    let marker = &schema.citation.marker;
    for record in &findings.off_format_citations {
        if outside.is_some_and(|scope| scope.contains(&record.file)) {
            continue;
        }
        report.errors.push(Diagnostic {
            code: Code::OFF_FORMAT_CITATION,
            path: Some(record.file.clone()),
            line: Some(record.line),
            column: Some(record.column),
            message: format!(
                "off-format citation {marker}{token}: {kind} IDs follow {format}, and nothing \
                 declares this one; {remedy} — unchecked in grund \
                 {OFF_FORMAT_RULE_PRIOR_RELEASE}, an error in {OFF_FORMAT_RULE_RELEASE}",
                token = record.token,
                kind = record.kind,
                format = record.format,
                remedy = remedies::off_format_citation(format!("<{marker}>{}", record.token)),
            ),
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }
}
