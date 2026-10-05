//! Test module: who the engine says authored a finding (§FS-rules.6,
//! §FS-rules.7.6) — the origin set that becomes the record's `authority` field,
//! and the message tail joined from that same set.
//!
//! The behaviour these units sit under is pinned black-box in
//! `crates/grund-cli/tests/rules_contract/authority.rs`. What is here is the
//! seam those cases rest on: that field and tail are one set rendered twice, in
//! one order, and that a diagnostic *about* a rule still names it.

use super::RuleAnchor;
use super::engine::{evaluate, unresolved_subject_diagnostic};
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts};
use super::sentence::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    TargetMode,
};
use std::collections::BTreeMap;

fn anchor(path: &str, line: usize) -> RuleAnchor {
    RuleAnchor {
        path: path.into(),
        line,
        column: None,
    }
}

/// One `FS` that cites no `GOAL`, so the rule below reports exactly once.
fn facts() -> RuleFacts {
    let fs = NodeKey("opaque-fs".into());
    RuleFacts {
        header: FactHeader {
            schema: 1,
            project: "demo".into(),
            producer: "test".into(),
            completeness: Completeness::Complete,
        },
        decl: vec![(fs.clone(), "FS".into())],
        chapter: Vec::new(),
        contains: Vec::new(),
        cites: Vec::new(),
        site_in: Vec::new(),
        nodes: BTreeMap::from([(
            fs,
            NodeMeta {
                label: "FS-demo".into(),
                anchor: anchor("docs/fs.md", 2),
            },
        )]),
        sites: BTreeMap::new(),
    }
}

/// "Each FS must cite at least one GOAL.", under whichever origin is given.
fn rule(origin: &str) -> ParsedRule {
    ParsedRule {
        origin: origin.into(),
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

/// §FS-rules.7.6: one rule's evaluation names that rule and nothing else.
#[test]
fn one_rules_finding_carries_that_rules_origin() {
    let diagnostics = evaluate(&[rule("RULE-goalref")], &[], &facts()).0;
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].authority, vec!["RULE-goalref".to_string()]);
}

/// §FS-rules.6: rules that mean the same thing collapse into one finding whose
/// authority is every contributing origin, in bytewise order rather than in the
/// order the rules arrived — which is what keeps the record deterministic
/// (§REQ-deterministic-output).
#[test]
fn a_collapsed_groups_authority_is_every_origin_in_bytewise_order() {
    let diagnostics = evaluate(
        &[rule("RULE-second"), rule("--rule"), rule("RULE-first")],
        &[],
        &facts(),
    )
    .0;
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].authority,
        vec![
            "--rule".to_string(),
            "RULE-first".to_string(),
            "RULE-second".to_string()
        ]
    );
}

/// §FS-rules.6: the field and the message tail are one set rendered twice, so
/// the words a reader sees name exactly what a script would read.
#[test]
fn the_message_tail_is_the_authority_joined() {
    let diagnostics = evaluate(&[rule("RULE-second"), rule("--rule")], &[], &facts()).0;
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message,
        format!(
            "FS-demo must cite GOAL ({})",
            diagnostics[0].authority.join(", ")
        )
    );
    // And those bytes are the ones the tail has always had.
    assert_eq!(
        diagnostics[0].message,
        "FS-demo must cite GOAL (--rule, RULE-second)"
    );
}

/// §FS-rules.7.6, §FS-rules.7.1: an `invalid-rule` is a diagnostic *about* a
/// rule, so it never reaches the group join — and it names its one rule anyway,
/// which is what keeps a trial sentence whose subject does not resolve inside a
/// scoped report instead of letting a typo read as `success`.
#[test]
fn an_unresolved_subject_diagnostic_carries_its_one_rules_origin() {
    let unresolved = ParsedRule {
        subject: RuleSubject::ExactDeclaration("FS-nosuch".into()),
        ..rule("--rule")
    };
    let diagnostic =
        unresolved_subject_diagnostic(&unresolved, &facts()).expect("the subject does not resolve");
    assert_eq!(diagnostic.code, "invalid-rule");
    assert_eq!(diagnostic.authority, vec!["--rule".to_string()]);
}

/// §FS-rules.6: a finding no chapter rule authored carries no authority, which
/// is what renders as `null` and what `--only-rule` cannot retain
/// (§FS-errors.5.1).
#[test]
fn a_suppressed_group_leaves_no_authority_behind() {
    let declared = rule("RULE-goalref");
    let suppressed = evaluate(
        std::slice::from_ref(&declared),
        std::slice::from_ref(&declared),
        &facts(),
    )
    .0;
    assert!(suppressed.is_empty());
}
