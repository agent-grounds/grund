//! The document reading of §FS-citations.wrap: which link, in which of the forms
//! §FS-citations.wrap.enclosing.scope lists, holds each span of a Markdown file,
//! and which bytes no pass rewrites (§FS-citations.zones.authored-markup,
//! §FS-citations.zones.link-destination).
//!
//! A reference finds its definition anywhere in the file and a link's text may
//! run onto later lines of its paragraph (§FS-citations.wrap.enclosing.shape),
//! so one line cannot answer alone. The file is walked once, lazily, to settle
//! what each `[` turns out to be; then the line a caller holds is walked again
//! from the state its paragraph left it in, to place those spans in its bytes.
//! Every pass of `fmt` rewrites only tokens and never a bracket, a backtick or a
//! `<`, so the `[`s of a line it has already rewritten are the `[`s of the
//! line as written, one for one, and the settled forms still apply.
//!
//! The walk is skipped where it can find nothing: a file with no `[` and no `<`
//! is never read, and nothing is read before a caller asks about a citation
//! (§FS-citations.wrap, §GOAL-fast-feedback).

use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use super::fmt_cross_refs::CrossRefWrap;
use super::line_scan::LineScan;
use super::link_walk::{After, Open, Sink, WalkState, walk_line};
use super::markdown_blocks::MarkdownBlocks;
use super::markdown_paragraphs::{Paragraphs, blockquote_markers};

/// What one `[` turned out to open, settled over its whole paragraph
/// (§FS-citations.wrap.enclosing.shape).
#[derive(Clone, Debug)]
enum Shape {
    /// An inline link; an entry where its destination is not empty.
    Inline { entry: bool },
    /// An image's description (§FS-citations.wrap.enclosing.images).
    Image,
    /// A full or collapsed reference, and the key its definition is found by.
    Reference { key: String, collapsed: bool },
    /// A definition's label.
    Definition,
    /// Brackets followed by nothing a form names: a shortcut reference where a
    /// definition matches `key`, plain text where none does.
    Bare { key: String },
    /// Opens no link: never balanced, or a destination that never closes.
    NoLink,
    /// Never balanced, with an unclosed backtick run after it: an incomplete
    /// label, protected to its paragraph's end.
    Incomplete,
}

/// The settled reading of one document.
#[derive(Default)]
struct DocReading {
    shapes: Vec<Shape>,
    /// The id of the first `[` on each line, and one past the last line.
    first: Vec<usize>,
    /// The state a line begins in, kept only where something is still open.
    carry: HashMap<usize, WalkState>,
    /// Whether a definition may start on each line (§FS-citations.wrap.enclosing.shape).
    defines: Vec<bool>,
    /// Each definition's key (§FS-citations.wrap.enclosing.shape), and whether its
    /// destination is not empty; the first definition of a key wins.
    definitions: HashMap<String, bool>,
}

impl DocReading {
    fn of(text: &str) -> Self {
        let lines: Vec<&str> = text.lines().collect();
        let mut reading = Self::default();
        if !text.bytes().any(|byte| matches!(byte, b'[' | b'<')) {
            return reading;
        }
        let mut builder = Builder {
            lines: &lines,
            line: 0,
            opens: Vec::new(),
            ticked: Vec::new(),
            defined: false,
            reading: &mut reading,
        };
        // §FS-check.1.1.5.2: a whole file's front matter is a block of its own, so
        // a `[` opened in it never pairs with a `]` below it.
        let mut blocks = MarkdownBlocks::lines(lines.iter().copied());
        let front_matter = blocks.front_matter().unwrap_or(0);
        let mut paragraphs = Paragraphs::default();
        let mut state = WalkState::default();
        for (index, line) in lines.iter().enumerate() {
            builder.line = index;
            builder.reading.first.push(builder.reading.shapes.len());
            if index > 0 && index == front_matter {
                paragraphs.fenced();
                builder.finish(&mut state);
            }
            if blocks.line(line).in_fence() {
                paragraphs.fenced();
                builder.finish(&mut state);
                builder.reading.defines.push(false);
                continue;
            }
            let paragraph = paragraphs.line(line);
            if paragraph.opens {
                builder.finish(&mut state);
            }
            builder.reading.defines.push(paragraph.may_define);
            state = state.carried();
            if !state.stack.is_empty() || state.anchor {
                builder.reading.carry.insert(index, state.clone());
            }
            builder.defined = false;
            walk_line(line, paragraph.may_define, &mut state, &mut builder);
            paragraphs.defined(builder.defined || state.stack.iter().any(|open| open.may_define));
        }
        builder.finish(&mut state);
        reading.first.push(reading.shapes.len());
        reading
    }

