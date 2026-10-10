//! The per-file structure a grounding unit is cut out of (§AR-scanner.2.7): the
//! pass that fills the `FileStructure` records `model` carries on `Catalog`
//! (§AR-system.2.2). The records themselves are plain data and sit there; what
//! is here is the reading of a file that produces one.
//!
//! This is a *description* of a file, not a list of units: the level that turns
//! headings and doc-comment blocks into units belongs to the `[[kinds]]` row that
//! governs the file (§FS-config.3.4.8.2), and the cut is made in
//! `checker/grounding.rs` so the level rule is written once. It runs per file,
//! only where that file's own row asks for a unit finer than the file, so one
//! fine-grained place does not describe the whole tree and a project at level `1`
//! — every configuration written before the keys existed — records nothing and
//! pays nothing (§GOAL-fast-feedback).

use std::path::Path;

use super::context::{file_home_kind, markdown_heading_level};
use crate::config::{Frame, Schema};
use crate::grammar::{
    DocCommentRule, MarkdownBlocks, block_is_doc_comment, comment_blocks, doc_comment_rule,
    first_content_line,
};
use crate::model::{Catalog, DocCommentBlock, FileHeading, FileStructure};

/// Record `path`'s grounding structure into `findings`, or do nothing when the
/// row this file belongs to asks for no unit finer than the file
/// (§AR-scanner.2.7). Called once per file from the per-file scan, with the text
/// it already read.
pub(super) fn record_file_structure(
    path: &Path,
    text: &str,
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
    let extension = path.extension().and_then(|ext| ext.to_str());
    let structure = if extension == Some("md") {
        markdown_structure(text)
    } else {
        source_structure(path, text, extension == Some("py"), schema, frame)
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
/// (§AR-scanner.2.7, §FS-check.1.1.5.1). The state is the one §AR-scanner.2.3.3
/// keeps for citations, for the same reason: a `##` inside either is an example
/// of a document, not a section of this one.
fn markdown_structure(text: &str) -> FileStructure {
    let mut structure = FileStructure::default();
    let mut blocks = MarkdownBlocks::default();
    for (index, line) in text.lines().enumerate() {
        structure.total_lines = index + 1;
        if !blocks.line(line).may_be_heading() {
            continue;
        }
        let trimmed = line.trim_start();
        if let Some(level) = markdown_heading_level(trimmed) {
            structure.headings.push(FileHeading {
                line: index + 1,
                level,
                text: heading_text(trimmed, level),
            });
        }
    }
    structure
}

/// A heading's text: the line without its opening `#`s and without the optional
/// closing run Markdown allows (`## Steps ##`), so a finding quotes the title
/// rather than the syntax (§FS-check.3.6.3).
pub(super) fn heading_text(trimmed: &str, level: usize) -> String {
    let suffix = trimmed[level..].trim_end_matches([' ', '\t']);
    let closing_start = suffix.trim_end_matches('#').len();
    let without_closing = if closing_start < suffix.len()
        && suffix[..closing_start]
            .as_bytes()
            .last()
            .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        &suffix[..closing_start]
    } else {
        suffix
    };
    without_closing.trim_matches([' ', '\t']).to_string()
}

/// Every doc-comment block in a source file, with its indentation
/// (§AR-scanner.2.7). Which blocks are documentation is the per-language rule of
/// §FS-inline-citation-style.1.1.1, read once per file from the extension and
/// applied to each block with one comparison — the same call the inline-site pass
/// makes, so a block cannot be a doc comment for one rule and a note for the
/// other.
fn source_structure(
    path: &Path,
    text: &str,
    is_py: bool,
    schema: &Schema,
    frame: Frame<'_>,
) -> FileStructure {
    let lines = text.lines().collect::<Vec<_>>();
    let doc_rule = doc_comment_rule(path);
    let leading_limit = match doc_rule {
        DocCommentRule::Position(_) => first_content_line(&lines),
        _ => 0,
    };
    let mut structure = FileStructure {
        total_lines: lines.len(),
        ..FileStructure::default()
    };
    for (start, end, kind) in comment_blocks(&lines, is_py, frame.compiled.lexical(schema)) {
        let block = &lines[start..=end];
        if block_is_doc_comment(
            doc_rule,
            &kind,
            block,
            lines.get(end + 1).copied(),
            start <= leading_limit,
        ) {
            structure.doc_comments.push(DocCommentBlock {
                start: start + 1,
                end: end + 1,
                indented: lines[start].starts_with(char::is_whitespace),
            });
        }
    }
    structure
}
