//! A refused rule's accepted form is the sentence its author typed with only
//! the refused part replaced (§FS-rule-refusals.sentences.rewrite), read through `parse_rule` on a
//! vocabulary whose one kind is `FS`, named sections on. The message is what a
//! configured rule declaration's `invalid-rule` finding carries after
//! `is not a valid rule: ` (§FS-rules.7.1), and `unrecovered` is whether
//! `check --rule` follows it with `known kinds:`, which
//! `rules_contract/refusal_forms.rs` pins black-box.

use super::RuleAnchor;
use super::sentence::{RuleVocabulary, parse_rule};
use crate::config::{Config, KindTable};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// The one kind `FS` under `{kind}-{slug}`, named sections on.
fn vocabulary() -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("fs-only-vocabulary"));
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.named_sections = true;
    config
        .rebuild_grammar()
        .expect("{kind}-{slug} grammar with named sections");
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

/// One refused sentence, the message it is refused with, and whether what
/// could not be supplied is a kind.
type Row = (&'static str, &'static str, bool);

/// Every row whose refusal is not exactly its own, so one never hides another.
fn wrong(rows: &[Row]) -> Vec<String> {
    let vocabulary = vocabulary();
    rows.iter()
        .filter_map(|&(sentence, expected, unrecovered)| {
            let anchor = RuleAnchor {
                path: "docs/rules/RULE-x.md".into(),
                line: 1,
                column: None,
            };
            let refusal = match parse_rule(sentence, "RULE-x".into(), anchor, &vocabulary) {
                Ok(_) => return Some(format!("{sentence:?} was accepted")),
                Err(refusal) => refusal,
            };
            (refusal.message != expected || refusal.unrecovered != unrecovered).then(|| {
                format!(
                    "{sentence:?}\n  message  {:?}, unrecovered {}\n  expected {expected:?}, unrecovered {unrecovered}",
                    refusal.message, refusal.unrecovered
                )
            })
        })
        .collect()
}

