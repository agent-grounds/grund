//! Every code stands on one side of §FS-scan.identity.findings, §AR-scanner.1.8.

use super::*;
use crate::model::Site;

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

fn finding(code: Code, sites: Vec<Site>) -> Diagnostic {
    Diagnostic {
        code,
        path: None,
        line: None,
        column: None,
        message: String::new(),
        sites,
        authority: Vec::new(),
    }
}

/// A direction code is Place about a file and Content about a declaration, and the one mark of the
/// second is the site it carries. No other code splits, with or without sites.
#[test]
fn only_a_direction_code_splits_by_its_form() {
    let site = || Site {
        path: "docs/fs/a.md".into(),
        line: 1,
    };
    for code in Code::ALL.iter().copied() {
        let bare = Side::of_finding(&finding(code, Vec::new()));
        assert_eq!(bare, Side::of(code), "{code:?} with no site");
        let sited = Side::of_finding(&finding(code, vec![site()]));
        let want = if DIRECTION.contains(&code) {
            Some(Side::Content)
        } else {
            Side::of(code)
        };
        assert_eq!(sited, want, "{code:?} with a site");
    }
    for code in DIRECTION {
        assert_eq!(Side::of(*code), Some(Side::Place), "{code:?}");
    }
}
