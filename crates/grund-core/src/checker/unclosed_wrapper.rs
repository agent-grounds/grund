//! The unclosed-wrapper warning (§FS-citations.checks.unclosed-wrapper): a wrap
//! whose destination never closes before a later `)` on its line, which `fmt`
//! leaves as written until grund 0.19.0 and wraps fresh from it. The scanner
//! notes the citation where it read it; this reports the note.

use crate::model::{Catalog, CheckReport, Diagnostic};

/// §FS-citations.checks.unclosed-wrapper: one warning per noted citation `fmt
/// --cross-refs` would link. An unqualified citation of an ID nothing declares is
/// skipped as `fmt` skips it (§FS-fmt.6.4); `check` reports it as dangling. The
/// warning leaves the exit status where the rest of the run put it (§FS-check.2).
pub(super) fn check_unclosed_wrappers(findings: &Catalog, report: &mut CheckReport) {
    report.warnings.extend(
        findings
            .unclosed_wrapper_notes
            .iter()
            .filter(|note| note.namespace.is_some() || findings.declarations.contains_key(&note.id))
            .map(|note| {
                let label = &note.label;
                Diagnostic {
                    code: "unclosed-wrapper",
                    path: Some(note.file.clone()),
                    line: Some(note.line),
                    column: None,
                    message: format!(
                        "the link after {label} never closes, so grund fmt --cross-refs leaves \
                         it as written, and from grund 0.19.0 it is not read as {label}'s link \
                         and {label} is linked on its own — add the ')' that ends the link to \
                         keep it"
                    ),
                    sites: Vec::new(),
                    authority: Vec::new(),
                }
            }),
    );
}
