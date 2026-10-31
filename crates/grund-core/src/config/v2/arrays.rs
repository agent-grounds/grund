//! An array written across lines (§FS-config-v2.reader.5): the lines it spans, joined
//! into the one value the walk reads at its key's line, with its one trailing comma
//! taken off (§FS-config-v2.reader.5.commas). A line that ends inside a string is the
//! array's last, and the array is refused there (§FS-config-v2.reader.5.strings.refusal);
//! the quote-at-a-time reading of the readers before 0.22.0 is kept only to tell whether
//! their window loaded it. The ancestor climb reads `[workspace] members` through this
//! too (§FS-workspace.6.1.7.5). Apart from `walk.rs` so the line walk stays one pass and
//! how an array's lines are read is one file.

use crate::config::v1::{list_elements, string_close, strip_comment};
use crate::grammar::is_escaped;

/// The value a `key = value` line starts, as [`joined_value`] reads it.
pub(crate) struct Joined {
    /// The value, or `None` when the array is refused as `unclosed array`: still open at the
    /// end of the text, or at its open line (§FS-config-v2.reader.5.strings.refusal).
    pub(crate) value: Option<String>,
    /// The array's open line, the first that ends inside a string and its last
    /// (§FS-config-v2.reader.5.strings); `None` when every string closes on its line.
    pub(crate) open: Option<OpenLine>,
    /// For an array that loads, whether its text read through the 0.21.x trailing-comma
    /// window, so the refusal of its empty element names grund 0.22.0: whether taking off
    /// the whole run of commas before its `]` would leave no empty element where the one
    /// trailing comma leaves one (§FS-config-v2.reader.5.commas.refusal).
    pub(crate) through_window: bool,
    /// For an array refused at its open line, its text as the readers before 0.22.0 read it,
    /// where their join closed it: its lines joined a quote at a time, each break read as one
    /// space, and the whole run of commas before its `]` taken off. Whether its elements then
    /// read as they read them tells whether their window loaded the array, and so whether its
    /// refusal names grund 0.22.0 (§FS-config-v2.reader.5.strings.refusal).
    pub(crate) as_before: Option<String>,
}

/// A line of an array that ends inside a string (§FS-config-v2.reader.5.strings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OpenLine {
    /// The line, 1-based.
    pub(crate) line: usize,
    /// Whether the string open at its end is a multi-line one, opened by three quotes in a
    /// row, which version 2 does not read (§FS-config-v2.reader.5.strings.multi-line).
    pub(crate) multi_line: bool,
}

impl OpenLine {
    /// The parenthetical `unclosed array` appends for this open line
    /// (§FS-config-v2.reader.5, §FS-config-v2.reader.5.strings.multi-line).
    pub(crate) fn parenthetical(self) -> String {
        let line = self.line;
        if self.multi_line {
            format!(" (a multi-line string opens on line {line}, which version 2 does not read)")
        } else {
            format!(" (a string is still open at the end of line {line})")
        }
    }
}

/// The value a `key = value` line starts, read as the reader reads it
/// (§FS-config-v2.reader.5): an array written across lines takes the lines it
/// spans from `lines`, joined into one, and loses the one trailing comma after its last
/// element (§FS-config-v2.reader.5.commas). `line` is the key's line, 1-based, and `lines`
/// yields each line after it with its 0-based index, as `str::lines().enumerate()` does.
/// The ancestor climb reads `[workspace] members` through this too, so a value the
/// config loads is never one the climb cannot read (§FS-workspace.6.1.7.5).
///
/// A line that ends inside a string is the array's open line and its last, so the array
/// is refused there, and `lines` is left after it (§FS-config-v2.reader.5.strings.refusal).
/// Only then are the lines read again from the key's, as the readers before 0.22.0 joined
/// them, for the text whose elements tell whether their window loaded the array.
pub(crate) fn joined_value<'a, I>(value: &str, line: usize, lines: &mut I) -> Joined
where
    I: Iterator<Item = (usize, &'a str)> + Clone,
{
    let value = value.trim();
    if !value.starts_with('[') {
        return Joined {
            value: Some(value.to_string()),
            open: None,
            through_window: false,
            as_before: None,
        };
    }
    let mut before = lines.clone();
    match join(value, line, lines, Quotes::Toml) {
        Join::Closed(value) => {
            let (value, through_window) = array_value(&value);
            Joined {
                value: Some(value),
                open: None,
                through_window,
                as_before: None,
            }
        }
        Join::Open(open) => {
            // §FS-config-v2.reader.5.strings.refusal: the text 0.21.x read, whether or not
            // their reading met an open line in it.
            let read_before = open.map(|_| join(value, line, &mut before, Quotes::OneAtATime));
            let as_before = match read_before {
                Some(Join::Closed(value)) => Some(commas_off(&value)),
                _ => None,
            };
            Joined {
                value: None,
                open,
                through_window: false,
                as_before,
            }
        }
    }
}

/// How [`join`] ended.
enum Join {
    /// The array's `[` closed: its lines joined with one space.
    Closed(String),
    /// The array is still open, at its open line or, with `None`, at the end of the text.
    Open(Option<OpenLine>),
}

/// Join the lines of the array whose key's line, `line`, holds `value`, each with its `#`
/// comment taken off on its own (§FS-config-v2.reader.5), until its `[` closes outside a
/// string. Read as TOML reads quotes, the open line is the array's last
/// (§FS-config-v2.reader.5.strings.refusal); read a quote at a time, as the 0.18.0 to 0.21.x
/// readers read them, a string open at a line's end runs on into the next, the break read
/// as one space (§FS-config-v2.reader.5.strings.window).
fn join<'a>(
    value: &str,
    line: usize,
    lines: &mut impl Iterator<Item = (usize, &'a str)>,
    quotes: Quotes,
) -> Join {
    let mut value = value.to_string();
    let mut brackets = Brackets::new(quotes);
    brackets.read(&value);
    let mut open = brackets.open_line(line);
    loop {
        if brackets.depth <= 0 {
            return Join::Closed(value);
        }
        if open.is_some() && quotes == Quotes::Toml {
            return Join::Open(open);
        }
        let Some((idx, next)) = lines.next() else {
            return Join::Open(open);
        };
        let next = strip_comment(next).trim();
        value.push(' ');
        value.push_str(next);
        brackets.read(next);
        open = open.or(brackets.open_line(idx + 1));
    }
}

/// How [`Brackets`] reads a run of quotes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Quotes {
    /// As TOML reads them, the reading version 2 reads by: three in a row that open a
    /// string open a multi-line string, which closes only at the next run of three or
    /// more, the run taken whole (§FS-config-v2.reader.5.strings).
    Toml,
    /// One at a time, each opening or closing a basic string, as the readers before 0.22.0
    /// read them, which only tells whether they loaded an array that version 2 refuses
    /// (§FS-config-v2.reader.5.strings.refusal).
    OneAtATime,
}

