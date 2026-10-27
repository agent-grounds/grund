//! Test module: which Markdown lines a heading can stand on (§FS-check.1.1.5.1,
//! §FS-check.1.1.5.2).
//! The e2e cases pin what `check` and `show` make of a raw-text HTML block; these
//! pin the boundaries of the one state machine every reader walks.

use super::*;

fn kinds(text: &str) -> Vec<MarkdownLine> {
    let mut blocks = MarkdownBlocks::default();
    text.lines().map(|line| blocks.line(line)).collect()
}

use MarkdownLine::{FenceDelimiter, Fenced, FrontMatter, RawHtml, Text};

/// A whole file read as every whole-file reader reads it (§FS-check.1.1.5.2).
fn file_kinds(text: &str) -> Vec<MarkdownLine> {
    let mut blocks = MarkdownBlocks::file(text);
    text.lines().map(|line| blocks.line(line)).collect()
}

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

/// §FS-check.1.1.5.2: front matter opens only on a line 1 that is exactly `---` or
/// `+++`, and closes at the next line exactly the same.
#[test]
fn front_matter_delimiters_are_exact() {
    assert_eq!(
        file_kinds("---\n# a\n---\n# b"),
        [FrontMatter, FrontMatter, FrontMatter, Text]
    );
    assert_eq!(
        file_kinds("+++\n# a\n+++\n# b"),
        [FrontMatter, FrontMatter, FrontMatter, Text]
    );
    for opener in [" ---", "--- ", "----", "-- -", "+++ ", "\u{feff}---"] {
        assert_eq!(
            front_matter_extent(format!("{opener}\n# a\n---\n").lines()),
            None,
            "{opener:?}"
        );
    }
    // A closer that is not exactly the delimiter does not close.
    assert_eq!(front_matter_extent("---\n# a\n --- \n".lines()), None);
    assert_eq!(
        front_matter_extent("---\n# a\n----\n---\n".lines()),
        Some(4)
    );
}

/// §FS-check.1.1.5.2: `...` closes a `---` block, never a `+++` block, and a block
/// that never closes is no front matter.
#[test]
fn three_dots_close_only_a_yaml_block() {
    assert_eq!(front_matter_extent("---\na: b\n...\n".lines()), Some(3));
    assert_eq!(front_matter_extent("+++\na = 1\n...\n".lines()), None);
    assert_eq!(front_matter_extent("+++\n...\n+++\n".lines()), Some(3));
    assert_eq!(front_matter_extent("---\na: b\n+++\n".lines()), None);
    assert_eq!(file_kinds("---\n# a\n"), [Text, Text]);
}

/// §FS-check.1.1.5.2: a trailing `\r` is the line ending, not part of the line.
#[test]
fn a_crlf_delimiter_is_exact() {
    assert_eq!(
        front_matter_extent("---\r\na: b\r\n---\r\n".lines()),
        Some(3)
    );
    assert_eq!(front_matter_extent(["---\r", "a: b", "...\r"]), Some(3));
}

/// §FS-check.1.1.5.2: a first line led by a byte-order mark opens nothing.
#[test]
fn a_byte_order_mark_opens_no_front_matter() {
    assert_eq!(file_kinds("\u{feff}---\n# a\n---\n")[1], Text);
}

/// §FS-check.1.1.5.2: front matter is decided first, so a fence opener or a
/// `<pre>` inside it opens nothing and nothing is left open behind it.
#[test]
fn a_fence_or_pre_inside_front_matter_opens_nothing() {
    assert_eq!(
        file_kinds("---\n```\n<pre>\n---\n# a"),
        [FrontMatter, FrontMatter, FrontMatter, FrontMatter, Text]
    );
}

/// §FS-check.1.1.5.2: a body reader starts with `default()` and reads no front
/// matter, and a front-matter line is no heading, not fenced and never rewritten.
#[test]
fn front_matter_is_live_but_neither_heading_nor_rewritable() {
    assert_eq!(kinds("---\n# a\n---\n"), [Text, Text, Text]);
    assert!(!FrontMatter.may_be_heading() && !FrontMatter.in_fence());
    assert!(!FrontMatter.rewritable() && Text.rewritable() && RawHtml.rewritable());
}
