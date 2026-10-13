//! Today's comment envelope, line by line (§FS-show.2.3, §DF-one-parse-answers-from-0-19):
//! what marks a line as part of a declaration's comment by today's fixed markers,
//! whether the block is line-style or `/* … */`, and what is left of a body line or
//! a `--toc` title once its envelope is off. `compat.rs` is its one reader, and it
//! goes with that file in 0.19.0, when the record's envelope
//! (§FS-languages.comment-wrapper, `grammar::comment_envelope`) answers instead.

use crate::grammar::{
    LexicalSettings, line_comment_block_marker, prefix_forms, strip_block_closer,
};

/// The line-style markers this file has always read on its own, by a fixed list
/// rather than by the declaration's marker. Every other marker the configured
/// `comment_prefixes` know — `--`, `;`, or a repository's own — is read as the
/// declaration's own comment family instead (`own_line_marker`). This file's lists
/// are the one sanctioned copy of markers outside `grammar::prefix_forms`: they are
/// 0.17's reading, frozen until 0.19.0, and must not follow the forms table.
const FIXED_LINE_MARKERS: [&str; 4] = ["///", "//!", "//", "#"];

/// Strip one of the fixed comment markers (`///`, `//!`, `//`, `#`, `*`, `/**`,
/// `/*!`, `/*`, `*/`) off a body line when the declaration lives in a code/`"""` doc-comment — Markdown bodies
/// pass through unchanged (§FS-show.2.3.2). The whitespace in front of the marker
/// is envelope and goes with it, so an indented `///` or a class member's ` * `
/// shows its body at column 0; what follows the marker and its one space is kept
/// verbatim. A line with no marker is kept as-is.
pub(super) fn clean_body_line(line: &str, is_md: bool) -> String {
    if is_md {
        return line.to_string();
    }

    let body = line.trim_start();
    // §FS-declarations.line.block-comment: `/**` and `/*!` are openers as `/*` is.
    for prefix in ["///", "//!", "//", "*/", "#", "*", "/**", "/*!", "/*"] {
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

/// The marker a line-style declaration's body lines open with, when it is not one
/// of the `FIXED_LINE_MARKERS` (§FS-show.2.3.1.1), and how much of a body line it
/// takes off (§FS-show.2.3.2).
pub(super) struct OwnMarker {
    /// The scanner's marker for the declaration line, which keys its block.
    marker: String,
    /// The declaration line is written in a form beyond the bare marker (`;;`,
    /// `---`, `-- |`, …), so 0.17 never found it and its body loses the whole form
    /// of each line (§FS-languages.prefixes.forms). A declaration behind the bare
    /// `;` or `--` was found before, and keeps 0.17's reading — the bare marker and
    /// nothing more — until 0.19.0 (§DF-line-prefix-forms-declare.4).
    whole_form: bool,
}

impl OwnMarker {
    #[cfg(test)]
    pub(super) fn bare(marker: &str) -> Self {
        Self {
            marker: marker.to_string(),
            whole_form: false,
        }
    }
}

/// The marker a line-style declaration's body lines must open with, when it is
/// not one of the `FIXED_LINE_MARKERS`: the scanner's marker for the declaration
/// line, so a `--` or `;` block is read line by line as the same comment prefix
/// family (§FS-show.2.3.1.1). `None` leaves the body on the fixed lists.
pub(super) fn own_line_marker(
    declaration_line: &str,
    lexical: LexicalSettings<'_>,
) -> Option<OwnMarker> {
    let marker = line_comment_block_marker(declaration_line, lexical)
        .filter(|marker| !FIXED_LINE_MARKERS.contains(&marker.as_str()))?;
    let whole_form = prefix_forms(&marker)
        .len_at(declaration_line.trim_start())
        .is_some_and(|len| len > marker.len());
    Some(OwnMarker { marker, whole_form })
}

/// Whether a non-blank body line continues the declaration's comment block
/// (§FS-show.2.3.1.2): behind its own marker when it has one — the scanner's
/// marker for the line must be the same (§FS-show.2.3.1.1) — and by the fixed
/// markers otherwise.
pub(super) fn continues_comment_body(
    line: &str,
    own_marker: Option<&OwnMarker>,
    lexical: LexicalSettings<'_>,
) -> bool {
    match own_marker {
        Some(own) => line_comment_block_marker(line, lexical).as_deref() == Some(&own.marker),
        None => is_comment_body_line(line),
    }
}

/// What is left of a body line once its envelope is off (§FS-show.2.3.2): behind
/// the declaration's own marker, the whitespace in front of it, the marker, and
/// one following space go and nothing else does; otherwise `clean_body_line`. For
/// a declaration 0.17 never found, the marker that goes is the whole form the line
/// was written in (§FS-languages.prefixes.forms, so a `;;` run goes whole).
pub(super) fn clean_envelope_line(
    line: &str,
    is_md: bool,
    own_marker: Option<&OwnMarker>,
) -> String {
    let Some(own) = own_marker.filter(|_| !is_md) else {
        return clean_body_line(line, is_md);
    };
    let trimmed = line.trim_start();
    let len = if own.whole_form {
        prefix_forms(&own.marker).len_at(trimmed)
    } else {
        trimmed.starts_with(&own.marker).then_some(own.marker.len())
    };
    match len {
        Some(len) => {
            let rest = &trimmed[len..];
            rest.strip_prefix(' ').unwrap_or(rest).to_string()
        }
        None => line.to_string(),
    }
}

/// Whether a declaration-body line closes the declaration's own `/* … */`
/// block, so that the body ends after it (§FS-show.2.3.1.2). Line-style
/// comments, Python docstrings and Markdown never close on `*/`.
pub(super) fn closes_comment_block(
    line: &str,
    line_style_comment: bool,
    in_py_docstring: bool,
    is_md: bool,
) -> bool {
    !is_md && !line_style_comment && !in_py_docstring && line.contains("*/")
}

/// A `--toc` section's title: the label its author wrote after the coordinate,
/// with the comment envelope — a source line's closing `*/` included — left out,
/// so it agrees with the title the scanner records (§FS-rules.5.1.1, §FS-show.3.1.3).
pub(super) fn section_title(
    line: &str,
    section: &str,
    markdown_heading: bool,
    own_marker: Option<&OwnMarker>,
) -> String {
    let clean = clean_envelope_line(line, markdown_heading, own_marker);
    let clean = if markdown_heading {
        &clean
    } else {
        strip_block_closer(&clean)
    };
    clean
        .trim_start()
        .trim_start_matches('#')
        .trim_start()
        .trim_start_matches(section)
        .trim_start_matches(['.', ':'])
        .trim_start()
        .to_string()
}

#[cfg(test)]
#[path = "tests_compat_envelope.rs"]
mod tests_compat_envelope;
