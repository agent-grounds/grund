//! Test module: the one bracket test of §FS-citations.wrap.recognized and the one
//! end it returns for a wrap's destination, the `)` that balances its `(`.

use super::*;

fn wrap_of(line: &str) -> Option<fmt_cross_refs::CrossRefWrap> {
    let start = line.find('\u{a7}').expect("a citation");
    let end = start + "\u{a7}FS-x".len();
    cross_ref_wrap(line, start, end)
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
