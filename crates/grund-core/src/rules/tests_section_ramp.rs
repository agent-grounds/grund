//! The ramp split over hand-built facts (§FS-rules.7.8): a required finding
//! that only a newly counted numbered-section site produces warns and names its
//! error release; one the count breaks without those sites stays an error.

use super::engine::{evaluate, evaluate_suggestions};
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use super::{
    Cardinality, ParsedRule, RuleAnchor, RuleLevel, RulePolarity, RuleRelation, RuleSubject,
    RuleTargets, TargetMode,
};
use crate::model::Diagnostic;
use std::collections::BTreeMap;

const CLAUSE: &str = "; a citation to a numbered section now counts, and this warning becomes an error in grund 0.18.0";

fn anchor(line: usize) -> RuleAnchor {
    RuleAnchor {
        path: "docs/bench.md".into(),
        line,
        column: None,
    }
}

/// `BENCH-a` cites `GOAL-a` twice: site `old` at line 7 as before, and site
/// `new` at line 8 only because its numbered section now counts (§FS-rules.5.1).
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
    let site = |key: &str, line, newly_counted| {
        (
            SiteKey(key.into()),
            SiteMeta {
                label: "GOAL-a".into(),
                anchor: RuleAnchor {
                    column: Some(3),
                    ..anchor(line)
                },
                newly_counted,
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
        sites: BTreeMap::from([site("old", 7, false), site("new", 8, true)]),
    }
}

/// `BENCH must cite at most <maximum> GOAL.`
fn at_most(maximum: usize, mode: TargetMode) -> ParsedRule {
    ParsedRule {
        origin: "RULE-bound".into(),
        anchor: anchor(30),
        subject: RuleSubject::Kind("BENCH".into()),
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
    diagnostics.iter().map(|d| (d.code, d.line)).collect()
}

#[test]
fn a_count_only_the_newly_counted_site_breaks_warns() {
    for mode in [TargetMode::Aggregate, TargetMode::PerTarget] {
        let (errors, warnings) = evaluate(&[at_most(1, mode)], &[], &facts());
        assert!(errors.is_empty(), "{mode:?}: {:?}", codes(&errors));
        assert_eq!(codes(&warnings), [("citation-cardinality", Some(1))]);
        assert!(
            warnings[0].message.ends_with(CLAUSE),
            "{mode:?}: {}",
            warnings[0].message
        );
    }
}

#[test]
fn a_count_broken_without_the_newly_counted_site_stays_an_error() {
    for mode in [TargetMode::Aggregate, TargetMode::PerTarget] {
        let (errors, warnings) = evaluate(&[at_most(0, mode)], &[], &facts());
        assert!(warnings.is_empty(), "{mode:?}: {:?}", codes(&warnings));
        assert_eq!(codes(&errors), [("citation-cardinality", Some(1))]);
        assert!(!errors[0].message.contains(CLAUSE));
    }
}

#[test]
fn an_inbound_count_only_the_newly_counted_site_breaks_warns() {
    let inbound = ParsedRule {
        subject: RuleSubject::Kind("GOAL".into()),
        relation: RuleRelation::BeCitedBy,
        targets: RuleTargets::Kinds {
            values: vec!["BENCH".into()],
            mode: TargetMode::Aggregate,
        },
        ..at_most(1, TargetMode::Aggregate)
    };
    let (errors, warnings) = evaluate(&[inbound], &[], &facts());
    assert!(errors.is_empty(), "{:?}", codes(&errors));
    assert_eq!(codes(&warnings), [("uncited-unit", Some(20))]);
    assert!(warnings[0].message.ends_with(CLAUSE));
}

#[test]
fn a_prohibition_warns_at_the_newly_counted_site_only() {
    let ban = ParsedRule {
        polarity: RulePolarity::Prohibiting,
        cardinality: Cardinality::NONE,
        ..at_most(0, TargetMode::Aggregate)
    };
    let (errors, warnings) = evaluate(std::slice::from_ref(&ban), &[], &facts());
    assert_eq!(codes(&errors), [("forbidden-citation", Some(7))]);
    assert_eq!(codes(&warnings), [("forbidden-citation", Some(8))]);
    assert_eq!(
        warnings[0].message,
        format!("{}{CLAUSE}", errors[0].message),
        "the warning is the error's message unchanged, then the clause"
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
