//! The YAML span tree (§DA-yaml-value-reader): the events of `yaml-rust2`'s
//! parser, read against the source text, as documents, mappings, sequences,
//! scalars and aliases with exact byte spans (§DA-yaml-value-reader.2.2). It
//! applies no value rule: a value source's subset is a separate pass
//! (`value_yaml.rs`), so a front-matter reader can read the same tree with the
//! block scalars and nesting that pass refuses.

use std::str::Chars;

use yaml_rust2::parser::{Event, Parser};
use yaml_rust2::scanner::TScalarStyle;

use super::yaml_text::YamlText;

/// A node's bytes in the source text, the byte order mark included in the count
/// and excluded from every span (§DA-yaml-value-reader.2.3).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct YamlSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScalarStyle {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
    Folded,
}

#[derive(Debug)]
pub(crate) enum YamlKind {
    /// `value` is the parser's decoding; `single_line` says whether the
    /// scalar's own bytes stay on one line.
    Scalar {
        value: String,
        style: ScalarStyle,
        single_line: bool,
    },
    Sequence {
        items: Vec<YamlNode>,
    },
    /// Entries in source order, a repeated key kept (§FS-values.2.2.2).
    Mapping {
        entries: Vec<(YamlNode, YamlNode)>,
    },
    Alias,
}

/// One node as written: its anchor and tag are kept as data, never applied.
#[derive(Debug)]
pub(crate) struct YamlNode {
    pub(crate) kind: YamlKind,
    pub(crate) span: YamlSpan,
    pub(crate) anchored: bool,
    pub(crate) tag: Option<String>,
}

#[derive(Debug)]
pub(crate) struct YamlDocument {
    pub(crate) root: YamlNode,
    /// Where the document's `---` stands, `None` for an implicit start.
    pub(crate) explicit_start: Option<usize>,
}

#[derive(Debug)]
pub(crate) struct YamlStream {
    pub(crate) documents: Vec<YamlDocument>,
}

/// A stream the parser rejects, at a one-based line and character column read
/// from its `Marker`, never from the byte count its message names
/// (§DA-yaml-value-reader.2.3).
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct YamlError {
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) message: String,
}

/// Read `text` into its span tree (§DA-yaml-value-reader.2.2).
pub(crate) fn parse_yaml(text: &str) -> Result<YamlStream, YamlError> {
    // §DA-yaml-value-reader.2.3: the parser keeps a BOM as part of the first key.
    let bom = if text.starts_with('\u{feff}') { 3 } else { 0 };
    let body = &text[bom..];
    let mut tree = Tree {
        parser: Parser::new_from_str(body),
        text: YamlText::new(body),
        bom,
        flow: 0,
        depth: 0,
    };
    tree.stream()
}

/// §DA-yaml-value-reader.2.2: how deep collections may nest before the tree,
/// which recurses once per collection, refuses one (§REQ-never-crashes.1).
pub(crate) const MAX_NESTING: usize = 128;

struct Tree<'a> {
    parser: Parser<Chars<'a>>,
    text: YamlText<'a>,
    bom: usize,
    /// How many flow collections enclose the event being read.
    flow: usize,
    /// How many collections enclose the event being read, the root included.
    depth: usize,
}

impl<'a> Tree<'a> {
    fn next(&mut self) -> Result<(Event, usize), YamlError> {
        let (event, mark) = self.parser.next_token().map_err(|error| YamlError {
            line: error.marker().line(),
            column: error.marker().col() + 1,
            message: error.info().to_string(),
        })?;
        Ok((event, self.text.offset(mark)))
    }

    fn peek_offset(&mut self) -> Result<usize, YamlError> {
        let mark = match self.parser.peek() {
            Ok((_, mark)) => *mark,
            Err(error) => {
                return Err(YamlError {
                    line: error.marker().line(),
                    column: error.marker().col() + 1,
                    message: error.info().to_string(),
                });
            }
        };
        Ok(self.text.offset(mark))
    }

