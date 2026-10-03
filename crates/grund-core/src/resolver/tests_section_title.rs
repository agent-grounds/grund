//! Test module: the title a `--toc` section map gives a source chapter is the
//! label its author wrote (§FS-rules.5.1.1, §FS-show.3.1.3) — the same title
//! the scanner records, so `--toc`, `list --selector` and the rules agree.

use super::section_title;

/// §FS-rules.5.1.1: a block comment's closing `*/` on the heading's own line is
/// envelope, like the ` * ` that opens it; `///` is the control.
#[test]
fn a_toc_title_drops_a_block_comment_closer_on_the_heading_line() {
    assert_eq!(
        section_title("/// ## terms: Terms", "terms", false),
        "Terms"
    );
    assert_eq!(section_title(" * ## terms: Terms", "terms", false), "Terms");
    assert_eq!(
        section_title(" * ## terms: Terms */", "terms", false),
        "Terms"
    );
    assert_eq!(
        section_title(" * ## terms: Terms   */", "terms", false),
        "Terms"
    );
}

/// §FS-rules.5.1.1: a Markdown heading is never trimmed, so a `*/` its author
/// wrote stays in the title.
#[test]
fn a_markdown_toc_title_keeps_a_trailing_closer() {
    assert_eq!(
        section_title("## terms: Terms */", "terms", true),
        "Terms */"
    );
}
