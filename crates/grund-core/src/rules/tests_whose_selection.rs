//! A whose clause over a hand-built snapshot (§FS-rules.2.2, §FS-rules.5.2):
//! `selected(d)` is read over the `decl`, `contains` and `chapter` facts alone,
//! a declaration whose field holds another value is outside the rule, and only a
//! selected declaration can be unreached
//! (§FS-rules.checks.unreached-declaration).

use super::RuleAnchor;
use super::engine::evaluate;
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts};
use super::sentence::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    TargetMode, Whose,
};
use std::collections::BTreeMap;

fn anchor(path: &str, line: usize) -> RuleAnchor {
    RuleAnchor {
        path: path.into(),
        line,
        column: None,
    }
}

/// Three SWR declarations, each with a direct `verification` chapter: `tested`
/// (Test) and `inspected` (Inspection) have an `evidence` chapter, `untested`
/// (Test) has none; `nested` holds `Test` only at `notes.verification`, which
/// is no field.
fn facts() -> RuleFacts {
    let mut facts = RuleFacts {
        header: FactHeader {
            schema: 1,
            project: "demo".into(),
            producer: "test".into(),
            completeness: Completeness::Complete,
        },
        decl: Vec::new(),
        chapter: Vec::new(),
        contains: Vec::new(),
        cites: Vec::new(),
        site_in: Vec::new(),
        nodes: BTreeMap::new(),
        sites: BTreeMap::new(),
    };
    let rows: [(&str, &[(&str, &str, &str)]); 4] = [
        (
            "tested",
            &[("", "verification", "Test"), ("", "evidence", "Evidence")],
        ),
        (
            "inspected",
            &[
                ("", "verification", "Inspection"),
                ("", "evidence", "Evidence"),
            ],
        ),
        ("untested", &[("", "verification", "Test")]),
        (
            "nested",
            &[
                ("", "notes", "Notes"),
                ("notes", "notes.verification", "Test"),
            ],
        ),
    ];
    for (slug, chapters) in rows {
        let label = format!("SWR-{slug}");
        let path = format!("req/{label}.md");
        let declaration = NodeKey(format!("opaque-{slug}"));
        facts.decl.push((declaration.clone(), "SWR".into()));
        facts.nodes.insert(
            declaration.clone(),
            NodeMeta {
                label: label.clone(),
                anchor: anchor(&path, 1),
            },
        );
        for (line, (parent, section, display)) in chapters.iter().enumerate() {
            let chapter = NodeKey(format!("opaque-{slug}-{section}"));
            let parent = match *parent {
                "" => declaration.clone(),
                parent => NodeKey(format!("opaque-{slug}-{parent}")),
            };
            facts.chapter.push((
                chapter.clone(),
                section.to_string(),
                Some(display.to_string()),
            ));
            facts.contains.push((parent, chapter.clone()));
            facts.nodes.insert(
                chapter,
                NodeMeta {
                    label: format!("{label}.{section}"),
                    anchor: anchor(&path, line + 3),
                },
            );
        }
    }
    facts
}

fn tested() -> Option<Whose> {
    Some(Whose {
        field: "verification".into(),
        value: "Test".into(),
    })
}

fn rule(subject: RuleSubject) -> ParsedRule {
    ParsedRule {
        origin: "RULE-tested".into(),
        anchor: anchor("rules/RULE-tested.md", 1),
        subject,
        level: RuleLevel::Required,
        polarity: RulePolarity::Positive,
        relation: RuleRelation::BeCitedBy,
        targets: RuleTargets::Kinds {
            values: vec!["TC".into()],
            mode: TargetMode::Aggregate,
        },
        cardinality: Cardinality::AT_LEAST_ONE,
    }
}

/// The declarations each finding is reported at, by file.
fn reported(rules: &[ParsedRule]) -> Vec<String> {
    let mut paths = evaluate(rules, &[], &facts())
        .into_iter()
        .map(|diagnostic| {
            let path = diagnostic
                .path
                .expect("located")
                .to_string_lossy()
                .into_owned();
            format!("{path} {}", diagnostic.code.as_str())
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

/// §FS-rules.2.2: `Each SWR whose verification is Test` selects the two whose
/// direct `verification` chapter displays `Test`; the Inspection one and the
/// one holding `Test` only in a nested chapter are outside the rule.
#[test]
fn the_kind_form_selects_only_declarations_whose_field_holds_the_value() {
    assert_eq!(
        reported(&[rule(RuleSubject::Kind("SWR".into(), tested()))]),
        [
            "req/SWR-tested.md uncited-unit",
            "req/SWR-untested.md uncited-unit"
        ]
    );
}

/// §FS-rules.checks.unreached-declaration: under the chapter form, the selected
/// declaration without the chapter is unreached, and the one the clause leaves
/// out is never reported, chapter or not.
#[test]
fn only_a_selected_declaration_is_unreached() {
    let subject = RuleSubject::ChapterOfKind {
        kind: "SWR".into(),
        name: "evidence".into(),
        whose: tested(),
    };
    assert_eq!(
        reported(&[rule(subject)]),
        [
            "req/SWR-tested.md uncited-unit",
            "req/SWR-untested.md unreached-declaration"
        ]
    );
}
