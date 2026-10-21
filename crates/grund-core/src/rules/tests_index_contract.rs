//! Index rewrite equivalence over opaque, hand-built facts (§FS-rules.5.3,
//! §AR-rules.3.1). These semantic guards pass before the optimization; the
//! failing cost regression is the optimized black-box test in tests/e2e/.

use super::engine::{evaluate, evaluate_suggestions};
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use super::{
    Cardinality, ParsedRule, RuleAnchor, RuleLevel, RulePolarity, RuleRelation, RuleSubject,
    RuleTargets, TargetMode,
};
use std::collections::BTreeMap;

fn key(value: &str) -> NodeKey {
    NodeKey(value.into())
}

fn anchor(line: usize) -> RuleAnchor {
    RuleAnchor {
        path: "producer/graph.data".into(),
        line,
        column: Some(4),
    }
}

fn facts() -> RuleFacts {
    // Keys deliberately do not encode ancestry or authored labels. Relations
    // are not sorted by keys, and site-key order is opposite physical order.
    let nodes = [
        ("z", "FS-one", 1),
        ("q", "FS-one.needs", 3),
        ("b", "FS-one.needs.detail", 5),
        ("h", "REQ-a", 10),
        ("j", "REQ-a.part", 12),
        ("a", "REQ-a.part.detail", 14),
        ("c", "REQ-b", 20),
    ];
    RuleFacts {
        header: FactHeader {
            schema: 1,
            project: "test-project".into(),
            producer: "independent-graph-producer".into(),
            completeness: Completeness::Complete,
        },
        decl: vec![
            (key("z"), "FS".into()),
            (key("h"), "REQ".into()),
            (key("c"), "REQ".into()),
        ],
        chapter: vec![
            (key("a"), "part.detail".into(), "Detail".into()),
            (key("b"), "needs.detail".into(), "Terms".into()),
            (key("q"), "needs".into(), "tErMs".into()),
            (key("j"), "part".into(), "Part".into()),
        ],
        contains: vec![
            (key("j"), key("a")),
            (key("q"), key("b")),
            (key("h"), key("j")),
            (key("z"), key("q")),
        ],
        cites: vec![
            (SiteKey("zz".into()), key("b"), key("a")),
            (SiteKey("aa".into()), key("b"), key("a")),
        ],
        site_in: ["zz", "aa"]
            .into_iter()
            .flat_map(|site| ["b", "q", "z"].map(|node| (SiteKey(site.into()), key(node))))
            .collect(),
        nodes: nodes
            .into_iter()
            .map(|(node, label, line)| {
                (
                    key(node),
                    NodeMeta {
                        label: label.into(),
                        anchor: anchor(line),
                    },
                )
            })
            .collect(),
        sites: BTreeMap::from([
            (
                SiteKey("zz".into()),
                SiteMeta {
                    label: "REQ-a.part.detail".into(),
                    anchor: anchor(7),
                },
            ),
            (
                SiteKey("aa".into()),
                SiteMeta {
                    label: "REQ-a.part.detail".into(),
                    anchor: anchor(8),
                },
            ),
        ]),
    }
}

fn rule(mode: TargetMode, count: usize) -> ParsedRule {
    ParsedRule {
        origin: "RULE-count".into(),
        anchor: anchor(30),
        subject: RuleSubject::Kind("FS".into()),
        level: RuleLevel::Required,
        polarity: RulePolarity::Positive,
        relation: RuleRelation::Cite,
        targets: RuleTargets::Kinds {
            values: vec!["REQ".into()],
            mode,
        },
        cardinality: Cardinality {
            minimum: Some(count),
            maximum: Some(count),
        },
    }
}

/// §FS-rules.5.1: repeated written targets are two physical sites, while a
/// site's membership in three containing units does not triple its count.
#[test]
fn aggregate_counts_physical_sites_in_nested_units_once_each() {
    for subject in [
        RuleSubject::Kind("FS".into()),
        RuleSubject::ChapterOfKind {
            kind: "FS".into(),
            name: "needs".into(),
        },
        RuleSubject::ExactChapter {
            declaration: "FS-one".into(),
            path: "needs.detail".into(),
            separator: ".".into(),
        },
    ] {
        let two = ParsedRule {
            subject,
            ..rule(TargetMode::Aggregate, 2)
        };
        assert!(evaluate(&[two.clone()], &[], &facts()).is_empty());
        let one = ParsedRule {
            cardinality: rule(TargetMode::Aggregate, 1).cardinality,
            ..two
        };
        let diagnostics = evaluate(&[one], &[], &facts());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "citation-cardinality");
        assert!(diagnostics[0].message.contains("cites REQ 2 times"));
    }
}

/// §FS-rules.5.2: nested targets count for their owning declaration; the
/// uncited target remains in the universe, in stable target-key order.
#[test]
fn per_target_counts_owners_and_keeps_zero_citation_targets() {
    let diagnostics = evaluate(&[rule(TargetMode::PerTarget, 1)], &[], &facts());
    let messages: Vec<_> = diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert_eq!(
        messages,
        vec![
            "FS-one cites REQ-b 0 times; RULE-count requires exactly once",
            "FS-one cites REQ-a 2 times; RULE-count requires exactly once",
        ]
    );
    assert!(
        diagnostics
            .iter()
            .all(|d| d.line == Some(1) && d.column == Some(4))
    );
    assert!(
        diagnostics
            .iter()
            .all(|d| d.path.as_deref() == Some(std::path::Path::new("producer/graph.data")))
    );
}

