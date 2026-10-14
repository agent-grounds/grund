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

pub(crate) fn parse_string_list(path: &Path, line: usize, value: &str) -> Result<Vec<String>> {
    if !value.starts_with('[') || !value.ends_with(']') {
        return bail_config(path, line, "expected string list".to_string());
    }
    let inner = value[1..value.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|part| parse_string(path, line, part.trim()))
        .collect()
}
