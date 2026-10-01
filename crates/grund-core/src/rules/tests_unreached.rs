//! The declaration a chapter-scoped citation rule cannot reach, over a
//! hand-built snapshot (§FS-rules.checks.unreached-declaration, §FS-rules.5.2):
//! which families reach the second premise, which stay silent, and what one
//! semantic group yields.

use super::RuleAnchor;
use super::engine::evaluate;
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts};
use super::sentence::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    TargetMode,
};
use crate::model::Diagnostic;
use std::collections::BTreeMap;

fn anchor(path: &str, line: usize) -> RuleAnchor {
    RuleAnchor {
        path: path.into(),
        line,
        column: None,
    }
}

fn node(label: &str, path: &str) -> (NodeKey, NodeMeta) {
    (
        NodeKey(format!("opaque-{label}")),
        NodeMeta {
            label: label.into(),
            anchor: anchor(path, 1),
        },
    )
}

/// Two FS declarations, one of which has the `requirements` chapter the rules
/// below quantify over, plus the targets the three positive families need.
fn facts() -> RuleFacts {
    let (silent, silent_meta) = node("FS-silent", "docs/fs/FS-silent.md");
    let (good, good_meta) = node("FS-good", "docs/fs/FS-good.md");
    let (req, req_meta) = node("REQ-demo", "docs/req/REQ-demo.md");
    let (ar, ar_meta) = node("AR-demo", "docs/ar/AR-demo.md");
    let chapter = NodeKey("opaque-good-requirements".into());
    RuleFacts {
        header: FactHeader {
            schema: 1,
            project: "demo".into(),
            producer: "test".into(),
            completeness: Completeness::Complete,
        },
        decl: vec![
            (silent.clone(), "FS".into()),
            (good.clone(), "FS".into()),
            (req.clone(), "REQ".into()),
            (ar.clone(), "AR".into()),
        ],
        chapter: vec![(
            chapter.clone(),
            "requirements".into(),
            "Requirements".into(),
        )],
        contains: vec![(good.clone(), chapter.clone())],
        cites: Vec::new(),
        site_in: Vec::new(),
        nodes: BTreeMap::from([
            (silent, silent_meta),
            (good, good_meta),
            (req, req_meta),
            (ar, ar_meta),
            (
                chapter,
                NodeMeta {
                    label: "FS-good.requirements".into(),
                    anchor: anchor("docs/fs/FS-good.md", 3),
                },
            ),
        ]),
        sites: BTreeMap::new(),
    }
}

/// `facts()` plus a third FS declaration whose `requirements` chapter is
/// *displayed* as `Needs`. The handle and the display name differ, which is the
/// one shape that tells §FS-rules.5.2's `chapter_handle` join apart from the
/// presence family's display-name count (§FS-rules.3.1).
fn facts_with_displayed_chapter() -> RuleFacts {
    let mut facts = facts();
    let (displayed, displayed_meta) = node("FS-displayed", "docs/fs/FS-displayed.md");
    let chapter = NodeKey("opaque-displayed-requirements".into());
    facts.decl.push((displayed.clone(), "FS".into()));
    facts
        .chapter
        .push((chapter.clone(), "requirements".into(), "Needs".into()));
    facts.contains.push((displayed.clone(), chapter.clone()));
    facts.nodes.insert(displayed, displayed_meta);
    facts.nodes.insert(
        chapter,
        NodeMeta {
            label: "FS-displayed.requirements".into(),
            anchor: anchor("docs/fs/FS-displayed.md", 5),
        },
    );
    facts
}

fn chapter_rule(origin: &str) -> ParsedRule {
    ParsedRule {
        origin: origin.into(),
        anchor: anchor("docs/rules.md", 1),
        subject: RuleSubject::ChapterOfKind {
            kind: "FS".into(),
            name: "requirements".into(),
        },
        level: RuleLevel::Required,
        polarity: RulePolarity::Positive,
        relation: RuleRelation::Cite,
        targets: RuleTargets::Kinds {
            values: vec!["REQ".into()],
            mode: TargetMode::Aggregate,
        },
        cardinality: Cardinality::AT_LEAST_ONE,
    }
}

