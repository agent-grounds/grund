//! A rule subject refused for needing named sections, read through `parse_rule`
//! on a hand-written vocabulary (§FS-rules.3.5.2). The message is what a
//! configured rule declaration's `invalid-rule` finding carries after
//! `is not a valid rule: ` (§FS-rules.7.1); `check --rule` prints it after
//! `error: ` and adds its `known kinds:` line where no form is offered, which
//! `rules_contract/refusals.rs` pins black-box.

use super::RuleAnchor;
use super::sentence::{RuleVocabulary, parse_rule};
use crate::config::Config;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// The reason every subject refused for needing named sections gives.
const NAMED_OFF: &str = "named chapter subjects require [id] named_sections = true";

/// The configured `kinds` under `{kind}-{slug}`, named sections off.
fn vocabulary(kinds: &[&str]) -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("named-off-vocabulary"));
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
        kinds: kinds.clone(),
        target_kinds: kinds,
        target_namespaces: BTreeMap::new(),
        named_sections: false,
        id_grammars: vec![config.grammar],
        section_separators: vec![".".into()],
    }
}

/// The refusal of `<subject> must cite at least one REQ.`, as a finding's
/// message carries it.
fn message(subject: &str, vocabulary: &RuleVocabulary) -> String {
    let anchor = RuleAnchor {
        path: "docs/rules/RULE-x.md".into(),
        line: 1,
        column: None,
    };
    let sentence = format!("{subject} must cite at least one REQ.");
    parse_rule(&sentence, "RULE-x".into(), anchor, vocabulary)
        .expect_err("a subject needing named sections is refused while they are off")
        .message
}

/// Every pair whose message is not the expected one, so one wrong row never
/// hides another.
fn wrong(rows: &[(&str, String)], vocabulary: &RuleVocabulary) -> Vec<String> {
    rows.iter()
        .filter_map(|(subject, expected)| {
            let actual = message(subject, vocabulary);
            (&actual != expected)
                .then(|| format!("{subject:?}\n  message  {actual:?}\n  expected {expected:?}"))
        })
        .collect()
}

/// §FS-rules.3.5.2: every row of the exact table, and the two rewrites it
/// states beyond them: a selector's multi-component `KIND.NAME.NAME` is written
/// `The NAME.NAME chapter of each KIND`, and its bare `KIND` is `Each KIND`.
#[test]
fn a_subject_needing_named_sections_is_answered_with_one_they_make_valid() {
    let after = "accepted form after enabling it";
    let chapter = "The requirements chapter of each FS must cite at least one REQ.";
    let rows = [
        (
            "FS-login.requirements",
            format!("{NAMED_OFF}; {after}: FS-login.requirements must cite at least one REQ."),
        ),
        (
            "The requirements chapter of each FS",
            format!("{NAMED_OFF}; {after}: {chapter}"),
        ),
        (
            "FS-*.requirements",
            format!("{NAMED_OFF}; {after}: {chapter}"),
        ),
        (
            "FS.requirements",
            format!("{NAMED_OFF}; {after}: {chapter}"),
        ),
        (
            "The requirements.1 chapter of each FS",
            format!("{NAMED_OFF}; {after}: {chapter}"),
        ),
        (
            "FS-login.Requirements",
            format!("{NAMED_OFF}; accepted form: FS-login must cite at least one REQ."),
        ),
        (
            "The Requirements chapter of each FS",
            format!("{NAMED_OFF}; accepted form: Each FS must cite at least one REQ."),
        ),
        (
            "The * chapter of each FS",
            format!("{NAMED_OFF}; accepted form: Each FS must cite at least one REQ."),
        ),
        // §FS-rules.7.1: a finding that offers no form ends at the reason.
        ("POLICY.requirements", NAMED_OFF.to_string()),
        (
            "FS.requirements.security",
            format!(
                "{NAMED_OFF}; {after}: The requirements.security chapter of each FS must cite at least one REQ."
            ),
        ),
        (
            "FS-*.Requirements",
            format!("{NAMED_OFF}; accepted form: Each FS must cite at least one REQ."),
        ),
    ];
    let wrong = wrong(&rows, &vocabulary(&["FS", "REQ", "GOAL"]));
    assert!(
        wrong.is_empty(),
        "{} of {} subjects were not answered as FS-rules.3.5.2 says:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

/// §FS-rules.3.5.2 step 2, through §FS-rules.8.1 step 1: the kind is the
/// longest configured kind the text starts with.
#[test]
fn the_rebuilt_chapter_names_the_longest_configured_kind() {
    let rows = [(
        "FSA-*.requirements",
        format!(
            "{NAMED_OFF}; accepted form after enabling it: The requirements chapter of each FSA must cite at least one REQ."
        ),
    )];
    let wrong = wrong(&rows, &vocabulary(&["FS", "FSA", "REQ"]));
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
