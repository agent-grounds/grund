//! The per-file structure a grounding unit is cut out of (§AR-scanner.2.7): the
//! pass that fills the `FileStructure` records `model` carries on `Catalog`
//! (§AR-system.2.2). The records themselves are plain data and sit there; what
//! is here is the reading of a file that produces one.
//!
//! This is a *description* of a file, not a list of units: the level that turns
//! headings and doc-comment blocks into units belongs to the `[[kinds]]` row that
//! governs the file (§FS-config.3.4.8.2), and the cut is made in
//! `checker/grounding.rs` so the level rule is written once. The record it views
//! is filled on every run (§AR-scanner.2.7.2); the view is taken only where the
//! file's own row asks for a unit finer than the file, so a project at level `1`
//! copies nothing out of it (§GOAL-fast-feedback).

use std::path::Path;

use super::context::file_home_kind;
use crate::config::{Frame, Schema};
use crate::model::{Catalog, DocCommentBlock, FileHeading, FileRecord, FileStructure};

/// Record `path`'s grounding structure into `findings`, or do nothing when the
/// row this file belongs to asks for no unit finer than the file
/// (§AR-scanner.2.7). The structure is a view of the file's record
/// (§AR-scanner.2.8.2): its listed headings in Markdown, its doc-comment blocks
/// elsewhere, both read in the one pass that filled the record.
pub(super) fn record_file_structure(
    path: &Path,
    record: &FileRecord,
    schema: &Schema,
    frame: Frame<'_>,
    findings: &mut Catalog,
) {
    // §AR-scanner.2.7.3: the project-wide answer first — one field read exempts
    // every file of a level-1 tree, every configuration written before the keys
    // existed, from the per-file lookup below (§GOAL-fast-feedback).
    let demand = &frame.compiled.demand;
    if demand.is_empty() || !demand.records_structure(&file_row(path, schema, frame)) {
        return;
    }
    let structure = if record.markdown {
        markdown_structure(record)
    } else {
        source_structure(record)
    };
    findings
        .file_structure
        .insert(path.to_path_buf(), structure);
}

/// The row `path` belongs to (§AR-scanner.2.7.1) — its home kind, or the
/// homeless kind where no single home claims it (§AR-scanner.2.4.2). That lookup
/// is the one §FS-check.3.6.1 defers to for which row governs a file, and whether
/// the row asks for structure is the demand config computed with the checker's
/// own level rule (§AR-config.6.1), so what is recorded here and what is cut out
/// of it later are one rule.
///
/// A Markdown document in a *citable* home is recorded when its row asks for a
/// finer unit, though §FS-check.3.6 will not ask for its units: the row's level
/// is a statement about the place, and erring toward having the structure costs
/// one description, while a second home rule here could disagree with that one.
fn file_row(path: &Path, schema: &Schema, frame: Frame<'_>) -> String {
    file_home_kind(path, schema, frame).unwrap_or_else(|| schema.complement_name().to_string())
}

/// Every heading outside a fenced block or a raw-text HTML block, with its text
/// (§AR-scanner.2.7, §FS-check.1.1.5.1): the record's listed headings, a `##`
/// inside either being an example of a document, not a section of this one.
fn markdown_structure(record: &FileRecord) -> FileStructure {
    FileStructure {
        headings: record
            .headings
            .iter()
            .filter(|heading| heading.listed)
            .map(|heading| FileHeading {
                line: heading.line,
                level: heading.level,
                text: record.heading_text(heading).to_string(),
            })
            .collect(),
        doc_comments: Vec::new(),
        total_lines: record.total_lines,
    }
}

/// Every doc-comment block in a source file, with its indentation
/// (§AR-scanner.2.7): the record's blocks its language calls documentation
/// (§FS-inline-citation-style.1.1.1), the same flag the inline-site pass reads, so
/// a block cannot be a doc comment for one rule and a note for the other.
fn source_structure(record: &FileRecord) -> FileStructure {
    FileStructure {
        headings: Vec::new(),
        doc_comments: record
            .blocks
            .iter()
            .filter(|block| block.doc)
            .map(|block| DocCommentBlock {
                start: block.start,
                end: block.end,
                indented: record
                    .line(block.start)
                    .is_some_and(|line| line.starts_with(char::is_whitespace)),
            })
            .collect(),
        total_lines: record.total_lines,
    }
}
