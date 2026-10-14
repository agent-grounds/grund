//! The string literals that span lines (§FS-languages.multi-line-string): a
//! per-file pre-pass that finds every recognized form's opener and closer, and
//! hands each line the state it **begins** in — how many of its leading bytes
//! are still inside a literal an earlier line opened (§AR-scanner.4.4).
//!
//! A pass over the whole file rather than a state carried line by line, because
//! a form counts only once its closer is found later in the file
//! (§FS-languages.multi-line-string.bound): a reader standing on the opener's line
//! cannot know that yet. Recognition, not parsing (§FS-non-goals.3): the table of
//! §FS-languages.multi-line-string.forms is all it knows of each language.

use std::path::Path;

use super::multi_line_forms::{Closer, Language, Skip, is_identifier_byte, quote_run};
use super::never_rewrite::is_escaped;
use super::source_line::{BlockCommentScanState, PythonDocstringScanState, source_scan_line};

/// The entry state of a line that begins inside a form whose closer is on a later
/// line: every byte of it is inside the literal.
pub(crate) const WHOLE_LINE: usize = usize::MAX;

/// For each line of one file, how many of its leading bytes are inside a
/// recognized multi-line form an earlier line opened: `0` for a line that begins
/// in code, the end of the form's closer on the line that closes it, and
/// [`WHOLE_LINE`] on a line between (§FS-languages.multi-line-string). Empty for a
/// file whose extension the form table does not name.
#[derive(Default)]
pub(crate) struct MultiLineStrings {
    entries: Vec<usize>,
}

impl MultiLineStrings {
    /// No line begins inside a literal: what a reader passes that asks only where
    /// a docstring sits, never whether a column is inside a string.
    pub(crate) const NONE: &'static Self = &Self {
        entries: Vec::new(),
    };

    /// Read `text` by the row of §FS-languages.multi-line-string.forms its
    /// extension names. `docstring_python` is `[scan] docstring_python`: where it is
    /// on, a docstring and an assigned-data span keep their own rules and open no
    /// form (§FS-languages.multi-line-string.bound).
    pub(crate) fn of(path: &Path, text: &str, docstring_python: bool) -> Self {
        let Some(language) = path
            .extension()
            .and_then(|extension| extension.to_str())
            .and_then(Language::of)
        else {
            return Self::default();
        };
        let lines = text.lines().collect::<Vec<_>>();
        let mut reader = Reader {
            language,
            lines: &lines,
            failed: Vec::new(),
        };
        Self {
            entries: reader.read(docstring_python),
        }
    }

    /// The entry state of line `index` (0-based).
    pub(crate) fn entry(&self, index: usize) -> usize {
        self.entries.get(index).copied().unwrap_or(0)
    }

    /// The entry states from line `start` (0-based) on, for a reader that holds a
    /// slice of the file's lines — a comment block — rather than the whole file.
    pub(crate) fn starting_at(&self, start: usize) -> &[usize] {
        self.entries.get(start..).unwrap_or(&[])
    }
}

/// What opens on a line and runs past it.
enum Opened {
    /// A form: where its closer ends, and the first line of its body.
    Form(Pos, usize),
    /// A `/* … */` opened after code: where its `*/` ends.
    Comment(Pos),
}

/// A position in the file: a line index and a byte column on it.
type Pos = (usize, usize);

struct Reader<'a> {
    language: Language,
    lines: &'a [&'a str],
    /// Closers already searched for and not found, with the position each search
    /// started from: a later search for the same closer from at or after it cannot
    /// succeed either, so a file of unclosed openers is read once, not once each.
    failed: Vec<(Closer, Pos)>,
}

