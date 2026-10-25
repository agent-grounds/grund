//! What §FS-rule-refusals.selectors's four steps recover, spelled by both surfaces that paste
//! it back, on a hand-written vocabulary with named sections off
//! (§FS-rule-refusals.sentences.named-sections). A selector's `KIND.NAME[.NAME…]` is a rule's
//! `The NAME[.NAME…] chapter of each KIND`, and a selector's bare `KIND` is a
//! rule's `Each KIND`.

use super::{Recovered, as_if_enabled, enabled, recover};
use crate::config::{Config, KindTable};
use crate::rules::sentence::RuleVocabulary;
use crate::rules::sentence::subjects::parse_subject;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// The configured `kinds` under `{kind}-{slug}`, named sections off.
fn named_off(kinds: &[&str]) -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("recovery-vocabulary"));
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.named_sections = false;
    config
        .rebuild_grammar()
        .expect("{kind}-{slug} grammar without named sections");
    let kinds = kinds
        .iter()
        .map(|kind| kind.to_string())
        .collect::<BTreeSet<_>>();
    RuleVocabulary {
        objects: KindTable::of_citable(&kinds),
        kinds,
        named_sections: false,
        named_kinds: Default::default(),
        non_citable: Default::default(),
        id_grammars: vec![config.grammar],
        section_separators: vec![".".into()],
    }
}

/// The selector and the rule subject recovered from `subject`'s refusal once
/// named sections are on, or `None` where no configured kind is recovered.
fn spelled(subject: &str, off: &RuleVocabulary) -> Option<(String, String)> {
    let on = enabled(off).expect("{kind}-{slug} compiles with named sections");
    let refusal = parse_subject(subject, &on).expect_err("refused with named sections on");
    recover(&refusal, &on).map(|recovered| (recovered.selector(), recovered.rule_subject()))
}

/// §FS-rule-refusals.sentences.rewrite.recovered: recovery uses typed scope and
/// configured grammar alone, without consulting a scan for chapter names.
#[test]
fn offer_scope_keeps_recoverable_chapters_and_literals_or_offers_nothing() {
    let vocabulary = named_off(&["FS", "REQ"]);
    let mut wrong = Vec::new();
    for (subject, canonical) in [
        (
            "Each FS requirements chapter",
            "The requirements chapter of each FS",
        ),
        (
            "Each FS requirements",
            "The requirements chapter of each FS",
        ),
        (
            "Each FS.requirements",
            "The requirements chapter of each FS",
        ),
        ("FS-store requirements chapter", "FS-store.requirements"),
        ("Each FS-store.requirements", "FS-store.requirements"),
        ("Each FS-store", "FS-store"),
    ] {
        if let Some((selector, rule)) = spelled(subject, &vocabulary) {
            let selectors = if canonical.starts_with("FS-store") {
                vec![canonical]
            } else {
                vec![canonical, "FS.requirements"]
            };
            if rule != canonical || !selectors.contains(&selector.as_str()) {
                wrong.push(format!(
                    "{subject}: recovered selector {selector:?}, rule subject {rule:?}; \
                     expected scope {canonical:?} or no offer"
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// §FS-rule-refusals.sentences.rewrite.recovered: the invalid-component policy
/// still applies when `Each` left the refused path in the subject head.
#[test]
fn each_path_recovery_keeps_the_valid_prefix_on_both_surfaces() {
    let vocabulary = named_off(&["FS"]);
    for (subject, canonical) in [
        ("Each FS.*", "Each FS"),
        (
            "Each FS.requirements.*",
            "The requirements chapter of each FS",
        ),
        (
            "Each FS.requirements.1",
            "The requirements chapter of each FS",
        ),
    ] {
        assert_eq!(
            spelled(subject, &vocabulary),
            Some((canonical.into(), canonical.into())),
            "{subject}"
        );
    }
}

/// §FS-rule-refusals.sentences.rewrite.recovered: knowing a kind supplies no
/// chapter name for an unnamed quantified chapter subject.
#[test]
fn offer_scope_suppresses_unnamed_chapter_quantifiers() {
    assert_eq!(
        spelled("Each chapter of each FS", &named_off(&["FS", "REQ"])),
        None,
        "a whole-declaration offer would change the typed chapter scope"
    );
}

/// §FS-rule-refusals.sentences.named-sections step 2: every rebuilt row of the table, and the rewrites the
/// point states beyond it, from one set of recovered parts.
#[test]
fn every_rebuilt_row_is_spelled_as_a_selector_and_as_a_rule_subject() {
    let off = named_off(&["FS", "REQ", "GOAL"]);
    let chapter = "The requirements chapter of each FS";
    for (subject, selector, rule) in [
        ("FS-*.requirements", "FS.requirements", chapter),
        ("FS.requirements", "FS.requirements", chapter),
        ("The requirements.1 chapter of each FS", chapter, chapter),
        ("FS-login.Requirements", "FS-login", "FS-login"),
        ("The Requirements chapter of each FS", "Each FS", "Each FS"),
        ("The * chapter of each FS", "Each FS", "Each FS"),
        (
            "FS.requirements.security",
            "FS.requirements.security",
            "The requirements.security chapter of each FS",
        ),
        ("FS-*.Requirements", "FS", "Each FS"),
        // A declaration written as a valid ID is kept, with its named run.
        (
            "FS-login.requirements.Scope",
            "FS-login.requirements",
            "FS-login.requirements",
        ),
    ] {
        assert_eq!(
            spelled(subject, &off),
            Some((selector.to_string(), rule.to_string())),
            "{subject}"
        );
    }
}

/// §FS-rule-refusals.sentences.named-sections step 4: nothing is recovered where no configured kind is,
/// and the kind is the longest configured one the text starts with.
#[test]
fn the_kind_is_recovered_from_the_configured_kinds_or_not_at_all() {
    assert_eq!(
        spelled("POLICY.requirements", &named_off(&["FS", "REQ", "GOAL"])),
        None
    );
    assert_eq!(
        spelled("FSA-*.requirements", &named_off(&["FS", "FSA", "REQ"])),
        Some((
            "FSA.requirements".to_string(),
            "The requirements chapter of each FSA".to_string()
        ))
    );
}

/// §FS-rule-refusals.sentences.named-sections steps 1 and 3 on the rule side: a subject enabling makes
/// valid is kept as typed, and the label follows from parsing the suggestion
/// as configured.
#[test]
fn a_subject_enabling_makes_valid_is_kept_and_the_label_is_decided_by_parsing() {
    let off = named_off(&["FS", "REQ", "GOAL"]);
    let suggest = |subject| {
        as_if_enabled(subject, &off, parse_subject, Recovered::rule_subject)
            .map(|suggested| (suggested.text, suggested.after_enabling))
    };
    let chapter = "The requirements chapter of each FS";
    for (subject, text, after_enabling) in [
        ("FS-login.requirements", "FS-login.requirements", true),
        (chapter, chapter, true),
        ("FS-*.requirements", chapter, true),
        ("FS-login.Requirements", "FS-login", false),
        ("The * chapter of each FS", "Each FS", false),
    ] {
        assert_eq!(
            suggest(subject),
            Some((text.to_string(), after_enabling)),
            "{subject}"
        );
    }
    assert_eq!(suggest("POLICY.requirements"), None);
}
