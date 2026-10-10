//! A chapter subject refused for its chapter path, read through `parse_rule` on
//! a hand-written vocabulary (§FS-rules.3.5.3). The message is what a
//! configured rule declaration's `invalid-rule` finding carries after
//! `is not a valid rule: ` and what `check --rule` prints after `error: `
//! (§FS-rules.7.1); `rules_contract/chapter_path_reasons.rs` pins both
//! black-box.

use super::RuleAnchor;
use super::sentence::{RuleVocabulary, parse_rule};
use crate::config::Config;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const NUMBERED: &str = "numbered chapter subjects can detach when headings move";
const WILDCARD: &str = "section-component wildcards are not accepted in phase 1";
/// What follows a reason whose accepted form has the subject `subject`
/// (§FS-rules.3.5.4.2).
fn form(subject: &str) -> String {
    format!("; accepted form: {subject} must cite at least one REQ.")
}

/// Kinds `FS` and `REQ` under `{kind}-{slug}`, named sections as given.
fn vocabulary(named_sections: bool) -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("chapter-path-vocabulary"));
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.named_sections = named_sections;
    config.rebuild_grammar().expect("{kind}-{slug} grammar");
    let kinds = ["FS", "REQ"]
        .iter()
        .map(|kind| kind.to_string())
        .collect::<BTreeSet<_>>();
    RuleVocabulary {
        kinds: kinds.clone(),
        target_kinds: kinds,
        target_namespaces: BTreeMap::new(),
        named_sections,
        id_grammars: vec![config.grammar],
        section_separators: vec![".".into()],
    }
}

/// The section-grammar reason for `subject`.
fn grammar(subject: &str) -> String {
    format!("named chapter subject \"{subject}\" does not match the configured section grammar")
}

/// Every row whose refusal of `<subject> must cite at least one REQ.` is not
/// the expected message, so one wrong row never hides another.
fn wrong(rows: &[(&str, String)], vocabulary: &RuleVocabulary) -> Vec<String> {
    let anchor = RuleAnchor {
        path: "docs/rules/RULE-x.md".into(),
        line: 1,
        column: None,
    };
    rows.iter()
        .filter_map(|(subject, expected)| {
            let sentence = format!("{subject} must cite at least one REQ.");
            let actual = match parse_rule(&sentence, "RULE-x".into(), anchor.clone(), vocabulary) {
                Ok(_) => "accepted".to_string(),
                Err(error) => error.message,
            };
            (&actual != expected)
                .then(|| format!("{subject:?}\n  message  {actual:?}\n  expected {expected:?}"))
        })
        .collect()
}

fn assert_rows(rows: &[(&str, String)], vocabulary: &RuleVocabulary) {
    let wrong = wrong(rows, vocabulary);
    assert!(
        wrong.is_empty(),
        "{} of {} chapter paths were not refused as FS-rules.3.5.3 says:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

/// §FS-rules.3.5.3: the exact table. A numbered or wildcard component in
/// `The PATH chapter of each KIND` gets the reason its literal spelling gets,
/// and an empty component is refused off the section grammar, not as numbered.
/// The form keeps the named components typed before it (§FS-rules.3.5.4.2).
#[test]
fn a_chapter_path_is_refused_for_the_component_that_failed() {
    let chapter = form("The requirements chapter of each FS");
    let rows = [
        (
            "The requirements.1 chapter of each FS",
            format!("{NUMBERED}{chapter}"),
        ),
        (
            "The requirements.* chapter of each FS",
            format!("{WILDCARD}{chapter}"),
        ),
        (
            "FS-login.requirements.",
            grammar("FS-login.requirements.") + &form("FS-login.requirements"),
        ),
        (
            "FS-login..requirements",
            grammar("FS-login..requirements") + &form("FS-login"),
        ),
        ("FS-login.", grammar("FS-login.") + &form("FS-login")),
    ];
    assert_rows(&rows, &vocabulary(true));
}

/// §FS-rules.3.5.3: the first refused component decides. In the literal
/// spelling that is the first empty or all-digit one; in
/// `The PATH chapter of each KIND` the first that is not a named one.
#[test]
fn the_first_refused_component_decides_the_reason() {
    let rows = [
        ("FS-login..1", grammar("FS-login..1") + &form("FS-login")),
        ("FS-login.1.", format!("{NUMBERED}{}", form("FS-login"))),
        (
            "The 1.requirements chapter of each FS",
            format!("{NUMBERED}{}", form("Each FS")),
        ),
        (
            "The requirements..1 chapter of each FS",
            grammar("The requirements..1 chapter of each FS")
                + &form("The requirements chapter of each FS"),
        ),
        (
            "The Requirements.1 chapter of each FS",
            grammar("The Requirements.1 chapter of each FS") + &form("Each FS"),
        ),
    ];
    assert_rows(&rows, &vocabulary(true));
}

/// §FS-rules.3.5.4.2: a literal keeps its own declaration and the named
/// components typed before the one that failed, whichever component failed.
#[test]
fn a_literal_keeps_its_declaration_and_the_components_before_the_failed_one() {
    let rows = [
        (
            "FS-demo.requirements.",
            grammar("FS-demo.requirements.") + &form("FS-demo.requirements"),
        ),
        (
            "FS-demo..requirements",
            grammar("FS-demo..requirements") + &form("FS-demo"),
        ),
        ("FS-demo.2", format!("{NUMBERED}{}", form("FS-demo"))),
    ];
    assert_rows(&rows, &vocabulary(true));
}

/// §FS-rules.3.5.3: the literal spelling judges its path before it asks about
/// named sections, so an empty component is refused off the section grammar
/// with them off too, while the chapter spelling, which asks first, keeps its
/// named-sections-off refusal (§FS-rules.3.5.2). With them off the literal is
/// offered `FS-login`, the subject `list --selector` gives it then too.
#[test]
fn an_empty_literal_component_is_judged_before_named_sections_are_asked_about() {
    let rows = [
        (
            "FS-login.requirements.",
            grammar("FS-login.requirements.") + &form("FS-login"),
        ),
        (
            "FS-login..requirements",
            grammar("FS-login..requirements") + &form("FS-login"),
        ),
        ("FS-login.2", format!("{NUMBERED}{}", form("FS-login"))),
        (
            "The requirements.1 chapter of each FS",
            "named chapter subjects require [id] named_sections = true; accepted form after enabling it: The requirements chapter of each FS must cite at least one REQ.".into(),
        ),
    ];
    assert_rows(&rows, &vocabulary(false));
}
