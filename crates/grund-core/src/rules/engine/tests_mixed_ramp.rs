//! The window of a citation of a name under a number (§FS-rules.7.12): a count only
//! the mixed sites break warns and names 0.19.0, one that fails without them stays
//! an error, a finding also in an earlier window carries each clause in subsection
//! order, and the release it names is still ahead of this crate
//! (§FS-citations.checks.mixed-path).

use super::super::{Cardinality, RuleLevel};
use super::ramp::{LENT_CLAUSE, MIXED_CLAUSE, MIXED_PATH_FLIP, Marks, Windows, route};
use super::route_stubbed_rules;
use crate::model::Diagnostic;
use std::collections::BTreeSet;

const STUB: &str = "; a rule declared through a stub now applies, and this warning becomes an error in grund 0.19.0";

const AT_MOST_ONE: Cardinality = Cardinality {
    minimum: None,
    maximum: Some(1),
};

const PLAIN: Marks = Marks {
    matched: false,
    lent: false,
    mixed: false,
};
const MIXED: Marks = Marks {
    mixed: true,
    ..PLAIN
};
const LENT: Marks = Marks {
    lent: true,
    ..PLAIN
};

fn windows(sites: &[Marks]) -> Windows {
    Windows::of_count(RuleLevel::Required, AT_MOST_ONE, sites.iter().copied())
}

fn mixed_only() -> Windows {
    Windows {
        mixed: true,
        ..Windows::default()
    }
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

/// §FS-rules.7.12: the window closes at a release still to come; once this crate
/// reaches it, the window, its clauses and the site mark must go, and this test
/// says so, as `parse-moves` is held to its release.
#[test]
fn the_mixed_path_flip_is_still_ahead() {
    let parse = |version: &str| -> Vec<u64> {
        version
            .split(['.', '-'])
            .take(3)
            .map(|part| part.parse().expect("numeric version"))
            .collect()
    };
    assert!(
        parse(env!("CARGO_PKG_VERSION")) < parse(MIXED_PATH_FLIP),
        "grund {} has reached {MIXED_PATH_FLIP}: make the mixed-path window errors",
        env!("CARGO_PKG_VERSION")
    );
    assert!(MIXED_CLAUSE.ends_with(&format!("an error in grund {MIXED_PATH_FLIP}")));
}

#[test]
fn a_count_only_the_mixed_sites_break_warns_until_0_19_0() {
    assert_eq!(windows(&[PLAIN, MIXED]), mixed_only());
    let (mut out, mut ramp) = (Vec::new(), Vec::new());
    let (sink, message) = route(
        mixed_only(),
        &mut out,
        &mut ramp,
        "FS must not cite GOAL (RULE-r)".into(),
    );
    sink.push(finding(message));
    assert!(out.is_empty(), "a ramped finding leaves the errors");
    assert_eq!(
        ramp[0].message,
        format!("FS must not cite GOAL (RULE-r){}", *MIXED_CLAUSE)
    );
}

#[test]
fn a_count_that_fails_without_the_mixed_sites_stays_an_error() {
    assert_eq!(windows(&[PLAIN, PLAIN, MIXED]), Windows::default());
}

#[test]
fn a_recommended_count_owes_no_ramp() {
    let windows = Windows::of_count(RuleLevel::Recommended, AT_MOST_ONE, [PLAIN, MIXED]);
    assert_eq!(windows, Windows::default());
}

#[test]
fn a_count_no_window_alone_mends_names_lent_before_mixed() {
    let both = windows(&[PLAIN, LENT, MIXED]);
    assert_eq!(
        both,
        Windows {
            lent: true,
            mixed: true,
            ..Windows::default()
        }
    );
    let (mut out, mut ramp) = (Vec::new(), Vec::new());
    let (sink, message) = route(both, &mut out, &mut ramp, "FS-x cites GOAL 3 times".into());
    sink.push(finding(message));
    assert_eq!(
        ramp[0].message,
        format!("FS-x cites GOAL 3 times{LENT_CLAUSE}{}", *MIXED_CLAUSE)
    );
}

#[test]
fn a_mixed_site_a_stubbed_rule_forbids_names_the_stub_window_first() {
    let stubbed = BTreeSet::from(["RULE-r".to_string()]);
    let mut errors = Vec::new();
    let mut warnings = vec![finding(format!(
        "FS must not cite GOAL (RULE-r){LENT_CLAUSE}{}",
        *MIXED_CLAUSE
    ))];
    route_stubbed_rules(&stubbed, &mut errors, &mut warnings);
    assert_eq!(
        warnings[0].message,
        format!(
            "FS must not cite GOAL (RULE-r){STUB}{LENT_CLAUSE}{}",
            *MIXED_CLAUSE
        )
    );
}
