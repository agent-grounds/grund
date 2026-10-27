//! §FS-rules.3.3 and §FS-rule-refusals.sentences.rewrite.by-refusal:
//! per-target ceilings normalize to the existing bounds, with typed rewrites.

use super::RuleAnchor;
use super::sentence::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    RuleVocabulary, TargetMode, parse_rule,
};
use crate::config::{Config, KindTable};
use std::collections::BTreeSet;
use std::path::PathBuf;

fn vocabulary() -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("per-target-once"));
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.named_sections = true;
    config.rebuild_grammar().unwrap();
    let kinds = BTreeSet::from(["FS".into()]);
    RuleVocabulary {
        fields: Default::default(),
        objects: KindTable::of_citable(&kinds),
        kinds,
        named_sections: true,
        named_kinds: Default::default(),
        non_citable: Default::default(),
        id_grammars: vec![config.grammar],
        section_separators: vec![".".into()],
    }
}

fn parse(title: &str) -> Result<ParsedRule, String> {
    parse_rule(
        title,
        "RULE-once".into(),
        RuleAnchor {
            path: "docs/rules/once.md".into(),
            line: 1,
            column: None,
        },
        &vocabulary(),
    )
    .map(|(rule, unverifiable)| {
        assert!(unverifiable.is_none());
        rule
    })
    .map_err(|error| error.message)
}

/// §FS-rules.3.3: the new word spelling needs no evaluator representation.
#[test]
fn per_target_once_normalizes_without_a_minimum_and_with_maximum_one() {
    for (modality, level) in [
        ("must", RuleLevel::Required),
        ("should", RuleLevel::Recommended),
    ] {
        let title = format!("FS-login.requirements {modality} cite each FS at most once.");
        let rule = parse(&title).unwrap_or_else(|error| panic!("{title}: {error}"));
        assert_eq!(
            rule.cardinality,
            Cardinality {
                minimum: None,
                maximum: Some(1)
            }
        );
        assert_eq!(rule.level, level);
        assert_eq!(rule.polarity, RulePolarity::Positive);
        assert_eq!(rule.relation, RuleRelation::Cite);
        assert_eq!(
            rule.subject,
            RuleSubject::ExactChapter {
                declaration: "FS-login".into(),
                path: "requirements".into(),
                separator: ".".into(),
            }
        );
        assert_eq!(
            rule.targets,
            RuleTargets::Kinds {
                values: vec!["FS".into()],
                mode: TargetMode::PerTarget
            }
        );
    }
}

/// §FS-rule-refusals.sentences.rewrite.fs-only,
/// §FS-rule-refusals.sentences.rewrite.reparsed: exact FS-only row and
/// typed subject/modality preservation over every subject spelling and repair.
#[test]
fn per_target_once_numeric_rewrites_preserve_typed_subjects_and_reparse() {
    let mut wrong = Vec::new();
    for subject in [
        "Each FS",
        "FS-login",
        "FS-login.requirements",
        "The requirements chapter of each FS",
    ] {
        for modality in ["must", "should"] {
            for (count, reason) in [
                ("1 times", "numeric \"at most 1 times\" is not canonical"),
                ("1", "per-target counts must end in \"times\""),
                ("1 time", "per-target counts must end in \"times\""),
                (
                    "01 times",
                    "count must be a canonical positive base-10 integer",
                ),
            ] {
                let title = format!("{subject} {modality} cite each FS at most {count}.");
                let form = format!("{subject} {modality} cite each FS at most once.");
                let expected = format!("{reason}; accepted form: {form}");
                if parse(&title) != Err(expected.clone()) {
                    wrong.push(format!("{title}: {:?}; expected {expected}", parse(&title)));
                }
                if let Err(error) = parse(&form) {
                    wrong.push(format!("offered form refused: {form}: {error}"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// §FS-rule-refusals.sentences.order: earlier refusal stages keep their
/// reasons even when the predicate also contains the new invalid count.
#[test]
fn per_target_once_predicate_refusal_does_not_displace_earlier_stages() {
    for (title, reason) in [
        (
            "Each FS must cite each FS at most 1 times",
            "rule must end with \".\"",
        ),
        (
            "each FS must cite each FS at most 1 times.",
            "fixed word \"Each\" is case-sensitive",
        ),
        (
            "FS-x.* must cite each FS at most 1 times.",
            "section-component wildcards are not accepted in phase 1",
        ),
        (
            "Each FS must cite each FS at most 1 times and must cite at least one FS.",
            "conjunctions are not accepted",
        ),
        (
            "Each POLICY may not cite each FS at most 1 times.",
            "modality \"may not\" is not accepted",
        ),
        (
            "Each POLICY must cite each FS at most 1 times.",
            "unknown kind \"POLICY\"",
        ),
    ] {
        let error = parse(title).unwrap_err();
        assert!(
            error == reason || error.starts_with(&format!("{reason};")),
            "{title}: {error}"
        );
    }
}

/// §FS-rules.3, §FS-rules.3.3: unchanged bounds and other-family spelling
/// controls pass before the new spelling is implemented.
#[test]
fn per_target_once_other_count_bounds_and_families_are_unchanged() {
    for (predicate, minimum, maximum) in [
        ("cite each FS at least once", Some(1), None),
        ("cite each FS exactly once", Some(1), Some(1)),
        ("cite each FS at least 2 times", Some(2), None),
        ("cite each FS at most 2 times", None, Some(2)),
        ("cite each FS exactly 2 times", Some(2), Some(2)),
        ("have at most 1 requirements chapter", None, Some(1)),
        ("cite at most 1 FS", None, Some(1)),
        ("be cited by at most 1 FS", None, Some(1)),
    ] {
        let title = format!("Each FS must {predicate}.");
        assert_eq!(
            parse(&title).unwrap().cardinality,
            Cardinality { minimum, maximum },
            "{title}"
        );
    }
}
