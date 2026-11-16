//! The `distinct` spelling of the outbound count as the sentence front end reads it
//! (§FS-rules.3.2.1): where `distinct` is the keyword, decided from the kind table of the rule's
//! scope and never from the scan, and the refusals of a `distinct` the predicate does not admit,
//! each with the form §FS-rule-refusals.sentences gives it (§FS-rule-refusals.sentences.rewrite,
//! §FS-rule-refusals.sentences.rewrite.same-units). Issue #1367 is the report: the keyword was
//! read as part of a kind name, `unknown kind "distinct SRC"`.
//!
//! The controls pin that a kind named `distinct` keeps the readings it had before the keyword.

use super::RuleAnchor;
use super::sentence::{
    ParsedRule, RuleRelation, RuleTargets, RuleVocabulary, TargetMode, parse_rule,
};
use crate::config::{Config, KindTable};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// A vocabulary of `kinds`, judged in `objects`, with named sections on.
fn vocabulary_in(kinds: &[&str], objects: KindTable) -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("distinct-keyword"));
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.named_sections = true;
    config.rebuild_grammar().unwrap();
    let kinds: BTreeSet<String> = kinds.iter().map(|kind| kind.to_string()).collect();
    RuleVocabulary {
        fields: Default::default(),
        objects,
        kinds,
        named_sections: true,
        named_kinds: Default::default(),
        non_citable: Default::default(),
        id_grammars: vec![config.grammar],
        section_separators: vec![".".into()],
    }
}

/// A standalone project, with no workspace in scope, whose kinds are all citable.
fn vocabulary(kinds: &[&str]) -> RuleVocabulary {
    let names = kinds.iter().map(|kind| kind.to_string()).collect();
    vocabulary_in(kinds, KindTable::of_citable(&names))
}

/// The rule and its unverifiable reason, or the refusal's whole message.
fn parse(title: &str, vocab: &RuleVocabulary) -> Result<(ParsedRule, Option<String>), String> {
    let anchor = RuleAnchor {
        path: "docs/rules/RULE-distinct.md".into(),
        line: 1,
        column: None,
    };
    parse_rule(title, "RULE-distinct".into(), anchor, vocab).map_err(|error| error.message)
}

/// Whether `targets` are the kinds `values` counted neither by site nor per target.
fn distinct_over(targets: &RuleTargets, values: &[&str]) -> bool {
    matches!(
        targets,
        RuleTargets::Kinds { values: read, mode }
            if read == values && *mode != TargetMode::Aggregate && *mode != TargetMode::PerTarget
    )
}

/// §FS-rules.3.2.1: the issue's sentence is accepted, an outbound count over `SRC` in a mode of
/// its own, with its count and relation as the plain count reads them.
#[test]
fn the_issue_sentence_is_a_distinct_outbound_count() {
    let vocab = vocabulary(&["COUNTRY", "SRC", "RULE"]);
    let title = "The truck-mass chapter of each COUNTRY must cite at least 3 distinct SRC.";
    let (rule, unverifiable) = parse(title, &vocab).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(unverifiable, None);
    assert_eq!(rule.relation, RuleRelation::Cite);
    assert_eq!(rule.cardinality.minimum, Some(3));
    assert_eq!(rule.cardinality.maximum, None);
    assert!(distinct_over(&rule.targets, &["SRC"]), "{:?}", rule.targets);
    let (plain, _) = parse(
        "The truck-mass chapter of each COUNTRY must cite at least 3 SRC.",
        &vocab,
    )
    .expect("the plain count");
    assert_ne!(
        plain.targets, rule.targets,
        "the counting mode separates the two"
    );
    assert_eq!(
        plain.targets,
        RuleTargets::Kinds {
            values: vec!["SRC".into()],
            mode: TargetMode::Aggregate
        }
    );
}

