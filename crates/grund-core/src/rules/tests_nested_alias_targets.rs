//! A target pinned at the rule's own nested member by its whole alias path
//! matches what the bare kind matches (§FS-rules.2, §FS-config.3.9.3), in both
//! relations; a required finding only that corrected match produces warns and
//! names its error release, one the rule produces without it stays an error
//! (§FS-rules.7.9).

use super::engine::{evaluate, evaluate_suggestions};
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use super::{
    Cardinality, ParsedRule, RuleAnchor, RuleLevel, RulePolarity, RuleRelation, RuleSubject,
    RuleTargets, TargetMode,
};
use crate::model::Diagnostic;
use std::collections::BTreeMap;

const CLAUSE: &str = "; a target pinned at the rule's own nested alias path now matches, and this warning becomes an error in grund 0.18.0";

fn anchor(line: usize) -> RuleAnchor {
    RuleAnchor {
        path: "docs/api.md".into(),
        line,
        column: None,
    }
}

/// The facts of member `group/api` (§FS-workspace.6.1): `API-a` cites its own
/// `AR-b` at line 3 and sibling member `group/web`'s `AR-c` at line 4. Local
/// kinds are bare and foreign ones carry the alias path, as the adapter writes them.
fn facts() -> RuleFacts {
    let node = |key: &str, label: &str, line| {
        (
            NodeKey(key.into()),
            NodeMeta {
                label: label.into(),
                anchor: anchor(line),
            },
        )
    };
    let site = |key: &str, label: &str, line| {
        (
            SiteKey(key.into()),
            SiteMeta {
                label: label.into(),
                anchor: RuleAnchor {
                    column: Some(3),
                    ..anchor(line)
                },
                newly_counted: false,
            },
        )
    };
    RuleFacts {
        header: FactHeader {
            schema: 1,
            project: "group/api".into(),
            producer: "hand-built".into(),
            completeness: Completeness::Complete,
        },
        decl: vec![
            (NodeKey("api".into()), "API".into()),
            (NodeKey("ar".into()), "AR".into()),
            (NodeKey("web".into()), "group/web/AR".into()),
        ],
        chapter: Vec::new(),
        contains: Vec::new(),
        cites: vec![
            (
                SiteKey("own".into()),
                NodeKey("api".into()),
                NodeKey("ar".into()),
            ),
            (
                SiteKey("sibling".into()),
                NodeKey("api".into()),
                NodeKey("web".into()),
            ),
        ],
        site_in: vec![
            (SiteKey("own".into()), NodeKey("api".into())),
            (SiteKey("sibling".into()), NodeKey("api".into())),
        ],
        nodes: BTreeMap::from([
            node("api", "API-a", 1),
            node("ar", "AR-b", 20),
            node("web", "group/web/AR-c", 30),
        ]),
        sites: BTreeMap::from([site("own", "AR-b", 3), site("sibling", "group/web/AR-c", 4)]),
    }
}

/// `Each API must cite …` over `targets`, with the given count.
fn cites(targets: &[&str], cardinality: Cardinality) -> ParsedRule {
    ParsedRule {
        origin: "RULE-x".into(),
        anchor: anchor(40),
        subject: RuleSubject::Kind("API".into()),
        level: RuleLevel::Required,
        polarity: RulePolarity::Positive,
        relation: RuleRelation::Cite,
        targets: RuleTargets::Kinds {
            values: targets.iter().map(|t| t.to_string()).collect(),
            mode: TargetMode::Aggregate,
        },
        cardinality,
    }
}

/// `Each API must not cite any …` over `targets`.
fn bans(targets: &[&str]) -> ParsedRule {
    ParsedRule {
        polarity: RulePolarity::Prohibiting,
        ..cites(targets, Cardinality::NONE)
    }
}

/// `Each AR must be cited by …` over `targets`, with the given count.
fn cited_by(targets: &[&str], cardinality: Cardinality) -> ParsedRule {
    ParsedRule {
        subject: RuleSubject::Kind("AR".into()),
        relation: RuleRelation::BeCitedBy,
        ..cites(targets, cardinality)
    }
}

const AT_LEAST_ONE: Cardinality = Cardinality {
    minimum: Some(1),
    maximum: None,
};

fn at_most(maximum: usize) -> Cardinality {
    Cardinality {
        minimum: None,
        maximum: Some(maximum),
    }
}

