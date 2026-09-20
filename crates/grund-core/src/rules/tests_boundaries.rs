//! Executable parser/facts/engine boundary drivers (§AR-rules.6).

use super::RuleAnchor;
use super::engine::evaluate;
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use super::markdown::adapt_markdown;
use super::sentence::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    RuleVocabulary, TargetMode, parse_rule,
};
use crate::testing::{legacy_fs_folder_config, scan_findings, test_root, write};
use std::collections::{BTreeMap, BTreeSet};

fn anchor(path: &str, line: usize) -> RuleAnchor {
    RuleAnchor {
        path: path.into(),
        line,
        column: None,
    }
}

fn rule() -> ParsedRule {
    ParsedRule {
        origin: "RULE-fs".into(),
        anchor: anchor("docs/rules.md", 1),
        subject: RuleSubject::Kind("FS".into()),
        level: RuleLevel::Required,
        polarity: RulePolarity::Positive,
        relation: RuleRelation::Cite,
        targets: RuleTargets::Kinds {
            values: vec!["GOAL".into()],
            mode: TargetMode::Aggregate,
        },
        cardinality: Cardinality::AT_LEAST_ONE,
    }
}

fn facts(completeness: Completeness) -> RuleFacts {
    let fs = NodeKey("opaque-fs".into());
    let goal = NodeKey("opaque-goal".into());
    RuleFacts {
        header: FactHeader {
            schema: 1,
            project: "demo".into(),
            producer: "test".into(),
            completeness,
        },
        decl: vec![(fs.clone(), "FS".into()), (goal.clone(), "GOAL".into())],
        chapter: Vec::new(),
        contains: Vec::new(),
        cites: Vec::new(),
        site_in: Vec::new(),
        nodes: BTreeMap::from([
            (
                fs,
                NodeMeta {
                    label: "FS-demo".into(),
                    anchor: anchor("docs/fs.md", 2),
                },
            ),
            (
                goal,
                NodeMeta {
                    label: "GOAL-demo".into(),
                    anchor: anchor("docs/goals.md", 3),
                },
            ),
        ]),
        sites: BTreeMap::new(),
    }
}

#[test]
fn sentence_front_end_returns_complete_parsed_rule_without_facts_or_diagnostics() {
    let parsed = parse_rule(
        "The requirements chapter of each FS should cite exactly one REQ.",
        "RULE-one".into(),
        anchor("docs/rules.md", 7),
        &RuleVocabulary {
            kinds: BTreeSet::from(["FS".into(), "REQ".into()]),
            target_kinds: BTreeSet::from(["FS".into(), "REQ".into()]),
            named_sections: true,
            id_grammars: Vec::new(),
        },
    )
    .expect("released sentence");
    assert_eq!(parsed.origin, "RULE-one");
    assert_eq!(parsed.anchor, anchor("docs/rules.md", 7));
    assert_eq!(
        parsed.subject,
        RuleSubject::ChapterOfKind {
            kind: "FS".into(),
            name: "requirements".into()
        }
    );
    assert_eq!(parsed.level, RuleLevel::Recommended);
    assert_eq!(parsed.polarity, RulePolarity::Positive);
    assert_eq!(parsed.relation, RuleRelation::Cite);
    assert_eq!(
        parsed.targets,
        RuleTargets::Kinds {
            values: vec!["REQ".into()],
            mode: TargetMode::Aggregate
        }
    );
    assert_eq!(
        parsed.cardinality,
        Cardinality {
            minimum: Some(1),
            maximum: Some(1)
        }
    );
}

#[test]
fn logic_engine_evaluates_hand_built_rule_and_facts_without_parser_or_scanner() {
    let diagnostics = evaluate(&[rule()], &facts(Completeness::Complete));
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].diagnostic.code, "missing-citation");
    assert_eq!(diagnostics[0].diagnostic.line, Some(2));
    assert_eq!(
        diagnostics[0].diagnostic.message,
        "FS-demo must cite GOAL (RULE-fs)"
    );
}

