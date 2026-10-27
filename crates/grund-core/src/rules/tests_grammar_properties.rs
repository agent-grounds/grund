//! Test module: the grammar properties that hold §AR-system.principle for what a
//! rule sentence or a selector means (one grammar), on a vocabulary whose kinds
//! are `FS` and `GOAL`, named sections on. Each property is asked of a fixed
//! corpus, and a violation is either new, which fails, or a row of `LEDGER`,
//! which fails as well once it stops reproducing, so the ledger only shrinks:
//!
//! - every suggestion a refusal prints parses again under the vocabulary it was
//!   offered for (§FS-rule-refusals.sentences.rewrite.reparsed,
//!   §FS-rule-refusals.selectors);
//! - a subject is accepted as a selector exactly when it is accepted as a rule's
//!   subject, and both read it as the same subject; refused, both name the same
//!   failure (§FS-rules.2, §FS-rule-refusals.selectors).
//!
//! `DEFERRED` names the properties today's code cannot be asked, each with the
//! step of #576 that makes it askable, listed live in
//! `tests/integration/one_answer_steps.toml` (§REQ-one-answer.proof). It is empty:
//! `--kind K` ⇔ `Each K` is asked by `kind_verdict_agreement.rs`, #626 left one
//! coordinate lexer for the primitive guard to hold, and no command renders a
//! parsed rule, so `parse(render(r)) == r` has nothing to ask that the reparse
//! property above does not.

use super::RuleAnchor;
use super::sentence::{RuleVocabulary, lower_case_each, parse_rule, parse_selector};
use crate::config::{Config, KindTable};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// The properties no test can ask yet, and the step that makes each askable.
const DEFERRED: &[(&str, &str)] = &[];

/// Known violations: `(property, input)`, each a row the step it names closes.
/// Seeded from today's code; a row whose violation is gone fails.
const LEDGER: &[(&str, &str)] = &[];

/// Rule sentences a refusal answers with a form.
const REFUSED_RULES: &[&str] = &[
    "Each FS may not cite any FS.",
    "Each FS should cite no FS.",
    "each FS must cite at least one FS.",
    "Each FS must be cited by at least one FS",
    "Each FS should be cited by exactly 1 FS.",
    "FS-login.requirements must cite each FS exactly 1 times.",
    "FS-login should cite each FS at least 2.",
    "Each FS must cite at most 007 FS.",
    "Each FS must cite a FS.",
    "Each FS must not cite exactly one FS.",
    "FS-login must have exactly one requirements section.",
    "FS-login must have at least 2 requirements chapter.",
    "Each FS must cite at most one FS.",
    "Each FS must cite at least one FS or GOAL.",
    "Each FS-* must cite at least one GOAL.",
    "Each FS.requirements must cite at least one GOAL.",
    "The requirements chapter of each FS-* must cite at least one GOAL.",
    "Each */FS must cite at least one GOAL.",
];

/// Subjects asked both as a selector and as a rule's subject.
const SUBJECTS: &[&str] = &[
    "Each FS",
    "Each GOAL",
    "FS-login",
    "FS-login.requirements",
    "The requirements chapter of each FS",
    "The requirements.performance chapter of each FS",
    "Each FSA",
    "Each FS-*",
    "FS-login.*",
    "FS-login.3",
    "The 3 chapter of each FS",
    "The requirements chapter of each FS-*",
    "Each */FS",
    "each FS",
];

/// The kinds `FS` and `GOAL` under `{kind}-{slug}`, named sections on.
fn vocabulary() -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("grammar-properties"));
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.named_sections = true;
    config
        .rebuild_grammar()
        .expect("{kind}-{slug} grammar with named sections");
    let kinds = BTreeSet::from(["FS".to_string(), "GOAL".to_string()]);
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

fn anchor() -> RuleAnchor {
    RuleAnchor {
        path: "docs/rules/RULE-x.md".into(),
        line: 1,
        column: None,
    }
}

fn rule(sentence: &str, vocabulary: &RuleVocabulary) -> Result<(), String> {
    parse_rule(sentence, "RULE-x".into(), anchor(), vocabulary)
        .map(|_| ())
        .map_err(|refusal| refusal.message)
}

/// The forms a refusal message offers: one after `accepted form: `, or each
/// quoted alternative after `accepted forms: `.
fn offered_forms(message: &str) -> Vec<String> {
    if let Some((_, many)) = message.split_once("accepted forms: ") {
        return many
            .split(" or ")
            .map(|form| form.trim().trim_matches('"').to_string())
            .collect();
    }
    match message.split_once("accepted form: ") {
        Some((_, one)) => vec![one.trim().to_string()],
        None => Vec::new(),
    }
}

/// What a refusal names as failed: its message up to the offer or the kinds line.
fn failure(message: &str) -> &str {
    let end = [";", "\n"]
        .iter()
        .filter_map(|stop| message.find(stop))
        .min()
        .unwrap_or(message.len());
    &message[..end]
}

/// Whether a selector's failure names the rule's: the same text, or, for a
/// `subject` opening with a lower-case `each`, a reason kept as a verbatim prefix
/// with the rule's appended after ` — `, the route §FS-rule-refusals.selectors
/// gives `each FS` (§DF-one-answer-rulings-grammar, G06).
fn names_the_failure(subject: &str, selector: &str, rule: &str) -> bool {
    selector == rule
        || (lower_case_each(subject).is_some() && selector.ends_with(&format!(" — {rule}")))
}

