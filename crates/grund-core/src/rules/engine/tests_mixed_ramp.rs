//! §FS-rules.7.12: a mixed site is an ordinary site, and with the nested-alias
//! window closed in 0.19.0 (§FS-rules.7.9) no window is left to relieve a count
//! that mixed, plain and pin-matched sites break together: each such count is the
//! code's ordinary error, through a pinned target as through the bare kind.

use super::super::facts::{NodeKey, RuleFacts, SiteKey, SiteMeta};
use super::super::tests_nested_alias_targets::{
    Required, assert_ordinary, at_most, cites, cites_each, codes, facts,
};
use super::super::{ParsedRule, RuleAnchor, RuleLevel};
use super::{evaluate, evaluate_suggestions};

/// [`facts`] with `API-a` citing its own `AR-b` once more at line 5, through
/// the mixed path `<§>AR-b.goals.1`, a name under a number.
fn facts_with_mixed_site() -> RuleFacts {
    let mut facts = facts();
    let site = SiteKey("mixed".into());
    facts
        .cites
        .push((site.clone(), NodeKey("api".into()), NodeKey("ar".into())));
    facts.site_in.push((site.clone(), NodeKey("api".into())));
    facts.sites.insert(
        site,
        SiteMeta {
            label: "AR-b.goals.1".into(),
            anchor: RuleAnchor {
                path: "docs/api.md".into(),
                line: 5,
                column: Some(3),
            },
        },
    );
    facts
}

/// Every mode of `Each API must cite at most 1 <target>`.
fn at_most_one(target: &str) -> [ParsedRule; 2] {
    [
        cites(&[target], at_most(1)),
        cites_each(&[target], at_most(1)),
    ]
}

#[test]
fn a_count_a_mixed_and_a_pinned_site_break_is_an_ordinary_error() {
    for rule in at_most_one("group/api/AR") {
        let mode = rule.targets.clone();
        let errors = evaluate(&[rule], &[], &facts_with_mixed_site()).required();
        assert_eq!(
            codes(&errors),
            [("citation-cardinality", Some(1))],
            "{mode:?}"
        );
        assert_ordinary(&errors);
    }
}

#[test]
fn a_count_a_mixed_site_breaks_through_the_bare_kind_is_an_ordinary_error() {
    for rule in at_most_one("AR") {
        let mode = rule.targets.clone();
        let errors = evaluate(&[rule], &[], &facts_with_mixed_site()).required();
        assert_eq!(
            codes(&errors),
            [("citation-cardinality", Some(1))],
            "{mode:?}"
        );
        assert_ordinary(&errors);
    }
}

#[test]
fn a_bound_the_mixed_and_pinned_sites_keep_within_passes() {
    let errors = evaluate(
        &[cites(&["group/api/AR"], at_most(2))],
        &[],
        &facts_with_mixed_site(),
    )
    .required();
    assert!(errors.is_empty(), "{:?}", codes(&errors));
}

#[test]
fn a_recommended_count_is_an_ordinary_suggestion() {
    let recommended = ParsedRule {
        level: RuleLevel::Recommended,
        ..cites(&["group/api/AR"], at_most(1))
    };
    let suggestions = evaluate_suggestions(&[recommended], &[], &facts_with_mixed_site());
    assert_eq!(codes(&suggestions).len(), 1, "{:?}", codes(&suggestions));
    assert_ordinary(&suggestions);
}
