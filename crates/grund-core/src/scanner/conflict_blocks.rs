//! The conflict-block recognizer of the file pass (§AR-scanner.2.9). Which lines
//! are markers, how they cut blocks, where a block counts and whom it names is
//! §FS-declarations.body.conflict; this file is the one place that reads it, so
//! `check`'s finding and `fmt`'s refusal come from one record (§AR-system.principle).
//!
//! The pass hands every raw line to [`MarkerLines::read`] before its fence skip,
//! because a block inside a fence counts and the skip would hide it. A line that
//! does not open with `<`, `|`, `=` or `>` costs one byte comparison, so a file
//! with no marker line records nothing and pays nothing more.

use std::path::Path;

use crate::grammar::{Grammar, declaration_captures, parse_id};
use crate::model::{BlockForm, Catalog, ConflictBlock, FileRecord, Id};

/// The marker lines of one file, each with its character, in line order.
#[derive(Default)]
pub(super) struct MarkerLines(Vec<(usize, u8)>);

impl MarkerLines {
    /// Note `line` if it is a marker line (§FS-declarations.body.conflict.markers).
    pub(super) fn read(&mut self, lineno: usize, line: &str) {
        if let Some(marker) = marker_of(line) {
            self.0.push((lineno, marker));
        }
    }

    /// Whether the file holds any marker line: such a file forces the body-span
    /// pass, because owners are read from body spans (§AR-scanner.2.9).
    pub(super) fn any(&self) -> bool {
        !self.0.is_empty()
    }

    /// Cut the blocks, keep those §FS-declarations.body.conflict.where looks at
    /// and give each its owners (§FS-declarations.body.conflict.owners). Runs once
    /// the body spans are fixed, and before the file's record goes into `findings`.
    pub(super) fn record(
        self,
        path: &Path,
        record: &FileRecord,
        grammar: &Grammar,
        findings: &mut Catalog,
    ) {
        if self.0.is_empty() {
            return;
        }
        let spans = owner_spans(record, findings);
        // §FS-declarations.body.conflict.where: in Markdown, every line of a file that
        // declares, decided on the raw lines, fences ignored.
        let markdown_declares = record.markdown
            && (!spans.is_empty()
                || record
                    .text
                    .as_str()
                    .lines()
                    .any(|line| declares(grammar, line)));
        let kept: Vec<ConflictBlock> = blocks(&self.0)
            .into_iter()
            .map(|(first, last)| ConflictBlock {
                file: path.to_path_buf(),
                first,
                last,
                owners: owners_of(&spans, first, last),
            })
            .filter(|block| markdown_declares || !block.owners.is_empty())
            .collect();
        findings.conflict_blocks.extend(kept);
    }
}

/// Whether a raw Markdown line would declare an ID by §FS-declarations.line, were
/// no fence open (§FS-declarations.body.conflict.where).
fn declares(grammar: &Grammar, line: &str) -> bool {
    declaration_captures(grammar, line, false, true)
        .is_some_and(|caps| parse_id(&caps, grammar).is_some())
}

/// The marker character of a marker line: column 0, exactly seven of one of
/// `<`, `|`, `=`, `>`, then a space or the end of the line, a CR before the
/// LF being its end (§FS-declarations.body.conflict.markers).
pub(super) fn marker_of(line: &str) -> Option<u8> {
    let bytes = line.as_bytes();
    let first = *bytes
        .first()
        .filter(|byte| matches!(byte, b'<' | b'|' | b'=' | b'>'))?;
    let ends = matches!(bytes.get(7..), Some([] | [b' ', ..] | [b'\r']));
    (ends && bytes[..7].iter().all(|byte| *byte == first)).then_some(first)
}

/// The blocks the marker lines make, as first and last line
/// (§FS-declarations.body.conflict.block): `<` opens, the next `>` closes, a
/// middle marker or a second `<` only extends an open block, a `>` with none
/// open is a block of its own line, and an unclosed block ends at its last
/// marker line.
pub(super) fn blocks(markers: &[(usize, u8)]) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let mut open: Option<(usize, usize)> = None;
    for &(line, marker) in markers {
        match (marker, open) {
            (b'>', Some((first, _))) => {
                blocks.push((first, line));
                open = None;
            }
            (b'>', None) => blocks.push((line, line)),
            (b'<', None) => open = Some((line, line)),
            (_, Some((first, _))) => open = Some((first, line)),
            (_, None) => {}
        }
    }
    blocks.extend(open);
    blocks
}

/// One declaration of the file as a block's owner sees it: its line, the last
/// line of its body, and the line right after the line-style run that body
/// ends with, when a marker line cut it there (§FS-declarations.body.conflict.where).
struct OwnerSpan<'a> {
    id: &'a Id,
    line: usize,
    body_end: usize,
    after_cut_run: Option<usize>,
}

/// The file's declarations in the order they open, with the body spans every
/// other command reads (§FS-declarations.body).
fn owner_spans<'a>(record: &FileRecord, findings: &'a Catalog) -> Vec<OwnerSpan<'a>> {
    let mut spans: Vec<OwnerSpan<'a>> = findings
        .declarations
        .values()
        .flatten()
        .map(|decl| OwnerSpan {
            id: &decl.id,
            line: decl.line,
            body_end: decl.body_end,
            after_cut_run: record
                .block_at(decl.line)
                .filter(|run| matches!(run.form, BlockForm::Line(_)) && run.end == decl.body_end)
                .map(|run| run.end + 1),
        })
        .collect();
    spans.sort_by_key(|span| span.line);
    spans
}

/// Every declaration whose line or body the block overlaps, and the one whose
/// line-style run it opens right after, each ID once, in the order they open
/// (§FS-declarations.body.conflict.owners).
fn owners_of(spans: &[OwnerSpan<'_>], first: usize, last: usize) -> Vec<Id> {
    let mut owners: Vec<Id> = Vec::new();
    for span in spans {
        let overlaps = span.line <= last && first <= span.body_end;
        if (overlaps || span.after_cut_run == Some(first)) && !owners.contains(span.id) {
            owners.push(span.id.clone());
        }
    }
    owners
}
