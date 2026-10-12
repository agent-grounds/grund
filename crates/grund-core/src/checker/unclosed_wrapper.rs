//! The unclosed-wrapper warning (§FS-citations.checks.unclosed-wrapper): a wrap
//! whose destination never closes before a later `)` on its line, which `fmt`
//! leaves as written until grund 0.19.0 and wraps fresh from it. The scanner
//! notes the citation where it read it; this reports the notes `fmt
//! --cross-refs` would link, which is a question of presentation and of the
//! index entries, so it is the relational half's (§AR-checker.1.2).

use std::collections::BTreeMap;

use super::index_entries::KindIndexEntries;
use crate::model::{Catalog, CheckReport, Diagnostic, Expected, UnclosedWrapperNote};
use crate::resolver::WorkspaceCheckTarget;

/// §FS-citations.checks.unclosed-wrapper: one warning per noted citation `fmt
/// --cross-refs` would link. Its ordinary pass links wherever nothing suppresses
/// the line, whatever `[fmt.cross_refs] enabled` says, since that key gates only
/// the automatic pass (§FS-fmt.6.7.2); a kind's index entry is linked even in a
/// suppressed scope (§FS-fmt.2.5.3). A citation whose target declares nothing is skipped as
/// `fmt` skips it (§FS-fmt.6.4), and a qualified one resolves only in the
/// workspace that loaded its alias (§FS-workspace.8.5). The warning leaves the
/// exit status where the rest of the run put it (§FS-check.2).
pub(super) fn check_unclosed_wrappers(
    findings: &Catalog,
    expected: &Expected,
    index_entries: &KindIndexEntries,
    targets: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    report: &mut CheckReport,
) {
    let linked = |note: &UnclosedWrapperNote| {
        let ordinary = !note.suppressed && !expected.fmt_excluded.contains(&note.file);
        let resolves = match note.namespace.as_deref() {
            Some(alias) => targets
                .get(alias)
                .is_some_and(|target| target.catalog.declarations.contains_key(&note.id)),
            None => findings.declarations.contains_key(&note.id),
        };
        let entry = || {
            note.namespace.is_none()
                && index_entries
                    .entries_in(&note.file)
                    .is_some_and(|ids| ids.contains(&note.id))
        };
        resolves && (ordinary || entry())
    };
    report.warnings.extend(
        findings
            .unclosed_wrapper_notes
            .iter()
            .filter(|note| linked(note))
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
