//! The four steps that build a refused selector's suggestion, on a
//! hand-written vocabulary (§FS-rules.8.1).
//!
//! The named-sections-off suggestion is pinned black-box in
//! `rules_contract/selector_refusals.rs`, since it reads the configuration
//! as if named sections were on.

use super::parse_selector;
use crate::config::Config;
use crate::rules::sentence::{RuleSubject, RuleVocabulary};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const HINT: &str = "\nhint: grund show --batch --toc expands each selected unit into its sections";

/// The configured `kinds` under `{kind}-{slug}`, named sections on.
fn vocabulary(kinds: &[&str]) -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("selector-vocabulary"));
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.named_sections = true;
    config
        .rebuild_grammar()
        .expect("named {kind}-{slug} grammar");
    let kinds = kinds
        .iter()
        .map(|kind| kind.to_string())
        .collect::<BTreeSet<_>>();
    RuleVocabulary {
        kinds: kinds.clone(),
        target_kinds: kinds,
        target_namespaces: BTreeMap::new(),
        named_sections: true,
        id_grammars: vec![config.grammar],
        section_separators: vec![".".into()],
    }
}

fn refusal(selector: &str) -> String {
    parse_selector(selector, &vocabulary(&["FS"]))
        .expect_err("refused selector")
        .render("known kinds: FS")
}

/// What follows `accepted selector: `, or `None` where nothing is suggested.
fn suggestion(selector: &str) -> Option<String> {
    let rendered = refusal(selector);
    let (_, rest) = rendered.split_once("accepted selector: ")?;
    rest.lines().next().map(str::to_string)
}

#[test]
fn the_kind_is_the_configured_kind_the_text_names_or_starts_with() {
    assert_eq!(
        suggestion("FS-*.requirements").as_deref(),
        Some("FS.requirements")
    );
    assert_eq!(suggestion("FS-login.*").as_deref(), Some("FS-login"));
    assert_eq!(suggestion("*/FS").as_deref(), Some("FS"));
    let refused = parse_selector("FSA-*.requirements", &vocabulary(&["FS", "FSA"]))
        .expect_err("a wildcard declaration is refused")
        .render("known kinds: FS, FSA");
    assert!(
        refused.ends_with("; accepted selector: FSA.requirements"),
        "the longest configured kind wins: {refused}"
    );
}

#[test]
fn the_longest_run_of_named_components_before_the_first_refused_one_is_kept() {
    assert_eq!(
        suggestion("FS.requirements.1").as_deref(),
        Some("FS.requirements")
    );
    assert_eq!(
        suggestion("FS.requirements.scope.2").as_deref(),
        Some("FS.requirements.scope")
    );
    assert_eq!(suggestion("FS.1.requirements").as_deref(), Some("FS"));
    assert_eq!(suggestion("FS.*").as_deref(), Some("FS"));
}

#[test]
fn the_typed_spelling_is_kept_and_falls_back_only_where_nothing_survives() {
    for (selector, expected) in [
        ("FS.requirements.1", "FS.requirements"),
        ("FS-login.requirements.1", "FS-login.requirements"),
        (
            "The requirements.1 chapter of each FS",
            "The requirements chapter of each FS",
        ),
        ("Each chapter of each FS", "Each FS"),
        // An ID whose declaration is not a valid ID becomes `KIND[.NAME…]`.
        ("FS-*.requirements", "FS.requirements"),
        ("FS-Login.1", "FS"),
        // A chapter spelling with no surviving NAME becomes `Each KIND`.
        ("The 1 chapter of each FS", "Each FS"),
        ("The * chapter of each FS", "Each FS"),
    ] {
        assert_eq!(
            suggestion(selector).as_deref(),
            Some(expected),
            "{selector}"
        );
    }
}

#[test]
fn nothing_is_suggested_where_no_configured_kind_is_recovered() {
    assert_eq!(
        refusal("Each POLICY"),
        "unknown kind \"POLICY\"\nknown kinds: FS"
    );
    assert_eq!(
        refusal("FSX-login"),
        "literal subject \"FSX-login\" does not match the configured ID grammar\nknown kinds: FS"
    );
}

#[test]
fn the_hint_follows_the_known_kinds_line_where_no_kind_is_recovered() {
    for (selector, reason) in [
        (
            "Each chapter of each POLICY",
            "chapter-quantified subjects are not accepted in phase 1",
        ),
        (
            "API.requirements.1",
            "numbered chapter subjects can detach when headings move",
        ),
        (
            "API.*",
            "section-component wildcards are not accepted in phase 1",
        ),
    ] {
        assert_eq!(
            refusal(selector),
            format!("{reason}\nknown kinds: FS{HINT}"),
            "{selector}"
        );
    }
}

#[test]
fn each_namespaced_kind_gets_the_namespace_reason_the_rule_sentence_gets() {
    assert_eq!(
        refusal("Each */FS"),
        "subject namespaces must be local in phase 1; accepted selector: Each FS"
    );
    assert_eq!(
        refusal("Each */POLICY"),
        "subject namespaces must be local in phase 1\nknown kinds: FS"
    );
}

#[test]
fn a_pasted_rule_sentence_is_answered_with_its_subject() {
    assert_eq!(
        refusal("FS-login must cite at least one GOAL."),
        "a rule sentence is not a selector; accepted selector: FS-login"
    );
    assert_eq!(
        refusal("The requirements chapter of each FS should not cite any AR."),
        "a rule sentence is not a selector; accepted selector: The requirements chapter of each FS"
    );
    // A subject that is itself refused answers with its own refusal.
    assert_eq!(
        refusal("FS.requirements.1 must cite at least one REQ."),
        format!(
            "numbered chapter subjects can detach when headings move; accepted selector: FS.requirements{HINT}"
        )
    );
    // A chapter named `must` or `should` is a selector, not a pasted sentence.
    assert_eq!(
        parse_selector("The must chapter of each FS", &vocabulary(&["FS"])),
        Ok(RuleSubject::ChapterOfKind {
            kind: "FS".into(),
            name: "must".into(),
        })
    );
}

/// §FS-rules.3.6: the subject ends where the rule sentence's does, the first
/// ` must not `, ` should not `, ` must ` or ` should ` in that order.
#[test]
fn a_pasted_rule_sentence_is_split_where_the_rule_sentence_is() {
    for (sentence, subject) in [
        (
            "The should chapter of each FS must cite at least one FS.",
            "The should chapter of each FS",
        ),
        (
            "The should chapter of each FS must not cite any AR.",
            "The should chapter of each FS",
        ),
        (
            "The must chapter of each FS should not cite any AR.",
            "The must chapter of each FS",
        ),
    ] {
        assert_eq!(
            refusal(sentence),
            format!("a rule sentence is not a selector; accepted selector: {subject}"),
            "{sentence}"
        );
    }
    // Guard: where the rule sentence's subject is `The`, so is the selector's.
    assert_eq!(
        refusal("The must chapter of each NOPE"),
        "literal subject \"The\" does not match the configured ID grammar\nknown kinds: FS"
    );
}
