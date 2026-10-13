//! The fifth shape of §FS-declarations.checks.parse-moves.2: a complete-path heading whose path
//! puts a name under a number, in a source doc-comment declaration body. 0.18.0 holds it back as
//! the prose it was before, so a doc-comment's sections move once, with the rest of #639
//! (§DF-number-name-sections.decision.doc-comments), and 0.19.0 gives it its section, so it warns
//! at its line. Which lines those are is the scanner's one reading, the same one the fourth
//! shape seeds its parents from, so the two shapes cannot disagree about a line.

use super::{ONE_PARSE_FLIP, push_warning, text_declarations};
use crate::checker::sizes::site_coordinate;
use crate::config::{Frame, Schema};
use crate::model::{Catalog, CheckReport, HeadingKind};
use crate::scanner::held_mixed_headings;

/// One finding per held-back line, named by its own path, with the last message of
/// §FS-declarations.checks.parse-moves.1: no spelling makes it that section in both releases.
/// It warns whether 0.19.0 makes it a section, a duplicate claim or an orphan, as the fourth
/// shape does; with `named_sections` off no such line is a heading in either release.
pub(super) fn check_mixed_headings(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    if !schema.ids.named_sections {
        return;
    }
    let mut by_file = None;
    for (path, record) in &findings.files {
        if record.markdown
            || !record
                .headings
                .iter()
                .any(|heading| heading.kind == HeadingKind::Plain)
        {
            continue;
        }
        let declarations = by_file
            .get_or_insert_with(|| text_declarations(findings))
            .get(path.as_path())
            .map(Vec::as_slice)
            .unwrap_or_default();
        for owner in record
            .headings
            .iter()
            .filter(|heading| heading.kind == HeadingKind::Declaration)
        {
            let Some((id, declaration)) = declarations
                .iter()
                .find(|(_, declaration)| declaration.line == owner.line)
            else {
                continue;
            };
            let plain = record
                .headings_between(owner.line + 1, owner.end)
                .iter()
                .filter(|heading| heading.kind == HeadingKind::Plain);
            for (heading, section) in held_mixed_headings(record, declaration, plain) {
                let coordinate = site_coordinate(schema, frame, id, Some(section));
                push_warning(report, path, heading.line, held_message(&coordinate));
            }
        }
    }
}

/// The last message of §FS-declarations.checks.parse-moves.1, which the fifth shape takes and
/// the fourth takes at a mixed canonical path: no spelling makes the line that section in both
/// releases, so only keeping it prose is offered.
pub(super) fn held_message(coordinate: &str) -> String {
    format!(
        "from grund {ONE_PARSE_FLIP} this line is the heading of {coordinate}; make it a bold label or a code span to keep it prose in both releases"
    )
}
