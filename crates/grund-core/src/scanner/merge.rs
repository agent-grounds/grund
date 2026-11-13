//! Fold one file's findings into the run's (§AR-scanner.2): every per-file
//! record list the pass fills is appended here, so a record the pass learns to
//! produce is carried to the run by one more line in one place, and taken back
//! from it by one more line in the function beside it.

use std::path::Path;

use crate::model::{Catalog, scanned_path_key};

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
        .relative_section_parents
        .append(&mut source.relative_section_parents);
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
        .off_format_citations
        .append(&mut source.off_format_citations);
    target
        .near_miss_headings
        .append(&mut source.near_miss_headings);
    target.conflict_blocks.append(&mut source.conflict_blocks);
    target.scanned_files.append(&mut source.scanned_files);
    target.file_structure.extend(source.file_structure);
    target.files.extend(source.files);
    target.line_ownership.append(&mut source.line_ownership);
}

/// Take every record of `file` that [`merge_findings`] folded into `target` back out of it, so
/// the file can be read again on new text and folded in once more (§AR-system.2.8.1). The two
/// name the same lists: one folded in and not taken back leaves the old reading of the file
/// beside the new one.
pub(super) fn drop_file_findings(target: &mut Catalog, file: &Path) {
    let key = scanned_path_key(file);
    let kept = |path: &Path| scanned_path_key(path) != key;
    for declarations in target.declarations.values_mut() {
        declarations.retain(|decl| kept(&decl.file));
    }
    target
        .declarations
        .retain(|_, declarations| !declarations.is_empty());
    target.citations.retain(|cite| kept(&cite.file));
    target
        .section_headings_outside_declarations
        .retain(|heading| kept(&heading.file));
    target
        .unmarked_headings
        .retain(|heading| kept(&heading.file));
    target
        .relative_section_parents
        .retain(|heading| kept(&heading.file));
    target
        .legacy_citation_candidates
        .retain(|candidate| kept(&candidate.file));
    target
        .local_section_citation_candidates
        .retain(|candidate| kept(&candidate.file));
    target.value_bindings.retain(|binding| kept(&binding.file));
    target
        .invalid_value_declarations
        .retain(|site| kept(&site.file));
    target
        .invalid_value_bindings
        .retain(|site| kept(&site.file));
    target.escaped_citations.retain(|cite| kept(&cite.file));
    target.glob_citations.retain(|cite| kept(&cite.file));
    target.off_format_citations.retain(|cite| kept(&cite.file));
    target
        .near_miss_headings
        .retain(|heading| kept(&heading.file));
    target.conflict_blocks.retain(|block| kept(&block.file));
    target.scanned_files.retain(|path| kept(path));
    target.file_structure.retain(|path, _| kept(path));
    target.files.retain(|path, _| kept(path));
    target.line_ownership.retain(|owners| kept(&owners.file));
}
