//! The file record, filled in the per-file pass (§AR-scanner.2.8.2): the comment
//! blocks walked once before the line loop, each heading as the loop meets it,
//! and, once every declaration is in, what each heading turned out to be. The
//! rules that read it — where a body, a section and a lead end — are
//! `extents.rs`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::config::{Frame, Schema};
use crate::grammar::{
    CommentBlockKind, DocCommentRule, MarkdownFence, SourceScanLine, atx_heading_text,
    block_is_doc_comment, comment_blocks, comment_envelope, doc_comment_rule, first_content_line,
    is_bare_block_line, markdown_fence_delimiter,
};
use crate::model::{
    BlockForm, Declaration, FileRecord, HeadingKind, LineZone, RecordedBlock, RecordedHeading,
};

use super::context::markdown_heading_level;

/// Every comment block of a source file, by form, with whether its language calls
/// it documentation (§FS-languages.comment-block.forms, §FS-inline-citation-style.1.1.1):
/// the one walk the body span, the inline-note sites and the grounding structure
/// all read (§AR-scanner.2.8.1).
pub(super) fn walk_blocks(
    path: &Path,
    lines: &[&str],
    is_py: bool,
    schema: &Schema,
    frame: Frame<'_>,
) -> Vec<RecordedBlock> {
    let doc_rule = doc_comment_rule(path);
    // Only a position language asks where the file's leading comment is.
    let leading_limit = match doc_rule {
        DocCommentRule::Position(_) => first_content_line(lines),
        _ => 0,
    };
    comment_blocks(lines, is_py, frame.compiled.lexical(schema))
        .into_iter()
        .map(|(start, end, kind)| RecordedBlock {
            start: start + 1,
            end: end + 1,
            closed: !matches!(kind, CommentBlockKind::Block { closed: false }),
            doc: block_is_doc_comment(
                doc_rule,
                &kind,
                &lines[start..=end],
                lines.get(end + 1).copied(),
                start <= leading_limit,
            ),
            form: match kind {
                CommentBlockKind::Line(marker) => BlockForm::Line(marker),
                CommentBlockKind::Block { .. } => BlockForm::Block,
                CommentBlockKind::PythonDocstring => BlockForm::Docstring,
            },
        })
        .collect()
}

/// Whether line `lineno` of a source file is a bare line of a closed `/* … */`
/// block, read off the blocks the record walked (§FS-declarations.line.block-comment):
/// the declaration gate, the near-miss gate and `show`'s reading of the
/// declaration such a line opens all ask the record this, never the line alone.
pub(crate) fn bare_block_line(record: &FileRecord, lineno: usize) -> bool {
    !record.markdown
        && record.block_at(lineno).is_some_and(|block| {
            block.form == BlockForm::Block
                && record
                    .line(lineno)
                    .is_some_and(|line| is_bare_block_line(line, block.closed))
        })
}

/// One line the loop recognized as a heading, located in the file's text.
pub(super) struct HeadingLine<'a> {
    pub(super) text: &'a str,
    pub(super) line: &'a str,
    pub(super) lineno: usize,
    pub(super) scan: &'a SourceScanLine<'a>,
    pub(super) is_md: bool,
}

impl HeadingLine<'_> {
    /// The record of this heading at the `level` the scan read it, with the section
    /// `path` parsed off it. Its rendered text is read through the comment's
    /// envelope (§FS-languages.comment-wrapper.headings) and then as an ATX heading
    /// (§FS-declarations.heading.rendered-text), so a code-form declaration's text
    /// is its line without the marker.
    pub(super) fn record(
        &self,
        record: &FileRecord,
        level: usize,
        path: Option<String>,
        kind: HeadingKind,
    ) -> RecordedHeading {
        let line_offset = self.line.as_ptr() as usize - self.text.as_ptr() as usize;
        let (base, base_offset) = if self.is_md {
            (self.line, line_offset)
        } else {
            (
                self.scan.text.as_ref(),
                line_offset + self.scan.column_offset,
            )
        };
        let content = match record.block_at(self.lineno) {
            Some(block) if !self.is_md => comment_envelope(base, &block.form),
            _ => base,
        };
        let trimmed = content.trim_start();
        let atx = markdown_heading_level(trimmed);
        let rendered = match atx {
            Some(run) => atx_heading_text(trimmed, run),
            None => trimmed.trim_end(),
        };
        // `rendered` is a slice of `base`, which an assigned-data mask copies at its own length.
        let start = base_offset + (rendered.as_ptr() as usize - base.as_ptr() as usize);
        RecordedHeading {
            line: self.lineno,
            level,
            atx: self.is_md && markdown_heading_level(self.line).is_some(),
            listed: self.is_md && atx.is_some(),
            span: line_offset..line_offset + self.line.len(),
            text: start..start + rendered.len(),
            path,
            relative: false,
            kind,
            end: 0,
            lead_end: 0,
        }
    }
}

/// The docstring zone of one scanned line, if it is a docstring line (§AR-scanner.4.4).
pub(super) fn zone_of(lineno: usize, scan: &SourceScanLine<'_>) -> Option<LineZone> {
    scan.in_py_docstring.then(|| LineZone {
        line: lineno,
        start: scan.column_offset,
        end: scan.column_offset + scan.text.len(),
        closes: scan.closed_py_docstring,
    })
}