impl Reader<'_> {
    fn read(&mut self, docstring_python: bool) -> Vec<usize> {
        let mut entries = vec![0; self.lines.len()];
        let mut blocks = BlockCommentScanState::default();
        let mut docstrings = PythonDocstringScanState::default();
        let python_docstrings = docstring_python && self.language == Language::Python;
        // Where a `/* … */` opened after code on an earlier line closes: nothing
        // before it opens a form (§FS-languages.multi-line-string.bound).
        let mut comment_until: Option<Pos> = None;
        for index in 0..self.lines.len() {
            let line = self.lines[index];
            // §FS-languages.multi-line-string: block and docstring state are read
            // as without this rule, so both advance on every line.
            let in_block = self.language.has_block_comments() && blocks.advance(line).inside;
            let excluded = python_docstrings
                .then(|| python_excluded(line, &mut docstrings))
                .flatten();
            let mut start = entries[index];
            if let Some((close_line, close_end)) = comment_until {
                if index < close_line {
                    continue;
                }
                start = start.max(close_end);
                comment_until = None;
            }
            if start == WHOLE_LINE || in_block {
                continue;
            }
            match self.open_on_line(index, start, excluded) {
                Some(Opened::Form((close_line, close_end), body_from)) => {
                    for entry in &mut entries[body_from..close_line] {
                        *entry = WHOLE_LINE;
                    }
                    entries[close_line] = close_end;
                }
                Some(Opened::Comment(close)) => comment_until = Some(close),
                None => {}
            }
        }
        entries
    }

    /// The first form that opens on line `index` and closes on a later line, read
    /// from column `start` — where the previous form closed — with the per-line
    /// toggle starting neutral there (§FS-languages.multi-line-string). Returns
    /// where it closes and the first line of its body, or where a `/* … */` that
    /// opens here closes on a later line. A form that closes on this line carries
    /// nothing, and the search resumes after its closer.
    fn open_on_line(
        &mut self,
        index: usize,
        start: usize,
        excluded: Option<(usize, usize)>,
    ) -> Option<Opened> {
        let line = self.lines[index].as_bytes();
        let tail = &line[start..];
        let (mut single, mut double, mut backtick) = (false, false, false);
        // The end of a one-line form already read: nothing inside it opens another.
        let mut inside_until = start;
        let mut i = start;
        while i < line.len() {
            let neutral = !single && !double && !backtick;
            let open_here =
                neutral && i >= inside_until && !excluded.is_some_and(|(s, e)| s <= i && i < e);
            if open_here && self.language.comment_at(line, i) {
                return line[i..]
                    .starts_with(b"/*")
                    .then(|| self.block_from(index, i))
                    .flatten();
            }
            if open_here {
                match self.language.skip_at(line, i) {
                    Some(Skip::To(end)) => {
                        i = end;
                        continue;
                    }
                    Some(Skip::Line) => return self.block_from(index, i),
                    None => {}
                }
            }
            if open_here && let Some((opener_end, closer)) = self.language.opener_at(line, i) {
                let from = if self.language.heredoc() {
                    (index + 1, 0)
                } else {
                    (index, opener_end)
                };
                match self.find(&closer, from) {
                    Some((close_line, close_end)) if close_line > index => {
                        return Some(Opened::Form((close_line, close_end), index + 1));
                    }
                    Some((_, close_end)) => inside_until = close_end,
                    // §FS-languages.multi-line-string.bound: no closer, so the
                    // opener opens nothing — and its own quote opens nothing else.
                    None => inside_until = opener_end,
                }
            }
            // The per-line toggle of §FS-fmt.2.3.1, exactly as
            // `is_inside_string_literal` reads it, from `start` — except that Rust
            // reads its own `'` above and never as a quote.
            let escaped = is_escaped(tail, i - start);
            match line[i] {
                b'\'' if !double && !backtick && !escaped && self.language.is_quote(b'\'') => {
                    single = !single;
                }
                b'"' if !single && !backtick && !escaped => double = !double,
                b'`' if !single && !double && !escaped => backtick = !backtick,
                _ => {}
            }
            i += 1;
        }
        None
    }

    /// The first `/* … */` that opens at or after column `from` of line `index` and
    /// closes on a later line: the search stops to its `*/`
    /// (§FS-languages.multi-line-string.bound).
    fn block_from(&self, index: usize, from: usize) -> Option<Opened> {
        let line = self.lines[index].as_bytes();
        let open = from + line[from..].windows(2).position(|pair| pair == b"/*")?;
        self.block_close((index, open + 2))
            .filter(|(close_line, _)| *close_line > index)
            .map(Opened::Comment)
    }

    /// Where the `*/` that closes a block opened just before `from` ends: the first
    /// one, or in Rust, whose blocks nest, the one that balances it.
    fn block_close(&self, from: Pos) -> Option<Pos> {
        let nests = self.language == Language::Rust;
        let mut depth = 0;
        for index in from.0..self.lines.len() {
            let line = self.lines[index].as_bytes();
            let mut i = if index == from.0 { from.1 } else { 0 };
            while i + 1 < line.len() {
                match &line[i..i + 2] {
                    b"*/" if depth == 0 => return Some((index, i + 2)),
                    b"*/" => depth -= 1,
                    b"/*" if nests => depth += 1,
                    _ => {
                        i += 1;
                        continue;
                    }
                }
                i += 2;
            }
        }
        None
    }

    /// Where `closer` ends, searching from `from` (§FS-languages.multi-line-string.bound).
    fn find(&mut self, closer: &Closer, from: Pos) -> Option<Pos> {
        if self
            .failed
            .iter()
            .any(|(failed, at)| failed == closer && *at <= from)
        {
            return None;
        }
        let found = match closer {
            Closer::Template => template_close(self.lines, from),
            Closer::Heredoc { word, strip_tabs } => self.line_close(from, |line| {
                let body = if *strip_tabs {
                    line.trim_start_matches('\t')
                } else {
                    line
                };
                (body == word).then_some(line.len())
            }),
            Closer::PhpHeredoc { word } => self.line_close(from, |line| {
                let trimmed = line.trim_start();
                let after = trimmed.strip_prefix(word.as_str())?;
                let bounded = !after.bytes().next().is_some_and(is_identifier_byte);
                bounded.then(|| line.len() - after.len())
            }),
            _ => self.span_close(closer, from),
        };
        // A template's search depends on its nesting, not only on where it starts.
        if found.is_none() && *closer != Closer::Template {
            self.failed.push((closer.clone(), from));
        }
        found
    }

    /// The first line at or after `from` that `closes` reads as the closer, with the
    /// end of the closer on it — the heredoc rows.
    fn line_close(&self, from: Pos, closes: impl Fn(&str) -> Option<usize>) -> Option<Pos> {
        (from.0..self.lines.len())
            .find_map(|index| closes(self.lines[index]).map(|end| (index, end)))
    }

    /// The closers that end at a byte run inside the text.
    fn span_close(&self, closer: &Closer, from: Pos) -> Option<Pos> {
        for index in from.0..self.lines.len() {
            let line = self.lines[index].as_bytes();
            let mut i = if index == from.0 { from.1 } else { 0 };
            while i < line.len() {
                let end = match closer {
                    Closer::Text { needle, escapable } => (line[i..].starts_with(needle)
                        && !(*escapable && is_escaped(line, i)))
                    .then_some(i + needle.len()),
                    Closer::QuoteRun(len) if line[i] == b'"' => {
                        let run = quote_run(line, i);
                        if run == *len {
                            Some(i + run)
                        } else {
                            i += run;
                            continue;
                        }
                    }
                    // §FS-languages.multi-line-string.forms: `""` is a quote inside.
                    Closer::Verbatim if line[i] == b'"' => {
                        if line.get(i + 1) == Some(&b'"') {
                            i += 2;
                            continue;
                        }
                        Some(i + 1)
                    }
                    _ => None,
                };
                if let Some(end) = end {
                    return Some((index, end));
                }
                i += 1;
            }
        }
        None
    }
}