/// §FS-rules.3.1, §FS-rules.5.2: presence is case-insensitive display-name
/// matching of direct children; chapter subjects use the section handle.
#[test]
fn presence_uses_direct_children_and_display_names() {
    let presence = ParsedRule {
        relation: RuleRelation::HaveChapter,
        targets: RuleTargets::Chapter("Terms".into()),
        ..rule(TargetMode::Aggregate, 1)
    };
    assert!(evaluate(&[presence], &[], &facts()).is_empty());
    let nested = ParsedRule {
        subject: RuleSubject::ChapterOfKind {
            kind: "FS".into(),
            name: "detail".into(),
        },
        ..rule(TargetMode::Aggregate, 1)
    };
    let diagnostics = evaluate(&[nested], &[], &facts());
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "unreached-declaration");
    assert_eq!(diagnostics[0].line, Some(1));
}

/// §FS-rules.3.4: inbound counts recover the source kind through nested
/// containment but match the immediate target, not its owning declaration.
#[test]
fn inbound_recovers_source_kind_without_promoting_target() {
    let inbound = ParsedRule {
        subject: RuleSubject::ExactChapter {
            declaration: "REQ-a".into(),
            path: "part.detail".into(),
            separator: ".".into(),
        },
        relation: RuleRelation::BeCitedBy,
        targets: RuleTargets::Kinds {
            values: vec!["FS".into()],
            mode: TargetMode::Aggregate,
        },
        ..rule(TargetMode::Aggregate, 2)
    };
    assert!(evaluate(&[inbound.clone()], &[], &facts()).is_empty());
    let owner = ParsedRule {
        subject: RuleSubject::ExactDeclaration("REQ-a".into()),
        ..inbound
    };
    let diagnostics = evaluate(&[owner], &[], &facts());
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "uncited-unit");
    assert_eq!(diagnostics[0].line, Some(10));
    assert_eq!(
        diagnostics[0].message,
        "REQ-a is cited by FS 0 times; RULE-count requires exactly 2"
    );
}

/// §FS-rules.4, §FS-rules.7.5: known prohibited physical sites survive an
/// incomplete snapshot on both levels, ordered and anchored as authored.
#[test]
fn incomplete_snapshots_keep_prohibition_sites_and_levels() {
    let prohibition = ParsedRule {
        polarity: RulePolarity::Prohibiting,
        cardinality: Cardinality::NONE,
        ..rule(TargetMode::Aggregate, 1)
    };
    let mut partial = facts();
    partial.header.completeness = Completeness::Incomplete;
    for level in [RuleLevel::Required, RuleLevel::Recommended] {
        let positive = ParsedRule {
            level,
            ..rule(TargetMode::PerTarget, 1)
        };
        let ban = ParsedRule {
            level,
            ..prohibition.clone()
        };
        let diagnostics = if level == RuleLevel::Required {
            evaluate(&[positive, ban], &[], &partial)
        } else {
            evaluate_suggestions(&[positive, ban], &[], &partial)
        };
        assert_eq!(
            diagnostics.iter().map(|d| d.line).collect::<Vec<_>>(),
            vec![Some(7), Some(8)]
        );
        let expected = if level == RuleLevel::Required {
            "forbidden-citation"
        } else {
            "discouraged-citation"
        };
        assert!(
            diagnostics
                .iter()
                .all(|d| d.code == expected && d.column == Some(4))
        );
        assert!(
            diagnostics
                .iter()
                .all(|d| d.authority == vec!["RULE-count"])
        );
    }
}

/// §FS-rules.5.2: an empty target universe makes per-target coverage vacuous;
/// an empty subject universe produces neither counts nor unreached findings.
#[test]
fn empty_universes_are_not_inferred_from_citations() {
    let mut empty_targets = facts();
    empty_targets.decl.retain(|(_, kind)| kind == "FS");
    empty_targets
        .nodes
        .retain(|node, _| ["z", "q", "b"].contains(&node.0.as_str()));
    empty_targets
        .chapter
        .retain(|(node, _, _)| empty_targets.nodes.contains_key(node));
    empty_targets.contains.retain(|(parent, child)| {
        empty_targets.nodes.contains_key(parent) && empty_targets.nodes.contains_key(child)
    });
    empty_targets.cites.clear();
    empty_targets.site_in.clear();
    empty_targets.sites.clear();
    assert!(evaluate(&[rule(TargetMode::PerTarget, 1)], &[], &empty_targets).is_empty());
    let diagnostics = evaluate(&[rule(TargetMode::Aggregate, 1)], &[], &empty_targets);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("cites REQ 0 times"));

    let mut empty_subjects = facts();
    empty_subjects.decl.clear();
    empty_subjects.chapter.clear();
    empty_subjects.contains.clear();
    empty_subjects.cites.clear();
    empty_subjects.site_in.clear();
    empty_subjects.nodes.clear();
    empty_subjects.sites.clear();
    let chapter = ParsedRule {
        subject: RuleSubject::ChapterOfKind {
            kind: "FS".into(),
            name: "needs".into(),
        },
        ..rule(TargetMode::Aggregate, 1)
    };
    assert!(evaluate(&[chapter], &[], &empty_subjects).is_empty());
}