fn codes(diagnostics: &[Diagnostic]) -> Vec<(&str, Option<usize>)> {
    diagnostics.iter().map(|d| (d.code, d.line)).collect()
}

#[test]
fn an_own_nested_alias_pin_satisfies_a_cite_rule_like_the_bare_kind() {
    for target in ["AR", "group/api/AR"] {
        let (errors, warnings) = evaluate(&[cites(&[target], AT_LEAST_ONE)], &[], &facts());
        assert!(errors.is_empty(), "{target}: {:?}", codes(&errors));
        assert!(warnings.is_empty(), "{target}: {:?}", codes(&warnings));
    }
}

#[test]
fn an_own_nested_alias_pin_satisfies_an_inbound_rule_like_the_bare_kind() {
    for target in ["API", "group/api/API"] {
        let (errors, warnings) = evaluate(&[cited_by(&[target], AT_LEAST_ONE)], &[], &facts());
        assert!(errors.is_empty(), "{target}: {:?}", codes(&errors));
        assert!(warnings.is_empty(), "{target}: {:?}", codes(&warnings));
    }
}

#[test]
fn a_prohibition_only_the_own_nested_alias_pin_reaches_warns() {
    let (errors, warnings) = evaluate(&[bans(&["group/api/AR"])], &[], &facts());
    assert!(errors.is_empty(), "{:?}", codes(&errors));
    assert_eq!(codes(&warnings), [("forbidden-citation", Some(3))]);
    assert!(
        warnings[0].message.ends_with(CLAUSE),
        "{}",
        warnings[0].message
    );
}

#[test]
fn an_inbound_ceiling_only_the_own_nested_alias_pin_breaks_warns() {
    let (errors, warnings) = evaluate(&[cited_by(&["group/api/API"], at_most(0))], &[], &facts());
    assert!(errors.is_empty(), "{:?}", codes(&errors));
    assert_eq!(codes(&warnings), [("uncited-unit", Some(20))]);
    assert!(
        warnings[0].message.ends_with(CLAUSE),
        "{}",
        warnings[0].message
    );
}

#[test]
fn a_count_only_the_corrected_match_breaks_warns_and_one_broken_without_it_errs() {
    let targets = ["group/api/AR", "group/web/AR"];
    let (errors, warnings) = evaluate(&[cites(&targets, at_most(1))], &[], &facts());
    assert!(errors.is_empty(), "{:?}", codes(&errors));
    assert_eq!(codes(&warnings), [("citation-cardinality", Some(1))]);
    assert!(
        warnings[0].message.ends_with(CLAUSE),
        "{}",
        warnings[0].message
    );

    let (errors, warnings) = evaluate(&[cites(&targets, at_most(0))], &[], &facts());
    assert!(warnings.is_empty(), "{:?}", codes(&warnings));
    assert_eq!(codes(&errors), [("citation-cardinality", Some(1))]);
    assert!(!errors[0].message.contains(CLAUSE));
}

#[test]
fn a_site_the_bare_kind_already_matched_stays_an_error() {
    let (errors, warnings) = evaluate(&[bans(&["AR", "group/api/AR"])], &[], &facts());
    assert!(warnings.is_empty(), "{:?}", codes(&warnings));
    assert_eq!(codes(&errors), [("forbidden-citation", Some(3))]);
    assert!(!errors[0].message.contains(CLAUSE));
}

#[test]
fn bare_kinds_any_member_and_other_member_pins_judge_as_before() {
    for (target, lines) in [
        ("AR", vec![3]),
        ("*/AR", vec![3, 4]),
        ("group/web/AR", vec![4]),
    ] {
        let (errors, warnings) = evaluate(&[bans(&[target])], &[], &facts());
        assert!(warnings.is_empty(), "{target}: {:?}", codes(&warnings));
        let found: Vec<_> = errors.iter().map(|d| (d.code, d.line.unwrap())).collect();
        let expected: Vec<_> = lines.iter().map(|&l| ("forbidden-citation", l)).collect();
        assert_eq!(found, expected, "{target}");
    }
}

#[test]
fn a_recommended_prohibition_through_the_pin_is_an_ordinary_suggestion() {
    let recommended = ParsedRule {
        level: RuleLevel::Recommended,
        ..bans(&["group/api/AR"])
    };
    let suggestions = evaluate_suggestions(&[recommended], &[], &facts());
    assert_eq!(codes(&suggestions), [("discouraged-citation", Some(3))]);
    assert!(!suggestions[0].message.contains(CLAUSE));
}
