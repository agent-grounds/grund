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

/// The forms one configured comment prefix stands for (§FS-languages.prefixes.forms,
/// §AR-scanner.4.3): the prefix itself, and the doc-comment spellings its languages
/// write by doubling or marking it. The one table every reader of a line's comment
/// marker reads — the declaration and section-heading alternation below, the
/// note-text stripper and the comment wrapper (`comment_line.rs`), the block keying
/// (`comment_block.rs`), the lone-opener test of the embedded-value readers and the
/// near-miss prefilter — so no reader widens a prefix by hand. The frozen 0.17 reader
/// (`scanner/compat_envelope.rs`) keeps its own list.
#[derive(Clone, Copy)]
pub(crate) struct PrefixForms<'a> {
    prefix: &'a str,
    /// The literal forms beyond the prefix itself.
    widened: &'static [&'static str],
    /// The prefix is one character and any run of it is a form (`;`, `;;`, `;;;`, …).
    run: bool,
    /// A widened form counts only where whitespace or the end of the line follows
    /// it (§FS-languages.prefixes.forms), so `----` and `-- |---|` open with `--`.
    /// Set for the forms 0.18 added; `//` and `/*` read as they always have.
    bounded: bool,
    /// Each form keys a comment block of its own, so a `///` run is a block apart
    /// from a `//` run (§AR-scanner.4.3); otherwise every form
    /// keys the prefix's block, so a `;` block stays keyed on `;`.
    keyed_by_form: bool,
}

/// The forms `prefix` stands for, by the table of §FS-languages.prefixes.forms. A
/// prefix outside it, such as a custom `%`, stands only for itself.
pub(crate) fn prefix_forms(prefix: &str) -> PrefixForms<'_> {
    let (widened, run, bounded, keyed_by_form): (&'static [&'static str], _, _, _) = match prefix {
        "//" => (&["///", "//!"], false, false, true),
        "/*" => (&["/**", "/*!"], false, false, false),
        "--" => (&["---", "-- |", "--|", "-- ^", "--^"], false, true, false),
        ";" => (&[], true, false, false),
        _ => (&[], false, false, false),
    };
    PrefixForms {
        prefix,
        widened,
        run,
        bounded,
        keyed_by_form,
    }
}

impl<'a> PrefixForms<'a> {
    /// The byte length of the longest form `rest` opens with, or `None` when it
    /// opens with none. A bounded widened form needs whitespace or the end of the
    /// line after it, so `----` and `-- |---|` open with `--`; the bare prefix and a
    /// run need nothing, so `--x` and `;x` open with theirs as they always have.
    pub(crate) fn len_at(&self, rest: &str) -> Option<usize> {
        if self.prefix.is_empty() {
            return None;
        }
        if self.run {
            let len = rest.len() - rest.trim_start_matches(self.prefix).len();
            return (len > 0).then_some(len);
        }
        let bounded = |form: &str| {
            !self.bounded
                || form == self.prefix
                || rest[form.len()..]
                    .chars()
                    .next()
                    .is_none_or(char::is_whitespace)
        };
        self.widened
            .iter()
            .copied()
            .chain([self.prefix])
            .filter(|form| rest.starts_with(form) && bounded(form))
            .map(str::len)
            .max()
    }

    /// Whether `text` is exactly one of these forms and nothing more, such as a
    /// lone `/**` opener line.
    pub(crate) fn is_form(&self, text: &str) -> bool {
        self.len_at(text) == Some(text.len())
    }

    /// The first byte every form opens with: the prefix's own, since each form
    /// widens the prefix rather than replacing it (§FS-languages.prefixes.forms).
    pub(crate) fn first_byte(&self) -> Option<u8> {
        self.prefix.as_bytes().first().copied()
    }

    /// The marker a line opening with one of these forms keys its comment block
    /// by: the form itself where forms key their own blocks, the prefix otherwise.
    pub(crate) fn block_key<'r>(&self, rest: &'r str) -> Option<&'r str>
    where
        'a: 'r,
    {
        let len = self.len_at(rest)?;
        Some(if self.keyed_by_form {
            &rest[..len]
        } else {
            &rest[..self.prefix.len()]
        })
    }

    /// One regex alternative per form, each a literal except a run's `;+`.
    fn regex_alternatives(&self) -> Vec<String> {
        if self.run {
            return vec![format!("{}+", regex::escape(self.prefix))];
        }
        self.widened
            .iter()
            .copied()
            .chain([self.prefix])
            .map(regex::escape)
            .collect()
    }
}

