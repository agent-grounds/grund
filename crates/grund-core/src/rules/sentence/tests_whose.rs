//! The whose clause on a hand-written vocabulary (§FS-rules.2.2): what the
//! parser reads a value as, and the order its refusals are checked in
//! (§FS-rule-refusals.sentences.whose, §FS-rule-refusals.sentences.order).
//! `crates/grund-cli/tests/rule_field_value*.rs` pin the same black-box.

use super::super::{RuleSubject, RuleVocabulary, Whose, parse_rule, parse_selector};
use crate::config::{Config, KindField, KindTable};
use crate::rules::RuleAnchor;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn scalar(name: &str, one_of: &[&str]) -> KindField {
    KindField {
        name: name.into(),
        one_of: Some(one_of.iter().map(|value| value.to_string()).collect()),
    }
}

/// Kinds `SWR`, `TC` and `AN` under `{kind}-{slug}`, named sections on: `SWR`
/// has the scalar `verification` and the prose `rationale`, `AN` the scalar
/// `status`, whose values hold spaces, a capitalised modal word and a `.`.
fn vocabulary() -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("whose-vocabulary"));
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.named_sections = true;
    config.rebuild_grammar().expect("{kind}-{slug} grammar");
    let kinds = BTreeSet::from(["SWR".to_string(), "TC".into(), "AN".into()]);
    RuleVocabulary {
        fields: BTreeMap::from([
            (
                "SWR".to_string(),
                Some(vec![
                    scalar("verification", &["Test", "Inspection"]),
                    KindField {
                        name: "rationale".into(),
                        one_of: None,
                    },
                ]),
            ),
            ("TC".to_string(), Some(Vec::new())),
            (
                "AN".to_string(),
                Some(vec![scalar(
                    "status",
                    &["Must fix", "fix and retest", "Approx."],
                )]),
            ),
        ]),
        objects: KindTable::of_citable(&kinds),
        kinds,
        named_sections: true,
        named_kinds: Default::default(),
        non_citable: Default::default(),
        id_grammars: vec![config.grammar],
        section_separators: vec![".".into()],
    }
}

fn anchor() -> RuleAnchor {
    RuleAnchor {
        path: "rules/RULE-x.md".into(),
        line: 1,
        column: None,
    }
}

/// The subject `parse_rule` reads `sentence` with, or its message.
fn subject(sentence: &str) -> Result<RuleSubject, String> {
    parse_rule(sentence, "RULE-x".into(), anchor(), &vocabulary())
        .map(|(rule, _)| rule.subject)
        .map_err(|error| error.message)
}

fn filtered(kind: &str, field: &str, value: &str) -> RuleSubject {
    RuleSubject::Kind(
        kind.into(),
        Some(Whose {
            field: field.into(),
            value: value.into(),
        }),
    )
}

/// §FS-rule-refusals.sentences.whose: the split looks for a lowercase ` must `, so a
/// capitalised `Must fix`, a value holding ` and `, and a value ending in `.`
/// are each read whole, on a rule and on a selector alike.
#[test]
fn values_with_spaces_modal_capitals_and_a_period_parse_whole() {
    for value in ["Must fix", "fix and retest", "Approx."] {
        let sentence = format!("Each AN whose status is {value} must cite at least one TC.");
        assert_eq!(
            subject(&sentence),
            Ok(filtered("AN", "status", value)),
            "{sentence}"
        );
        let selector = format!("Each AN whose status is {value}");
        assert_eq!(
            parse_selector(&selector, &vocabulary()),
            Ok(filtered("AN", "status", value)),
            "{selector}"
        );
    }
}

/// §FS-rules.2.2: the chapter form carries the clause on its chapter subject.
#[test]
fn the_chapter_form_carries_the_clause() {
    assert_eq!(
        subject(
            "The evidence chapter of each SWR whose verification is Test must cite at least one TC."
        ),
        Ok(RuleSubject::ChapterOfKind {
            kind: "SWR".into(),
            name: "evidence".into(),
            whose: Some(Whose {
                field: "verification".into(),
                value: "Test".into(),
            }),
        })
    );
}

/// §FS-rule-refusals.sentences.whose: a kind that is not configured fails before its
/// clause is read, so an unknown field of it is never what is named; since the window of
/// §FS-errors.3.10 closed the kind is named alone.
#[test]
fn an_unknown_kind_is_refused_before_an_unknown_field() {
    assert_eq!(
        subject("Each POLICY whose verfication is Test must cite at least one TC."),
        Err("unknown kind \"POLICY\"".into())
    );
}

/// §FS-rule-refusals.sentences.order: stage 3 refuses a conjunction before stage 5
/// reads the clause, so the bad value is never named.
#[test]
fn a_conjunction_is_refused_before_the_clause() {
    let message = subject(
        "Each SWR whose verification is Tset must cite at least one TC and must not cite any AN.",
    )
    .unwrap_err();
    assert!(
        message.starts_with("conjunctions are not accepted"),
        "{message}"
    );
}

/// §FS-rule-refusals.sentences.whose: within the clause, the shape is checked first,
/// then the field, then whether it is a scalar, then the value's whitespace,
/// then the value. Each reason is the whole message (§FS-errors.3.10).
#[test]
fn the_clause_is_checked_in_its_order() {
    let reason = |sentence: &str| subject(sentence).unwrap_err();
    assert_eq!(
        reason("Each SWR whose verfication is  must cite at least one TC."),
        "whose clause \"whose verfication is \" names no value; write whose FIELD is VALUE"
    );
    assert_eq!(
        reason("Each SWR whose rationale is Tset  must cite at least one TC."),
        "field \"rationale\" of kind SWR is not a scalar: it sets no one_of; its scalar \
         fields: verification"
    );
    assert_eq!(
        reason("Each SWR whose verification is Tset  must cite at least one TC."),
        "value \"Tset \" of SWR.verification has surrounding whitespace, which no one_of \
         value can hold"
    );
}