#[test]
fn markdown_adapter_and_second_producer_drive_the_same_engine_result() {
    let root = test_root("rules_producer_replacement");
    write(
        &root.join("docs/functional-spec/FS-001-demo.md"),
        "# FS-001-demo: Demo\n\nNo goal citation.\n",
    );
    let config = legacy_fs_folder_config(root.clone());
    let findings = scan_findings(&config, &root);
    let markdown = adapt_markdown(&findings, &config, true);
    // A producer replacement builds the schema independently: its opaque keys
    // and anchors deliberately share nothing with Markdown's (§AR-rules.6).
    let replacement_fs = NodeKey("replacement-node-17".into());
    let second = RuleFacts {
        header: FactHeader {
            schema: 1,
            project: "local".into(),
            producer: "replacement-v1".into(),
            completeness: Completeness::Complete,
        },
        decl: vec![(replacement_fs.clone(), "FS".into())],
        chapter: Vec::new(),
        contains: Vec::new(),
        cites: Vec::new(),
        site_in: Vec::new(),
        nodes: BTreeMap::from([(
            replacement_fs,
            NodeMeta {
                label: "FS-001-demo".into(),
                anchor: anchor("replacement/spec.graph", 41),
            },
        )]),
        sites: BTreeMap::new(),
    };
    let left = evaluate(&[rule()], &markdown)
        .into_iter()
        .map(|d| (d.diagnostic.code, d.diagnostic.message))
        .collect::<Vec<_>>();
    let right = evaluate(&[rule()], &second)
        .into_iter()
        .map(|d| (d.diagnostic.code, d.diagnostic.message))
        .collect::<Vec<_>>();
    assert_eq!(left, right);
}

#[test]
fn incomplete_fact_snapshot_suppresses_absence_and_count_conclusions() {
    assert_eq!(evaluate(&[rule()], &facts(Completeness::Complete)).len(), 1);
    assert!(evaluate(&[rule()], &facts(Completeness::Incomplete)).is_empty());

    let fs = NodeKey("partial-fs".into());
    let ar = NodeKey("partial-ar".into());
    let site = SiteKey("partial-site".into());
    let partial = RuleFacts {
        header: FactHeader {
            schema: 1,
            project: "demo".into(),
            producer: "partial-producer".into(),
            completeness: Completeness::Incomplete,
        },
        decl: vec![(fs.clone(), "FS".into()), (ar.clone(), "AR".into())],
        chapter: Vec::new(),
        contains: Vec::new(),
        cites: vec![(site.clone(), fs.clone(), ar.clone())],
        site_in: vec![(site.clone(), fs.clone())],
        nodes: BTreeMap::from([
            (
                fs,
                NodeMeta {
                    label: "FS-partial".into(),
                    anchor: anchor("facts/spec.graph", 3),
                },
            ),
            (
                ar,
                NodeMeta {
                    label: "AR-partial".into(),
                    anchor: anchor("facts/architecture.graph", 5),
                },
            ),
        ]),
        sites: BTreeMap::from([(
            site,
            SiteMeta {
                label: "AR-partial".into(),
                anchor: anchor("facts/spec.graph", 7),
            },
        )]),
    };
    let exact_count = ParsedRule {
        targets: RuleTargets::Kinds {
            values: vec!["AR".into()],
            mode: TargetMode::Aggregate,
        },
        cardinality: Cardinality {
            minimum: Some(2),
            maximum: Some(2),
        },
        ..rule()
    };
    let prohibition = ParsedRule {
        polarity: RulePolarity::Prohibiting,
        targets: RuleTargets::Kinds {
            values: vec!["AR".into()],
            mode: TargetMode::Aggregate,
        },
        cardinality: Cardinality::NONE,
        ..rule()
    };
    let diagnostics = evaluate(&[rule(), exact_count, prohibition], &partial);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].diagnostic.code, "forbidden-citation");
}
