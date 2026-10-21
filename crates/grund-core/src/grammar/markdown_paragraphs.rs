//! Where the paragraphs of a Markdown document begin, for the document reading of
//! §FS-citations.wrap.enclosing.shape: a link's text runs to the end of its
//! paragraph, and a definition only starts one. `markdown_links.rs` asks it of
//! every line in order; the fences are the caller's (§FS-check.1.1.5).
//!
//! Not a Markdown parser (§FS-citations.wrap.enclosing.scope): a paragraph ends
//! at the lines that point names and continues on every other line, a quoted line
//! of its own quote and a lazy one included.

use super::fence::markdown_fence_delimiter;

/// A Markdown line's block-quote depth and its text after the `>` markers, each
/// marker with the one space that may follow it.
pub(crate) fn blockquote_markers(line: &str) -> (usize, &str) {
    let mut depth = 0;
    let mut text = line;
    while let Some(after) = text.trim_start_matches([' ', '\t']).strip_prefix('>') {
        depth += 1;
        text = after.strip_prefix(' ').unwrap_or(after);
    }
    (depth, text)
}

/// What a line's text after its `>` markers opens.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Block {
    /// Nothing: the line is blank.
    Blank,
    /// A block of one line: an ATX heading, a thematic break, or a fence opener
    /// behind `>` markers.
    Leaf,
    /// A list item, whose text opens a paragraph of the item's.
    Item,
    /// Text, which continues the open paragraph or opens one.
    Text,
}

/// How one line sits in its paragraph.
#[derive(Clone, Copy)]
pub(super) struct ParagraphLine {
    /// The line ends the paragraph before it.
    pub(super) opens: bool,
    /// A definition may start on the line: it starts its paragraph, or every
    /// line of the paragraph before it was a definition
    /// (§FS-citations.wrap.enclosing.shape).
    pub(super) may_define: bool,
}

/// The paragraph a document read line by line has open
/// (§FS-citations.wrap.enclosing.shape).
#[derive(Default)]
pub(super) struct Paragraphs {
    /// The block-quote depth the open paragraph began at; `None` between them.
    depth: Option<usize>,
    /// Every line of the open paragraph so far was a definition.
    defining: bool,
}

impl Paragraphs {
    /// Read `line`. A quoted line continues the paragraph of its quote, and a
    /// line with fewer `>` markers continues it lazily, unless its text is blank
    /// or opens a block; a line with more opens a quote of its own.
    pub(super) fn line(&mut self, line: &str) -> ParagraphLine {
        let (depth, text) = blockquote_markers(line);
        let block = block_start(text);
        if matches!(block, Block::Blank | Block::Leaf) {
            self.depth = None;
            return ParagraphLine {
                opens: true,
                may_define: false,
            };
        }
        let opens = block == Block::Item || self.depth.is_none_or(|open| depth > open);
        if opens {
            self.depth = Some(depth);
            self.defining = true;
        }
        ParagraphLine {
            opens,
            may_define: self.defining,
        }
    }

    /// A fenced line: no paragraph reaches across it.
    pub(super) fn fenced(&mut self) {
        self.depth = None;
    }

    /// Whether the line just read ended a definition or a label that may still
    /// become one: only a run of definitions lets the next line hold another.
    pub(super) fn defined(&mut self, defined: bool) {
        self.defining &= defined;
    }
}

/// §FS-citations.wrap.enclosing.shape: the block `text`, a line after its `>`
/// markers, opens — a blank line, an ATX heading, a list item, a thematic break,
/// or a fence a block quote holds; a fence outside one is the caller's.
fn block_start(text: &str) -> Block {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return Block::Blank;
    }
    if markdown_fence_delimiter(&mut None, text) {
        return Block::Leaf;
    }
    let hashes = trimmed.bytes().take_while(|b| *b == b'#').count();
    if (1..=6).contains(&hashes)
        && trimmed[hashes..]
            .chars()
            .next()
            .is_none_or(char::is_whitespace)
    {
        return Block::Leaf;
    }
    let bullet = trimmed.as_bytes()[0];
    if matches!(bullet, b'-' | b'*' | b'+') {
        let rest = &trimmed[1..];
        if trimmed.bytes().filter(|b| *b == bullet).count() >= 3
            && trimmed
                .bytes()
                .all(|b| b == bullet || b == b' ' || b == b'\t')
        {
            return Block::Leaf;
        }
        if rest.is_empty() || rest.starts_with([' ', '\t']) {
            return Block::Item;
        }
        return Block::Text;
    }
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    if (1..=9).contains(&digits)
        && let Some(after) = trimmed[digits..].strip_prefix(['.', ')'])
        && (after.is_empty() || after.starts_with([' ', '\t']))
    {
        return Block::Item;
    }
    if trimmed.len() >= 3 && trimmed.bytes().all(|b| b == b'_' || b == b' ') {
        return Block::Leaf;
    }
    Block::Text
}
