//! The file record (§AR-scanner.2.8): what the scan read of one file, kept in
//! the `Catalog` so no command reads the file again or parses it a second way.
//! Plain data and the slicing that needs no rule; the per-file pass fills it
//! (`scanner/record.rs`), and the rules that read it — where a body, a section
//! and a lead end — are functions over it there.

use std::ops::Range;
use std::sync::{Arc, OnceLock};

/// A scanned file's text as `sources` returned it (§AR-scanner.2.8.1): the string
/// a disk read produced, or the editor's open buffer itself, shared rather than
/// copied (§AR-scanner.2.8.4).
#[derive(Clone)]
pub(crate) enum FileText {
    Disk(Arc<String>),
    Buffer(Arc<str>),
}

impl FileText {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Disk(text) => text.as_str(),
            Self::Buffer(text) => text,
        }
    }
}

/// What a recorded heading is to the scan (§AR-scanner.2.8.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HeadingKind {
    /// A declaration line, near misses included (§AR-scanner.2.1).
    Declaration,
    /// An accepted section of the declaration whose body holds it (§AR-scanner.2.2).
    Section,
    /// A later heading claiming a path its declaration already holds (§AR-scanner.2.2.3).
    Duplicate,
    /// A section path no declaration accepted: too shallow, outside every body, or
    /// in a comment the scan reads as a heading its declaration cannot hold.
    Rejected,
    /// A heading with no path.
    Plain,
}

/// One heading outside a fence, in Markdown or inside a comment (§AR-scanner.2.8.1).
#[derive(Clone, Debug)]
pub(crate) struct RecordedHeading {
    /// 1-indexed line.
    pub(crate) line: usize,
    /// The `#` run the scan read the heading at: the declaration's or section's
    /// heading level, or a plain heading's.
    pub(crate) level: usize,
    /// A Markdown heading by CommonMark's indentation rule, the set a Markdown body
    /// and section end at (§AR-scanner.2.4.1).
    pub(crate) atx: bool,
    /// A Markdown heading once its indentation is set aside: the set the grounding
    /// structure lists (§AR-scanner.2.7).
    pub(crate) listed: bool,
    /// The heading line's bytes in the file text.
    pub(crate) span: Range<usize>,
    /// Its rendered text's bytes in the file text (§FS-declarations.heading.rendered-text):
    /// no opening `#` run, no closing sequence, no surrounding whitespace.
    pub(crate) text: Range<usize>,
    /// The section path parsed off it, if any: for a relative heading, the
    /// canonical path its parent supplied (§FS-declarations.section.relative).
    pub(crate) path: Option<String>,
    /// Written `.<component>` and resolved against its parent; its path is
    /// canonical and not the bytes the line carries.
    pub(crate) relative: bool,
    pub(crate) kind: HeadingKind,
    /// On a declaration, an accepted section or a duplicate claimant: the last line of
    /// its body (§FS-declarations.body, §AR-scanner.2.4.4); `0` on any other heading.
    pub(crate) end: usize,
    /// On a declaration, an accepted section or a duplicate claimant: the last line of
    /// its lead (§FS-declarations.lead); `0` on any other heading.
    pub(crate) lead_end: usize,
}

/// The comment form a block is written in (§FS-languages.comment-block).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BlockForm {
    /// A run of lines behind one marker: `//`, `///`, `//!`, `#`, `--`, …
    Line(String),
    /// A `/* … */` block, `/** … */` included.
    Block,
    /// A Python docstring.
    Docstring,
}

/// One comment block of a source file, walked once (§AR-scanner.2.8.1).
#[derive(Clone, Debug)]
pub(crate) struct RecordedBlock {
    /// 1-indexed inclusive line span.
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) form: BlockForm,
    /// Whether the file's language calls it documentation (§FS-inline-citation-style.1.1.1).
    pub(crate) doc: bool,
    /// Whether a line in declaration position in it may declare: always in v1, and
    /// in v2 as the declaration gate decides (§FS-config-v2.schema.sources.declarations.gate).
    pub(crate) declares: bool,
    /// Whether a `/* … */` block's `*/` was found; only a closed block has bare
    /// lines in declaration position (§FS-declarations.line.block-comment). Always
    /// `true` for the other forms, which have no closer to miss.
    pub(crate) closed: bool,
}

