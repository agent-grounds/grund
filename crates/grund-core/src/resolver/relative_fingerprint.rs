//! The text a locked section is fingerprinted from, its nested headings written
//! relative to it (§FS-section-lock.holds.fingerprint): what the section says,
//! not where its parent stands, so a renumber that respells a complete-path
//! heading nested in it is still reported as a move
//! (§DF-section-lock-relative-fingerprint.decision). One function, read by the
//! one place a fingerprint is computed, for the recorded entry and for every
//! section it is compared with (§DF-section-lock-relative-fingerprint.decision.one-place).

use crate::grammar::{path_at_or_under, relative_to_ancestor};
use crate::model::{Declaration, DeclarationSource};

/// `below`, the text `show` prints beneath the heading of `section` at `line`,
/// with each line the section map of `decl` records as the heading of a section
/// nested in it rewritten with its path relative to `section`: `.` then the
/// components beneath it, the `#` run, the punctuation and the title kept. A line
/// is matched to the map by its position, never by its shape, so a heading-shaped
/// line in a fence or in prose comes back as written
/// (§DF-section-lock-relative-fingerprint.decision.map). The slice keeps one line
/// per file line from the heading down, so the map's line numbers index it. A JSON
/// value and an e2e case have no heading lines to rewrite.
pub(super) fn relative_nested_headings(
    below: &str,
    decl: &Declaration,
    section: &str,
    line: usize,
) -> String {
    if !matches!(decl.source, DeclarationSource::Text) || decl.e2e_case.is_some() {
        return below.to_string();
    }
    let mut lines = below.split('\n').map(str::to_string).collect::<Vec<_>>();
    for (path, info) in &decl.sections {
        if path == section || !path_at_or_under(path, section, ".") {
            continue;
        }
        let Some(text) = info
            .line
            .checked_sub(line + 1)
            .and_then(|index| lines.get_mut(index))
        else {
            continue;
        };
        let nested = &path[section.len() + 1..];
        // A relative heading spells its last component; a complete one, its path.
        let written = match info.relative {
            true => format!(".{}", path.rsplit('.').next().unwrap_or(path)),
            false => path.clone(),
        };
        *text = relative_to_ancestor(text, &written, nested);
    }
    lines.join("\n")
}
