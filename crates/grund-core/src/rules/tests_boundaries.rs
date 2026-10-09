//! Executable parser/facts/engine boundary drivers (§AR-rules.6).

use super::RuleAnchor;
use super::engine::{evaluate, evaluate_suggestions};
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use super::markdown::adapt_markdown;
use super::sentence::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    RuleVocabulary, TargetMode, parse_rule,
};
use crate::checker::markdown_project;
use crate::resolver::WorkspaceCheckTarget;
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

/// §FS-rules.3.5.1, §FS-errors.3: whitespace at any position remains refused,
/// with the entire released reason followed by actionable guidance. Between
/// them stands the typed sentence with the `NAME` trimmed, where trimming
/// leaves a token, and nothing where the `NAME` is empty or holds whitespace.
#[test]
fn chapter_diagnostic_parser_refuses_whitespace_anywhere_in_presence_name() {
    let vocabulary = RuleVocabulary {
        kinds: BTreeSet::from(["FS".into()]),
        target_kinds: BTreeSet::from(["FS".into()]),
        target_namespaces: BTreeMap::new(),
        named_sections: true,
        id_grammars: Vec::new(),
        section_separators: vec![".".into()],
    };
    let reason = "chapter name must be a non-empty NAME with no surrounding whitespace; ";
    let trimmed = "accepted form: Each FS must have exactly one goal chapter. ";
    for (name, form) in [
        ("Goal and hypothesis", ""),
        ("Goal\tand", ""),
        ("Goal\u{a0}and", ""),
        (" goal", trimmed),
        ("goal ", trimmed),
        ("", ""),
    ] {
        let sentence = format!("Each FS must have exactly one {name} chapter.");
        let refusal = parse_rule(
            &sentence,
            "RULE-presence".into(),
            anchor("docs/rules.md", 1),
            &vocabulary,
        )
        .expect_err("presence NAME with whitespace or no token stays refused");
        let expected = format!("{reason}{form}NAME forbids whitespace anywhere.");
        assert_eq!(refusal.message, expected, "presence NAME {name:?}");
        assert!(
            !refusal.unrecovered,
            "a chapter name is not a kind: {name:?}"
        );
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

/// §FS-rules.11: the sentence front end returns a complete `ParsedRule` and
/// knows neither facts nor findings.
#[test]
fn sentence_front_end_returns_complete_parsed_rule_without_facts_or_diagnostics() {
    let parsed = parse_rule(
        "The requirements chapter of each FS should cite exactly one REQ.",
        "RULE-one".into(),
        anchor("docs/rules.md", 7),
        &RuleVocabulary {
            kinds: BTreeSet::from(["FS".into(), "REQ".into()]),
            target_kinds: BTreeSet::from(["FS".into(), "REQ".into()]),
            target_namespaces: BTreeMap::new(),
            named_sections: true,
            id_grammars: Vec::new(),
            section_separators: vec![".".into()],
        },
    )
    .expect("released sentence");
    // A local object kind the vocabulary holds: nothing unverifiable here.
    let (parsed, unverifiable) = parsed;
    assert_eq!(unverifiable, None);
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

/// §FS-rules.11, §FS-rules.5.2: the engine evaluates a family clause over
/// hand-built facts, with no sentence text, Markdown or scanner record.
#[test]
fn logic_engine_evaluates_hand_built_rule_and_facts_without_parser_or_scanner() {
    let diagnostics = evaluate(&[rule()], &[], &facts(Completeness::Complete)).0;
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "missing-citation");
    assert_eq!(diagnostics[0].line, Some(2));
    assert_eq!(diagnostics[0].message, "FS-demo must cite GOAL (RULE-fs)");
}

#[test]
fn logic_engine_owns_precedence_and_existing_report_channels() {
    let hard = rule();
    let suppressed = evaluate(
        std::slice::from_ref(&hard),
        std::slice::from_ref(&hard),
        &facts(Completeness::Complete),
    )
    .0;
    assert!(suppressed.is_empty());

    let recommended = ParsedRule {
        level: RuleLevel::Recommended,
        ..rule()
    };
    let diagnostics = evaluate_suggestions(&[recommended], &[], &facts(Completeness::Complete));
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "suggested-citation");
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
    let markdown = adapt_markdown(
        &findings,
        markdown_project(
            config.frame().name,
            &WorkspaceCheckTarget::of(&findings, config.schema(), config.frame()),
        ),
        true,
    );
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
    let left = evaluate(&[rule()], &[], &markdown)
        .0
        .into_iter()
        .map(|d| (d.code, d.message))
        .collect::<Vec<_>>();
    let right = evaluate(&[rule()], &[], &second)
        .0
        .into_iter()
        .map(|d| (d.code, d.message))
        .collect::<Vec<_>>();
    assert_eq!(left, right);
}

/// §FS-rules.5.2: negation and aggregates are closed-world only over a
/// complete snapshot.
#[test]
fn incomplete_fact_snapshot_suppresses_absence_and_count_conclusions() {
    assert_eq!(
        evaluate(&[rule()], &[], &facts(Completeness::Complete))
            .0
            .len(),
        1
    );
    assert!(
        evaluate(&[rule()], &[], &facts(Completeness::Incomplete))
            .0
            .is_empty()
    );

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
                newly_counted: false,
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
    let diagnostics = evaluate(&[rule(), exact_count, prohibition], &[], &partial).0;
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "forbidden-citation");

    // §FS-rules.checks.unreached-declaration, §FS-rules.5.2: the chapter
    // subject's second premise is closed-world like the counts above, so the
    // same snapshot withholds it.
    let chapter_scoped = ParsedRule {
        subject: RuleSubject::ChapterOfKind {
            kind: "FS".into(),
            name: "requirements".into(),
        },
        ..rule()
    };
    let withheld = evaluate(std::slice::from_ref(&chapter_scoped), &[], &partial).0;
    assert!(withheld.is_empty());

    // The same facts, declared complete: the silence above is the snapshot's
    // rather than the rule's, and the finding arrives on the errors channel the
    // promotion put it on (§FS-rules.7).
    let mut complete = partial;
    complete.header.completeness = Completeness::Complete;
    let reported = evaluate(&[chapter_scoped], &[], &complete).0;
    assert_eq!(reported.len(), 1);
    assert_eq!(reported[0].code, "unreached-declaration");
}