fn per_target(origin: &str) -> ParsedRule {
    ParsedRule {
        targets: RuleTargets::Kinds {
            values: vec!["REQ".into()],
            mode: TargetMode::PerTarget,
        },
        ..chapter_rule(origin)
    }
}

fn inbound(origin: &str) -> ParsedRule {
    ParsedRule {
        relation: RuleRelation::BeCitedBy,
        targets: RuleTargets::Kinds {
            values: vec!["AR".into()],
            mode: TargetMode::Aggregate,
        },
        ..chapter_rule(origin)
    }
}

fn prohibition(origin: &str) -> ParsedRule {
    ParsedRule {
        polarity: RulePolarity::Prohibiting,
        targets: RuleTargets::Kinds {
            values: vec!["AR".into()],
            mode: TargetMode::Aggregate,
        },
        cardinality: Cardinality::NONE,
        ..chapter_rule(origin)
    }
}

/// §FS-rules.7: the absence is an error on the ordinary `must` channel, so the
/// rules report has no warnings channel left to carry it
/// (§FS-rules.checks.unreached-declaration). Every test below reads the
/// absences through here, off that one channel, which is where the promotion
/// is pinned at the unit level.
fn absences(rules: &[ParsedRule]) -> Vec<Diagnostic> {
    evaluate(rules, &[], &facts())
        .into_iter()
        .filter(|diagnostic| diagnostic.code == "unreached-declaration")
        .collect()
}

fn one_absence(rules: &[ParsedRule]) -> Diagnostic {
    let mut absences = absences(rules);
    let rendered = absences
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(absences.len(), 1, "expected one absence: {rendered:?}");
    absences.remove(0)
}

/// §FS-rules.checks.unreached-declaration: the outbound-count family reports
/// the declaration that has no such chapter, at the declaration's own title
/// line, with the landed release in its own bytes (§FS-distribution.4.2.3).
#[test]
fn the_outbound_count_family_reports_the_declaration_with_no_such_chapter() {
    let absence = one_absence(&[chapter_rule("RULE-outbound")]);
    assert_eq!(absence.code, "unreached-declaration");
    assert_eq!(
        absence
            .path
            .as_deref()
            .map(|path| path.to_string_lossy().into_owned()),
        Some("docs/fs/FS-silent.md".to_string())
    );
    assert_eq!(absence.line, Some(1));
    assert_eq!(
        absence.message,
        "FS-silent has no requirements chapter, so RULE-outbound cannot reach it; \
         add the chapter, or narrow the rule to the declarations that have one; \
         this became an error in grund 0.16.0"
    );
    assert_eq!(absence.authority, vec!["RULE-outbound".to_string()]);
}

/// §FS-rules.5.2: per-target coverage reaches the premise too, and reports the
/// declaration once rather than once per target — the absence is about the
/// subject, not about what it failed to cite.
#[test]
fn the_per_target_family_reports_the_declaration_once() {
    let absence = one_absence(&[per_target("RULE-coverage")]);
    assert_eq!(absence.code, "unreached-declaration");
    assert!(
        absence.message.starts_with(
            "FS-silent has no requirements chapter, so RULE-coverage cannot reach it;"
        )
    );
}

/// §FS-rules.5.2: the inbound-count family is the third positive family, so it
/// reports the absence on the same reading as the outbound one.
#[test]
fn the_inbound_count_family_reports_the_declaration_with_no_such_chapter() {
    let absence = one_absence(&[inbound("RULE-inbound")]);
    assert_eq!(absence.code, "unreached-declaration");
    assert!(absence.message.contains("so RULE-inbound cannot reach it;"));
}

