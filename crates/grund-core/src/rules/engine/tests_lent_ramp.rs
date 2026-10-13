//! The window of a citation or chapter a stub's target outside the walk lends
//! (§FS-rules.7.11): a count only the lent sites break warns and names 0.19.0, one
//! that fails without them stays an error, a count it shares with §FS-rules.7.8
//! names each window the count needs in subsection order, and a finding
//! §FS-rules.7.10 also ramps carries that window's clause before this one.

use super::super::{Cardinality, RuleLevel};
use super::ramp::{LENT_CLAUSE, Marks, Windows, route};
use super::route_stubbed_rules;
use crate::model::Diagnostic;
use std::collections::BTreeSet;

const COUNTED: &str = "; a citation to a numbered section now counts, and this warning becomes an error in grund 0.18.0";
const STUB: &str = "; a rule declared through a stub now applies, and this warning becomes an error in grund 0.19.0";

const AT_MOST_ONE: Cardinality = Cardinality {
    minimum: None,
    maximum: Some(1),
};

const PLAIN: Marks = Marks {
    counted: false,
    matched: false,
    lent: false,
    mixed: false,
};
const LENT: Marks = Marks {
    lent: true,
    ..PLAIN
};
const COUNTED_SITE: Marks = Marks {
    counted: true,
    ..PLAIN
};
const COUNTED_AND_LENT: Marks = Marks {
    counted: true,
    lent: true,
    ..PLAIN
};

fn windows(sites: &[Marks]) -> Windows {
    Windows::of_count(RuleLevel::Required, AT_MOST_ONE, sites.iter().copied())
}

fn lent_only() -> Windows {
    Windows {
        lent: true,
        ..Windows::default()
    }
}

#[test]
fn a_count_only_the_lent_sites_break_warns_until_0_19_0() {
    assert_eq!(windows(&[PLAIN, LENT]), lent_only());
}

#[test]
fn a_count_that_fails_without_the_lent_sites_stays_an_error() {
    assert_eq!(windows(&[PLAIN, PLAIN, LENT]), Windows::default());
}

#[test]
fn a_recommended_count_owes_no_ramp() {
    let sites = [PLAIN, LENT];
    let windows = Windows::of_count(RuleLevel::Recommended, AT_MOST_ONE, sites);
    assert_eq!(windows, Windows::default());
}

#[test]
fn a_count_that_dropping_the_lent_sites_alone_mends_names_only_that_window() {
    // Without the lent sites one plain site is left; without the newly counted
    // one, a plain and a lent site, which still break it (§FS-rules.7.11).
    assert_eq!(windows(&[PLAIN, LENT, COUNTED_AND_LENT]), lent_only());
}

#[test]
fn a_count_no_window_alone_mends_names_both_in_subsection_order() {
    let both = windows(&[PLAIN, COUNTED_SITE, LENT]);
    assert_eq!(
        both,
        Windows {
            counted: true,
            lent: true,
            ..Windows::default()
        }
    );
    let (mut out, mut ramp) = (Vec::new(), Vec::new());
    let (sink, message) = route(both, &mut out, &mut ramp, "FS-x cites GOAL 3 times".into());
    sink.push(finding(message));
    assert!(out.is_empty(), "a ramped count leaves the errors");
    assert_eq!(
        ramp[0].message,
        format!("FS-x cites GOAL 3 times{COUNTED}{LENT_CLAUSE}")
    );
}

#[test]
fn a_finding_about_a_lent_chapter_is_in_the_window_whatever_its_count() {
    let windows = Windows::default().of_subject(RuleLevel::Required, true);
    assert_eq!(windows, lent_only());
    let recommended = Windows::default().of_subject(RuleLevel::Recommended, true);
    assert_eq!(recommended, Windows::default());
}

fn finding(message: String) -> Diagnostic {
    Diagnostic {
        code: "forbidden-citation",
        path: Some("src/x.rs".into()),
        line: Some(3),
        column: None,
        message,
        sites: Vec::new(),
        authority: vec!["RULE-r".to_string()],
    }
}

#[test]
fn a_lent_site_a_stubbed_rule_forbids_names_the_stub_window_first() {
    let stubbed = BTreeSet::from(["RULE-r".to_string()]);
    let mut errors = Vec::new();
    let mut warnings = vec![finding(format!(
        "FS must not cite GOAL (RULE-r){LENT_CLAUSE}"
    ))];
    route_stubbed_rules(&stubbed, &mut errors, &mut warnings);
    assert_eq!(
        warnings[0].message,
        format!("FS must not cite GOAL (RULE-r){STUB}{LENT_CLAUSE}")
    );
}
