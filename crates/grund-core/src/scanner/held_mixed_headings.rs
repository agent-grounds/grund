//! The complete-path headings of a source doc-comment declaration body that put a
//! name under a number (§FS-config.3.3.1). 0.18.0 holds each back as the prose it
//! was before, and 0.19.0 reads it as a section
//! (§FS-declarations.checks.parse-moves.2, shape 5). The scan records such a line
//! as a plain heading, so this is the one place that reads the path back off it,
//! for the fifth shape's warning and for the fourth's preview of 0.19.0's parents.

use super::relative_sections::claim;
use crate::grammar::{mixed_heading_path, reduce_heading_text};
use crate::model::{Declaration, FileRecord, HeadingKind, RecordedHeading, SectionInfo};

/// The held-back headings among `plain`, the plain headings of one doc-comment
/// body of `decl`: deeper than the declaration, each with its path.
pub(crate) fn held_mixed_headings<'a>(
    record: &'a FileRecord,
    decl: &Declaration,
    plain: impl Iterator<Item = &'a RecordedHeading>,
) -> impl Iterator<Item = (&'a RecordedHeading, &'a str)> {
    let level = decl.heading_level;
    plain.filter_map(move |heading| {
        let path = mixed_heading_path(record.heading_text(heading))?;
        (heading.kind == HeadingKind::Plain && heading.level > level).then_some((heading, path))
    })
}

/// Claim each held-back heading of `decl`'s body on `settled`, the copy the
/// fourth shape previews 0.19.0's parents on, as the per-file pass records a
/// complete heading before the parent pass runs: a relative heading below one is
/// named under it (§FS-declarations.checks.parse-moves.1).
pub(super) fn seed_held_mixed_headings<'a>(
    record: &'a FileRecord,
    decl: &Declaration,
    plain: impl Iterator<Item = &'a RecordedHeading>,
    settled: &mut Declaration,
) {
    for (heading, path) in held_mixed_headings(record, decl, plain) {
        let info = SectionInfo {
            title: reduce_heading_text(record.heading_text(heading)),
            line: heading.line,
            heading_level: heading.level,
            value: None,
            value_root: None,
            relative: false,
        };
        claim(settled, path.to_string(), info);
    }
}
