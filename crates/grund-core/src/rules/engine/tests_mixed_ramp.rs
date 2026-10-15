//! §FS-rules.7.12: mixed sites are ordinary when computing the window that remains.
//! These assertions pin eligibility as well as the clause, so merely stripping
//! the mixed clause after routing is insufficient.

use super::super::{Cardinality, RuleLevel};
use super::ramp::{Marks, Windows, route};
use crate::model::Diagnostic;

const MATCH: &str = "; a target pinned at the rule's own nested alias path now matches, and this warning becomes an error in grund 0.19.0";
const AT_MOST_ONE: Cardinality = Cardinality {
    minimum: None,
    maximum: Some(1),
};
const PLAIN: Marks = Marks {
    matched: false,
    mixed: false,
};
const MIXED: Marks = Marks {
    mixed: true,
    ..PLAIN
};
const MATCHED: Marks = Marks {
    matched: true,
    ..PLAIN
};

fn windows(sites: &[Marks]) -> Windows {
    Windows::of_count(RuleLevel::Required, AT_MOST_ONE, sites.iter().copied())
}

fn finding(message: String) -> Diagnostic {
    Diagnostic {
        code: "forbidden-citation",
        path: Some("docs/use.md".into()),
        line: Some(3),
        column: None,
        message,
        sites: Vec::new(),
        authority: vec!["RULE-r".to_string()],
    }
}

#[test]
fn a_count_only_the_mixed_site_breaks_is_an_ordinary_error() {
    let flags = windows(&[PLAIN, MIXED]);
    assert_eq!(flags, Windows::default());
    let (mut errors, mut warnings) = (Vec::new(), Vec::new());
    let (sink, message) = route(flags, &mut errors, &mut warnings, "ordinary message".into());
    sink.push(finding(message));
    assert!(warnings.is_empty());
    assert_eq!(errors[0].message, "ordinary message");
}

#[test]
fn a_count_that_fails_without_mixed_sites_stays_an_error() {
    assert_eq!(windows(&[PLAIN, PLAIN, MIXED]), Windows::default());
}

#[test]
fn a_recommended_count_owes_no_ramp() {
    assert_eq!(
        Windows::of_count(RuleLevel::Recommended, AT_MOST_ONE, [PLAIN, MIXED]),
        Windows::default()
    );
}

#[test]
fn removing_mixed_sites_cannot_make_a_matched_count_eligible() {
    // Even without the matched site, the two ordinary sites exceed the bound.
    assert_eq!(windows(&[PLAIN, MATCHED, MIXED]), Windows::default());
}

#[test]
fn an_independently_matched_mixed_site_keeps_only_alias_relief() {
    let flags = windows(&[
        PLAIN,
        Marks {
            matched: true,
            ..MIXED
        },
    ]);
    assert_eq!(
        flags,
        Windows {
            matched: true,
            ..Windows::default()
        }
    );
    let (mut errors, mut warnings) = (Vec::new(), Vec::new());
    let (sink, message) = route(flags, &mut errors, &mut warnings, "ordinary message".into());
    sink.push(finding(message));
    assert!(errors.is_empty());
    assert_eq!(warnings[0].message, format!("ordinary message{MATCH}"));
}

#[test]
fn mixed_ordinary_sites_do_not_erase_the_remaining_relief() {
    // The matched sites alone repair this count; the mixed site counts as ordinary.
    let flags = windows(&[MIXED, MATCHED, MATCHED]);
    assert_eq!(
        flags,
        Windows {
            matched: true,
            ..Windows::default()
        }
    );
    let (mut errors, mut warnings) = (Vec::new(), Vec::new());
    let (sink, message) = route(flags, &mut errors, &mut warnings, "ordinary message".into());
    sink.push(finding(message));
    assert!(errors.is_empty());
    assert_eq!(warnings[0].message, format!("ordinary message{MATCH}"));
}