    /// §DA-yaml-value-reader.2.2, §FS-values.5.3, §FS-fields.front-matter.reading:
    /// open the collection that starts at `at`, refused at its line when it would
    /// nest past [`MAX_NESTING`], before the tree recurses into it.
    fn enter(&mut self, at: usize) -> Result<(), YamlError> {
        self.depth += 1;
        if self.depth <= MAX_NESTING {
            return Ok(());
        }
        let line = self.text.line_of(at);
        let start = self.text.lines[line - 1];
        Err(YamlError {
            line,
            column: self.text.body[start..at].chars().count() + 1,
            message: format!("collections are nested more than {MAX_NESTING} deep"),
        })
    }

    fn stream(&mut self) -> Result<YamlStream, YamlError> {
        let mut documents = Vec::new();
        loop {
            match self.next()? {
                (Event::DocumentStart, at) => {
                    let explicit_start = self.text.body[at..]
                        .starts_with("---")
                        .then_some(at + self.bom);
                    let (event, at) = self.next()?;
                    let root = self.node(event, at)?;
                    self.next()?; // DocumentEnd
                    documents.push(YamlDocument {
                        root,
                        explicit_start,
                    });
                }
                (Event::StreamEnd, _) => return Ok(YamlStream { documents }),
                _ => {}
            }
        }
    }

    fn node(&mut self, event: Event, at: usize) -> Result<YamlNode, YamlError> {
        let (kind, start, end, anchor, tag) = match event {
            Event::Scalar(value, style, anchor, tag) => {
                let style = match style {
                    TScalarStyle::Plain => ScalarStyle::Plain,
                    TScalarStyle::SingleQuoted => ScalarStyle::SingleQuoted,
                    TScalarStyle::DoubleQuoted => ScalarStyle::DoubleQuoted,
                    TScalarStyle::Literal => ScalarStyle::Literal,
                    TScalarStyle::Folded => ScalarStyle::Folded,
                };
                let (start, end, single_line) = self.scalar_bytes(at, &value, style)?;
                let kind = YamlKind::Scalar {
                    value,
                    style,
                    single_line,
                };
                (kind, start, end, anchor, tag)
            }
            Event::SequenceStart(anchor, tag) => {
                self.enter(at)?;
                let flow = self.text.body[at..].starts_with('[');
                self.flow += usize::from(flow);
                let mut items = Vec::new();
                let close = loop {
                    match self.next()? {
                        (Event::SequenceEnd, close) => break close,
                        (event, at) => items.push(self.node(event, at)?),
                    }
                };
                self.flow -= usize::from(flow);
                self.depth -= 1;
                let start = if flow { at } else { self.text.dash_before(at) };
                if !flow {
                    self.locate_items(start, close, &mut items);
                }
                let end = if flow {
                    close + 1
                } else {
                    items
                        .last()
                        .map_or(start + 1, |item| item.span.end - self.bom)
                };
                (YamlKind::Sequence { items }, start, end, anchor, tag)
            }
            Event::MappingStart(anchor, tag) => {
                self.enter(at)?;
                let flow = self.text.body[at..].starts_with('{');
                self.flow += usize::from(flow);
                let mut entries = Vec::new();
                let close = loop {
                    match self.next()? {
                        (Event::MappingEnd, close) => break close,
                        (event, at) => {
                            let key = self.node(event, at)?;
                            let (event, at) = self.next()?;
                            let mut value = self.node(event, at)?;
                            if is_empty_plain(&value) {
                                // An absent value sits at the next token; it belongs to its key.
                                value.span = YamlSpan {
                                    start: key.span.end,
                                    end: key.span.end,
                                };
                            }
                            entries.push((key, value));
                        }
                    }
                };
                self.flow -= usize::from(flow);
                self.depth -= 1;
                // §DA-yaml-value-reader.2.3: a block mapping starts at its first key.
                let start = match entries.first() {
                    Some((key, _)) if !flow => key.span.start - self.bom,
                    _ => at,
                };
                let end = match entries.last() {
                    Some((_, value)) if !flow => value.span.end - self.bom,
                    _ => close + 1,
                };
                (YamlKind::Mapping { entries }, start, end, anchor, tag)
            }
            Event::Alias(_) => {
                let name = self.text.body[at + 1..]
                    .find(|c: char| c.is_whitespace() || ",[]{}".contains(c))
                    .map_or(self.text.body.len(), |len| at + 1 + len);
                (YamlKind::Alias, at, name, 0, None)
            }
            other => {
                return Err(YamlError {
                    line: self.text.line_of(at),
                    column: 1,
                    message: format!("unexpected YAML event {other:?}"),
                });
            }
        };
        Ok(YamlNode {
            kind,
            span: YamlSpan {
                start: start + self.bom,
                end: end.max(start) + self.bom,
            },
            anchored: anchor != 0,
            tag: tag.map(|tag| format!("{}{}", tag.handle, tag.suffix)),
        })
    }