    /// Walk `line`, as a caller holds it, from the state line `index` of the
    /// document begins in, and say where each span falls in its bytes.
    fn scan(&self, index: usize, line: &str) -> LineScan {
        let mut state = self.carry.get(&index).cloned().unwrap_or_default();
        let mut sink = Placer {
            reading: self,
            first: self.first.get(index).copied().unwrap_or(0),
            count: match (self.first.get(index), self.first.get(index + 1)) {
                (Some(first), Some(next)) => next - first,
                _ => 0,
            },
            ordinal: 0,
            len: line.len(),
            scan: LineScan::default(),
        };
        if state.stack.is_empty()
            && !state.anchor
            && !line.bytes().any(|byte| matches!(byte, b'[' | b']' | b'<'))
        {
            return sink.scan;
        }
        let may_define = self.defines.get(index).copied().unwrap_or(false);
        walk_line(line, may_define, &mut state, &mut sink);
        for open in state.stack {
            sink.place(open, line.len(), None);
        }
        sink.scan
    }
}

/// §FS-citations.wrap.enclosing.shape: a key read as CommonMark reads one, case
/// folded, each run of whitespace one space, trimmed.
fn normalized_key(key: &str) -> String {
    key.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Whether the definition whose destination starts at `dest_start` has one.
fn destination_present(line: &str, dest_start: usize) -> bool {
    let rest = line.get(dest_start..).unwrap_or("").trim_end();
    match rest.strip_prefix('<') {
        Some(inner) => inner.split('>').next().is_some_and(|dest| !dest.is_empty()),
        None => !rest.is_empty(),
    }
}

/// The document walk: names every `[`, settles its shape where it closes, and
/// collects the definitions.
struct Builder<'b, 'd> {
    lines: &'b [&'d str],
    line: usize,
    /// Where each `[` stands, by id.
    opens: Vec<(usize, usize)>,
    ticked: Vec<bool>,
    /// A definition closed on the current line.
    defined: bool,
    reading: &'b mut DocReading,
}

impl Builder<'_, '_> {
    /// A paragraph ended: what it still holds open opens no link, or is an
    /// incomplete label.
    fn finish(&mut self, state: &mut WalkState) {
        for open in state.stack.drain(..) {
            if let Some(id) = open.id
                && self.ticked[id]
            {
                self.reading.shapes[id] = Shape::Incomplete;
            }
        }
        state.anchor = false;
    }

    /// The text of `[` `id` up to the `]` at `close` on the current line. A
    /// later line's `>` markers are its quote's, not the text's, so a key spelled
    /// across quoted lines is the key its definition spells on one
    /// (§FS-citations.wrap.enclosing.shape).
    fn text(&self, id: usize, close: usize) -> String {
        let (line, at) = self.opens[id];
        if line == self.line {
            return self.lines[line][at + 1..close].to_string();
        }
        let mut text = self.lines[line][at + 1..].to_string();
        for middle in &self.lines[line + 1..self.line] {
            text.push('\n');
            text.push_str(blockquote_markers(middle).1);
        }
        let last = self.lines[self.line];
        let from = (last.len() - blockquote_markers(last).1.len()).min(close);
        text.push('\n');
        text.push_str(&last[from..close]);
        text
    }
}

impl Sink for Builder<'_, '_> {
    fn opened(&mut self, at: usize) -> Option<usize> {
        let id = self.reading.shapes.len();
        self.reading.shapes.push(Shape::NoLink);
        self.opens.push((self.line, at));
        self.ticked.push(false);
        Some(id)
    }

    fn closed(&mut self, open: Open, at: usize, after: After) {
        let Some(id) = open.id else { return };
        let shape = match after {
            After::Inline { .. } if open.image => Shape::Image,
            After::Inline { url_start, close } => Shape::Inline {
                entry: !self.lines[self.line][url_start..close].trim().is_empty(),
            },
            After::Unclosed => Shape::NoLink,
            _ if open.image => Shape::NoLink,
            After::Key { start, end } if start == end => Shape::Reference {
                key: normalized_key(&self.text(id, at)),
                collapsed: true,
            },
            After::Key { start, end } => Shape::Reference {
                key: normalized_key(&self.lines[self.line][start..end]),
                collapsed: false,
            },
            After::Definition { dest_start } => {
                self.defined = true;
                let present = destination_present(self.lines[self.line], dest_start);
                self.reading
                    .definitions
                    .entry(normalized_key(&self.text(id, at)))
                    .or_insert(present);
                Shape::Definition
            }
            After::Other => Shape::Bare {
                key: normalized_key(&self.text(id, at)),
            },
        };
        self.reading.shapes[id] = shape;
    }

