//! The stages a rule sentence is read in, in the order
//! §FS-rule-refusals.sentences.order gives them, each refusing with what it
//! offers in place of what it read.

use super::conjunction::{conjunction_forms, first_clause};
use super::forms::{Form, Refusal, refuse};
use super::modality::split_modality;
use super::predicate::parse_predicate;
use super::subjects::{
    EACH_IS_CASE_SENSITIVE, Spelling, SubjectFault, SubjectRefusal, lower_case_each, parse_subject,
    refused,
};
use super::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    RuleVocabulary,
};
use crate::rules::RuleAnchor;

/// Whether `subject` holds a modality followed by a verb, the clause of a
/// conjunction the modality split read as part of the subject (§FS-rules.3.6).
fn holds_clause(subject: &str) -> bool {
    [" must ", " should "].iter().any(|modality| {
        subject.match_indices(modality).any(|(at, modality)| {
            let rest = &subject[at + modality.len()..];
            let rest = rest.strip_prefix("not ").unwrap_or(rest);
            ["cite ", "be cited by ", "have "]
                .iter()
                .any(|verb| rest.starts_with(verb))
        })
    })
}

/// §FS-rule-refusals.sentences.order: the stages a sentence is read in, in
/// order, each named as the specification's table names its row. `read` runs
/// them in this order and stops at the first that refuses, so no later stage is
/// read, and a test holds the names to that table.
pub(super) const STAGES: [(&str, Stage); 6] = [
    ("terminal period", terminal_period),
    ("fixed word and phase-1 subject", phase_one_subject),
    ("conjunction", conjunction),
    ("modality", modality),
    ("the rest of the subject", rest_of_subject),
    ("the predicate", predicate),
];

/// One stage of [`STAGES`]: it refuses the sentence, or leaves in the reading
/// what a later stage reads.
pub(super) type Stage = fn(&mut Reading<'_>) -> Result<(), Refusal>;

/// One sentence as far as [`STAGES`] have read it.
pub(super) struct Reading<'t> {
    title: &'t str,
    /// The sentence without its terminal period.
    sentence: &'t str,
    vocabulary: &'t RuleVocabulary,
    /// §FS-rules.3.6: the subject ends at the modality found first in a fixed
    /// order. Stage 4 refuses a sentence that has none.
    split: Option<(&'t str, RuleLevel, RulePolarity, &'t str)>,
    subject: Option<RuleSubject>,
    predicate: Option<(RuleRelation, RuleTargets, Cardinality)>,
    unverifiable: Option<String>,
}

/// Read a sentence, or refuse it with what its refusing production offers in
/// place of what it read, wrapped back into the whole sentence
/// (§FS-rule-refusals.sentences.rewrite.replaced), at the first of [`STAGES`]
/// it fails.
pub(super) fn read(
    title: &str,
    origin: String,
    anchor: RuleAnchor,
    vocabulary: &RuleVocabulary,
) -> Result<(ParsedRule, Option<String>), Refusal> {
    // §FS-rule-refusals.sentences.order: two documented sentences are refused for
    // their own row before stage 1; both end in `.`, so no sentence can tell.
    if let Some(refusal) = documented(title) {
        return Err(refusal);
    }
    let sentence = title.strip_suffix('.').unwrap_or(title);
    let mut reading = Reading {
        title,
        sentence,
        vocabulary,
        split: split_modality(sentence),
        subject: None,
        predicate: None,
        unverifiable: None,
    };
    for (_, stage) in STAGES {
        stage(&mut reading)?;
    }
    let (Some((_, level, polarity, _)), Some(subject), Some((relation, targets, cardinality))) =
        (reading.split, reading.subject, reading.predicate)
    else {
        unreachable!("a sentence every stage read has a modality, a subject and a predicate");
    };
    Ok((
        ParsedRule {
            origin,
            anchor,
            subject,
            level,
            polarity,
            relation,
            targets,
            cardinality,
        },
        reading.unverifiable,
    ))
}

