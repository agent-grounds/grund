//! The per-line readers that ask the compiled grammar a question about one
//! source line (§AR-scanner.4): the comment-prefix alternation a declaration or
//! section heading may sit behind, and the normalization that makes a Python
//! docstring line scannable while keeping the original file's column offset.
//!
//! Separate from `compiled.rs` because building the grammar and walking a line
//! with it are two jobs (§AR-core-module-layout.3) — this file is the second,
//! and it holds no pattern of its own.

/// Build the alternation a declaration/section heading may be prefixed by — one
/// entry per `[scan] comment_prefixes` value (§FS-config.3.5), with `//` widened to
/// also catch Rust/JS doc-comment forms `///` and `//!` so inline declarations in
/// code are seen (§AR-scanner.4). Longest-first so `//` does not shadow `///`.
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

#[derive(Default)]
pub(crate) struct PythonDocstringScanState {
    pub(super) quote: Option<&'static str>,
}

pub(crate) struct SourceScanLine<'a> {
    pub(crate) text: &'a str,
    pub(crate) in_py_docstring: bool,
    pub(crate) column_offset: usize,
    pub(crate) closed_py_docstring: bool,
}

/// Normalize one source line for scanner-style declaration/section/citation
/// detection while preserving the original-file column offset for emitted
/// ranges (§AR-scanner.4).
pub(crate) fn source_scan_line<'a>(
    line: &'a str,
    is_py: bool,
    docstring_python: bool,
    py_docstring: &mut PythonDocstringScanState,
) -> SourceScanLine<'a> {
    if !docstring_python || !is_py {
        return SourceScanLine {
            text: line,
            in_py_docstring: false,
            column_offset: 0,
            closed_py_docstring: false,
        };
    }

    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();
    if let Some(quote) = py_docstring.quote {
        if let Some(close) = trimmed.find(quote) {
            py_docstring.quote = None;
            return SourceScanLine {
                text: &trimmed[..close],
                in_py_docstring: true,
                column_offset: indent,
                closed_py_docstring: true,
            };
        }
        return SourceScanLine {
            text: trimmed,
            in_py_docstring: true,
            column_offset: indent,
            closed_py_docstring: false,
        };
    }

    let Some(quote) = python_docstring_quote(line) else {
        return SourceScanLine {
            text: line,
            in_py_docstring: false,
            column_offset: 0,
            closed_py_docstring: false,
        };
    };
    let after_open = &trimmed[quote.len()..];
    if let Some(close) = after_open.find(quote) {
        return SourceScanLine {
            text: &after_open[..close],
            in_py_docstring: true,
            column_offset: indent + quote.len(),
            closed_py_docstring: true,
        };
    }
    py_docstring.quote = Some(quote);
    SourceScanLine {
        text: after_open,
        in_py_docstring: true,
        column_offset: indent + quote.len(),
        closed_py_docstring: false,
    }
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
