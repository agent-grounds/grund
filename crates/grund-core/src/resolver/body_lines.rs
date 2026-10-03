//! The line-text helpers of declaration-body extraction (§FS-show.2.3): the
//! title a `--toc` section gets, where a comment block ends, and how `--toc` and
//! `--brief` reshape a finished body. They are pure functions of text, split out
//! of `body.rs` so that file keeps to the walk itself; what one comment line reads
//! as once its envelope is off is `comment_envelope.rs`.

use crate::grammar::strip_block_closer;
use crate::model::ShowSection;

use super::comment_envelope::clean_body_line;

pub(super) fn push_outline_section(
    lines: &mut Vec<String>,
    sections: &mut Vec<ShowSection>,
    line: &str,
    section: &str,
    depth: usize,
    markdown_heading: bool,
) {
    lines.push(clean_body_line(line, markdown_heading));
    sections.push(ShowSection {
        path: section.to_string(),
        title: section_title(line, section, markdown_heading),
        depth,
    });
}

/// A `--toc` section's title: the label its author wrote after the coordinate,
/// with the comment envelope — a source line's closing `*/` included — left out,
/// so it agrees with the title the scanner records (§FS-rules.5.1.1, §FS-show.3.1.3).
fn section_title(line: &str, section: &str, markdown_heading: bool) -> String {
    let clean = clean_body_line(line, markdown_heading);
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

/// `--toc` joins the default body with the section-map body, separated by one
/// blank line. Empty halves are dropped; if both are empty the result is empty.
/// Each body already ends with `\n`, so `{a}\n{b}` produces `<a>\n\n<b>\n`
/// (§FS-show.2.1.2.2).
pub(super) fn join_with_blank(default_body: &str, outline_body: &str) -> String {
    match (default_body.is_empty(), outline_body.is_empty()) {
        (true, true) => String::new(),
        (true, false) => outline_body.to_string(),
        (false, true) => default_body.to_string(),
        (false, false) => format!("{default_body}\n{outline_body}"),
    }
}

/// `--brief` truncates the (default-mode, heading-included) body to its first
/// blank-line-separated paragraph (§FS-show.2.1.1). Keeps the heading line and
/// at most one blank-line separator before the first paragraph; stops at the
/// next blank line (or end of body).
pub(super) fn truncate_to_first_paragraph(body: &str) -> String {
    let mut lines: Vec<&str> = body.split('\n').collect();
    // `body` ends with `\n`, so the split produces a trailing empty element.
    if lines.last() == Some(&"") {
        lines.pop();
    }
    if lines.is_empty() {
        return String::new();
    }
    let mut out: Vec<&str> = vec![lines[0]];
    let mut i = 1;
    let mut kept_separator = false;
    while i < lines.len() && lines[i].trim().is_empty() {
        if !kept_separator {
            out.push(lines[i]);
            kept_separator = true;
        }
        i += 1;
    }
    while i < lines.len() && !lines[i].trim().is_empty() {
        out.push(lines[i]);
        i += 1;
    }
    while out.last().is_some_and(|line| line.trim().is_empty()) {
        out.pop();
    }
    if out.is_empty() {
        String::new()
    } else {
        format!("{}\n", out.join("\n"))
    }
}

#[cfg(test)]
#[path = "tests_section_title.rs"]
mod tests_section_title;