/// §FS-rules.3.2.1: `distinct` covers the whole `or` list, and every count spelling but
/// `at least one` takes it.
#[test]
fn distinct_covers_the_or_list_at_every_bound() {
    let vocab = vocabulary(&["FS", "GOAL", "REQ"]);
    for count in [
        "at least 2",
        "at most 1",
        "at most 3",
        "exactly one",
        "exactly 2",
    ] {
        let title = format!("Each FS must cite {count} distinct REQ or GOAL.");
        let (rule, _) = parse(&title, &vocab).unwrap_or_else(|error| panic!("{title}: {error}"));
        assert!(
            distinct_over(&rule.targets, &["GOAL", "REQ"]),
            "{title}: {:?}",
            rule.targets
        );
    }
}

/// §FS-rules.3.2.1: where `distinct` is no citable kind, it is the keyword whenever an object
/// follows it, so an unknown kind after it is refused under its own name.
#[test]
fn an_unknown_kind_after_distinct_is_refused_under_its_own_name() {
    let vocab = vocabulary(&["COUNTRY", "SRC", "RULE"]);
    assert_eq!(
        parse("Each COUNTRY must cite at least 3 distinct SRCX.", &vocab).map(|_| ()),
        Err("unknown kind \"SRCX\"".to_string())
    );
    // §DF-distinct-target-counts: a later refusal names the real failure.
    assert_eq!(
        parse(
            "Each COUNTRY must cite at least 3 distinct SRC or DOCX.",
            &vocab
        )
        .map(|_| ()),
        Err("unknown kind \"DOCX\"".to_string())
    );
}

/// §FS-rules.3.2.1: with a citable kind named `distinct`, the bare count keeps counting it and
/// an object `distinct` does not admit is read as before. Controls.
#[test]
fn a_kind_named_distinct_keeps_its_readings() {
    let vocab = vocabulary(&["COUNTRY", "SRC", "distinct"]);
    let (rule, _) = parse("Each COUNTRY must cite at least 3 distinct.", &vocab)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        rule.targets,
        RuleTargets::Kinds {
            values: vec!["distinct".into()],
            mode: TargetMode::Aggregate
        }
    );
    assert_eq!(
        parse("Each COUNTRY must cite at least 3 distinct SRCX.", &vocab).map(|_| ()),
        Err("unknown kind \"distinct SRCX\"".to_string())
    );
}

/// §FS-rules.3.2.1: with a citable kind named `distinct`, `distinct` followed by a citable kind
/// is the keyword.
#[test]
fn a_kind_named_distinct_still_lets_the_keyword_count_a_citable_kind() {
    let vocab = vocabulary(&["COUNTRY", "SRC", "distinct"]);
    let (rule, _) = parse("Each COUNTRY must cite at least 3 distinct SRC.", &vocab)
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(distinct_over(&rule.targets, &["SRC"]), "{:?}", rule.targets);
}

