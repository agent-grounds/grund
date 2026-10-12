//! Test module: the `/* … */` state the line reader carries across lines, and the
//! spans `comment_blocks` reads off it (§FS-declarations.line.block-comment,
//! §AR-scanner.4.4). A block opens only on a line that starts with an opener and
//! closes at the first `*/` after it; a bare line is in declaration position only
//! in a block whose `*/` is found.

use super::source_line::{BlockCommentLine, BlockCommentScanState};
use super::{CommentBlockKind, bare_block_lines, comment_blocks};
use crate::testing::{numbered_config, test_root};

/// Where each line stands to a block, read by one state from the top.
fn states(lines: &[&str]) -> Vec<(bool, bool)> {
    let mut state = BlockCommentScanState::default();
    lines
        .iter()
        .map(|line| {
            let BlockCommentLine { inside, closes } = state.advance(line);
            (inside, closes)
        })
        .collect()
}

/// A `/*` after code, in a string, or behind `//` opens nothing.
#[test]
fn a_block_opens_only_at_the_start_of_a_line() {
    let lines = [
        "x = 1; /* note",
        "let s = \"/*\";",
        "// see /* here",
        "FS-b: Beta",
        "    /** indented opener",
        "FS-b: Beta",
    ];
    assert_eq!(
        states(&lines),
        [
            (false, false),
            (false, false),
            (false, false),
            (false, false),
            (true, false),
            (true, false),
        ]
    );
}

/// The first `*/` closes the block, and a line after it is outside again.
#[test]
fn a_block_closes_at_the_first_closer() {
    let lines = [
        "/**",
        "FS-b: Beta */ tail */",
        "FS-c: Gamma",
        "/*! x */",
        "after",
    ];
    assert_eq!(
        states(&lines),
        [
            (true, false),
            (true, true),
            (false, false),
            (true, true),
            (false, false),
        ]
    );
}

/// A one-line block opens and closes on its line; the closer must follow the
/// opener, so `/*/` is no closer while `/**/` is.
#[test]
fn a_one_line_block_opens_and_closes_on_its_line() {
    assert_eq!(states(&["/** FS-b: Beta */"]), [(true, true)]);
    assert_eq!(states(&["/**/"]), [(true, true)]);
    assert_eq!(states(&["/*/", "*/"]), [(true, false), (true, true)]);
}

/// A block with no `*/` runs to the end of the file, and none of its bare lines
/// is in declaration position; a closed block's bare lines are, and its opener,
/// starred and closer lines are not bare.
#[test]
fn only_a_closed_block_has_bare_lines() {
    let config = numbered_config(test_root("only_a_closed_block_has_bare_lines"));
    let closed = ["/**", "FS-b: Beta", " * starred", "*/", "/*", "FS-c: Gamma"];
    assert_eq!(
        bare_block_lines(&closed.join("\n"), false, false, config.lexical()),
        [false, true, false, false, false, false]
    );
}

/// `comment_blocks` reads its `/* … */` spans off the line reader's state: each
/// block starts where the state opens and ends where it closes, or at the end of
/// the file when it never does.
#[test]
fn comment_block_spans_equal_the_state_spans() {
    let config = numbered_config(test_root("comment_block_spans_equal_the_state_spans"));
    let lines = [
        "/**",
        "FS-b: Beta",
        "*/",
        "code();",
        "/** one */",
        "/*",
        "never closed",
    ];
    let blocks = comment_blocks(&lines, false, config.lexical())
        .into_iter()
        .filter_map(|(start, end, kind)| match kind {
            CommentBlockKind::Block { closed } => Some((start, end, closed)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(blocks, [(0, 2, true), (4, 4, true), (5, 6, false)]);

    let mut spans = Vec::new();
    let mut open = None;
    for (index, (inside, closes)) in states(&lines).into_iter().enumerate() {
        if inside && open.is_none() {
            open = Some(index);
        }
        if inside && closes {
            spans.push((open.take().unwrap(), index, true));
        }
    }
    if let Some(start) = open {
        spans.push((start, lines.len() - 1, false));
    }
    assert_eq!(blocks, spans);
}
