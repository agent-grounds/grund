//! A source declaration's comment envelope, line by line (§FS-show.2.3): what
//! marks a line as part of the comment block, whether the block is line-style or
//! `/* … */`, and what is left of a body line once its envelope is stripped.

/// Strip the comment marker (`///`, `//!`, `//`, `#`, `*`, `/*`, `*/`) off a body
/// line when the declaration lives in a code/`"""` doc-comment — Markdown bodies
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

/// Whether a line still looks like part of the comment block — used to decide
/// where an inline declaration's body ends (§FS-show.2.3.1.2).
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
