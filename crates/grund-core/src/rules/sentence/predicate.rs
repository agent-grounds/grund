//! The predicate of a rule sentence (§FS-rules.3.1–3.4), and the form each of
//! its refusals offers in place of the predicate it read (§FS-rule-refusals.sentences.rewrite.by-refusal).
//! The subject and modality are the caller's, so a form here keeps them as
//! typed by never seeing them.

use super::count::{CountSpelling, count_prefix, per_target};
use super::forms::{Form, Refusal, refuse};
use super::subjects::EVERY_IS_UNSUPPORTED;
use super::targets::{Objects, objects};
use super::{Cardinality, RulePolarity, RuleRelation, RuleTargets, RuleVocabulary, TargetMode};

pub(super) fn parse_predicate(
    text: &str,
    polarity: RulePolarity,
    vocab: &RuleVocabulary,
    unverifiable: &mut Option<String>,
) -> Result<(RuleRelation, RuleTargets, Cardinality), Refusal> {
    // §FS-rule-refusals.sentences.every: keep the kind and count for the ordinary reparse.
    if let Some(rest) = text.strip_prefix("cite every ") {
        return Err(refuse(
            EVERY_IS_UNSUPPORTED,
            Form::One(format!("cite each {rest}")),
        ));
    }
    if polarity == RulePolarity::Prohibiting {
        let Some(rest) = text.strip_prefix("cite any ") else {
            return Err(refuse(
                "a prohibition must use \"cite any\"",
                prohibition(text),
            ));
        };
        let objects = objects(rest, vocab, unverifiable)?;
        if objects.distinct() {
            // §FS-rules.3.2.1: nothing is counted, so nothing is counted distinctly.
            return Err(refuse(
                "\"distinct\" has no meaning in a prohibition",
                Form::One(format!("cite any {}", objects.plain)),
            ));
        }
        return Ok((
            RuleRelation::Cite,
            objects.targets(TargetMode::Aggregate),
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
        let objects = objects(kind, vocab, unverifiable)?;
        if objects.distinct() {
            // §FS-rules.3.2.1: each target is counted alone, so `distinct` has nothing to merge.
            return Err(refuse(
                "\"distinct\" has no meaning per target",
                Form::One(format!(
                    "cite each {}{}",
                    objects.plain,
                    &rest[kind.len()..]
                )),
            ));
        }
        return Ok((
            RuleRelation::Cite,
            objects.targets(TargetMode::PerTarget),
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
        let count = &rest[..rest.len() - kinds.len()];
        let objects = objects(kinds, vocab, unverifiable)?;
        let mode = outbound_mode(&objects, card)
            .map_err(|refusal| refusal.within(|kinds| format!("cite {count}{kinds}")))?;
        return Ok((RuleRelation::Cite, objects.targets(mode), card));
    }
    if let Some(rest) = text.strip_prefix("be cited by ") {
        let (card, kinds, _) = count_prefix(rest)
            .map_err(|refusal| refusal.within(|rest| format!("be cited by {rest}")))?;
        let objects = objects(kinds, vocab, unverifiable)?;
        if objects.distinct() {
            // §FS-rule-refusals.sentences.rewrite.same-units: no accepted sentence counts
            // different citers, and dropping `distinct` counts sites instead.
            return Err(refuse(
                "\"distinct\" counts only outbound citations",
                Form::None { kind: false },
            ));
        }
        return Ok((
            RuleRelation::BeCitedBy,
            objects.targets(TargetMode::Aggregate),
            card,
        ));
    }
    // §FS-rule-refusals.sentences.rewrite.no-form: the sentence does not say which verb it meant.
    Err(refuse("verb is not accepted", Form::None { kind: false }))
}

/// How an outbound count counts its object kinds (§FS-rules.3.2.1): by site, or, where
/// `distinct` follows the count, by the different declarations the sites name. A `distinct`
/// the count does not admit is refused with the object kinds it offers in place of the ones
/// it read (§FS-rule-refusals.sentences.rewrite.by-refusal): `at least one` already counts one
/// declaration, so `distinct` is dropped; a `distinct` before a later kind is dropped where
/// one also follows the count, and otherwise no form is offered, because either repair counts
/// something not written (§FS-rule-refusals.sentences.rewrite.same-units).
fn outbound_mode(objects: &Objects, card: Cardinality) -> Result<TargetMode, Refusal> {
    if !objects.distinct() {
        return Ok(TargetMode::Aggregate);
    }
    if card == Cardinality::AT_LEAST_ONE {
        return Err(refuse(
            "\"distinct\" is redundant after \"at least one\"",
            Form::One(objects.plain.clone()),
        ));
    }
    if objects.later {
        let form = if objects.counted {
            Form::One(format!("distinct {}", objects.plain))
        } else {
            Form::None { kind: false }
        };
        return Err(refuse("\"distinct\" must follow the count", form));
    }
    Ok(TargetMode::Distinct)
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
    if joins_with_or(name) {
        // §FS-rule-refusals.sentences.presence-or: refused for the `or`, before its whitespace.
        return Err(refuse(
            "\"or\" between chapter names is not accepted",
            Form::None { kind: false },
        )
        .trailing("a presence rule counts one NAME."));
    }
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

/// Whether a presence `NAME`, split at whitespace, holds the lower-case word
/// `or` with a word before it and a word after it: a choice between chapters
/// rather than one title with spaces in it
/// (§FS-rule-refusals.sentences.presence-or). `or` alone, an `or` at either
/// end, `OR` and `order` are not.
fn joins_with_or(name: &str) -> bool {
    let words: Vec<&str> = name.split_whitespace().collect();
    words.len() > 2 && words[1..words.len() - 1].contains(&"or")
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
