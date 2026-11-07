//! Every code stands on one side of §FS-scan.identity.findings, §AR-scanner.1.8.

use super::*;

/// A new code fails here until someone classifies it, which is what having no default means.
#[test]
fn every_code_has_exactly_one_side() {
    for code in Code::ALL {
        let sides = SIDES
            .iter()
            .filter(|(_, codes)| codes.contains(code))
            .count();
        assert_eq!(sides, 1, "{code:?} stands on {sides} sides");
    }
    let classified: usize = SIDES.iter().map(|(_, codes)| codes.len()).sum();
    assert_eq!(classified, Code::ALL.len(), "a side lists a code twice");
}

/// The four the first version left with no side, which a resolution has to name.
#[test]
fn the_four_read_off_the_text_are_content() {
    for code in [
        Code::UNREACHED_DECLARATION,
        Code::SECTION_ADDRESS_CHANGED,
        Code::MISSING_SNAPSHOT,
        Code::UNCITED_UNIT,
    ] {
        assert_eq!(Side::of(code), Some(Side::Content), "{code:?}");
    }
}
