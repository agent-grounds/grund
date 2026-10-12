//! Today's answers, for one release (§AR-scanner.2.8.3): the body end, section
//! end, doc-comment owner and table of contents 0.17 gives, read from the same
//! file record rather than from a second parse. Everything here exists because
//! §DF-one-parse-answers-from-0-19 keeps those answers byte for byte until 0.19.0,
//! and goes with this file then ([#639](https://github.com/agent-grounds/grund/issues/639)):
//! the record's own rules are `extents.rs`.
//!
//! The comment envelope today's body reads is `compat_envelope.rs`. The two
//! readings at the end, the record's lead and a block line 0.19.0 makes a section,
//! are what §FS-declarations.checks.parse-moves compares today's answers with.

use crate::grammar::{LexicalSettings, comment_envelope};
use crate::model::{
    BlockForm, FileRecord, HeadingKind, RecordedHeading, ShowRenderMode, ShowSection,
};

use super::compat_envelope::{
    clean_envelope_line, closes_comment_block, continues_comment_body, is_line_style_comment_line,
    own_line_marker, section_title,
};
use super::context::markdown_heading_level;
use super::record::bare_block_line;

/// The catalog site a slice is pinned to (§FS-declarations.section.map, §FS-list.3.4):
/// its declaration line, the body end the catalog publishes when it publishes one,
/// and the line of the selected section heading.
#[derive(Clone, Copy)]
pub(crate) struct TodaySite {
    pub(crate) declaration_line: usize,
    pub(crate) declaration_body_end: Option<usize>,
    pub(crate) section_line: Option<usize>,
}

/// One view a walk fills (§FS-show.2.1): the lead (`Default`), the whole body
/// (`Full`) or the section map (`Outline`), with or without the heading line.
pub(crate) struct TodaySink {
    view: ShowRenderMode,
    include_heading: bool,
    open: bool,
    lines: Vec<String>,
    sections: Vec<ShowSection>,
}

impl TodaySink {
    pub(crate) fn new(view: ShowRenderMode, include_heading: bool) -> Self {
        Self {
            view,
            include_heading,
            open: true,
            lines: Vec::new(),
            sections: Vec::new(),
        }
    }

    /// The view's text, blank lines trimmed from both ends and `\n`-terminated,
    /// and the section map an `Outline` view filled.
    pub(crate) fn finish(self) -> (String, Vec<ShowSection>) {
        let first = self.lines.iter().position(|line| !line.trim().is_empty());
        let last = self.lines.iter().rposition(|line| !line.trim().is_empty());
        let body = match (first, last) {
            (Some(first), Some(last)) => format!("{}\n", self.lines[first..=last].join("\n")),
            _ => String::new(),
        };
        (body, self.sections)
    }
}

/// What a walk could not find.
pub(crate) enum TodayMiss {
    Declaration,
    Section,
}

/// What one line asks of a sink.
enum Step {
    /// The view ends before this line.
    Close,
    /// The line is the view's, already taken.
    Taken,
    /// The line is body text, if the section has been found.
    Body,
}

