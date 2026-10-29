//! The scalar value parsers every section of a version-1 `grund.toml` shares
//! (§FS-config.3), and the comment stripper the line pass reads each line
//! through: a string, a boolean, a count and a one-line string list, each refused
//! at its line in the located shape of §FS-config.4.3. Apart from `parse.rs` so
//! the section walk stays one state machine and the values it reads are one
//! grammar, which the v2 reader's tables read too (§AR-config.2).

use anyhow::Result;
use std::path::Path;

use crate::grammar::is_escaped;
use crate::model::format_path;

/// Drop a trailing `#`-comment from a `grund.toml` line (§FS-config.3).
pub(crate) fn strip_comment(line: &str) -> &str {
    // A `#` inside a quoted string is not a comment marker. Walk the line, passing over each
    // string to where it closes, and stop at the first `#` outside one.
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' if !is_escaped(bytes, i) => match string_close(bytes, i + 1) {
                Some(close) => i = close,
                None => return line,
            },
            b'#' => return &line[..i],
            _ => {}
        }
        i += 1;
    }
    line
}

/// Where a basic string closes (§FS-config.3.11): at the first `"` at or after `from` that is
/// not part of a `\"` escape, or `None` when `bytes` ends inside the string. `from` is the byte
/// after its opening quote, or `0` for a string a line before this one opened. Every reader that
/// asks where a `grund.toml` string ends asks this, so no two can disagree
/// (§AR-system.principle).
pub(in crate::config) fn string_close(bytes: &[u8], from: usize) -> Option<usize> {
    (from..bytes.len()).find(|&i| bytes[i] == b'"' && !is_escaped(bytes, i))
}

/// Cut `inner`, the text between a string list's brackets or an inline table's
/// braces, at each comma outside a basic string, and only there (§FS-config.3.11).
/// A string opens at a `"` that is not part of a `\"` escape and closes where
/// [`string_close`] says, the walk `strip_comment` reads a line by, so a comma inside a
/// string stays in its piece. Like `str::split`, an empty `inner` is one empty piece.
pub(in crate::config) fn split_outside_strings(inner: &str) -> Vec<&str> {
    let bytes = inner.as_bytes();
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' if !is_escaped(bytes, i) => match string_close(bytes, i + 1) {
                Some(close) => i = close,
                None => break,
            },
            b',' => {
                pieces.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    pieces.push(&inner[start..]);
    pieces
}

/// Whether `piece`, already trimmed, is one basic string: it opens with `"` and closes, where
/// [`string_close`] says, at its last byte (§FS-config.3.11). Only such a piece keeps the commas
/// `split_outside_strings` left inside it; any other piece is cut at each of its commas, so a
/// malformed list is read exactly as it was before that rule. Version 1 keeps that reading for
/// good (§FS-config.5.2), a piece whose string closes early included, and meets the rule only
/// when it is migrated (§FS-config-migrate.3.1.8); version 2 reads the same pieces, and refuses
/// a value where one closes early (§FS-config-v2.reader.6.refusal).
fn is_one_basic_string(piece: &str) -> bool {
    piece.starts_with('"') && string_close(piece.as_bytes(), 1) == Some(piece.len() - 1)
}

/// Whether `line`, its comment stripped and trimmed, is a table header,
/// `[name]` or `[[name]]`: the one line shape the reader reads that is not
/// `key = value`.
pub(crate) fn is_header(line: &str) -> bool {
    line.starts_with('[') && line.ends_with(']')
}

/// Fail config parsing with a `path:line: message` error — the located-finding
/// shape applied to a malformed `grund.toml` (§FS-config.4.3, §FS-errors.2.1).
pub(in crate::config) fn bail_config<T>(path: &Path, line: usize, message: String) -> Result<T> {
    // §FS-distribution.3.3.2: preserve the parser's location without parsing Display.
    let mut failure = crate::model::OperationDiagnostic::new(
        "config",
        "config",
        format!("{}:{}: {}", format_path(path), line, message),
    );
    failure.path = Some(format_path(path));
    failure.line = Some(line);
    Err(failure.into())
}

pub(in crate::config) fn parse_string(path: &Path, line: usize, value: &str) -> Result<String> {
    if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2) {
        return bail_config(path, line, "expected string".to_string());
    }
    let inner = &value[1..value.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some(other) => {
                return bail_config(
                    path,
                    line,
                    format!("invalid escape sequence `\\{other}` in string"),
                );
            }
            None => {
                return bail_config(path, line, "trailing backslash in string".to_string());
            }
        }
    }
    Ok(out)
}

pub(in crate::config) fn parse_bool(path: &Path, line: usize, value: &str) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail_config(path, line, "expected boolean".to_string()),
    }
}

pub(in crate::config) fn parse_usize(path: &Path, line: usize, value: &str) -> Result<usize> {
    value
        .parse::<usize>()
        .or_else(|_| bail_config(path, line, "expected non-negative integer".to_string()))
}

/// Read a `["…", "…"]` string list, its elements ending only at a comma outside a
/// string (§FS-config.3.11); a piece that is not one basic string is cut at each of its
/// commas as before, and each element is refused at `line` when it is not one (§FS-config.4.3).
pub(crate) fn parse_string_list(path: &Path, line: usize, value: &str) -> Result<Vec<String>> {
    let Some(elements) = list_elements(value) else {
        return bail_config(path, line, "expected string list".to_string());
    };
    elements
        .into_iter()
        .map(|element| parse_string(path, line, element))
        .collect()
}

/// The elements of a `["…", "…"]` string list as written, each still quoted and trimmed, or
/// `None` when `value` is not bracketed: its text cut only at a comma outside a string, and a
/// piece that is not one basic string cut at each of its commas, as [`parse_string_list`] reads
/// them (§FS-config.3.11). Each is a slice of `value`, so version 2 can tell a `"""` string
/// that a joined line break runs through (§FS-config-v2.reader.6.multi-line).
pub(in crate::config) fn list_elements(value: &str) -> Option<Vec<&str>> {
    let inner = value.strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let mut elements = Vec::new();
    for piece in split_outside_strings(inner) {
        let piece = piece.trim();
        if is_one_basic_string(piece) {
            elements.push(piece);
        } else {
            elements.extend(piece.split(',').map(str::trim));
        }
    }
    Some(elements)
}
