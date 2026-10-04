//! A source declaration's comment envelope, line by line (§FS-show.2.3): what
//! marks a line as part of the comment block, whether the block is line-style or
//! `/* … */`, and what is left of a body line once its envelope is stripped.

use crate::grammar::{LexicalSettings, line_comment_block_marker};

/// The line-style markers this file has always read on its own, by a fixed list
/// rather than by the declaration's marker. Every other marker the configured
/// `comment_prefixes` know — `--`, `;`, or a repository's own — is read as the
/// declaration's own comment family instead (`own_line_marker`).
const FIXED_LINE_MARKERS: [&str; 4] = ["///", "//!", "//", "#"];

/// Strip one of the fixed comment markers (`///`, `//!`, `//`, `#`, `*`, `/*`,
/// `*/`) off a body line when the declaration lives in a code/`"""` doc-comment — Markdown bodies
/// pass through unchanged (§FS-show.2.3.2). The whitespace in front of the marker
/// is envelope and goes with it, so an indented `///` or a class member's ` * `
/// shows its body at column 0; what follows the marker and its one space is kept
/// verbatim. A line with no marker is kept as-is.
pub(super) fn clean_body_line(line: &str, is_md: bool) -> String {
    if is_md {
        return line.to_string();
    }

    let body = line.trim_start();
    for prefix in ["///", "//!", "//", "*/", "#", "*", "/*"] {
        if let Some(rest) = body.strip_prefix(prefix) {
            if prefix == "*/" && rest.trim().is_empty() {
                return String::new();
            }
            return rest.strip_prefix(' ').unwrap_or(rest).to_string();
        }
    }
    line.to_string()
}

/// Whether a line still looks like part of the comment block by the fixed
/// markers — used to decide where an inline declaration's body ends
/// (§FS-show.2.3.1.2) when it has no `own_line_marker`.
pub(super) fn is_comment_body_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    ["///", "//!", "//", "#", "*", "/*", "*/"]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
}

/// Whether a declaration heading line sits inside a *line-style* comment
/// (`//`-family, `#`, `;`, `--`) as opposed to a `/* … */` block (which opens
/// `*` continuation lines). Line-style blocks end at a blank line; block-style
/// ones end at `*/` (§FS-show.2.3.1.2).
pub(super) fn is_line_style_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed.starts_with(';')
        || trimmed.starts_with("--")
}

/// The marker a line-style declaration's body lines must open with, when it is
/// not one of the `FIXED_LINE_MARKERS`: the scanner's marker for the declaration
/// line, so a `--` or `;` block is read line by line as the same comment prefix
/// family (§FS-show.2.3.1.1). `None` leaves the body on the fixed lists.
pub(super) fn own_line_marker(
    declaration_line: &str,
    lexical: LexicalSettings<'_>,
) -> Option<String> {
    line_comment_block_marker(declaration_line, lexical)
        .filter(|marker| !FIXED_LINE_MARKERS.contains(&marker.as_str()))
}

/// Whether a non-blank body line continues the declaration's comment block
/// (§FS-show.2.3.1.2): behind its own marker when it has one — the scanner's
/// marker for the line must be the same (§FS-show.2.3.1.1) — and by the fixed
/// markers otherwise.
pub(super) fn continues_comment_body(
    line: &str,
    own_marker: Option<&str>,
    lexical: LexicalSettings<'_>,
) -> bool {
    match own_marker {
        Some(marker) => line_comment_block_marker(line, lexical).as_deref() == Some(marker),
        None => is_comment_body_line(line),
    }
}

/// What is left of a body line once its envelope is off (§FS-show.2.3.2): behind
/// the declaration's own marker, the whitespace in front of it, the marker, and
/// one following space go and nothing else does; otherwise `clean_body_line`.
pub(super) fn clean_envelope_line(line: &str, is_md: bool, own_marker: Option<&str>) -> String {
    let Some(marker) = own_marker.filter(|_| !is_md) else {
        return clean_body_line(line, is_md);
    };
    match line.trim_start().strip_prefix(marker) {
        Some(rest) => rest.strip_prefix(' ').unwrap_or(rest).to_string(),
        None => line.to_string(),
    }
}