/// One frame of a template literal's nesting (§FS-languages.multi-line-string.forms).
enum Frame {
    Template,
    /// An open `${`, with how many `{` it has opened since.
    Interpolation(usize),
}

/// Where the template literal whose content begins at `from` closes: at a `` ` ``
/// with no interpolation open, a `` ` `` inside an interpolation opening a nested
/// template. `None` when the nesting never balances.
fn template_close(lines: &[&str], from: Pos) -> Option<Pos> {
    let mut stack = vec![Frame::Template];
    for (index, line) in lines.iter().enumerate().skip(from.0) {
        let line = line.as_bytes();
        let mut i = if index == from.0 { from.1 } else { 0 };
        while i < line.len() {
            match (stack.last_mut()?, line[i]) {
                (Frame::Template, b'\\') => i += 1,
                (Frame::Template, b'`') => {
                    stack.pop();
                    if stack.is_empty() {
                        return Some((index, i + 1));
                    }
                }
                (Frame::Template, b'$') if line.get(i + 1) == Some(&b'{') => {
                    stack.push(Frame::Interpolation(0));
                    i += 1;
                }
                (Frame::Interpolation(depth), b'{') => *depth += 1,
                (Frame::Interpolation(0), b'}') => {
                    stack.pop();
                }
                (Frame::Interpolation(depth), b'}') => *depth -= 1,
                (Frame::Interpolation(_), b'`') => stack.push(Frame::Template),
                _ => {}
            }
            i += 1;
        }
    }
    None
}

/// The raw-line columns of a `.py` line that a docstring or an assigned-data span
/// claims, where no form opens (§FS-languages.multi-line-string.bound): through
/// the docstring's closer, or the whole line while it stays open.
fn python_excluded(line: &str, state: &mut PythonDocstringScanState) -> Option<(usize, usize)> {
    let scan = source_scan_line(line, true, true, state);
    if let Some(span) = scan.assigned_data_span {
        return Some(span);
    }
    if !scan.in_py_docstring {
        return None;
    }
    let content_end = scan.column_offset + scan.text.len();
    Some(if scan.closed_py_docstring {
        (0, content_end + 3)
    } else {
        (0, line.len())
    })
}
