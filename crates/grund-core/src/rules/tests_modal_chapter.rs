//! A chapter subject whose name is a modal word, read through `parse_rule` and
//! `parse_selector` on a hand-written vocabulary
//! (§FS-rule-refusals.sentences.modal-chapter). The split of §FS-rules.3.6 cuts
//! it at that name and leaves the subject `The`; the released ID-grammar reason
//! stays a prefix and the name follows it. `rules_contract/modal_chapter.rs`
//! pins the same rows black-box.

use super::RuleAnchor;
use super::sentence::{RuleVocabulary, parse_rule, parse_selector};
use crate::config::{Config, KindTable};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Kind `FS` under `{kind}-{slug}`, named sections on.
fn vocabulary() -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("modal-chapter-vocabulary"));
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.named_sections = true;
    config.rebuild_grammar().expect("{kind}-{slug} grammar");
    let kinds = BTreeSet::from(["FS".to_string()]);
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

/// §FS-rule-refusals.sentences.modal-chapter: the released reason, then the
/// modal word the split took as the chapter's name.
fn modal(name: &str) -> String {
    format!(
        "literal subject \"The\" does not match the configured ID grammar — chapter name \
         \"{name}\" is read as the rule's modality, so rename the chapter"
    )
}

/// What `parse_rule` answers `sentence` with: `accepted`, or the message.
fn rule(sentence: &str) -> String {
    let anchor = RuleAnchor {
        path: "docs/rules/RULE-x.md".into(),
        line: 1,
        column: None,
    };
    match parse_rule(sentence, "RULE-x".into(), anchor, &vocabulary()) {
        Ok(_) => "accepted".into(),
        Err(error) => error.message,
    }
}

/// What `parse_selector` answers `selector` with: `accepted`, or its lines.
fn selector(selector: &str) -> String {
    match parse_selector(selector, &vocabulary()) {
        Ok(_) => "accepted".into(),
        Err(refusal) => refusal.render("known kinds: FS"),
    }
}

/// Every row whose answer is not the expected one, so one never hides another.
fn wrong(rows: &[(&str, String)], answer: fn(&str) -> String) -> Vec<String> {
    rows.iter()
        .filter_map(|(input, expected)| {
            let actual = answer(input);
            (&actual != expected)
                .then(|| format!("{input:?}\n  answer   {actual:?}\n  expected {expected:?}"))
        })
        .collect()
}

/// §FS-rule-refusals.sentences.modal-chapter: each modal name, cut by each
/// modality that reaches it first, is refused for that name and not for `The`.
#[test]
fn a_chapter_named_after_a_modal_word_is_refused_for_that_name() {
    let rows = [
        (
            "The must chapter of each FS must cite at least one FS.",
            modal("must"),
        ),
        (
            "The must chapter of each FS should cite at least one FS.",
            modal("must"),
        ),
        (
            "The should chapter of each FS should cite at least one FS.",
            modal("should"),
        ),
        (
            "The must not chapter of each FS must cite at least one FS.",
            modal("must not"),
        ),
        (
            "The must not chapter of each FS should not cite any FS.",
            modal("must not"),
        ),
        (
            "The should not chapter of each FS should cite at least one FS.",
            modal("should not"),
        ),
        (
            "The should not chapter of each FS should not cite any FS.",
            modal("should not"),
        ),
        (
            "The must chapter of each FS must have exactly one terms chapter.",
            modal("must"),
        ),
    ];
    let wrong = wrong(&rows, rule);
    assert!(
        wrong.is_empty(),
        "{} of {} modal-word chapters were not refused for their name:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

/// §FS-rule-refusals.sentences.modal-chapter: `list --selector` splits where the
/// rule does, so it gives the same reason, then its `known kinds:` line.
#[test]
fn a_selector_cut_at_a_modal_chapter_name_gives_the_same_reason() {
    let rows = [
        (
            "The should chapter of each FS should cite at least one FS.",
            format!("{}\nknown kinds: FS", modal("should")),
        ),
        (
            "The must not chapter of each FS",
            format!("{}\nknown kinds: FS", modal("must not")),
        ),
    ];
    let wrong = wrong(&rows, selector);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Guard, green before and after §FS-rule-refusals.sentences.modal-chapter: a
/// split that takes the rule's own modality first is accepted, and a subject
/// `The` that does not open a chapter subject keeps the released reason alone.
#[test]
fn a_split_that_takes_the_rules_own_modality_first_is_untouched() {
    for sentence in [
        "The should chapter of each FS must cite at least one FS.",
        "The must chapter of each FS should not cite any FS.",
        "The should chapter of each FS must not cite any FS.",
    ] {
        assert_eq!(rule(sentence), "accepted", "{sentence:?}");
    }
    assert_eq!(selector("The must chapter of each FS"), "accepted");
    assert_eq!(
        rule("The must cite at least one FS."),
        "literal subject \"The\" does not match the configured ID grammar"
    );
}
