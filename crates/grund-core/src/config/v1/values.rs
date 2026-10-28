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
    // A `#` inside a quoted string is not a comment marker. Walk the line and stop at the
    // first unquoted `#`; otherwise return the line unchanged.
    let bytes = line.as_bytes();
    let mut in_string = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' if !is_escaped(bytes, i) => in_string = !in_string,
            b'#' if !in_string => return &line[..i],
            _ => {}
        }
        i += 1;
    }
    line
}

/// Cut `inner`, the text between a string list's brackets or an inline table's
/// braces, at each comma outside a basic string, and only there (§FS-config.3.11).
/// A string opens and closes at a `"` that is not part of a `\"` escape, the walk
/// `strip_comment` reads a line by, so a comma inside a string stays in its piece.
/// Like `str::split`, an empty `inner` is one empty piece.
pub(in crate::config) fn split_outside_strings(inner: &str) -> Vec<&str> {
    let bytes = inner.as_bytes();
    let mut pieces = Vec::new();
    let mut in_string = false;
    let mut start = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        match byte {
            b'"' if !is_escaped(bytes, i) => in_string = !in_string,
            b',' if !in_string => {
                pieces.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    pieces.push(&inner[start..]);
    pieces
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
/// string (§FS-config.3.11); each element is still one basic string, refused at
/// `line` as before when it is not (§FS-config.4.3).
pub(crate) fn parse_string_list(path: &Path, line: usize, value: &str) -> Result<Vec<String>> {
    if !value.starts_with('[') || !value.ends_with(']') {
        return bail_config(path, line, "expected string list".to_string());
    }
    let inner = value[1..value.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    split_outside_strings(inner)
        .into_iter()
        .map(|part| parse_string(path, line, part.trim()))
        .collect()
}
