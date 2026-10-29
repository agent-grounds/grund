//! How the version-2 reader reads a basic string (§FS-config-v2.reader.6): a scalar
//! value, a quoted key, or each element of a string list. A string ends at its first
//! `"` that is not part of a `\"` escape, where the one close every reader shares says
//! (§FS-config.3.11). Through grund 0.21.x a string that closes early is read as the
//! readers before it read it, by version 1's own functions, and the reader records the
//! first one in each value for its warning (§FS-config-v2.reader.6.window); every other
//! malformed string is refused as before. Apart from `walk.rs` so the line walk stays
//! one pass and the string rule is one file.

use anyhow::Result;
use std::path::Path;

use super::walk::{OpenLine, Reader};
use crate::config::project::StringClosesEarly;
use crate::config::v1::{list_elements, parse_string, parse_string_list, string_close};

/// A string the version-2 reader reads past its closing quote
/// (§FS-config-v2.reader.6.window).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EarlyClose<'a> {
    /// The string as written, from its opening quote to the last quote of its value or
    /// its element, both kept.
    pub(crate) written: &'a str,
    /// Where `written` starts in the value, as a byte offset.
    start: usize,
    /// Where in the value the quote it closes early at stands.
    close: usize,
}

impl EarlyClose<'_> {
    /// The text the warning names, the string's from its opening quote to its last, as
    /// written (§FS-config-v2.reader.6.window).
    pub(crate) fn text(&self) -> &str {
        &self.written[1..self.written.len() - 1]
    }
}

/// `written`, a slice of `value` read as one string the way the readers before 0.21.0
/// read it, when it closes before its last quote (§FS-config-v2.reader.6).
fn closes_early<'a>(value: &'a str, written: &'a str) -> Option<EarlyClose<'a>> {
    if written.len() < 2 || !written.starts_with('"') || !written.ends_with('"') {
        return None;
    }
    let close = string_close(written.as_bytes(), 1)?;
    // A slice of `value`, so its offset is the distance between the two.
    let start = written.as_ptr() as usize - value.as_ptr() as usize;
    (close + 1 < written.len()).then_some(EarlyClose {
        written,
        start,
        close: start + close,
    })
}

/// §FS-config-v2.reader.6: a version-2 scalar value or quoted key, read as one basic
/// string, with where it closes early. Through grund 0.21.x it is read as the readers
/// before it read it (§FS-config-v2.reader.6.window), and refused as they refused it.
/// A reader that records no finding reads through this and drops the place
/// (§FS-config-v2.reader.6.refusal).
pub(crate) fn read_string<'a>(
    path: &Path,
    line: usize,
    value: &'a str,
) -> Result<(String, Option<EarlyClose<'a>>)> {
    let read = parse_string(path, line, value)?;
    Ok((read, closes_early(value, value)))
}

/// §FS-config-v2.reader.6: a version-2 string list, read as the readers before it read
/// it through grund 0.21.x, with every element that closes early, in order
/// (§FS-config-v2.reader.6.window).
pub(crate) fn read_string_list<'a>(
    path: &Path,
    line: usize,
    value: &'a str,
) -> Result<(Vec<String>, Vec<EarlyClose<'a>>)> {
    let read = parse_string_list(path, line, value)?;
    let early = list_elements(value)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|element| closes_early(value, element))
        .collect();
    Ok((read, early))
}

/// The lines a value was written across, as `joined_value` joined them: its key's line,
/// where each later one starts in the value, and its open line (§FS-config-v2.reader.5).
#[derive(Default)]
pub(super) struct JoinedLines {
    line: usize,
    starts: Vec<usize>,
    open: Option<OpenLine>,
}

impl JoinedLines {
    pub(super) fn new(line: usize, starts: Vec<usize>, open: Option<OpenLine>) -> Self {
        Self { line, starts, open }
    }

    /// The line the byte at `offset` in the value was written on.
    fn line_at(&self, offset: usize) -> usize {
        self.line + self.starts.partition_point(|&start| start <= offset)
    }

    /// Whether `early` is the `"""` string the value's open line leaves open, the one
    /// construct §FS-config-v2.reader.5.strings.multi-line already warns for, which earns
    /// that warning alone (§FS-config-v2.reader.6.multi-line).
    fn is_open_multi_line(&self, early: &EarlyClose) -> bool {
        let end = early.start + early.written.len() - 1;
        self.open.is_some_and(|open| {
            open.multi_line
                && self.line_at(early.start) <= open.line
                && open.line < self.line_at(end)
        })
    }
}

impl Reader<'_> {
    /// §FS-config-v2.reader.6: a scalar value or a quoted key at `line`, read as one basic
    /// string; one that closes early is read as before and recorded for its warning.
    pub(super) fn string(&mut self, line: usize, value: &str) -> Result<String> {
        let (read, early) = read_string(self.path, line, value)?;
        if let Some(early) = early {
            self.record_early_close(line, &early, line, false);
        }
        Ok(read)
    }

    /// §FS-config-v2.reader.6: a string list at `line`, read as before; the value earns
    /// one warning, for the first of its elements that closes early, naming the line that
    /// element closes on (§FS-config-v2.reader.6.window).
    pub(super) fn string_list(&mut self, line: usize, value: &str) -> Result<Vec<String>> {
        let (read, early) = read_string_list(self.path, line, value)?;
        let lines = &self.value;
        if let Some(early) = early.iter().find(|early| !lines.is_open_multi_line(early)) {
            let close = lines.line_at(early.close);
            self.record_early_close(line, early, close, true);
        }
        Ok(read)
    }

    /// §FS-config-v2.checks.config-not-toml, §FS-config-v2.checks.config-multiline-string:
    /// record `early`, read in the value of the key at `line`, closing on line `close`.
    fn record_early_close(&mut self, line: usize, early: &EarlyClose, close: usize, element: bool) {
        let Some(at) = self.at(line) else {
            return;
        };
        self.project.strings_closing_early.push(StringClosesEarly {
            at,
            text: early.text().to_string(),
            line: close,
            element,
            // §FS-config-v2.reader.6.multi-line: TOML's multi-line string, on one line.
            multi_line: early.written.starts_with("\"\"\""),
        });
    }
}