/// Stage 1, the terminal period (§FS-rule-refusals.sentences.rewrite.by-refusal:
/// the `.` is appended).
fn terminal_period(reading: &mut Reading<'_>) -> Result<(), Refusal> {
    let title = reading.title;
    if title.ends_with('.') {
        return Ok(());
    }
    Err(refuse(
        "rule must end with \".\"",
        Form::One(format!("{title}.")),
    ))
}

/// Stage 2, the fixed word and the phase-1 subject refusals, read from the
/// sentence's start: `Each` capitalized, a path, `Each */`, `Each chapter of
/// each`, and a section-component wildcard.
fn phase_one_subject(reading: &mut Reading<'_>) -> Result<(), Refusal> {
    let (title, vocabulary) = (reading.title, reading.vocabulary);
    if let Some(rest) = lower_case_each(title) {
        return Err(refuse(
            EACH_IS_CASE_SENSITIVE,
            Form::One(format!("Each {rest}")),
        ));
    }
    // §FS-rule-refusals.sentences.rewrite.no-form: a path names no kind to put in its place.
    if title.starts_with("Each file in ") {
        return Err(refuse(
            "path subjects are not accepted in phase 1",
            Form::None { kind: true },
        ));
    }
    if let Some(refusal) = quantified(
        title,
        reading.split.map(|(subject, ..)| subject),
        vocabulary,
    ) {
        return Err(refusal);
    }
    // The wildcard is the subject of the first clause, so it is read only where
    // an accepted modality splits that subject off.
    let Some((subject, ..)) = split_modality(first_clause(reading.sentence)) else {
        return Ok(());
    };
    let Some(refusal) = section_wildcard(subject, vocabulary) else {
        return Ok(());
    };
    // §FS-errors.3.8.1: joined by `and` or not, the wildcard reason opens the
    // refusal; a subject rebuilt into a conjunction offers no form
    // (§FS-rule-refusals.sentences.rewrite.reparsed).
    Err(subject_refusal(refusal, subject, title, vocabulary))
}

/// Stage 3 (§FS-rule-refusals.sentences.conjunction).
fn conjunction(reading: &mut Reading<'_>) -> Result<(), Refusal> {
    match conjunction_forms(reading.sentence) {
        Some(forms) => Err(conjunction_refusal(forms)),
        None => Ok(()),
    }
}

fn conjunction_refusal(forms: Vec<String>) -> Refusal {
    refuse("conjunctions are not accepted", Form::Many(forms, "and"))
}

/// Stage 4, the modality.
fn modality(reading: &mut Reading<'_>) -> Result<(), Refusal> {
    if reading.split.is_some() {
        return Ok(());
    }
    Err(match reading.sentence.split_once(" may not ") {
        // §FS-rule-refusals.sentences.rewrite.by-refusal: `must not`, the rest as typed.
        Some((before, after)) => refuse(
            "modality \"may not\" is not accepted",
            Form::One(format!("{before} must not {after}.")),
        ),
        // §FS-rule-refusals.sentences.rewrite.no-form: the sentence does not say which modality
        // it meant.
        None => refuse("rule has no accepted modality", Form::None { kind: false }),
    })
}

/// Stage 5, the rest of the subject.
fn rest_of_subject(reading: &mut Reading<'_>) -> Result<(), Refusal> {
    let Some(split @ (subject, ..)) = reading.split else {
        return Ok(());
    };
    let parsed = parse_subject(subject, reading.vocabulary).map_err(|refusal| {
        // §FS-rule-refusals.sentences.modal-chapter
        let refusal = refusal.cut_at(split);
        subject_refusal(refusal, subject, reading.title, reading.vocabulary)
    })?;
    reading.subject = Some(parsed);
    Ok(())
}

/// A subject's refusal in place in the sentence, the rest of it as typed.
fn subject_refusal(
    refusal: SubjectRefusal,
    subject: &str,
    title: &str,
    vocabulary: &RuleVocabulary,
) -> Refusal {
    let refusal = refusal.rule_refusal(vocabulary);
    // §FS-rule-refusals.sentences.rewrite.recovered: a subject holding a clause of its own is
    // two rules.
    if holds_clause(subject) {
        return refusal.offering_none(true);
    }
    let after_subject = &title[subject.len()..];
    refusal.within(|subject| format!("{subject}{after_subject}"))
}