/// The `[` an array's text still holds open outside its strings, and the string
/// open at the end of the text read so far. A `"` outside a string opens one unless
/// it is part of a `\"` escape, and a basic string closes where `string_close` says, the
/// one close `strip_comment`, `split_outside_strings` and `is_one_basic_string`
/// read strings by (§FS-config.3.11).
struct Brackets {
    depth: i32,
    /// `Some(multi_line)` while inside a string: whether it is a multi-line one.
    string: Option<bool>,
    quotes: Quotes,
}

impl Brackets {
    fn new(quotes: Quotes) -> Self {
        Self {
            depth: 0,
            string: None,
            quotes,
        }
    }

    /// Read the next piece of the array's text. The pieces are joined with a space,
    /// so no `\"` escape reaches across two of them.
    fn read(&mut self, piece: &str) {
        let bytes = piece.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if let Some(multi_line) = self.string {
                let close = match multi_line {
                    true => multi_line_close(bytes, i),
                    false => string_close(bytes, i),
                };
                let Some(close) = close else {
                    return;
                };
                self.string = None;
                i = close + 1;
                continue;
            }
            match bytes[i] {
                // §FS-config-v2.reader.5.strings: three quotes in a row open one string.
                b'"' if !is_escaped(bytes, i) => {
                    let run = self.quotes == Quotes::Toml && bytes[i..].starts_with(b"\"\"\"");
                    self.string = Some(run);
                    if run {
                        i += 2;
                    }
                }
                b'[' => self.depth += 1,
                b']' => self.depth -= 1,
                _ => {}
            }
            i += 1;
        }
    }

    /// `line`, when the text read so far ends inside a string
    /// (§FS-config-v2.reader.5.strings).
    fn open_line(&self, line: usize) -> Option<OpenLine> {
        self.string.map(|multi_line| OpenLine { line, multi_line })
    }
}

/// Where a multi-line string closes in `bytes`, from `from`: at the last quote of the next
/// run of three or more, the run taken whole and its first quote the one `string_close`
/// finds, so none of them is part of a `\"` escape (§FS-config-v2.reader.5.strings).
fn multi_line_close(bytes: &[u8], mut from: usize) -> Option<usize> {
    loop {
        let quote = string_close(bytes, from)?;
        let run = bytes[quote..].iter().take_while(|&&b| b == b'"').count();
        if run >= 3 {
            return Some(quote + run - 1);
        }
        from = quote + run;
    }
}

/// An array value with its one trailing comma taken off, the one TOML allows: a comma
/// before the `]` whose text before it, whitespace skipped, is neither `[` nor `,`
/// (§FS-config-v2.reader.5.commas). Any other comma before the `]` has no element before
/// it and stays for the element reader to refuse (§FS-config-v2.reader.5.commas.refusal),
/// which is told whether the array's text read through the 0.21.x window: whether taking
/// off the whole run of adjacent commas, as the 0.18.0 to 0.21.x readers did, would have
/// left no empty element behind.
fn array_value(value: &str) -> (String, bool) {
    let Some(inner) = value.strip_suffix(']').filter(|_| value.starts_with('[')) else {
        return (value.to_string(), false);
    };
    let Some(before) = inner.trim_end().strip_suffix(',') else {
        return (value.to_string(), false);
    };
    if !before.trim_end().ends_with(['[', ',']) {
        return (format!("{before}]"), false);
    }
    let through_window = list_elements(&commas_off(value))
        .is_some_and(|elements| elements.iter().all(|element| !element.is_empty()));
    (value.to_string(), through_window)
}

/// An array's text with the whole run of adjacent commas before its `]` taken off, as the
/// 0.18.0 to 0.21.x readers took it off (§FS-config-v2.reader.5.commas.window).
fn commas_off(value: &str) -> String {
    match value.strip_suffix(']') {
        Some(inner) if value.starts_with('[') && inner.trim_end().ends_with(',') => {
            format!("{}]", inner.trim_end().trim_end_matches(','))
        }
        _ => value.to_string(),
    }
}