/// §FS-rules.3.2, §AR-rules.2: a floor above one is `{minimum, maximum: None}`
/// in the normalized representation, and the representation already words and
/// tests it - which is why the widening of §FS-rules.3 is a sentence change and
/// the engine does not move.
#[test]
fn a_floor_above_one_is_already_worded_and_tested_by_the_representation() {
    let floor = Cardinality {
        minimum: Some(2),
        maximum: None,
    };
    assert_eq!(floor.wording(), "at least 2");
    assert_eq!(floor.times_wording(), "at least 2");
    assert!(!floor.contains(0));
    assert!(!floor.contains(1));
    assert!(floor.contains(2));
    assert!(floor.contains(3));
}

/// §FS-rules.3.2: `missing-citation` is the ordinary `at least one` failure at
/// zero and nothing else, so a floor above one reports `citation-cardinality`
/// even when the count is zero.
#[test]
fn zero_citations_under_a_floor_above_one_is_a_cardinality_finding() {
    let floor = ParsedRule {
        cardinality: Cardinality {
            minimum: Some(2),
            maximum: None,
        },
        ..rule()
    };
    let diagnostics = evaluate(&[floor], &[], &facts(Completeness::Complete)).0;
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "citation-cardinality");
    assert_eq!(
        diagnostics[0].message,
        "FS-demo cites GOAL 0 times; RULE-fs requires at least 2"
    );
}
