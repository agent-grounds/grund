//! The per-line readers that ask the compiled grammar a question about one
//! source line (§AR-scanner.4): the comment-prefix alternation a declaration or
//! section heading may sit behind, and the normalization that makes a Python
//! docstring line scannable while keeping the original file's column offset.
//!
//! Separate from `compiled.rs` because building the grammar and walking a line
//! with it are two jobs (§AR-core-module-layout.3) — this file is the second,
//! and holds only the bounded line-local Python distinction of
//! §FS-check.1.1.3.1 beside the compiled grammar.

use std::borrow::Cow;

/// Build the alternation a declaration/section heading may be prefixed by — one
/// entry per `[scan] comment_prefixes` value (§FS-config.3.5), with `//` widened to
/// also catch Rust/JS doc-comment forms `///` and `//!` so inline declarations in
/// code are seen (§AR-scanner.4.3). Longest-first so `//` does not shadow `///`.
pub(super) fn comment_prefix_regex(comment_prefixes: &[String]) -> String {
    let mut prefixes = comment_prefixes
        .iter()
        .filter(|prefix| !prefix.is_empty())
        .map(|prefix| {
            if prefix == "//" {
                r"//[/!]?".to_string()
            } else {
                regex::escape(prefix)
            }
        })
        .collect::<Vec<_>>();
    prefixes.sort_by_key(|prefix| std::cmp::Reverse(prefix.len()));
    if prefixes.is_empty() {
        "(?!)".to_string()
    } else {
        format!("(?:{})", prefixes.join("|"))
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PythonDocstringScanState {
    pub(super) quote: Option<&'static str>,
    pub(super) assigned_data: bool,
}

pub(crate) struct SourceScanLine<'a> {
    pub(crate) text: Cow<'a, str>,
    pub(crate) in_py_docstring: bool,
    pub(crate) column_offset: usize,
    pub(crate) closed_py_docstring: bool,
    pub(crate) assigned_data_span: Option<(usize, usize)>,
}

/// Normalize one source line for scanner-style declaration/section/citation
/// detection while preserving the original-file column offset for emitted
/// ranges (§AR-scanner.4.4). A qualifying assigned Python triple-quoted data
/// span is blanked without changing its byte length, so every consumer skips
/// the data while source on either side keeps its raw position
/// (§FS-check.1.1.3.1).
pub(crate) fn source_scan_line<'a>(
    line: &'a str,
    is_py: bool,
    docstring_python: bool,
    py_docstring: &mut PythonDocstringScanState,
) -> SourceScanLine<'a> {
    if !docstring_python || !is_py {
        return SourceScanLine {
            text: Cow::Borrowed(line),
            in_py_docstring: false,
            column_offset: 0,
            closed_py_docstring: false,
            assigned_data_span: None,
        };
    }

    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    if let Some(quote) = py_docstring.quote {
        if py_docstring.assigned_data {
            let close_end =
                matching_unescaped_quote(line, 0, quote).map(|close| close + quote.len());
            let end = close_end.unwrap_or(line.len());
            if close_end.is_some() {
                py_docstring.quote = None;
                py_docstring.assigned_data = false;
            }
            return assigned_data_line(line, 0, end);
        }
        if let Some(close) = trimmed.find(quote) {
            py_docstring.quote = None;
            return SourceScanLine {
                text: Cow::Borrowed(&trimmed[..close]),
                in_py_docstring: true,
                column_offset: indent,
                closed_py_docstring: true,
                assigned_data_span: None,
            };
        }
        return SourceScanLine {
            text: Cow::Borrowed(trimmed),
            in_py_docstring: true,
            column_offset: indent,
            closed_py_docstring: false,
            assigned_data_span: None,
        };
    }

    if let Some((literal_start, quote_start, quote)) = python_assigned_data_quote(line) {
        let content_start = quote_start + quote.len();
        if let Some(close) = matching_unescaped_quote(line, content_start, quote) {
            return assigned_data_line(line, literal_start, close + quote.len());
        }
        py_docstring.quote = Some(quote);
        py_docstring.assigned_data = true;
        return assigned_data_line(line, literal_start, line.len());
    }

    let Some(quote) = python_docstring_quote(line) else {
        return SourceScanLine {
            text: Cow::Borrowed(line),
            in_py_docstring: false,
            column_offset: 0,
            closed_py_docstring: false,
            assigned_data_span: None,
        };
    };
    let after_open = &trimmed[quote.len()..];
    if let Some(close) = after_open.find(quote) {
        return SourceScanLine {
            text: Cow::Borrowed(&after_open[..close]),
            in_py_docstring: true,
            column_offset: indent + quote.len(),
            closed_py_docstring: true,
            assigned_data_span: None,
        };
    }
    py_docstring.quote = Some(quote);
    SourceScanLine {
        text: Cow::Borrowed(after_open),
        in_py_docstring: true,
        column_offset: indent + quote.len(),
        closed_py_docstring: false,
        assigned_data_span: None,
    }
}

fn assigned_data_line(line: &str, start: usize, end: usize) -> SourceScanLine<'_> {
    let mut masked = line.as_bytes().to_vec();
    masked[start..end].fill(b' ');
    SourceScanLine {
        text: Cow::Owned(String::from_utf8(masked).expect("masking preserves UTF-8")),
        in_py_docstring: false,
        column_offset: 0,
        closed_py_docstring: false,
        assigned_data_span: Some((start, end)),
    }
}

/// The bounded assignment opener of §FS-check.1.1.3.1. This intentionally
/// recognizes no Python expression grammar: one unindented identifier, an
/// optional lexical annotation, `=`, and an approved prefix plus delimiter.
fn python_assigned_data_quote(line: &str) -> Option<(usize, usize, &'static str)> {
    let bytes = line.as_bytes();
    let first = *bytes.first()?;
    if !first.is_ascii_alphabetic() && first != b'_' {
        return None;
    }
    let mut cursor = 1;
    while bytes
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        cursor += 1;
    }
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    if bytes.get(cursor) == Some(&b':') {
        cursor += 1;
        cursor += line[cursor..].find('=')?;
    }
    if bytes.get(cursor) != Some(&b'=') {
        return None;
    }
    cursor += 1;
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    let literal_start = cursor;
    while bytes.get(cursor).is_some_and(u8::is_ascii_alphabetic) {
        cursor += 1;
    }
    let prefix = &line[literal_start..cursor];
    if !matches!(
        prefix.to_ascii_lowercase().as_str(),
        "" | "r" | "u" | "b" | "f" | "t" | "br" | "rb" | "fr" | "rf" | "tr" | "rt"
    ) {
        return None;
    }
    let quote = if line[cursor..].starts_with("\"\"\"") {
        "\"\"\""
    } else if line[cursor..].starts_with("'''") {
        "'''"
    } else {
        return None;
    };
    Some((literal_start, cursor, quote))
}

/// Find a matching delimiter whose immediately preceding backslash run is even
/// (§FS-check.1.1.3.1).
fn matching_unescaped_quote(line: &str, mut from: usize, quote: &str) -> Option<usize> {
    while let Some(relative) = line[from..].find(quote) {
        let index = from + relative;
        let slash_count = line.as_bytes()[..index]
            .iter()
            .rev()
            .take_while(|byte| **byte == b'\\')
            .count();
        if slash_count % 2 == 0 {
            return Some(index);
        }
        from = index + quote.len();
    }
    None
}

pub(super) fn python_docstring_quote(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("\"\"\"") {
        Some("\"\"\"")
    } else if trimmed.starts_with("'''") {
        Some("'''")
    } else {
        None
    }
}