/// Every violation of the two askable properties over the corpus.
fn violations() -> BTreeSet<(&'static str, String)> {
    let vocabulary = vocabulary();
    let mut found = BTreeSet::new();
    for sentence in REFUSED_RULES {
        let Err(message) = rule(sentence, &vocabulary) else {
            continue;
        };
        for form in offered_forms(&message) {
            if rule(&form, &vocabulary).is_err() {
                found.insert(("offered form reparses", format!("{sentence} -> {form}")));
            }
        }
    }
    for subject in SUBJECTS {
        let as_rule = parse_rule(
            &format!("{subject} must cite at least one GOAL."),
            "RULE-x".into(),
            anchor(),
            &vocabulary,
        );
        let as_selector = parse_selector(subject, &vocabulary);
        match (as_rule, as_selector) {
            (Ok((parsed, _)), Ok(selected)) if parsed.subject != selected => {
                found.insert(("rule <=> selector", format!("{subject}: two subjects")));
            }
            (Ok(_), Ok(_)) => {}
            (Err(refused), Err(selector)) => {
                let selector = selector.render("known kinds: FS, GOAL");
                if let Some(form) = selector
                    .split_once("accepted selector: ")
                    .and_then(|(_, rest)| rest.lines().next())
                    && !selector.contains("after enabling")
                    && parse_selector(form, &vocabulary).is_err()
                {
                    found.insert(("offered form reparses", format!("{subject} -> {form}")));
                }
                if !names_the_failure(subject, failure(&selector), failure(&refused.message)) {
                    found.insert(("rule <=> selector", format!("{subject}: two failures")));
                }
            }
            (Ok(_), Err(_)) => {
                found.insert(("rule <=> selector", format!("{subject}: rule only")));
            }
            (Err(_), Ok(_)) => {
                found.insert(("rule <=> selector", format!("{subject}: selector only")));
            }
        }
    }
    found
}

/// What is wrong with `ledger` against what was `found`, both ways round.
fn problems(found: &BTreeSet<(&str, String)>, ledger: &[(&str, &str)]) -> Vec<String> {
    let listed: BTreeSet<(&str, String)> = ledger
        .iter()
        .map(|(property, input)| (*property, input.to_string()))
        .collect();
    let mut wrong: Vec<String> = found
        .difference(&listed)
        .map(|(property, input)| format!("new violation of `{property}`: {input}"))
        .collect();
    wrong.extend(listed.difference(found).map(|(property, input)| {
        format!("ledger row `{property}`: {input} no longer reproduces: take it out")
    }));
    wrong
}

#[test]
fn every_askable_grammar_property_holds_but_for_the_ledger() {
    let wrong = problems(&violations(), LEDGER);
    assert!(
        wrong.is_empty(),
        "{} grammar propert(y/ies) broken (deferred to their steps: {DEFERRED:?}):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn the_ledger_fails_on_a_new_violation_and_on_a_stale_row() {
    let found = BTreeSet::from([("p", "a".to_string()), ("p", "b".to_string())]);
    assert!(problems(&found, &[("p", "a"), ("p", "b")]).is_empty());
    assert_eq!(problems(&found, &[("p", "a")]), ["new violation of `p`: b"]);
    assert_eq!(
        problems(&BTreeSet::new(), &[("p", "a")]),
        ["ledger row `p`: a no longer reproduces: take it out"]
    );
}

/// The ids of `tests/integration/one_answer_steps.toml`, the one list of the live
/// steps of #576 (§REQ-one-answer.proof), read off its `id = "…"` lines.
fn listed_steps() -> BTreeSet<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/integration/one_answer_steps.toml");
    let text = std::fs::read_to_string(path).expect("read one_answer_steps.toml");
    text.lines().filter_map(step_id).collect()
}

/// The id an `id = "…"` line of one_answer_steps.toml sets, whatever its indent,
/// quote or trailing `# comment`; an id itself may start with `#`.
fn step_id(line: &str) -> Option<String> {
    let rest = line
        .trim()
        .strip_prefix("id")?
        .trim_start()
        .strip_prefix('=')?
        .trim_start();
    let quote = rest.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let (id, tail) = rest[1..].split_once(quote)?;
    let tail = tail.trim();
    (tail.is_empty() || tail.starts_with('#')).then(|| id.to_string())
}

#[test]
fn every_deferred_property_names_its_step() {
    let steps = listed_steps();
    for (property, step) in DEFERRED {
        assert!(
            steps.contains(*step),
            "{property} names no step listed in one_answer_steps.toml: {step}"
        );
    }
}

#[test]
fn the_offers_are_read_off_both_message_shapes() {
    assert_eq!(
        offered_forms("x; accepted form: Each FS must cite at least one FS."),
        ["Each FS must cite at least one FS."]
    );
    assert_eq!(
        offered_forms("x; accepted forms: \"A.\" or \"B.\""),
        ["A.", "B."]
    );
    assert!(offered_forms("conjunctions are not accepted").is_empty());
    assert_eq!(
        failure("unknown kind `FSA`\nknown kinds: FS"),
        "unknown kind `FSA`"
    );
}
