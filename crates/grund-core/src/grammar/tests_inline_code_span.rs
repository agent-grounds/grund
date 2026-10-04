//! Test module: where a Markdown inline code span begins and ends, the one
//! reading every pass of `fmt` and every inline-code reading of `check` share
//! (§FS-fmt.2.3.5).

use super::*;

/// The byte offset of `needle` in `line`, which every case below names by text
/// rather than by a column count. The lines spell their citations with `@`: the
/// predicate reads backticks and backslashes only, and a `§` here would be a live
/// citation of this repository.
fn at(line: &str, needle: &str) -> usize {
    line.find(needle)
        .unwrap_or_else(|| panic!("`{needle}` is not on `{line}`"))
}

/// §FS-fmt.2.3.5: a span opened by two backticks closes only at the next run of
/// exactly two, so a single backtick inside it is content and the citation after
/// it is prose. The issue's line, and the same line narrowed to the backtick.
#[test]
fn a_single_backtick_inside_a_double_backtick_span_does_not_close_it() {
    for line in [
        "Escaped `` \\ ` * `` then [@GOAL-a](goals.md#goal-a-stale) here.",
        "Min `` ` `` then [@GOAL-a](goals.md#goal-a-stale) here.",
    ] {
        let link = at(line, "[@GOAL-a]");
        assert!(
            !is_inside_inline_code(line, link),
            "the citation after the span is prose: {line}"
        );
        assert!(
            !never_rewrite_context(line, true, link),
            "check and fmt read the same prose: {line}"
        );
        let inner = line.rfind(" ` ").expect("the inner backtick") + 1;
        assert!(
            is_inside_inline_code(line, inner),
            "the inner backtick is content: {line}"
        );
    }
}

/// §FS-fmt.2.3.5: the control. A citation inside a double-backtick span, before
/// the single backtick it holds, is code: the parity reading called it prose.
#[test]
fn a_citation_inside_a_double_backtick_span_before_its_inner_backtick_is_code() {
    let line = "Inside `` [@GOAL-a](goals.md#goal-a-stale) ` `` stays.";
    let link = at(line, "[@GOAL-a]");
    assert!(is_inside_inline_code(line, link));
    assert!(never_rewrite_context(line, true, link));
    assert!(!is_inside_inline_code(line, at(line, "stays")));
}

/// §FS-fmt.2.3.5: a span opened by a run of two holds no backtick at all, and
/// what is between the runs is still code.
#[test]
fn the_content_of_a_double_backtick_span_is_code() {
    let line = "Inside `` @FS-042 `` stays.";
    assert!(is_inside_inline_code(line, at(line, "@FS-042")));
    assert!(!is_inside_inline_code(line, at(line, "stays")));
}

/// §FS-fmt.2.3.5: inside a span a backslash is literal, so it does not escape the
/// closing run, and the text after `` `C:\` `` is prose.
#[test]
fn a_backslash_inside_a_span_does_not_stop_its_closing_run() {
    let line = "Path `C:\\` then [@GOAL-a](goals.md#goal-a-stale) here.";
    assert!(is_inside_inline_code(line, at(line, "C:")));
    assert!(!is_inside_inline_code(line, at(line, "[@GOAL-a]")));
}

/// §FS-fmt.2.3.5: a longer run inside a span is content too, not only a shorter
/// one, and a span closes on the run of its own length.
#[test]
fn a_longer_run_inside_a_span_is_content() {
    let line = "Fence ``` a `` b ``` then @FS-042 here.";
    assert!(is_inside_inline_code(line, at(line, "b ")));
    assert!(!is_inside_inline_code(line, at(line, "@FS-042")));
}

/// §FS-fmt.2.3.5: two spans on one line leave the text between them prose.
#[test]
fn the_text_between_two_spans_is_prose() {
    let line = "`a` then @FS-042 and `b` here.";
    assert!(is_inside_inline_code(line, at(line, "a`")));
    assert!(!is_inside_inline_code(line, at(line, "@FS-042")));
    assert!(is_inside_inline_code(line, at(line, "b`")));
    assert!(!is_inside_inline_code(line, at(line, "here")));
}

/// §FS-fmt.2.3.5: an escaped backtick opens nothing, so the citation after it is
/// prose.
#[test]
fn an_escaped_backtick_opens_no_span() {
    let line = "Literal \\` then @FS-042 here.";
    assert!(!is_inside_inline_code(line, at(line, "@FS-042")));
}

/// §FS-fmt.2.3.5: a single backtick with no closing run on its line leaves the
/// rest of the line code, because the span may continue onto the next line.
/// Unchanged from the parity reading, and pinned so that it stays a decision.
#[test]
fn a_stray_backtick_leaves_the_rest_of_its_line_code() {
    let line = "Stray ` then [@GOAL-a](goals.md#goal-a-stale) stays.";
    assert!(is_inside_inline_code(line, at(line, "[@GOAL-a]")));
}

/// §FS-fmt.2.3.5: the same holds for a longer opener, and for one whose line
/// holds only runs of another length, which the parity reading called prose.
#[test]
fn an_opener_that_does_not_close_on_its_line_leaves_the_rest_code() {
    for line in [
        "Stray `` then [@GOAL-a](goals.md#goal-a-stale) stays.",
        "Short ` then `` then [@GOAL-a](goals.md#goal-a-stale) stays.",
    ] {
        assert!(
            is_inside_inline_code(line, at(line, "[@GOAL-a]")),
            "an unclosed opener runs to the end of its line: {line}"
        );
    }
}