/// A line whose scanned text is a part of it (§AR-scanner.2.8.1): a Python
/// docstring line, read inside its delimiters (§AR-scanner.4.4).
#[derive(Clone, Copy, Debug)]
pub(crate) struct LineZone {
    pub(crate) line: usize,
    /// The scanned bytes, relative to the line's start.
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// Every scanned file's record, by the path the scan read it under (§AR-scanner.2.8).
pub(crate) type FileRecords = std::collections::BTreeMap<std::path::PathBuf, Arc<FileRecord>>;

/// One scanned file's record (§AR-scanner.2.8).
pub(crate) struct FileRecord {
    pub(crate) text: FileText,
    pub(crate) markdown: bool,
    pub(crate) total_lines: usize,
    /// Every heading outside a fence, in line order.
    pub(crate) headings: Vec<RecordedHeading>,
    /// Every comment block, in line order; empty for Markdown.
    pub(crate) blocks: Vec<RecordedBlock>,
    /// The docstring lines, in line order; empty outside Python.
    pub(crate) zones: Vec<LineZone>,
    /// The source lines inside a fence of their comment block, delimiters
    /// included, in line order; empty for Markdown (§FS-languages.fence.comment-block).
    pub(crate) fenced: Vec<usize>,
    /// The last line of a Markdown file's front matter, `None` where it has none
    /// (§FS-check.1.1.5.2).
    pub(crate) front_matter: Option<usize>,
    line_starts: OnceLock<Vec<usize>>,
}

impl FileRecord {
    pub(crate) fn new(text: FileText, markdown: bool) -> Self {
        Self {
            text,
            markdown,
            total_lines: 0,
            headings: Vec::new(),
            blocks: Vec::new(),
            zones: Vec::new(),
            fenced: Vec::new(),
            front_matter: None,
            line_starts: OnceLock::new(),
        }
    }

    /// Whether line `line` (1-indexed) sits in the file's front matter, the
    /// region whose citations its first declaration owns and `fmt` never
    /// rewrites (§FS-check.1.1.5.2, §FS-check.1.1.5.2.1).
    pub(crate) fn in_front_matter(&self, line: usize) -> bool {
        self.front_matter
            .is_some_and(|last| line >= 1 && line <= last)
    }

    pub(crate) fn text(&self) -> &str {
        self.text.as_str()
    }

    /// Line `line` (1-indexed) as `str::lines` yields it, or `None` past the end.
    pub(crate) fn line(&self, line: usize) -> Option<&str> {
        let starts = self.line_starts();
        let start = *starts.get(line.checked_sub(1)?)?;
        let text = self.text();
        // As `str::lines`: a `\r` goes only with the `\n` after it.
        Some(match text[start..].find('\n') {
            Some(at) => {
                let raw = &text[start..start + at];
                raw.strip_suffix('\r').unwrap_or(raw)
            }
            None => &text[start..],
        })
    }

    /// The byte offset line `line` (1-indexed) starts at.
    pub(crate) fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts().get(line.checked_sub(1)?).copied()
    }

    /// The heading recorded at `line`.
    pub(crate) fn heading_at(&self, line: usize) -> Option<&RecordedHeading> {
        let index = self
            .headings
            .binary_search_by_key(&line, |heading| heading.line)
            .ok()?;
        self.headings.get(index)
    }

    /// The headings on lines `first..=last`.
    pub(crate) fn headings_between(&self, first: usize, last: usize) -> &[RecordedHeading] {
        let from = self
            .headings
            .partition_point(|heading| heading.line < first);
        let to = self
            .headings
            .partition_point(|heading| heading.line <= last);
        &self.headings[from..to.max(from)]
    }

    /// The docstring zone of `line`, if it is a docstring line.
    pub(crate) fn zone_at(&self, line: usize) -> Option<&LineZone> {
        let index = self
            .zones
            .binary_search_by_key(&line, |zone| zone.line)
            .ok()?;
        self.zones.get(index)
    }

    /// Whether `line` is inside a fence of its comment block, so it is never a
    /// heading (§FS-languages.fence.comment-block).
    pub(crate) fn in_comment_fence(&self, line: usize) -> bool {
        self.fenced.binary_search(&line).is_ok()
    }

    /// The comment block holding `line`.
    pub(crate) fn block_at(&self, line: usize) -> Option<&RecordedBlock> {
        let index = self.blocks.partition_point(|block| block.end < line);
        self.blocks.get(index).filter(|block| block.start <= line)
    }

    /// A recorded heading's rendered text.
    pub(crate) fn heading_text(&self, heading: &RecordedHeading) -> &str {
        &self.text()[heading.text.clone()]
    }

    fn line_starts(&self) -> &[usize] {
        self.line_starts.get_or_init(|| {
            let text = self.text();
            let mut starts = Vec::with_capacity(self.total_lines);
            let mut start = 0;
            for line in text.split_inclusive('\n') {
                starts.push(start);
                start += line.len();
            }
            starts
        })
    }
}

impl std::fmt::Debug for FileRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileRecord")
            .field("bytes", &self.text().len())
            .field("headings", &self.headings)
            .field("blocks", &self.blocks)
            .finish_non_exhaustive()
    }
}
