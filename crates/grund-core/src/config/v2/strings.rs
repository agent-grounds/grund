//! How the version-2 reader reads a basic string (§FS-config-v2.reader.6): a scalar
//! value, a quoted key, or each element of a string list. A string ends at its first
//! `"` that is not part of a `\"` escape, where the one close every reader shares says
//! (§FS-config.3.11). Through grund 0.21.x a string that closes early is read as the
//! readers before it read it, by version 1's own functions, and the reader records the
//! first one in each value for its warning (§FS-config-v2.reader.6.window); every other
//! malformed string is refused as before, and a TOML literal string is refused naming its
//! form (§FS-config-v2.reader.6.literal). Apart from `walk.rs` so the line walk stays
//! one pass and the string rule is one file.

use anyhow::Result;
use std::path::Path;

use super::walk::Reader;
use crate::config::project::StringClosesEarly;
use crate::config::v1::{
    bail_config, list_elements, parse_string, parse_string_list, string_close,
};

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
    let read = basic_string(path, line, value)?;
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
    // A value that is not bracketed is refused as version 1 refuses it.
    let Some(elements) = list_elements(value) else {
        return Ok((parse_string_list(path, line, value)?, Vec::new()));
    };
    let read = elements
        .iter()
        .map(|element| basic_string(path, line, element))
        .collect::<Result<_>>()?;
    let early = elements
        .into_iter()
        .filter_map(|element| closes_early(value, element))
        .collect();
    Ok((read, early))
}

/// §FS-config-v2.reader.6, §FS-config-v2.reader.6.literal: `written`, a scalar value or one
/// element, read as one basic string, refused as before where it is not one, and naming
/// the form where it opens as a TOML literal string, which version 2 does not read.
fn basic_string(path: &Path, line: usize, written: &str) -> Result<String> {
    if let Some(form) = literal_form(written) {
        return bail_config(
            path,
            line,
            format!(
                "expected string ({form}, which version 2 does not read); write it as a basic \
                 string, \"...\", with \\\" for a quote inside it"
            ),
        );
    }
    parse_string(path, line, written)
}

/// §FS-config-v2.reader.6.literal: the TOML literal string form `written` opens with, if it
/// opens with `'`: `'''` opens a multi-line literal string, any other `'` a literal string.
fn literal_form(written: &str) -> Option<&'static str> {
    if written.starts_with("'''") {
        Some("a multi-line literal string")
    } else if written.starts_with('\'') {
        Some("a literal string")
    } else {
        None
    }
}

/// The lines a value was written across, as `joined_value` joined them: its key's line
/// and where each later one starts in the value (§FS-config-v2.reader.5).
#[derive(Default)]
pub(super) struct JoinedLines {
    line: usize,
    starts: Vec<usize>,
}

impl JoinedLines {
    pub(super) fn new(line: usize, starts: Vec<usize>) -> Self {
        Self { line, starts }
    }

    /// The line the byte at `offset` in the value was written on.
    fn line_at(&self, offset: usize) -> usize {
        self.line + self.starts.partition_point(|&start| start <= offset)
    }

    /// Whether `early` opens with a `"""` string a joined line break runs through, at any
    /// of the array's open lines: the one construct §FS-config-v2.reader.5.strings.multi-line
    /// already warns for, which earns that warning alone (§FS-config-v2.reader.6.multi-line).
    /// The string is the one its third quote opens, as the open line is read, so a break
    /// inside a plain string after it, as in `"""a""" "b` / `c"`, is not one.
    fn is_multi_line_over_break(&self, early: &EarlyClose) -> bool {
        let close = string_close(early.written.as_bytes(), 3);
        early.written.starts_with("\"\"\"")
            && close.is_some_and(|close| {
                self.line_at(early.start + 2) < self.line_at(early.start + close)
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
        if let Some(early) = early
            .iter()
            .find(|early| !lines.is_multi_line_over_break(early))
        {
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