/// Today's slice of one declaration or section into every sink in one walk
/// (§DF-one-parse-answers-from-0-19, P01, P02, P03, P07, P10, D17a, D17d): the
/// body runs from the declaration line until a declaration-shaped line, the end
/// of its comment by today's markers, the `*/` that closes it, an empty
/// docstring-closing line, or the published body end; a section runs until any
/// numbered heading as deep as it, or a second claimant of its path; a lead stops
/// at any numbered heading. Answers the line the slice starts at.
pub(crate) fn today_slices(
    record: &FileRecord,
    site: TodaySite,
    section: Option<&str>,
    sinks: &mut [TodaySink],
    lexical: LexicalSettings<'_>,
) -> Result<usize, TodayMiss> {
    let is_md = record.markdown;
    let decl_line = site.declaration_line;
    if !record
        .heading_at(decl_line)
        .is_some_and(|heading| heading.kind == HeadingKind::Declaration)
    {
        return Err(TodayMiss::Declaration);
    }
    let (scan_line, in_docstring, closes) = scanned(record, decl_line);
    let line_style = is_line_style_comment_line(scan_line);
    // §FS-show.2.3.1.1: a `--` or `;` body is read behind its own marker.
    let own_marker = (!is_md && !in_docstring)
        .then(|| own_line_marker(scan_line, lexical))
        .flatten();
    let own_marker = own_marker.as_deref();
    // §FS-declarations.line.block-comment: a declaration 0.17 never found has no bytes to keep,
    // so its block is read by the record's envelope, down to the block's `*/`.
    let record_block_end = bare_declaration_block_end(record, decl_line);
    let clean = |line: &str, markdown_heading: bool| match record_block_end {
        Some(_) => comment_envelope(line, &BlockForm::Block).to_string(),
        None => clean_envelope_line(line, markdown_heading, own_marker),
    };
    let mut block_closed = closes_comment_block(scan_line, line_style, in_docstring, is_md);
    for sink in sinks.iter_mut().filter(|sink| sink.include_heading) {
        let markdown_heading = is_md || in_docstring;
        sink.lines.push(clean(scan_line, markdown_heading));
    }
    let mut found_section = section.is_none();
    let mut target_depth = usize::MAX;
    let mut output_line = decl_line;
    let mut lineno = decl_line;
    while !closes && sinks.iter().any(|sink| sink.open) {
        lineno += 1;
        // §FS-show.2.3.1.2: the `*/` closing the declaration's own block ends it.
        let Some(line) = record.line(lineno).filter(|_| !block_closed) else {
            break;
        };
        if site
            .declaration_body_end
            .is_some_and(|body_end| lineno > body_end)
        {
            break;
        }
        let (scan_line, in_docstring, closes_docstring) = scanned(record, lineno);
        if in_docstring && closes_docstring && scan_line.trim().is_empty() {
            break;
        }
        // §FS-show.2.5: only a line outside a fence is recorded as a heading.
        let heading = record.heading_at(lineno);
        if heading.is_some_and(|heading| heading.kind == HeadingKind::Declaration) {
            break;
        }
        if let Some(block_end) = record_block_end {
            // Every line of the block is the body's, bare or behind ` * `, to its closer.
            block_closed = lineno >= block_end;
        } else if !is_md {
            // A blank line ends a line-style comment; a docstring line is its body.
            if !in_docstring
                && (if line.trim().is_empty() {
                    line_style
                } else {
                    !continues_comment_body(scan_line, own_marker, lexical)
                })
            {
                break;
            }
            block_closed = closes_comment_block(scan_line, line_style, in_docstring, is_md);
        }
        let markdown_heading = is_md || in_docstring;
        let path = heading.and_then(|heading| heading.path.as_deref());
        let depth = path.map_or(0, |path| path.split('.').count());
        // §DF-duplicate-section-path.1: a second claimant of the path ends the section.
        let target_found = match (path, section) {
            (Some(path), Some(target)) => {
                path == target
                    && !found_section
                    && site.section_line.is_none_or(|line| line == lineno)
            }
            _ => false,
        };
        if target_found {
            found_section = true;
            target_depth = depth;
            output_line = lineno;
        }
        for sink in sinks.iter_mut().filter(|sink| sink.open) {
            let step = match path {
                _ if target_found => {
                    if sink.view != ShowRenderMode::Outline {
                        sink.lines.push(clean(scan_line, markdown_heading));
                    }
                    Step::Taken
                }
                None => Step::Body,
                Some(_) if !found_section => Step::Body,
                // A lead stops at the first numbered heading (§FS-declarations.lead.section).
                Some(_) if sink.view == ShowRenderMode::Default => Step::Close,
                Some(_) if section.is_some() && depth <= target_depth => Step::Close,
                Some(path) if sink.view == ShowRenderMode::Outline => {
                    let depth = section.map_or(depth, |_| depth - target_depth);
                    let line = clean(scan_line, markdown_heading);
                    let title = match record_block_end {
                        Some(_) => section_title(&line, path, true, None),
                        None => section_title(scan_line, path, markdown_heading, own_marker),
                    };
                    sink.lines.push(line);
                    sink.sections.push(ShowSection {
                        path: path.to_string(),
                        title,
                        depth,
                    });
                    Step::Taken
                }
                Some(_) => Step::Body,
            };
            match step {
                Step::Close => sink.open = false,
                Step::Taken => {}
                Step::Body => {
                    if found_section && sink.view != ShowRenderMode::Outline {
                        sink.lines.push(clean(scan_line, markdown_heading));
                    }
                    if closes_docstring {
                        sink.open = false;
                    }
                }
            }
        }
    }
    if !found_section {
        return Err(TodayMiss::Section);
    }
    Ok(output_line)
}

/// The last line of the closed `/* … */` block a declaration on a bare line of it
/// opens, or `None` for any other declaration (§FS-declarations.line.block-comment,
/// §DF-bare-block-comment-line-declares.2.2): such a declaration's body, sections
/// and lead are its block's by the record's own rules at once, because 0.17 printed
/// nothing for it and there are no bytes of today's to keep.
fn bare_declaration_block_end(record: &FileRecord, decl_line: usize) -> Option<usize> {
    bare_block_line(record, decl_line)
        .then(|| record.block_at(decl_line).map(|block| block.end))
        .flatten()
}