/// §FS-rules.5.2: a prohibition admits a chapter subject and still stays
/// silent, because a citation site inside the chapter body is deleted along
/// with that body — for `must not cite` the forbidden site genuinely no longer
/// exists.
#[test]
fn a_prohibition_over_the_same_absent_chapter_stays_silent() {
    assert!(absences(&[prohibition("RULE-prohibition")]).is_empty());
}

/// §FS-rules.5.2: `chapter_handle` joins the chapter's last accepted section
/// component, so `## requirements: Needs` is one of the units `The requirements
/// chapter of each FS` selects and its declaration is not unreached. Joining the
/// display name instead would report that FS-displayed has no `requirements`
/// chapter in the same run as a finding located inside that chapter.
#[test]
fn a_chapter_displayed_under_another_name_is_reached_by_its_handle() {
    let facts = facts_with_displayed_chapter();
    let diagnostics = evaluate(&[chapter_rule("RULE-outbound")], &[], &facts);
    let absent: Vec<&str> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "unreached-declaration")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(absent.len(), 1, "only FS-silent is unreached: {absent:?}");
    assert!(absent[0].starts_with("FS-silent has no requirements chapter,"));
    let rendered = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert!(
        rendered
            .iter()
            .any(|message| message.starts_with("FS-displayed.requirements must cite REQ")),
        "the displayed chapter is a unit of the rule: {rendered:?}"
    );
}

/// §FS-rules.checks.unreached-declaration: the premise is a chapter subject's,
/// so a kind subject and an exact chapter subject report nothing — the exact
/// spelling keeps its own `invalid-rule` instead (§FS-rules.2).
#[test]
fn a_kind_or_exact_subject_reports_no_absence() {
    let kind = ParsedRule {
        subject: RuleSubject::Kind("FS".into()),
        ..chapter_rule("RULE-kind")
    };
    let exact = ParsedRule {
        subject: RuleSubject::ExactChapter {
            declaration: "FS-good".into(),
            path: "requirements".into(),
            separator: ".".into(),
        },
        ..chapter_rule("RULE-exact")
    };
    assert!(absences(&[kind]).is_empty());
    assert!(absences(&[exact]).is_empty());
}

/// §FS-rules.6: the absence is authored by a semantic group like every other
/// rule finding, so two byte-identical rules yield one line naming both
/// origins, in bytewise order, in the field and in the tail alike.
#[test]
fn one_semantic_group_yields_one_line_naming_both_origins() {
    let absence = one_absence(&[inbound("RULE-inbound-copy"), inbound("RULE-inbound")]);
    assert_eq!(
        absence.authority,
        vec!["RULE-inbound".to_string(), "RULE-inbound-copy".to_string()]
    );
    assert!(
        absence
            .message
            .contains("so RULE-inbound, RULE-inbound-copy cannot reach it;")
    );
}

/// §FS-rules.checks.unreached-declaration: two rules that mean different things
/// each produce their own line about the same declaration, and nothing
/// suppresses either.
#[test]
fn two_different_rules_each_report_the_same_declaration() {
    let absences = absences(&[chapter_rule("RULE-outbound"), inbound("RULE-inbound")]);
    assert_eq!(absences.len(), 2);
    assert!(absences.iter().all(|a| a.code == "unreached-declaration"));
}

/// §FS-rules.checks.unreached-declaration: the recommended level reports the
/// absence as an ordinary suggestion, with no landed clause — a suggestion
/// never moved the exit status at any release, so it owed no promotion.
#[test]
fn the_recommended_level_reports_the_absence_without_a_ramp_clause() {
    let recommended = ParsedRule {
        level: RuleLevel::Recommended,
        ..chapter_rule("RULE-outbound")
    };
    let suggestions = super::engine::evaluate_suggestions(&[recommended], &[], &facts());
    let absence = suggestions
        .iter()
        .find(|diagnostic| diagnostic.code == "unreached-declaration")
        .expect("the recommended absence");
    assert_eq!(
        absence.message,
        "FS-silent has no requirements chapter, so RULE-outbound cannot reach it; \
         add the chapter, or narrow the rule to the declarations that have one"
    );
}