    fn zone(&mut self, _: usize, _: usize) {}

    fn anchor_text(&mut self, _: usize, _: usize) {}

    fn unclosed_tick(&mut self, stack: &[Open]) {
        for id in stack.iter().filter_map(|open| open.id) {
            self.ticked[id] = true;
        }
    }
}

/// The line walk: places each settled span in the caller's bytes.
struct Placer<'r> {
    reading: &'r DocReading,
    first: usize,
    count: usize,
    ordinal: usize,
    len: usize,
    scan: LineScan,
}

impl Placer<'_> {
    /// Place `open`'s text, which ends at `end`; `after` is what follows its
    /// `]` where that stands on this line.
    fn place(&mut self, open: Open, end: usize, after: Option<After>) {
        let Some(shape) = open.id.and_then(|id| self.reading.shapes.get(id)) else {
            return;
        };
        let start = open.at.map_or(0, |at| at + 1);
        let scan = &mut self.scan;
        match shape {
            Shape::Inline { entry } => scan.texts.push((start, end, *entry)),
            Shape::Image => {
                if let Some(After::Inline { url_start, close }) = after {
                    scan.images
                        .push((start, end, CrossRefWrap { url_start, close }));
                }
            }
            Shape::Reference { key, collapsed } => {
                if *collapsed {
                    scan.zones.push((start, end));
                } else if let Some(After::Key { start, end }) = after {
                    scan.zones.push((start, end));
                }
                let entry = self.reading.definitions.get(key) == Some(&true);
                scan.texts.push((start, end, entry));
            }
            Shape::Definition => {
                scan.zones.push((start, end));
                if let Some(After::Definition { dest_start }) = after {
                    scan.zones.push((dest_start, self.len));
                }
                scan.texts.push((start, end, false));
            }
            Shape::Bare { key } => {
                if let Some(&entry) = self.reading.definitions.get(key) {
                    scan.zones.push((start, end));
                    scan.texts.push((start, end, entry));
                }
            }
            Shape::Incomplete => scan.texts.push((start, end, false)),
            Shape::NoLink => {}
        }
    }
}

impl Sink for Placer<'_> {
    fn opened(&mut self, _: usize) -> Option<usize> {
        let id = (self.ordinal < self.count).then_some(self.first + self.ordinal);
        self.ordinal += 1;
        id
    }

    fn closed(&mut self, open: Open, at: usize, after: After) {
        self.place(open, at, Some(after));
    }

    fn zone(&mut self, start: usize, end: usize) {
        self.scan.zones.push((start, end));
    }

    fn anchor_text(&mut self, start: usize, end: usize) {
        self.scan.texts.push((start, end, false));
    }

    fn unclosed_tick(&mut self, _: &[Open]) {}
}

/// One Markdown document's link reading, built on the first question asked of
/// it (§FS-citations.wrap). Every caller that reads a whole file holds one and
/// asks it line by line through [`LineLinks`].
pub(crate) struct MarkdownLinks<'d> {
    text: &'d str,
    reading: OnceCell<DocReading>,
    /// The line last placed: every pass asks of one citation's line in turn.
    last: RefCell<Option<(usize, String, Rc<LineScan>)>>,
}

impl<'d> MarkdownLinks<'d> {
    pub(crate) fn new(text: &'d str) -> Self {
        Self {
            text,
            reading: OnceCell::new(),
            last: RefCell::new(None),
        }
    }

    /// Line `index` of the document, 0-based as `str::lines` counts.
    pub(crate) fn line(&self, index: usize) -> LineLinks<'_> {
        LineLinks {
            doc: Some((self, index)),
        }
    }
}

/// One line's place in its document's link reading. The default is a line
/// read alone, as its own document, for a caller that holds no more than it.
#[derive(Clone, Copy, Default)]
pub(crate) struct LineLinks<'a> {
    doc: Option<(&'a MarkdownLinks<'a>, usize)>,
}

impl LineLinks<'_> {
    /// Where the document's spans fall in `line`, the caller's current bytes of
    /// this line.
    pub(crate) fn scan(self, line: &str) -> Rc<LineScan> {
        let Some((doc, index)) = self.doc else {
            return Rc::new(DocReading::of(line).scan(0, line));
        };
        if let Some((at, text, scan)) = &*doc.last.borrow()
            && *at == index
            && text == line
        {
            return Rc::clone(scan);
        }
        let reading = doc.reading.get_or_init(|| DocReading::of(doc.text));
        let scan = Rc::new(reading.scan(index, line));
        *doc.last.borrow_mut() = Some((index, line.to_string(), Rc::clone(&scan)));
        scan
    }
}
