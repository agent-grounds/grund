//! Test module: today's comment envelope (§FS-show.2.3.2,
//! §DF-one-parse-answers-from-0-19). A source body line loses its envelope — the
//! whitespace in front of the marker, the marker and one space — and keeps the rest
//! verbatim; the title a `--toc` section map gives a source chapter is the label its
//! author wrote (§FS-rules.5.1.1, §FS-show.3.1.3).

use super::{clean_body_line, section_title};

#[test]
fn block_continuation_drops_whitespace_before_star() {
    assert_eq!(
        clean_body_line(" * ## terms: Terms", false),
        "## terms: Terms"
    );
    assert_eq!(
        clean_body_line("     * ## terms: Terms", false),
        "## terms: Terms"
    );
    assert_eq!(clean_body_line("     *", false), "");
}

/// §FS-declarations.line.block-comment: every opener is envelope, so `/** ## 1. Sub`
/// shows as `/* ## 1. Sub` does.
#[test]
fn every_block_opener_drops_whole() {
    for line in ["/* ## 1. Sub", "/** ## 1. Sub", "/*! ## 1. Sub"] {
        assert_eq!(clean_body_line(line, false), "## 1. Sub", "line {line:?}");
    }
}

#[test]
fn indented_line_comment_drops_whitespace_before_marker() {
    assert_eq!(clean_body_line("    /// x", false), "x");
    assert_eq!(clean_body_line("\t//! x", false), "x");
    assert_eq!(clean_body_line("  # x", false), "x");
}

#[test]
fn indentation_after_the_marker_survives() {
    assert_eq!(clean_body_line("///   - nested", false), "  - nested");
    assert_eq!(clean_body_line("     *   - nested", false), "  - nested");
}

#[test]
fn closing_and_unmarked_lines() {
    assert_eq!(clean_body_line("     */", false), "");
    assert_eq!(clean_body_line("   plain text", false), "   plain text");
}

#[test]
fn markdown_lines_pass_through() {
    assert_eq!(clean_body_line("    /// x", true), "    /// x");
}

/// §FS-rules.5.1.1 (§FS-languages.comment-wrapper.headings): a block comment's closing `*/` on the heading's own line is
/// envelope, like the ` * ` that opens it; `///` is the control.
#[test]
fn a_toc_title_drops_a_block_comment_closer_on_the_heading_line() {
    assert_eq!(
        section_title("/// ## terms: Terms", "terms", false, None),
        "Terms"
    );
    assert_eq!(
        section_title(" * ## terms: Terms", "terms", false, None),
        "Terms"
    );
    assert_eq!(
        section_title(" * ## terms: Terms */", "terms", false, None),
        "Terms"
    );
    assert_eq!(
        section_title(" * ## terms: Terms   */", "terms", false, None),
        "Terms"
    );
}

/// §FS-rules.5.1.1: a Markdown heading is never trimmed, so a `*/` its author
/// wrote stays in the title.
#[test]
fn a_markdown_toc_title_keeps_a_trailing_closer() {
    assert_eq!(
        section_title("## terms: Terms */", "terms", true, None),
        "Terms */"
    );
}

/// §FS-show.2.3.2: a `--` or `;` heading line loses its own marker and one
/// space, so its title is the label its author wrote.
#[test]
fn a_toc_title_drops_its_own_dash_dash_or_semicolon_marker() {
    assert_eq!(
        section_title("-- ## terms: Terms", "terms", false, Some("--")),
        "Terms"
    );
    assert_eq!(
        section_title("; ## terms: Terms", "terms", false, Some(";")),
        "Terms"
    );
}
