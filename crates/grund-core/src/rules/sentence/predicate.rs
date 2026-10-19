//! The predicate of a rule sentence (§FS-rules.3.1–3.4), and the form each of
//! its refusals offers in place of the predicate it read (§FS-rule-refusals.sentences.rewrite.by-refusal).
//! The subject and modality are the caller's, so a form here keeps them as
//! typed by never seeing them.

use super::count::{CountSpelling, count_prefix, per_target};
use super::forms::{Form, Refusal, refuse};
use super::targets::kind_targets;
use super::{Cardinality, RulePolarity, RuleRelation, RuleTargets, RuleVocabulary, TargetMode};

pub(super) fn parse_predicate(
    text: &str,
    polarity: RulePolarity,
    vocab: &RuleVocabulary,
    unverifiable: &mut Option<String>,
) -> Result<(RuleRelation, RuleTargets, Cardinality), Refusal> {
    if polarity == RulePolarity::Prohibiting {
        let Some(rest) = text.strip_prefix("cite any ") else {
            return Err(refuse(
                "a prohibition must use \"cite any\"",
                prohibition(text),
            ));
        };
        return Ok((
            RuleRelation::Cite,
            kind_targets(rest, TargetMode::Aggregate, vocab, unverifiable)?,
            Cardinality::NONE,
        ));
    }
    if let Some(rest) = text.strip_prefix("have ") {
        let (card, name) =
            presence(rest).map_err(|refusal| refusal.within(|rest| format!("have {rest}")))?;
        return Ok((
            RuleRelation::HaveChapter,
            RuleTargets::Chapter(name.into()),
            card,
        ));
    }
    if let Some(rest) = text.strip_prefix("cite each ")
        && let Some(coverage) = per_target(rest)
    {
        let (kind, card) =
            coverage.map_err(|refusal| refusal.within(|rest| format!("cite each {rest}")))?;
        return Ok((
            RuleRelation::Cite,
            kind_targets(kind, TargetMode::PerTarget, vocab, unverifiable)?,
            card,
        ));
    }
    if let Some(rest) = text.strip_prefix("cite ") {
        if let Some(kinds) = rest.strip_prefix("no ") {
            // §FS-rule-refusals.sentences.rewrite.by-refusal: `<modality> not cite any`, the
            // modality as typed.
            return Err(refuse(
                "\"cite no\" is not accepted",
                Form::One(format!("not cite any {kinds}")),
            ));
        }
        if let Some(kinds) = rest.strip_prefix("a ") {
            let cite = |count| format!("cite {count} {kinds}");
            return Err(refuse(
                "quantifier \"a\" is ambiguous",
                Form::Many(vec![cite("at least one"), cite("exactly one")], "or"),
            ));
        }
        let (card, kinds, _) =
            count_prefix(rest).map_err(|refusal| refusal.within(|rest| format!("cite {rest}")))?;
        return Ok((
            RuleRelation::Cite,
            kind_targets(kinds, TargetMode::Aggregate, vocab, unverifiable)?,
            card,
        ));
    }
    if let Some(rest) = text.strip_prefix("be cited by ") {
        let (card, kinds, _) = count_prefix(rest)
            .map_err(|refusal| refusal.within(|rest| format!("be cited by {rest}")))?;
        return Ok((
            RuleRelation::BeCitedBy,
            kind_targets(kinds, TargetMode::Aggregate, vocab, unverifiable)?,
            card,
        ));
    }
    // §FS-rule-refusals.sentences.rewrite.no-form: the sentence does not say which verb it meant.
    Err(refuse("verb is not accepted", Form::None { kind: false }))
}

/// §FS-rule-refusals.sentences.rewrite.by-refusal: a prohibited `cite [count] KINDS` becomes
/// `cite any KINDS`, whatever count was written; any other prohibited
/// predicate is not one the sentence says how to spell.
fn prohibition(text: &str) -> Form {
    let Some(rest) = text
        .strip_prefix("cite ")
        .filter(|rest| !rest.starts_with("each "))
    else {
        return Form::None { kind: false };
    };
    let kinds = ["at least ", "at most ", "exactly "]
        .into_iter()
        .find_map(|bound| rest.strip_prefix(bound)?.split_once(' '))
        .map(|(_, kinds)| kinds)
        .or_else(|| {
            ["a ", "no "]
                .into_iter()
                .find_map(|word| rest.strip_prefix(word))
        })
        .unwrap_or(rest);
    Form::One(format!("cite any {kinds}"))
}

/// Chapter presence (§FS-rules.3.1): the count and the chapter `NAME`. A form
/// keeps the count as typed and replaces the noun or the `NAME` that failed
/// (§FS-rule-refusals.sentences.rewrite.by-refusal).
fn presence(rest: &str) -> Result<(Cardinality, &str), Refusal> {
    let (card, object, spelling) = count_prefix(rest)?;
    let count = &rest[..rest.len() - object.len()];
    let noun = if plural(spelling) {
        "chapters"
    } else {
        "chapter"
    };
    let Some((name, typed)) = [" chapters", " chapter"]
        .into_iter()
        .find_map(|typed| Some((object.strip_suffix(typed)?, &typed[1..])))
    else {
        // No chapter name is invented where no word precedes the noun.
        let form = object
            .rsplit_once(' ')
            .map_or(Form::None { kind: false }, |(name, _)| {
                Form::One(format!("{count}{name} {noun}"))
            });
        return Err(refuse("chapter presence must end in \"chapter\"", form));
    };
    if name.is_empty() || name.trim() != name || name.chars().any(char::is_whitespace) {
        // §FS-rule-refusals.sentences.whitespace: the NAME trimmed, where that leaves one token.
        let trimmed = name.trim();
        let form = if trimmed.is_empty() || trimmed.contains(char::is_whitespace) {
            Form::None { kind: false }
        } else {
            Form::One(format!("{count}{trimmed} {typed}"))
        };
        return Err(refuse(
            "chapter name must be a non-empty NAME with no surrounding whitespace",
            form,
        )
        .trailing("NAME forbids whitespace anywhere."));
    }
    if typed != noun {
        return Err(refuse(
            "chapter count has the wrong singular/plural spelling",
            Form::One(format!("{count}{name} {noun}")),
        ));
    }
    Ok((card, name))
}

/// Whether a presence count takes `chapters` (§FS-rules.3.1).
fn plural(spelling: CountSpelling) -> bool {
    match spelling {
        CountSpelling::AtLeastOne | CountSpelling::ExactlyOne => false,
        CountSpelling::AtMost(n) => n != 1,
        // §FS-rules.3.1: a numeric floor is never one, so it is always plural.
        CountSpelling::AtLeast | CountSpelling::Exactly => true,
    }
}
