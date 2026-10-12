//! Test module: the one bracket test of §FS-citations.wrap.recognized and the two
//! ends it returns for a wrap's destination until grund 0.19.0.

use super::*;

fn wrap_of(line: &str) -> Option<fmt_cross_refs::CrossRefWrap> {
    let start = line.find('\u{a7}').expect("a citation");
    let end = start + "\u{a7}FS-x".len();
    cross_ref_wrap(line, start, end)
}

/// A closed wrap is re-derived up to the first `)`, even where its parentheses
/// balance later; that first `)` is the end `show` collapses up to as well.
#[test]
fn a_closed_wrap_is_rederived_up_to_its_first_paren() {
    let line = "Balanced: [\u{a7}FS-x](old(1).md) after.";
    let wrap = wrap_of(line).expect("a wrap");
    assert_eq!(
        wrap.close,
        Some(line.find(".md)").unwrap() + 3),
        "balancing `)`"
    );
    assert_eq!(
        wrap.first_close,
        Some(line.find("1)").unwrap() + 1),
        "first `)`"
    );
    assert_eq!(wrap.rederived_close(), wrap.first_close);
    assert!(!wrap.unclosed_before_paren());
}

/// A wrap that never closes, with a later `)` on its line, is left as written and
/// is the line `unclosed-wrapper` warns at; one with no `)` at all is wrapped fresh.
#[test]
fn an_unclosed_wrap_has_no_rederived_end() {
    let aside =
        wrap_of("Then [\u{a7}FS-x](stale-dest and an aside (keep me) end.").expect("a wrap");
    assert_eq!(aside.close, None);
    assert!(aside.first_close.is_some());
    assert_eq!(aside.rederived_close(), None);
    assert!(aside.unclosed_before_paren());

    let bare = wrap_of("No close at all: [\u{a7}FS-x](stale-dest and more").expect("a wrap");
    assert_eq!((bare.close, bare.first_close), (None, None));
    assert!(!bare.unclosed_before_paren());
}

/// No `[` before the citation, or no `](` after it, is no wrap.
#[test]
fn a_citation_without_both_brackets_is_no_wrap() {
    assert_eq!(wrap_of("Plain \u{a7}FS-x](x) here."), None);
    assert_eq!(wrap_of("Half [\u{a7}FS-x] (x) here."), None);
    assert_eq!(wrap_of("\u{a7}FS-x](x) at the start."), None);
}