/// The byte length of the longest form of any of `prefixes` that `rest` opens
/// with (§FS-languages.prefixes.forms): what a reader strips as the line's comment
/// marker.
pub(crate) fn longest_form_at(rest: &str, prefixes: &[PrefixForms<'_>]) -> Option<usize> {
    prefixes.iter().filter_map(|forms| forms.len_at(rest)).max()
}

/// Whether a trimmed comment line is a `/* … */` block's envelope alone: a lone
/// opener in any form of `/*` (§FS-languages.prefixes.forms), a lone `*` gutter or
/// a lone closer — a line that says nothing (§FS-declarations.line.block-comment).
pub(crate) fn is_lone_block_marker(content: &str) -> bool {
    prefix_forms("/*").is_form(content) || matches!(content, "*" | "*/")
}

/// Build the alternation a declaration/section heading may be prefixed by — every
/// form of every `[scan] comment_prefixes` value (§FS-config.3.5,
/// §FS-languages.prefixes.forms), so `///`, `/**`, `---`, `-- |` and `;;` declare
/// as their bare prefixes do (§AR-scanner.4.3, §FS-declarations.line.block-comment).
/// Longest-first so `//` does not shadow `///`; each alternative is a flat literal
/// or a run, so the near-miss prefilter reads every first byte off it.
pub(super) fn comment_prefix_regex(comment_prefixes: &[String]) -> String {
    let mut alternatives = comment_prefixes
        .iter()
        .filter(|prefix| !prefix.is_empty())
        .flat_map(|prefix| prefix_forms(prefix).regex_alternatives())
        .collect::<Vec<_>>();
    alternatives.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    alternatives.dedup();
    if alternatives.is_empty() {
        "(?!)".to_string()
    } else {
        format!("(?:{})", alternatives.join("|"))
    }
}

/// The `/* … */` state the line reader carries across lines, as it carries a
/// docstring's (§AR-scanner.4.4): a block opens only on a line whose first
/// non-blank characters are `/*` (so `/**` and `/*!` too) and closes at the first
/// `*/` after its opener, on that line or a later one
/// (§FS-declarations.line.block-comment). `comment_blocks` is its reader, and every
/// gate reads the spans it walks, so "is this line inside a block?" has one answer.
#[derive(Clone, Copy, Default)]
pub(crate) struct BlockCommentScanState {
    open: bool,
}

/// Where one line stands to a `/* … */` block: inside one (its opener included),
/// and whether it is the line that closes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BlockCommentLine {
    pub(crate) inside: bool,
    pub(crate) closes: bool,
}

impl BlockCommentScanState {
    /// Advance over one raw line. Recognition, not parsing: a `/*` after code or in
    /// a string on its line opens nothing, and a line-initial `/*` inside a
    /// multi-line string literal opens a block all the same.
    pub(crate) fn advance(&mut self, line: &str) -> BlockCommentLine {
        let rest = if self.open {
            line
        } else if let Some(rest) = line.trim_start().strip_prefix("/*") {
            rest
        } else {
            return BlockCommentLine {
                inside: false,
                closes: false,
            };
        };
        let closes = rest.contains("*/");
        self.open = !closes;
        BlockCommentLine {
            inside: true,
            closes,
        }
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
        cursor = annotation_assignment_index(line, cursor + 1)?;
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

/// Find the top-level assignment after an annotation without parsing the
/// annotation as a Python expression (§FS-check.1.1.3.1). Quotes and balanced
/// delimiters are enough to keep their internal `=` bytes from becoming the
/// assignment boundary; the caller still requires a direct triple-string RHS.
fn annotation_assignment_index(line: &str, mut cursor: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut nesting = 0usize;
    while let Some(byte) = bytes.get(cursor).copied() {
        match byte {
            b'\'' | b'"' => {
                let quote_len = if bytes.get(cursor..cursor + 3) == Some(&[byte, byte, byte][..]) {
                    3
                } else {
                    1
                };
                let quote = &line[cursor..cursor + quote_len];
                let close = matching_unescaped_quote(line, cursor + quote_len, quote)?;
                cursor = close + quote_len;
            }
            b'(' | b'[' | b'{' => {
                nesting += 1;
                cursor += 1;
            }
            b')' | b']' | b'}' => {
                nesting = nesting.saturating_sub(1);
                cursor += 1;
            }
            b'=' if nesting == 0 && is_assignment_equal(bytes, cursor) => return Some(cursor),
            b'#' if nesting == 0 => return None,
            _ => cursor += 1,
        }
    }
    None
}

fn is_assignment_equal(bytes: &[u8], index: usize) -> bool {
    !matches!(
        index
            .checked_sub(1)
            .and_then(|previous| bytes.get(previous)),
        Some(b'=' | b'!' | b'<' | b'>' | b':')
    ) && bytes.get(index + 1) != Some(&b'=')
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
        from = index + 1;
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
