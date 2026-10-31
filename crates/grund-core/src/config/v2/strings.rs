//! How the version-2 reader reads a basic string (§FS-config-v2.reader.6): a scalar
//! value, a quoted key, or each element of a string list. A string ends at its first
//! `"` that is not part of a `\"` escape, where the one close every reader shares says
//! (§FS-config.3.11). From grund 0.22.0 a string that closes early is refused, with the
//! release that made it one where the readers before it read it as strings
//! (§FS-config-v2.reader.6.refusal); every other malformed string is refused as before,
//! and a TOML literal string is refused naming its form (§FS-config-v2.reader.6.literal).
//! Apart from `walk.rs` so the line walk stays one pass and the string rule is one file.

use anyhow::Result;
use std::path::Path;

use super::walk::Reader;
use crate::config::v1::{
    bail_config, list_elements, parse_string, parse_string_list, string_close,
};

/// The clause a refusal ends with where the text it refuses read through one of the 0.21.x
/// windows: a string that closes early (§FS-config-v2.reader.6.refusal), a comma with no
/// element before it (§FS-config-v2.reader.5.commas.refusal) and an array's open line
/// (§FS-config-v2.reader.5.strings.refusal).
pub(super) const WINDOW_CLOSED: &str = "; this became an error in grund 0.22.0";

/// A string the readers before grund 0.22.0 read past its closing quote, the 0.21.x
/// window's (§FS-config-v2.reader.6.window), which version 2 now refuses
/// (§FS-config-v2.reader.6.refusal) and migrate refuses to carry
/// (§FS-config-migrate.3.1.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EarlyClose<'a> {
    /// The string as written, from its opening quote to the last quote of its value or
    /// its element, both kept.
    pub(crate) written: &'a str,
}

impl EarlyClose<'_> {
    /// The text between the string's opening quote and its last, as written, which
    /// migrate's refusal names (§FS-config-migrate.3.1.8).
    pub(crate) fn text(&self) -> &str {
        &self.written[1..self.written.len() - 1]
    }

    /// §FS-config-v2.reader.6.refusal, §FS-config-v2.reader.6.multi-line: refuse the value
    /// at `line` this string stands in, naming a `"""` string's form.
    fn refuse<T>(&self, path: &Path, line: usize) -> Result<T> {
        let form = match self.written.starts_with("\"\"\"") {
            true => " (a multi-line string, which version 2 does not read)",
            false => "",
        };
        bail_config(path, line, format!("expected string{form}{WINDOW_CLOSED}"))
    }
}

/// `written` read as one string the way the readers before 0.22.0 read it, when it closes
/// before its last quote (§FS-config-v2.reader.6).
fn closes_early(written: &str) -> Option<EarlyClose<'_>> {
    if written.len() < 2 || !written.starts_with('"') || !written.ends_with('"') {
        return None;
    }
    let close = string_close(written.as_bytes(), 1)?;
    (close + 1 < written.len()).then_some(EarlyClose { written })
}

/// A version-2 scalar value or quoted key read as the readers before grund 0.22.0 read it,
/// refused as they refused it, with where it closes early: the classifier that tells a
/// value the 0.21.x window loaded from one refused before (§FS-config-v2.reader.6.refusal),
/// and what migrate refuses (§FS-config-migrate.3.1.8).
pub(crate) fn read_string_as_before<'a>(
    path: &Path,
    line: usize,
    value: &'a str,
) -> Result<(String, Option<EarlyClose<'a>>)> {
    let read = basic_string(path, line, value)?;
    Ok((read, closes_early(value)))
}

/// A version-2 string list read as the readers before grund 0.22.0 read it, refused as
/// they refused it, with every element that closes early, in order: the list's half of
/// [`read_string_as_before`] (§FS-config-v2.reader.6.refusal).
pub(crate) fn read_string_list_as_before<'a>(
    path: &Path,
    line: usize,
    value: &'a str,
) -> Result<(Vec<String>, Vec<EarlyClose<'a>>)> {
    read_elements(path, line, value, false)
}

/// [`read_string_list`], with the empty element's `expected string` naming grund 0.22.0
/// where `through_window` says the array's text read through the 0.21.x trailing-comma
/// window (§FS-config-v2.reader.5.commas.refusal). Only an empty element's refusal takes
/// it: an element refused before the empty one is reached was refused in 0.21.x too.
fn read_elements<'a>(
    path: &Path,
    line: usize,
    value: &'a str,
    through_window: bool,
) -> Result<(Vec<String>, Vec<EarlyClose<'a>>)> {
    // A value that is not bracketed is refused as version 1 refuses it.
    let Some(elements) = list_elements(value) else {
        return Ok((parse_string_list(path, line, value)?, Vec::new()));
    };
    let read = elements
        .iter()
        .map(|element| {
            if element.is_empty() && through_window {
                bail_config(path, line, format!("expected string{WINDOW_CLOSED}"))
            } else {
                basic_string(path, line, element)
            }
        })
        .collect::<Result<_>>()?;
    let early = elements.into_iter().filter_map(closes_early).collect();
    Ok((read, early))
}

/// §FS-config-v2.reader.6.refusal: a version-2 scalar value or quoted key at `line`, read
/// as one basic string; one that closes early is refused with the release that made it
/// one. A reader that records no finding reads through this too, and passes over a value
/// it refuses.
pub(crate) fn read_string(path: &Path, line: usize, value: &str) -> Result<String> {
    match read_string_as_before(path, line, value)? {
        (_, Some(early)) => early.refuse(path, line),
        (read, None) => Ok(read),
    }
}

/// §FS-config-v2.reader.6.refusal: a version-2 string list at `line`, refused once at `line`
/// for the first element that closes early, and its empty element naming grund 0.22.0 where
/// `through_window` says its text read through the trailing-comma window
/// (§FS-config-v2.reader.5.commas.refusal). No string in it runs over a line break: an array
/// with an open line is refused before its elements are read
/// (§FS-config-v2.reader.5.strings.refusal).
pub(crate) fn read_string_list(
    path: &Path,
    line: usize,
    value: &str,
    through_window: bool,
) -> Result<Vec<String>> {
    let (read, early) = read_elements(path, line, value, through_window)?;
    match early.first() {
        Some(early) => early.refuse(path, line),
        None => Ok(read),
    }
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

impl Reader<'_> {
    /// §FS-config-v2.reader.6.refusal: a scalar value or a quoted key at `line`, read as
    /// one basic string.
    pub(super) fn string(&mut self, line: usize, value: &str) -> Result<String> {
        read_string(self.path, line, value)
    }

    /// §FS-config-v2.reader.6.refusal: a string list at `line`, its empty element naming
    /// grund 0.22.0 where the array read through the trailing-comma window.
    pub(super) fn string_list(&mut self, line: usize, value: &str) -> Result<Vec<String>> {
        read_string_list(self.path, line, value, self.through_window)
    }
}