/// A line as the scan read it: inside its docstring's delimiters where it is a
/// docstring line, with whether it is one and whether it closes it (§AR-scanner.4.4).
fn scanned(record: &FileRecord, lineno: usize) -> (&str, bool, bool) {
    let line = record.line(lineno).unwrap_or("");
    match record.zone_at(lineno) {
        Some(zone) => (&line[zone.start..zone.end], true, zone.closes),
        None => (line, false, false),
    }
}

/// Where an accepted section a citation sits in ends, as 0.17 answers it
/// (§AR-scanner.2.4.4, §DF-one-parse-answers-from-0-19, P12): in a Markdown file
/// with an ATX heading, before the next ATX heading as deep or shallower; anywhere
/// else, before the declaration's next accepted section as deep or shallower, so
/// a plain, rejected or duplicate heading in a doc-comment leaves it open.
/// `sections` are the declaration's accepted sections as `(path, line, level)`,
/// in line order; the answer is `usize::MAX` where nothing closes the section.
pub(crate) struct TodayOwnerEnds<'a> {
    record: &'a FileRecord,
    by_markdown_headings: bool,
}

impl<'a> TodayOwnerEnds<'a> {
    pub(crate) fn new(record: &'a FileRecord) -> Self {
        let by_markdown_headings =
            record.markdown && record.headings.iter().any(|heading| heading.atx);
        Self {
            record,
            by_markdown_headings,
        }
    }

    pub(crate) fn end(
        &self,
        sections: &[(String, usize, usize)],
        line: usize,
        level: usize,
    ) -> usize {
        let closer = if self.by_markdown_headings {
            self.record
                .headings_between(line + 1, usize::MAX)
                .iter()
                .find(|next| next.atx && next.level <= level)
                .map(|next| next.line)
        } else {
            sections
                .iter()
                .find(|(_, next_line, next_level)| *next_line > line && *next_level <= level)
                .map(|(_, next_line, _)| *next_line)
        };
        closer.map_or(usize::MAX, |next| next - 1)
    }
}

/// The lead the record gives a site, read through today's envelope: the lines
/// after `site_line` down to `lead_end` (§FS-declarations.lead), so the warning
/// of §FS-declarations.checks.parse-moves.2's third shape measures the lead 0.19.0 prints beside the one
/// [`today_slices`] prints now.
pub(crate) fn record_lead(
    record: &FileRecord,
    declaration_line: usize,
    site_line: usize,
    lead_end: usize,
    lexical: LexicalSettings<'_>,
) -> String {
    let is_md = record.markdown;
    let (scan_line, in_docstring, _) = scanned(record, declaration_line);
    let own_marker = (!is_md && !in_docstring)
        .then(|| own_line_marker(scan_line, lexical))
        .flatten();
    // §FS-declarations.line.block-comment: the envelope `today_slices` reads such a block with.
    let record_envelope = bare_declaration_block_end(record, declaration_line).is_some();
    let mut sink = TodaySink::new(ShowRenderMode::Default, false);
    for lineno in site_line + 1..=lead_end {
        let (scan_line, in_docstring, closes) = scanned(record, lineno);
        if in_docstring && closes && scan_line.trim().is_empty() {
            break;
        }
        let markdown_heading = is_md || in_docstring;
        sink.lines.push(if record_envelope {
            comment_envelope(scan_line, &BlockForm::Block).to_string()
        } else {
            clean_envelope_line(scan_line, markdown_heading, own_marker.as_deref())
        });
    }
    sink.finish().0
}

/// Whether a heading the record rejects is a section from 0.19.0: a `/** … */`
/// line without ` * ` that reads, kept as written, as a heading deeper than its
/// declaration's (§FS-languages.comment-wrapper.headings, the second shape of
/// §FS-declarations.checks.parse-moves.2).
/// Today the scan reads its first `#` as a comment marker, so it is too shallow.
pub(crate) fn becomes_section(
    record: &FileRecord,
    heading: &RecordedHeading,
    declaration_level: usize,
) -> bool {
    if heading.kind != HeadingKind::Rejected || heading.path.is_none() || record.markdown {
        return false;
    }
    let (Some(block), Some(line)) = (record.block_at(heading.line), record.line(heading.line))
    else {
        return false;
    };
    block.form == BlockForm::Block
        && !line.trim_start().starts_with('*')
        && markdown_heading_level(comment_envelope(line, &block.form).trim_start())
            .is_some_and(|level| level > declaration_level)
}
