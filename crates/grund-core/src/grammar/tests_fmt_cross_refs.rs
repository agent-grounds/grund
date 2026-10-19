//! Test module: the one bracket test of §FS-citations.wrap.recognized and the one
//! end it returns for a wrap's destination, the `)` that balances its `(`; and the
//! three-way reading beside it (§FS-citations.wrap): a wrap, a citation inside an
//! enclosing link (§FS-citations.wrap.enclosing), or unwrapped.

use super::*;

fn wrap_of(line: &str) -> Option<fmt_cross_refs::CrossRefWrap> {
    let start = line.find('\u{a7}').expect("a citation");
    let end = start + "\u{a7}FS-x".len();
    fmt_cross_refs::cross_ref_wrap(line, start, end)
}

/// A closed wrap ends at the `)` that balances its `(`, not at the first `)`;
/// that is the end `fmt` re-derives up to and `show` collapses up to.
#[test]
fn a_closed_wrap_ends_at_its_balancing_paren() {
    let line = "Balanced: [\u{a7}FS-x](old(1).md) after.";
    let wrap = wrap_of(line).expect("a wrap");
    assert_eq!(wrap.url_start, line.find("old").unwrap());
    assert_eq!(wrap.close, line.find(".md)").unwrap() + 3, "balancing `)`");
}

/// A `[§ID](` that never closes is no wrap, whether or not a later `)` stands on
/// its line: `fmt` wraps it fresh and `show` prints it as written.
#[test]
fn an_unclosed_wrap_is_no_wrap() {
    assert_eq!(
        wrap_of("Then [\u{a7}FS-x](stale-dest and an aside (keep me) end."),
        None
    );
    assert_eq!(
        wrap_of("No close at all: [\u{a7}FS-x](stale-dest and more"),
        None
    );
}

/// No `[` before the citation, or no `](` after it, is no wrap.
#[test]
fn a_citation_without_both_brackets_is_no_wrap() {
    assert_eq!(wrap_of("Plain \u{a7}FS-x](x) here."), None);
    assert_eq!(wrap_of("Half [\u{a7}FS-x] (x) here."), None);
    assert_eq!(wrap_of("\u{a7}FS-x](x) at the start."), None);
}

/// The reading of the `nth` citation `\u{a7}FS-x` on `line`.
fn reading_at(line: &str, nth: usize) -> CitationLink {
    let start = line.match_indices('\u{a7}').nth(nth).expect("a citation").0;
    citation_link(line, start, start + "\u{a7}FS-x".len())
}

fn encloses(reading: CitationLink) -> bool {
    matches!(reading, CitationLink::Enclosing { .. })
}

/// §FS-citations.wrap: the three readings on one line, each with the extent its
/// caller acts on — the wrap's destination, and the enclosing link's.
#[test]
fn a_wrap_an_enclosing_link_and_an_unwrapped_citation() {
    let line = "A [\u{a7}FS-x](a.md) wrap, [the \u{a7}FS-x spec](b.md) link, and \u{a7}FS-x bare.";
    let wrap_url = line.find("a.md").unwrap();
    assert_eq!(
        reading_at(line, 0),
        CitationLink::Wrap(fmt_cross_refs::CrossRefWrap {
            url_start: wrap_url,
            close: wrap_url + "a.md".len(),
        })
    );
    let link_url = line.find("b.md").unwrap();
    assert_eq!(
        reading_at(line, 1),
        CitationLink::Enclosing {
            url_start: link_url,
            close: link_url + "b.md".len(),
        }
    );
    assert_eq!(reading_at(line, 2), CitationLink::Unwrapped);
}

/// §FS-citations.wrap.enclosing.wrap-wins: a citation whose own brackets make a
/// wrap is one whatever stands around it, so the line an older `fmt` nested reads
/// its inner wrap, and the citation-only image is still read as a wrap
/// (§FS-citations.wrap.enclosing.images).
#[test]
fn a_wrap_wins_over_the_brackets_around_it() {
    for line in [
        "- [[\u{a7}FS-x](a.md#x): the title](x.md)",
        "See ![\u{a7}FS-x](diagram.png).",
    ] {
        assert!(
            matches!(reading_at(line, 0), CitationLink::Wrap(_)),
            "`{line}` is a wrap"
        );
    }
}

/// §FS-citations.wrap.enclosing.shape: every shape that encloses — an escaped and
/// a balanced bracket, a code span holding `]`, a link and an image before or
/// around the citation in the text, an escaped and a balanced parenthesis in the
/// destination, and an empty destination.
#[test]
fn every_enclosing_shape_encloses() {
    for line in [
        "[\u{a7}FS-x: in \\] for good](a.md)",
        "[\u{a7}FS-x: in [draft]](a.md)",
        "[\u{a7}FS-x: run `]` now](a.md)",
        "[a [b](c.md) and \u{a7}FS-x](a.md)",
        "[see ![a \u{a7}FS-x](i.png) here](a.md)",
        "[the \u{a7}FS-x spec](a\\).md)",
        "[the \u{a7}FS-x spec](a(1).md)",
    ] {
        assert!(encloses(reading_at(line, 0)), "`{line}` encloses");
    }
    let empty = "[the \u{a7}FS-x spec]()";
    let url_start = empty.find("()").unwrap() + 1;
    assert_eq!(
        reading_at(empty, 0),
        CitationLink::Enclosing {
            url_start,
            close: url_start,
        },
        "an empty destination encloses, and says it is empty"
    );
}

/// §FS-citations.wrap.enclosing.shape, §FS-citations.wrap.enclosing.images,
/// §FS-citations.wrap.enclosing.scope: what opens or closes no link leaves the
/// citation unwrapped — an escaped opener, an opener in a code span, an image's
/// description, a destination that never closes, a reference link, a definition
/// label, text that never closes, and brackets that close before the citation.
#[test]
fn what_makes_no_enclosing_link_leaves_the_citation_unwrapped() {
    for line in [
        "\\[the \u{a7}FS-x spec](a.md)",
        "`[` \u{a7}FS-x](a.md)",
        "![a diagram of \u{a7}FS-x](d.png)",
        "[the \u{a7}FS-x spec](a.md",
        "[the \u{a7}FS-x rationale][pg]",
        "[\u{a7}FS-x]: a.md",
        "[the \u{a7}FS-x",
        "[the] \u{a7}FS-x (a.md)",
    ] {
        assert_eq!(reading_at(line, 0), CitationLink::Unwrapped, "`{line}`");
    }
}

/// §FS-citations.wrap.enclosing.shape: only a citation in the text is enclosed;
/// one in a code span is code and one in a destination is a URL.
#[test]
fn a_citation_in_code_or_a_destination_is_in_no_text() {
    for line in [
        "[run `\u{a7}FS-x` now](a.md)",
        "[the record](a \u{a7}FS-x.md)",
    ] {
        assert_eq!(reading_at(line, 0), CitationLink::Unwrapped, "`{line}`");
    }
}

/// §FS-citations.wrap.enclosing.shape: each occurrence is read by its own position.
#[test]
fn each_occurrence_is_read_by_its_own_position() {
    let line =
        "[the \u{a7}FS-x spec](a.md), and \u{a7}FS-x itself, [\u{a7}FS-x and \u{a7}FS-x](b.md).";
    assert!(encloses(reading_at(line, 0)));
    assert_eq!(reading_at(line, 1), CitationLink::Unwrapped);
    assert!(encloses(reading_at(line, 2)));
    assert!(encloses(reading_at(line, 3)));
}