/// Settle what each section-shaped heading turned out to be: a path its
/// declaration accepted inside its body, a later claimant of one, or rejected
/// (§AR-scanner.2.2.3, §FS-declarations.section.map). Then add the plain headings
/// of the doc-comments that hold a declaration, read through their envelope
/// (§FS-languages.comment-wrapper.headings). `bodies` maps each declaration line
/// to its body end and heading level.
pub(super) fn settle_headings(
    record: &mut FileRecord,
    decls: &[&Declaration],
    bodies: &BTreeMap<usize, (usize, usize)>,
) {
    let mut settled = BTreeMap::new();
    for decl in decls {
        let Some(&(end, _)) = bodies.get(&decl.line) else {
            continue;
        };
        let body = decl.line..=end;
        for (path, info) in &decl.sections {
            if body.contains(&info.line) {
                settled.insert(info.line, (HeadingKind::Section, path.as_str()));
            }
        }
        for (path, info) in &decl.duplicate_sections {
            if body.contains(&info.line) {
                settled
                    .entry(info.line)
                    .or_insert((HeadingKind::Duplicate, path.as_str()));
            }
        }
    }
    for heading in &mut record.headings {
        if let Some((kind, path)) = settled.get(&heading.line)
            && heading.path.as_deref() == Some(*path)
        {
            heading.kind = *kind;
        }
    }
    if !record.markdown {
        let plain = doc_comment_plain_headings(record, bodies);
        if !plain.is_empty() {
            record.headings.extend(plain);
            record.headings.sort_by_key(|heading| heading.line);
        }
    }
}

/// The headings without a path inside each source declaration's body: a line
/// that reads as an ATX heading once its envelope is off, outside the fences the
/// scan recorded (§FS-languages.comment-wrapper.headings,
/// §FS-languages.fence.comment-block). Only a declaration's own block is read, so
/// a file without one pays nothing.
fn doc_comment_plain_headings(
    record: &FileRecord,
    bodies: &BTreeMap<usize, (usize, usize)>,
) -> Vec<RecordedHeading> {
    let mut plain = Vec::new();
    for (&decl_line, &(end, _)) in bodies {
        let Some(block) = record.block_at(decl_line) else {
            continue;
        };
        for lineno in decl_line + 1..=end {
            let Some(line) = record.line(lineno) else {
                break;
            };
            if record.in_comment_fence(lineno) || record.heading_at(lineno).is_some() {
                continue;
            }
            let content = comment_envelope(scanned_line(record, lineno, line), &block.form);
            let trimmed = content.trim_start();
            let Some(level) = markdown_heading_level(trimmed) else {
                continue;
            };
            let offset = record.line_start(lineno).unwrap_or(0);
            let rendered = atx_heading_text(trimmed, level);
            let start = offset + (rendered.as_ptr() as usize - line.as_ptr() as usize);
            plain.push(RecordedHeading {
                line: lineno,
                level,
                atx: false,
                listed: false,
                span: offset..offset + line.len(),
                text: start..start + rendered.len(),
                path: None,
                relative: false,
                kind: HeadingKind::Plain,
                end: 0,
                lead_end: 0,
            });
        }
    }
    plain
}

/// Source line `lineno` as its comment block's section reading sees it: inside its
/// docstring's delimiters, with the block's envelope off and the whitespace around it
/// trimmed (§FS-languages.comment-wrapper.headings), or `None` outside every block.
pub(crate) fn enveloped_line(record: &FileRecord, lineno: usize) -> Option<&str> {
    let block = record.block_at(lineno)?;
    let line = record.line(lineno)?;
    Some(comment_envelope(scanned_line(record, lineno, line), &block.form).trim())
}

/// The fences of a source file's comment blocks, read line by line as the scan
/// meets them (§FS-languages.fence.comment-block): a delimiter is read on the
/// line with its comment wrapper off, and a fence still open where its block
/// ends closes there. The scan records what it reads, and every heading reader
/// asks the record, so there is one reading of where such a fence runs.
#[derive(Default)]
pub(super) struct CommentFences {
    /// The first line of the block the open fence is in.
    block: usize,
    open: Option<MarkdownFence>,
}

impl CommentFences {
    /// Read line `lineno` of `record`, whose docstring zone is already in, and
    /// record it when it is a delimiter or inside a fence.
    pub(super) fn read(&mut self, record: &mut FileRecord, lineno: usize) {
        let Some(block) = record.block_at(lineno) else {
            self.open = None;
            return;
        };
        if block.start != self.block {
            self.block = block.start;
            self.open = None;
        }
        let Some(line) = record.line(lineno) else {
            return;
        };
        let content = comment_envelope(scanned_line(record, lineno, line), &block.form);
        let delimiter = markdown_fence_delimiter(&mut self.open, content);
        if delimiter || self.open.is_some() {
            record.fenced.push(lineno);
        }
    }
}

/// The part of `line` the scan read: a docstring line inside its delimiters
/// (§AR-scanner.4.4), any other line whole.
fn scanned_line<'a>(record: &FileRecord, lineno: usize, line: &'a str) -> &'a str {
    match record.zone_at(lineno) {
        Some(zone) => &line[zone.start..zone.end],
        None => line,
    }
}