    /// §DA-yaml-value-reader.2.4: a single-line scalar ends where a rescan of its
    /// own bytes does; any other ends at the next event, trimmed back.
    fn scalar_bytes(
        &mut self,
        at: usize,
        value: &str,
        style: ScalarStyle,
    ) -> Result<(usize, usize, bool), YamlError> {
        let body = self.text.body;
        let rest = &body[at..];
        let quoted = |quote: char, escape: Option<char>| {
            let mut chars = rest.char_indices().skip(1);
            let mut single_line = true;
            while let Some((index, c)) = chars.next() {
                match c {
                    '\n' => single_line = false,
                    _ if Some(c) == escape => {
                        chars.next();
                    }
                    _ if c == quote && quote == '\'' && rest[index + 1..].starts_with('\'') => {
                        chars.next();
                    }
                    _ if c == quote => return (at + index + 1, single_line),
                    _ => {}
                }
            }
            (body.len(), false)
        };
        Ok(match style {
            ScalarStyle::SingleQuoted => {
                let (end, single) = quoted('\'', None);
                (at, end, single)
            }
            ScalarStyle::DoubleQuoted => {
                let (end, single) = quoted('"', Some('\\'));
                (at, end, single)
            }
            ScalarStyle::Plain if !value.contains('\n') && rest.starts_with(value) => {
                (at, at + value.len(), true)
            }
            ScalarStyle::Plain => {
                let next = self.peek_offset()?;
                (at, self.text.trim_back(next, None, self.flow > 0), false)
            }
            ScalarStyle::Literal | ScalarStyle::Folded => {
                let next = self.peek_offset()?;
                let line_start = self.text.lines[self.text.line_of(at) - 1];
                let indent = self.text.body[line_start..at].chars().count();
                let end = self.text.trim_back(next, Some(indent), self.flow > 0);
                (self.text.block_indicator(at), end.max(at), false)
            }
        })
    }

    /// §DA-yaml-value-reader.2.3: each item of a block sequence starts at the
    /// `-` in the sequence's column, so an empty element, whose event sits at the
    /// next token, is located from its own dash.
    fn locate_items(&self, start: usize, close: usize, items: &mut [YamlNode]) {
        let first = self.text.line_of(start);
        let column = start - self.text.lines[first - 1];
        let last = self.text.line_of(close.min(self.text.body.len()));
        let dashes = (first..=last)
            .filter_map(|line| {
                let at = self.text.lines[line - 1] + column;
                let indent = self.text.body.get(self.text.lines[line - 1]..at)?;
                let item = self.text.body.get(at..)?.strip_prefix('-')?;
                let dash = item.is_empty() || item.starts_with([' ', '\t', '\r', '\n']);
                (dash && (line == first || indent.trim().is_empty())).then_some(at)
            })
            .collect::<Vec<_>>();
        if dashes.len() != items.len() {
            return;
        }
        for (item, dash) in items.iter_mut().zip(dashes) {
            if is_empty_plain(item) {
                item.span = YamlSpan {
                    start: dash + self.bom,
                    end: dash + 1 + self.bom,
                };
            }
        }
    }
}

fn is_empty_plain(node: &YamlNode) -> bool {
    matches!(&node.kind, YamlKind::Scalar { value, style: ScalarStyle::Plain, .. } if value.is_empty())
        && !node.anchored
        && node.tag.is_none()
}
