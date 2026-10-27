//! The counted-section window closed at 0.18.0 (§FS-rules.7.8), over hand-built
//! facts: a required finding that only a numbered-section site produces, a site
//! §FS-rules.5.1 counts, is the same code's ordinary error with no clause after
//! its message, and nothing is left on the warnings channel.

use super::engine::{evaluate, evaluate_suggestions};
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use super::{
    Cardinality, ParsedRule, RuleAnchor, RuleLevel, RulePolarity, RuleRelation, RuleSubject,
    RuleTargets, TargetMode,
};
use crate::model::Diagnostic;
use std::collections::BTreeMap;

/// The clause the window's warnings ended with, which no finding carries any more.
const CLAUSE: &str = "; a citation to a numbered section now counts";

fn anchor(line: usize) -> RuleAnchor {
    RuleAnchor {
        path: "docs/bench.md".into(),
        line,
        column: None,
    }
}

/// `BENCH-a` cites `GOAL-a` twice: site `old` at line 7, and site `new` at line 8
/// through a numbered section, which counts like any other (§FS-rules.5.1). The
/// producer marks it `newly_counted` until that mark is deleted with the window.
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
    let site = |key: &str, line| {
        (
            SiteKey(key.into()),
            SiteMeta {
                label: "GOAL-a".into(),
                anchor: RuleAnchor {
                    column: Some(3),
                    ..anchor(line)
                },
            },
        )
    };
    RuleFacts {
        header: FactHeader {
            schema: 1,
            project: "test-project".into(),
            producer: "hand-built".into(),
            completeness: Completeness::Complete,
        },
        decl: vec![
            (NodeKey("bench".into()), "BENCH".into()),
            (NodeKey("goal".into()), "GOAL".into()),
        ],
        chapter: Vec::new(),
        contains: Vec::new(),
        cites: ["old", "new"]
            .map(|key| {
                (
                    SiteKey(key.into()),
                    NodeKey("bench".into()),
                    NodeKey("goal".into()),
                )
            })
            .into(),
        site_in: ["old", "new"]
            .map(|key| (SiteKey(key.into()), NodeKey("bench".into())))
            .into(),
        nodes: BTreeMap::from([node("bench", "BENCH-a", 1), node("goal", "GOAL-a", 20)]),
        sites: BTreeMap::from([site("old", 7), site("new", 8)]),
    }
}

/// `BENCH must cite at most <maximum> GOAL.`
fn at_most(maximum: usize, mode: TargetMode) -> ParsedRule {
    ParsedRule {
        origin: "RULE-bound".into(),
        anchor: anchor(30),
        subject: RuleSubject::Kind("BENCH".into(), None),
        level: RuleLevel::Required,
        polarity: RulePolarity::Positive,
        relation: RuleRelation::Cite,
        targets: RuleTargets::Kinds {
            values: vec!["GOAL".into()],
            mode,
        },
        cardinality: Cardinality {
            minimum: None,
            maximum: Some(maximum),
        },
    }
}

fn codes(diagnostics: &[Diagnostic]) -> Vec<(&str, Option<usize>)> {
    diagnostics
        .iter()
        .map(|d| (d.code.as_str(), d.line))
        .collect()
}

#[test]
fn a_count_only_the_numbered_section_site_breaks_is_an_error() {
    for mode in [TargetMode::Aggregate, TargetMode::PerTarget] {
        let errors = evaluate(&[at_most(1, mode)], &[], &facts());
        assert_eq!(codes(&errors), [("citation-cardinality", Some(1))]);
        assert!(
            !errors[0].message.contains(CLAUSE),
            "{mode:?}: {}",
            errors[0].message
        );
    }
}

#[test]
fn a_count_broken_without_the_numbered_section_site_stays_an_error() {
    for mode in [TargetMode::Aggregate, TargetMode::PerTarget] {
        let errors = evaluate(&[at_most(0, mode)], &[], &facts());
        assert_eq!(codes(&errors), [("citation-cardinality", Some(1))]);
        assert!(!errors[0].message.contains(CLAUSE));
    }
}

#[test]
fn an_inbound_count_only_the_numbered_section_site_breaks_is_an_error() {
    let inbound = ParsedRule {
        subject: RuleSubject::Kind("GOAL".into(), None),
        relation: RuleRelation::BeCitedBy,
        targets: RuleTargets::Kinds {
            values: vec!["BENCH".into()],
            mode: TargetMode::Aggregate,
        },
        ..at_most(1, TargetMode::Aggregate)
    };
    let errors = evaluate(&[inbound], &[], &facts());
    assert_eq!(codes(&errors), [("uncited-unit", Some(20))]);
    assert!(!errors[0].message.contains(CLAUSE));
}

#[test]
fn a_prohibition_is_an_error_at_the_numbered_section_site_too() {
    let ban = ParsedRule {
        polarity: RulePolarity::Prohibiting,
        cardinality: Cardinality::NONE,
        ..at_most(0, TargetMode::Aggregate)
    };
    let errors = evaluate(std::slice::from_ref(&ban), &[], &facts());
    assert_eq!(
        codes(&errors),
        [
            ("forbidden-citation", Some(7)),
            ("forbidden-citation", Some(8))
        ]
    );
    assert_eq!(
        errors[0].message, errors[1].message,
        "the site through a numbered section reads as the other one does"
    );
    let recommended = ParsedRule {
        level: RuleLevel::Recommended,
        ..ban
    };
    let suggestions = evaluate_suggestions(&[recommended], &[], &facts());
    assert_eq!(
        codes(&suggestions),
        [
            ("discouraged-citation", Some(7)),
            ("discouraged-citation", Some(8))
        ]
    );
    assert!(suggestions.iter().all(|d| !d.message.contains(CLAUSE)));
}
