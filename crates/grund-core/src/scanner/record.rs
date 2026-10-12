//! The file record, filled in the per-file pass (§AR-scanner.2.8.2): the comment
//! blocks walked once before the line loop, each heading as the loop meets it,
//! and, once every declaration is in, what each heading turned out to be. The
//! rules that read it — where a body, a section and a lead end — are
//! `extents.rs`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::config::{Frame, Schema};
use crate::grammar::{
    CommentBlockKind, DocCommentRule, SourceScanLine, atx_heading_text, block_is_doc_comment,
    comment_blocks, comment_envelope, doc_comment_rule, first_content_line,
    markdown_fence_delimiter,
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
            doc: block_is_doc_comment(
                doc_rule,
                &kind,
                &lines[start..=end],
                lines.get(end + 1).copied(),
                start <= leading_limit,
            ),
            form: match kind {
                CommentBlockKind::Line(marker) => BlockForm::Line(marker),
                CommentBlockKind::Block => BlockForm::Block,
                CommentBlockKind::PythonDocstring => BlockForm::Docstring,
            },
        })
        .collect()
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
/// that reads as an ATX heading once its envelope is off, outside a fence
/// (§FS-languages.comment-wrapper.headings, §FS-show.2.5). Only a declaration's
/// own block is read, so a file without one pays nothing.
fn doc_comment_plain_headings(
    record: &FileRecord,
    bodies: &BTreeMap<usize, (usize, usize)>,
) -> Vec<RecordedHeading> {
    let mut plain = Vec::new();
    for (&decl_line, &(end, _)) in bodies {
        let Some(block) = record.block_at(decl_line) else {
            continue;
        };
        let mut fence = None;
        for lineno in decl_line + 1..=end {
            let Some(line) = record.line(lineno) else {
                break;
            };
            let scanned = match record.zone_at(lineno) {
                Some(zone) => &line[zone.start..zone.end],
                None => line,
            };
            let content = comment_envelope(scanned, &block.form);
            if markdown_fence_delimiter(&mut fence, content) {
                continue;
            }
            if fence.is_some() || record.heading_at(lineno).is_some() {
                continue;
            }
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
                kind: HeadingKind::Plain,
                end: 0,
                lead_end: 0,
            });
        }
    }
    plain
}
