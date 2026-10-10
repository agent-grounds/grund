//! Fold one file's findings into the run's (§AR-scanner.2): every per-file
//! record list the pass fills is appended here, so a record the pass learns to
//! produce is carried to the run by one more line in one place.

use crate::model::Catalog;

/// Append every record of one file's `source` to the run's `target`. A list
/// missing here is silently dropped from every surface, which is how
/// `escaped_citations` was once lost.
pub(super) fn merge_findings(target: &mut Catalog, mut source: Catalog) {
    for (id, mut declarations) in source.declarations {
        target
            .declarations
            .entry(id)
            .or_default()
            .append(&mut declarations);
    }
    target.citations.append(&mut source.citations);
    target
        .section_headings_outside_declarations
        .append(&mut source.section_headings_outside_declarations);
    target
        .unmarked_headings
        .append(&mut source.unmarked_headings);
    target
        .legacy_citation_candidates
        .append(&mut source.legacy_citation_candidates);
    target
        .local_section_citation_candidates
        .append(&mut source.local_section_citation_candidates);
    target.value_bindings.append(&mut source.value_bindings);
    target
        .invalid_value_declarations
        .append(&mut source.invalid_value_declarations);
    target
        .invalid_value_bindings
        .append(&mut source.invalid_value_bindings);
    target
        .escaped_citations
        .append(&mut source.escaped_citations);
    target.glob_citations.append(&mut source.glob_citations);
    target
        .near_miss_headings
        .append(&mut source.near_miss_headings);
    target.scanned_files.append(&mut source.scanned_files);
    target.file_structure.extend(source.file_structure);
    target.line_ownership.append(&mut source.line_ownership);
}