fn assert_rows(rows: &[Row]) {
    let wrong = wrong(rows);
    assert!(
        wrong.is_empty(),
        "{} of {} refusals were not as FS-rule-refusals.sentences.rewrite says:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

/// §FS-rule-refusals.sentences.rewrite.replaced: a refused token is replaced by its canonical spelling,
/// and the subject, modality and object kinds stay as typed.
#[test]
fn a_refused_token_is_replaced_under_the_typed_subject_and_kinds() {
    assert_rows(&[
        (
            "Each FS may not cite any FS.",
            "modality \"may not\" is not accepted; accepted form: Each FS must not cite any FS.",
            false,
        ),
        (
            "Each FS should cite no FS.",
            "\"cite no\" is not accepted; accepted form: Each FS should not cite any FS.",
            false,
        ),
        (
            "each FS must cite at least one FS.",
            "fixed word \"Each\" is case-sensitive; accepted form: Each FS must cite at least one FS.",
            false,
        ),
        (
            "Each FS must be cited by at least one FS",
            "rule must end with \".\"; accepted form: Each FS must be cited by at least one FS.",
            false,
        ),
        (
            "Each FS should be cited by exactly 1 FS.",
            "numeric \"exactly 1\" is not canonical; accepted form: Each FS should be cited by exactly one FS.",
            false,
        ),
        (
            "FS-login.requirements must cite each FS exactly 1 times.",
            "numeric \"exactly 1 times\" is not canonical; accepted form: FS-login.requirements must cite each FS exactly once.",
            false,
        ),
    ]);
}

/// §FS-rule-refusals.sentences.rewrite.by-refusal: a per-target count without `times` keeps the count typed
/// after its bound, and a count written with leading zeros loses them; a count
/// that is zero or no numeral has nothing to keep.
#[test]
fn a_count_keeps_the_number_typed_or_offers_no_form() {
    let canonical = "count must be a canonical positive base-10 integer";
    assert_rows(&[
        (
            "FS-login should cite each FS at least 2.",
            "per-target counts must end in \"times\"; accepted form: FS-login should cite each FS at least 2 times.",
            false,
        ),
        (
            "FS-login must cite each FS at least two.",
            "per-target counts must end in \"times\"",
            false,
        ),
        (
            "Each FS must cite at most 007 FS.",
            "count must be a canonical positive base-10 integer; accepted form: Each FS must cite at most 7 FS.",
            false,
        ),
        ("Each FS must cite at least 00 FS.", canonical, false),
        ("Each FS must cite at least two FS.", canonical, false),
    ]);
}

/// §FS-rule-refusals.sentences.rewrite.reparsed: a two-form refusal offers its pair only where both forms
/// are accepted, so the documented conjunction, whose clauses name kinds this
/// vocabulary lacks, offers none and leaves the kinds to `check --rule`.
#[test]
fn a_two_form_refusal_offers_both_or_neither() {
    assert_rows(&[
        (
            "Each FS must cite a FS.",
            "quantifier \"a\" is ambiguous; accepted forms: \"Each FS must cite at least one FS.\" or \"Each FS must cite exactly one FS.\"",
            false,
        ),
        (
            "Each FS must cite at least one GOAL and must not cite any AR.",
            "conjunctions are not accepted",
            true,
        ),
    ]);
}

/// §FS-rule-refusals.sentences.rewrite.by-refusal: a prohibition's `cite [count] KINDS` becomes
/// `cite any KINDS`, and any other prohibited predicate has no form.
#[test]
fn a_prohibition_becomes_cite_any_or_offers_no_form() {
    assert_rows(&[
        (
            "Each FS must not cite exactly one FS.",
            "a prohibition must use \"cite any\"; accepted form: Each FS must not cite any FS.",
            false,
        ),
        (
            "Each FS should not be cited by at least 2 FS.",
            "a prohibition must use \"cite any\"",
            false,
        ),
    ]);
}

/// §FS-rule-refusals.sentences.rewrite.by-refusal: a presence object's last word becomes `chapter` or
/// `chapters` as the count takes, under the typed subject and modality, and no
/// chapter name is invented where none was typed (§FS-rule-refusals.sentences.rewrite.recovered).
#[test]
fn a_presence_noun_is_respelled_under_the_typed_subject() {
    assert_rows(&[
        (
            "FS-login must have exactly one requirements section.",
            "chapter presence must end in \"chapter\"; accepted form: FS-login must have exactly one requirements chapter.",
            false,
        ),
        (
            "FS-login must have exactly one section.",
            "chapter presence must end in \"chapter\"",
            false,
        ),
        (
            "FS-login must have at least 2 requirements chapter.",
            "chapter count has the wrong singular/plural spelling; accepted form: FS-login must have at least 2 requirements chapters.",
            false,
        ),
        (
            "FS-login should have exactly one requirements chapters.",
            "chapter count has the wrong singular/plural spelling; accepted form: FS-login should have exactly one requirements chapter.",
            false,
        ),
    ]);
}

/// §FS-rule-refusals.sentences.whitespace: the presence `NAME` is trimmed under the typed subject.
#[test]
fn a_presence_name_is_trimmed_under_the_typed_subject() {
    assert_rows(&[(
        "FS-login should have at most 2  requirements chapters.",
        "chapter name must be a non-empty NAME with no surrounding whitespace; accepted form: FS-login should have at most 2 requirements chapters. NAME forbids whitespace anywhere.",
        false,
    )]);
}

/// §FS-rule-refusals.sentences.rewrite.by-refusal: a chapter subject in a presence rule becomes its
/// declaration, or `Each KIND`, and the presence predicate stays as typed.
#[test]
fn a_chapter_subject_with_chapters_becomes_its_declaration() {
    assert_rows(&[
        (
            "FS-login.requirements must have exactly one should chapter.",
            "chapter subjects cannot have chapters; accepted form: FS-login must have exactly one should chapter.",
            false,
        ),
        (
            "The requirements chapter of each FS should have at least one should chapter.",
            "chapter subjects cannot have chapters; accepted form: Each FS should have at least one should chapter.",
            false,
        ),
    ]);
}

/// §FS-rule-refusals.sentences.rewrite.recovered: a refused subject is rebuilt with §FS-rule-refusals.selectors's steps
/// and written as a rule subject, keeping the named components typed before
/// the refused one; where no configured kind is recovered there is no form.
#[test]
fn a_refused_subject_is_rebuilt_as_the_selector_rebuilds_it() {
    assert_rows(&[
        (
            "FS-*.requirements must cite at least one FS.",
            "literal subject \"FS-*.requirements\" does not match the configured ID grammar; accepted form: The requirements chapter of each FS must cite at least one FS.",
            false,
        ),
        (
            "FS-login.Requirements must cite at least one FS.",
            "literal subject \"FS-login.Requirements\" does not match the configured ID grammar; accepted form: FS-login must cite at least one FS.",
            false,
        ),
        (
            "The Requirements chapter of each FS must cite at least one FS.",
            "named chapter subject \"The Requirements chapter of each FS\" does not match the configured section grammar; accepted form: Each FS must cite at least one FS.",
            false,
        ),
        (
            "FS.* must cite at least one FS.",
            "section-component wildcards are not accepted in phase 1; accepted form: Each FS must cite at least one FS.",
            false,
        ),
        (
            "FS-login.requirements.2 should cite at most 2 FS.",
            "numbered chapter subjects can detach when headings move; accepted form: FS-login.requirements should cite at most 2 FS.",
            false,
        ),
        (
            "Each chapter of each FS must be cited by at least one FS.",
            "chapter-quantified subjects are not accepted in phase 1",
            false,
        ),
        (
            "Each */FS must cite at least one FS.",
            "subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one FS.",
            false,
        ),
        (
            "Each POLICY must cite at least one FS.",
            "unknown kind \"POLICY\"",
            true,
        ),
        (
            "Each */POLICY must cite at least one FS.",
            "subject namespaces must be local in phase 1",
            true,
        ),
        (
            "POLICY-x.* must cite at least one FS.",
            "section-component wildcards are not accepted in phase 1",
            true,
        ),
        (
            "Each chapter of each POLICY must cite at least one FS.",
            "chapter-quantified subjects are not accepted in phase 1",
            true,
        ),
    ]);
}

/// §FS-rule-refusals.sentences.rewrite.no-form: a path subject and an unknown or malformed object kind
/// offer no form, and what is missing is a kind.
#[test]
fn a_missing_kind_offers_no_form_and_asks_for_the_known_kinds() {
    assert_rows(&[
        (
            "Each file in vendor/ must cite at least one FS.",
            "path subjects are not accepted in phase 1",
            true,
        ),
        (
            "Each FS must cite at least one POLICY.",
            "unknown kind \"POLICY\"",
            true,
        ),
        (
            "Each FS must cite at least one FS or POLICY.",
            "unknown kind \"POLICY\"",
            true,
        ),
        (
            "Each FS must cite at least one fs.",
            "unknown kind \"fs\"",
            true,
        ),
    ]);
}

/// §FS-rule-refusals.sentences.rewrite.no-form: no modality, an unknown verb, a count that is not
/// accepted and a count with no object offer no form, and nothing missing is a
/// kind; the count catch-all keeps its list of canonical counts.
#[test]
fn a_sentence_that_does_not_say_what_belongs_offers_no_form() {
    let counts = concat!(
        "count is not accepted; the canonical counts are \"at least one\", ",
        "\"at least N\", \"at most N\", \"exactly one\" and \"exactly N\" for a base-10 N"
    );
    assert_rows(&[
        (
            "Each FS cites at least one FS.",
            "rule has no accepted modality",
            false,
        ),
        (
            "Each FS must reference at least one FS.",
            "verb is not accepted",
            false,
        ),
        ("Each FS must cite some FS.", counts, false),
        ("FS-login should cite each FS 2 times.", counts, false),
        (
            "Each FS must cite at least FS.",
            "count has no object",
            false,
        ),
    ]);
}

/// §FS-rule-refusals.sentences.rewrite.reparsed: a form refused for another part is repaired again before
/// it is offered, and one the passes cannot repair is not offered at all.
#[test]
fn a_form_is_repaired_until_the_parser_accepts_it() {
    assert_rows(&[
        (
            "Each FS must cite at least 1 FS",
            "rule must end with \".\"; accepted form: Each FS must cite at least one FS.",
            false,
        ),
        (
            "each FS should cite exactly 1 FS",
            "rule must end with \".\"; accepted form: Each FS should cite exactly one FS.",
            false,
        ),
        (
            "FS-*.requirements must cite at least one GOAL.",
            "literal subject \"FS-*.requirements\" does not match the configured ID grammar",
            true,
        ),
        (
            "Each FS may not cite any POLICY.",
            "modality \"may not\" is not accepted",
            true,
        ),
    ]);
}

/// §FS-rule-refusals.sentences.rewrite.reparsed: a sentence refused for several productions at once is
/// repaired one production per pass, the reason staying the first refusal's,
/// and the passes end in a form or in none rather than looping.
#[test]
fn a_many_fault_sentence_ends_without_looping() {
    assert_rows(&[
        (
            "each FS should have exactly 01  requirements section",
            "rule must end with \".\"; accepted form: Each FS should have exactly one requirements chapter.",
            false,
        ),
        (
            "each FS.* may not cite at least 1 FS",
            "rule must end with \".\"; accepted form: Each FS must not cite any FS.",
            false,
        ),
        (
            "FS-login should cite each FS exactly 01",
            "rule must end with \".\"; accepted form: FS-login should cite each FS exactly once.",
            false,
        ),
        (
            "each FS.* may not cite at least 1 POLICY",
            "rule must end with \".\"",
            true,
        ),
        (
            "each FS should have exactly 01 Goal and hypothesis section",
            "rule must end with \".\"",
            false,
        ),
    ]);
}

/// §FS-rule-refusals.selectors: a subject whose only `/` is leading names the empty alias, so the
/// kind is recovered from what follows it and the form is still offered.
#[test]
fn a_leading_slash_subject_still_recovers_its_kind() {
    assert_rows(&[
        (
            "/FS-x must cite at least one FS.",
            "subject namespaces must be local in phase 1; accepted form: FS-x must cite at least one FS.",
            false,
        ),
        (
            "Each /FS must cite at least one FS.",
            "unknown kind \"/FS\"; accepted form: Each FS must cite at least one FS.",
            false,
        ),
    ]);
}

/// The sentences that carry `count` in every count shape: the three bounds of a
/// relation count and the three of a per-target count.
fn every_count_shape(count: &str) -> Vec<String> {
    ["at least", "at most", "exactly"]
        .into_iter()
        .flat_map(|bound| {
            [
                format!("Each FS must cite {bound} {count} FS."),
                format!("FS-login.requirements must cite each FS {bound} {count} times."),
            ]
        })
        .collect()
}

/// §FS-rule-refusals.sentences.count-range: a count above the largest supported count is refused for
/// its range in every count shape, leading zeros or none, and is offered no form.
#[test]
fn a_count_above_the_largest_supported_count_is_refused_for_its_range() {
    let reason = format!(
        "count must be a canonical positive base-10 integer; \
         the count is above the largest supported count, {}",
        usize::MAX
    );
    let above = (usize::MAX as u128 + 1).to_string();
    let rows: Vec<Row> = [above.clone(), format!("0{above}")]
        .iter()
        .flat_map(|count| every_count_shape(count))
        .map(|sentence| -> Row { (sentence.leak(), reason.clone().leak(), false) })
        .collect();
    assert_rows(&rows);
    // The largest count with a leading zero keeps the leading-zeros refusal and its form.
    let largest = format!("Each FS must cite at most {} FS.", usize::MAX);
    let padded = largest.replace("at most ", "at most 0");
    assert_rows(&[(
        padded.leak(),
        format!("count must be a canonical positive base-10 integer; accepted form: {largest}")
            .leak(),
        false,
    )]);
}

/// Guard for §FS-rules.3, green before and after §FS-rule-refusals.sentences.count-range: the
/// largest supported count is accepted in every count shape.
#[test]
fn the_largest_supported_count_is_accepted_in_every_count_shape() {
    let vocabulary = vocabulary();
    let refused: Vec<String> = every_count_shape(&usize::MAX.to_string())
        .into_iter()
        .filter_map(|sentence| {
            let anchor = RuleAnchor {
                path: "docs/rules/RULE-x.md".into(),
                line: 1,
                column: None,
            };
            parse_rule(&sentence, "RULE-x".into(), anchor, &vocabulary)
                .err()
                .map(|refusal| format!("{sentence:?}: {}", refusal.message))
        })
        .collect();
    assert!(refused.is_empty(), "{}", refused.join("\n"));
}