/// §FS-rules.3.2.1, §FS-rules.4.1: `distinct alias/KIND` and `distinct */KIND` get what
/// `alias/KIND` and `*/KIND` get in the same scope: unverifiable in a member run with no
/// workspace in scope, accepted at the workspace root, and refused, with the same reason, where
/// the workspace holds no such alias.
#[test]
fn a_namespaced_object_after_distinct_reads_as_it_does_alone() {
    let member = vocabulary(&["COUNTRY", "SRC"]);
    let root = vocabulary_in(
        &["COUNTRY", "SRC"],
        KindTable::of_citable_projects(&[("", &["COUNTRY", "SRC"]), ("mem", &["SRC"])]),
    );
    let elsewhere = vocabulary_in(
        &["COUNTRY", "SRC"],
        KindTable::of_citable_projects(&[("", &["COUNTRY", "SRC"]), ("other", &["SRC"])]),
    );
    let mut wrong = Vec::new();
    for (scope, vocab) in [
        ("member", &member),
        ("root", &root),
        ("elsewhere", &elsewhere),
    ] {
        for object in ["mem/SRC", "*/SRC"] {
            let plain = format!("Each COUNTRY must cite at least 3 {object}.");
            let distinct = format!("Each COUNTRY must cite at least 3 distinct {object}.");
            let plain = parse(&plain, vocab).map(|(_, unverifiable)| unverifiable);
            let read = parse(&distinct, vocab).map(|(_, unverifiable)| unverifiable);
            if read != plain {
                wrong.push(format!("{scope}: {distinct}: {read:?}, alone {plain:?}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert!(
        parse("Each COUNTRY must cite at least 3 distinct mem/SRC.", &root)
            .is_ok_and(|(_, unverifiable)| unverifiable.is_none()),
        "the root resolves mem"
    );
}

/// §FS-rule-refusals.sentences, §FS-rule-refusals.sentences.rewrite.by-refusal: every refusal a
/// `distinct` moves, in the repository of the sentences table, with its exact reason and form.
#[test]
fn each_distinct_refusal_gives_its_reason_and_form() {
    let vocab = vocabulary(&["GOAL", "REQ", "FS", "AR", "RULE"]);
    let rows = [
        (
            "Each FS must cite at least one distinct GOAL.",
            "\"distinct\" is redundant after \"at least one\"; accepted form: Each FS must cite at least one GOAL.",
        ),
        (
            "Each FS must cite at least 1 distinct GOAL.",
            "numeric \"at least 1\" is not canonical; accepted form: Each FS must cite at least one GOAL.",
        ),
        (
            "Each FS must cite exactly 1 distinct GOAL.",
            "numeric \"exactly 1\" is not canonical; accepted form: Each FS must cite exactly one distinct GOAL.",
        ),
        (
            "Each FS must cite each distinct GOAL at least once.",
            "\"distinct\" has no meaning per target; accepted form: Each FS must cite each GOAL at least once.",
        ),
        (
            "Each FS must not cite any distinct GOAL.",
            "\"distinct\" has no meaning in a prohibition; accepted form: Each FS must not cite any GOAL.",
        ),
        (
            "Each FS must not cite at least 2 distinct GOAL.",
            "a prohibition must use \"cite any\"; accepted form: Each FS must not cite any GOAL.",
        ),
        (
            "Each FS must cite at least 3 distinct GOAL or distinct REQ.",
            "\"distinct\" must follow the count; accepted form: Each FS must cite at least 3 distinct GOAL or REQ.",
        ),
        (
            "Each FS must cite at least 3 GOAL or distinct REQ.",
            "\"distinct\" must follow the count",
        ),
        (
            "Each FS must be cited by at least 2 distinct GOAL.",
            "\"distinct\" counts only outbound citations",
        ),
        (
            "Each FS must cite at least 3 distinct POLICY.",
            "unknown kind \"POLICY\"",
        ),
        (
            "Each FS must cite at least one distinct POLICY.",
            "unknown kind \"POLICY\"",
        ),
        (
            "Each FS must cite at least 3 distinct GOAL or POLICY.",
            "unknown kind \"POLICY\"",
        ),
        (
            // §FS-rule-refusals.sentences.order: refused at the count stage, bytes kept.
            "Each FS must cite at most 0 distinct GOAL.",
            "count must be a canonical positive base-10 integer",
        ),
    ];
    let mut wrong = Vec::new();
    for (title, expected) in rows {
        let read = parse(title, &vocab).map(|_| ());
        if read != Err(expected.to_string()) {
            wrong.push(format!(
                "{title}\n  got      {read:?}\n  expected {expected}"
            ));
        }
        // §FS-rule-refusals.sentences.rewrite.reparsed: every offered form is accepted.
        if let Some((_, form)) = expected.split_once("; accepted form: ")
            && let Err(error) = parse(form, &vocab)
        {
            wrong.push(format!("offered form refused: {form}: {error}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
