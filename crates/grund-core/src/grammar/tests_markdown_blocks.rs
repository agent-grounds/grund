//! Test module: which Markdown lines a heading can stand on (§FS-check.1.1.5.1).
//! The e2e cases pin what `check` and `show` make of a raw-text HTML block; these
//! pin the boundaries of the one state machine every reader walks.

use super::*;

fn kinds(text: &str) -> Vec<MarkdownLine> {
    markdown_line_kinds(text.lines())
}

use MarkdownLine::{FenceDelimiter, Fenced, RawHtml, Text};

/// §FS-check.1.1.5.1: a block opened and closed on one line holds that line only.
#[test]
fn a_one_line_block_closes_on_its_opening_line() {
    assert_eq!(
        kinds("<pre class=\"x\"># FS-a: b</pre>\n# FS-c: d"),
        [RawHtml, Text]
    );
}

/// §FS-check.1.1.5.1: the block runs through the first line holding its end tag,
/// that line included.
#[test]
fn a_block_runs_through_its_closing_line() {
    assert_eq!(
        kinds("<pre>\n# FS-a: b\n</pre>\n# FS-c: d"),
        [RawHtml, RawHtml, RawHtml, Text]
    );
}

/// §FS-check.1.1.5.1: an unclosed block runs to end of file.
#[test]
fn an_unclosed_block_runs_to_end_of_file() {
    assert_eq!(kinds("<script>\n# a\n\n## b"), [RawHtml; 4]);
}

/// §FS-check.1.1.5.1: the tag name and its end tag are matched case-insensitively,
/// and only the opening tag's own end tag closes the block.
#[test]
fn tags_match_case_insensitively_and_only_their_own_end_tag_closes() {
    assert_eq!(
        kinds("<SCRIPT>\n</style>\n</Script>\n# a"),
        [RawHtml, RawHtml, RawHtml, Text]
    );
    assert_eq!(
        kinds("<TextArea\n</TEXTAREA>\n# a"),
        [RawHtml, RawHtml, Text]
    );
}

/// §FS-check.1.1.5.1: the tag name must end at whitespace, `>` or end of line.
#[test]
fn a_longer_tag_name_opens_nothing() {
    assert_eq!(kinds("<prefix>\n# a"), [Text, Text]);
    assert_eq!(kinds("<pre-wrap>\n# a"), [Text, Text]);
    assert_eq!(kinds("<style\ttype=x>\n# a"), [RawHtml, RawHtml]);
}

/// §FS-check.1.1.5.1: at most three leading spaces, as for a fence.
#[test]
fn four_spaces_of_indent_open_nothing() {
    assert_eq!(kinds("   <pre>\n# a"), [RawHtml, RawHtml]);
    assert_eq!(kinds("    <pre>\n# a"), [Text, Text]);
}

/// §FS-check.1.1.5.1: inside a fence an HTML opener is fence content.
#[test]
fn an_opener_inside_a_fence_opens_nothing() {
    assert_eq!(
        kinds("```\n<pre>\n```\n# a"),
        [FenceDelimiter, Fenced, FenceDelimiter, Text]
    );
}

/// §FS-check.1.1.5.1: inside the block a fence opener is block content, so the
/// block's end tag still closes it and no fence is left open behind it.
#[test]
fn a_fence_inside_a_block_opens_nothing() {
    assert_eq!(
        kinds("<pre>\n```\n</pre>\n# a"),
        [RawHtml, RawHtml, RawHtml, Text]
    );
}

/// §FS-check.1.1.5.1: only plain text may hold a heading; citations stay live in
/// a block but not in a fence.
#[test]
fn only_text_may_hold_a_heading() {
    assert!(Text.may_be_heading());
    assert!(!RawHtml.may_be_heading() && !RawHtml.in_fence());
    assert!(Fenced.in_fence() && FenceDelimiter.in_fence());
}