/// Stage 6, the predicate, and a chapter subject that would have chapters.
fn predicate(reading: &mut Reading<'_>) -> Result<(), Refusal> {
    let (Some((_, _, polarity, predicate)), Some(subject)) =
        (reading.split, reading.subject.as_ref())
    else {
        return Ok(());
    };
    let sentence = reading.sentence;
    // What precedes the predicate, as typed.
    let before_predicate = &sentence[..sentence.len() - predicate.len()];
    let (relation, targets, cardinality) = parse_predicate(
        predicate,
        polarity,
        reading.vocabulary,
        &mut reading.unverifiable,
    )
    .map_err(|refusal| refusal.within(|predicate| format!("{before_predicate}{predicate}.")))?;
    // §FS-rule-refusals.sentences.rewrite.same-units: no accepted sentence counts the chapters
    // under a chapter, so a chapter subject's presence rule is offered no form.
    if relation == RuleRelation::HaveChapter
        && matches!(
            subject,
            RuleSubject::ChapterOfKind { .. } | RuleSubject::ExactChapter { .. }
        )
    {
        return Err(refuse(
            "chapter subjects cannot have chapters",
            Form::None { kind: false },
        ));
    }
    reading.predicate = Some((relation, targets, cardinality));
    Ok(())
}

/// §FS-rule-refusals.sentences: two documented sentences are refused for their own
/// production whatever the vocabulary reads of the rest, and offer themselves
/// with that production replaced (§FS-rule-refusals.sentences.rewrite.by-refusal).
fn documented(title: &str) -> Option<Refusal> {
    let form = String::from;
    Some(match title {
        "Each FS must cite no AR." => refuse(
            "\"cite no\" is not accepted",
            Form::One(form("Each FS must not cite any AR.")),
        ),
        "Each FS must cite a GOAL." => refuse(
            "quantifier \"a\" is ambiguous",
            Form::Many(
                vec![
                    form("Each FS must cite at least one GOAL."),
                    form("Each FS must cite exactly one GOAL."),
                ],
                "or",
            ),
        ),
        _ => return None,
    })
}

/// Stage 2's wildcard row (§FS-rule-refusals.sentences.order): the subject's
/// refusal where it is refused for its section-component wildcard, decided by
/// the subject parser's own test, so a subject that fails an earlier component
/// of stage 5 first keeps that component's reason.
fn section_wildcard(subject: &str, vocabulary: &RuleVocabulary) -> Option<SubjectRefusal> {
    let refusal = parse_subject(subject, vocabulary).err()?;
    (refusal.fault.failed() == SubjectFault::SectionWildcard).then_some(refusal)
}

/// §FS-rules.12: a namespaced or chapter-quantified subject is refused for its
/// own production before the modality is sought. Its form is the subject
/// rebuilt as §FS-rule-refusals.sentences.rewrite.recovered says, and there is none where no modality
/// says where the subject ends.
fn quantified(title: &str, subject: Option<&str>, vocabulary: &RuleVocabulary) -> Option<Refusal> {
    let (fault, quantifier) = if title.starts_with("Each */") {
        (SubjectFault::Namespace, "Each ")
    } else if title.starts_with("Each chapter of each ") {
        (SubjectFault::ChapterQuantified, "Each chapter of each ")
    } else {
        return None;
    };
    let Some(subject) = subject else {
        return Some(refuse(fault.reason(title), Form::None { kind: false }));
    };
    let head = subject.strip_prefix(quantifier).unwrap_or_default();
    let refusal = refused(fault, Spelling::Each, subject, head, None, "").rule_refusal(vocabulary);
    // §FS-rule-refusals.sentences.rewrite.recovered: a subject holding a clause of its own is two
    // rules.
    if holds_clause(subject) {
        return Some(refusal.offering_none(true));
    }
    let after_subject = &title[subject.len()..];
    Some(refusal.within(|subject| format!("{subject}{after_subject}")))
}
