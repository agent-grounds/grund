//! The text helpers of declaration-body extraction (§FS-show.2.3): how `--toc`
//! and `--brief` reshape a finished body. They are pure functions of text, split
//! out of `body.rs` so that file keeps to choosing the views; the walk that fills
//! them, and what one comment line reads as once its envelope is off, are the
//! scanner's (`slices.rs`).

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
