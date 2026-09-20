//! The controlled-English sentence front end (§FS-rules.2–4, §AR-rules.2).

mod count;

use super::RuleAnchor;
use crate::grammar::{Grammar, parse_id_arg, render_id};
use count::{CountSpelling, count_prefix, positive};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum RuleSubject {
    Kind(String),
    ExactDeclaration(String),
    ChapterOfKind { kind: String, name: String },
    ExactChapter { declaration: String, path: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum RuleLevel {
    Required,
    Recommended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum RulePolarity {
    Positive,
    Prohibiting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum RuleRelation {
    HaveChapter,
    Cite,
    BeCitedBy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum TargetMode {
    Aggregate,
    PerTarget,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum RuleTargets {
    Chapter(String),
    Kinds {
        values: Vec<String>,
        mode: TargetMode,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Cardinality {
    pub(crate) minimum: Option<usize>,
    pub(crate) maximum: Option<usize>,
}

/// Immutable normalized rule; authored sentence text does not cross this
/// boundary (§FS-rules.11, §AR-rules.2).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ParsedRule {
    pub(crate) origin: String,
    pub(crate) anchor: RuleAnchor,
    pub(crate) subject: RuleSubject,
    pub(crate) level: RuleLevel,
    pub(crate) polarity: RulePolarity,
    pub(crate) relation: RuleRelation,
    pub(crate) targets: RuleTargets,
    pub(crate) cardinality: Cardinality,
}

#[derive(Clone)]
pub(crate) struct RuleVocabulary {
    pub(crate) kinds: BTreeSet<String>,
    pub(crate) target_kinds: BTreeSet<String>,
    pub(crate) named_sections: bool,
    /// Effective lexical grammars for the catalogs this vocabulary can select.
    /// The sentence front end sees syntax, never scan facts (§AR-rules.1).
    pub(crate) id_grammars: Vec<Grammar>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RuleParseError {
    pub(crate) message: String,
}

impl std::fmt::Display for RuleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}
impl std::error::Error for RuleParseError {}

fn error(message: impl Into<String>) -> RuleParseError {
    RuleParseError {
        message: message.into(),
    }
}

/// Parse the one subject-selector grammar shared by rule sentences and catalog
/// filtering (§FS-rules.2, §FS-rules.8). Resolution of exact literals remains a
/// post-scan concern for the caller.
pub(crate) fn parse_selector(
    text: &str,
    vocabulary: &RuleVocabulary,
) -> Result<RuleSubject, RuleParseError> {
    if vocabulary.kinds.contains(text) {
        return Ok(RuleSubject::Kind(text.into()));
    }
    if let Some((kind, name)) = text.split_once('.')
        && vocabulary.kinds.contains(kind)
    {
        validate_named_path(name, vocabulary, text)?;
        return Ok(RuleSubject::ChapterOfKind {
            kind: kind.into(),
            name: name.into(),
        });
    }
    parse_subject(text, vocabulary)
}

/// Parse exactly the five released families (§FS-rules.3). Near misses receive
/// their fixed accepted rewrite before generic production parsing.
pub(crate) fn parse_rule(
    title: &str,
    origin: String,
    anchor: RuleAnchor,
    vocabulary: &RuleVocabulary,
) -> Result<ParsedRule, RuleParseError> {
    match title {
        "Each FS may not cite any AR." => {
            return Err(error(
                "modality \"may not\" is not accepted; accepted form: Each FS must not cite any AR.",
            ));
        }
        "Each FS must cite no AR." => {
            return Err(error(
                "\"cite no\" is not accepted; accepted form: Each FS must not cite any AR.",
            ));
        }
        "Each FS must cite a GOAL." => {
            return Err(error(
                "quantifier \"a\" is ambiguous; accepted forms: \"Each FS must cite at least one GOAL.\" or \"Each FS must cite exactly one GOAL.\"",
            ));
        }
        "Each FS must cite at least one GOAL and must not cite any AR." => {
            return Err(error(
                "conjunctions are not accepted; accepted forms: \"Each FS must cite at least one GOAL.\" and \"Each FS must not cite any AR.\"",
            ));
        }
        _ => {}
    }
    if !title.ends_with('.') {
        return Err(error(format!(
            "rule must end with \".\"; accepted form: {title}."
        )));
    }
    if title.starts_with("each ") {
        return Err(error(
            "fixed word \"Each\" is case-sensitive; accepted form: Each FS must cite at least one GOAL.",
        ));
    }
    if title.starts_with("Each file in ") {
        return Err(error(
            "path subjects are not accepted in phase 1; accepted form: Each FS must cite at least one GOAL.",
        ));
    }
    if title.starts_with("Each */") {
        return Err(error(
            "subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one GOAL.",
        ));
    }
    if title.starts_with("Each chapter of each ") {
        return Err(error(
            "chapter-quantified subjects are not accepted in phase 1; accepted form: The requirements chapter of each FS must cite at least one REQ.",
        ));
    }
    let sentence = &title[..title.len() - 1];
    let (subject_text, level, polarity, predicate) = if let Some((a, b)) =
        sentence.split_once(" must not ")
    {
        (a, RuleLevel::Required, RulePolarity::Prohibiting, b)
    } else if let Some((a, b)) = sentence.split_once(" should not ") {
        (a, RuleLevel::Recommended, RulePolarity::Prohibiting, b)
    } else if let Some((a, b)) = sentence.split_once(" must ") {
        (a, RuleLevel::Required, RulePolarity::Positive, b)
    } else if let Some((a, b)) = sentence.split_once(" should ") {
        (a, RuleLevel::Recommended, RulePolarity::Positive, b)
    } else if sentence.contains(" may not ") {
        return Err(error(
            "modality \"may not\" is not accepted; accepted form: Each FS must not cite any AR.",
        ));
    } else {
        return Err(error(
            "rule has no accepted modality; accepted form: Each FS must cite at least one GOAL.",
        ));
    };
    let subject = parse_subject(subject_text, vocabulary)?;
    let (relation, targets, cardinality) = parse_predicate(predicate, polarity, vocabulary)?;
    if relation == RuleRelation::HaveChapter
        && matches!(
            subject,
            RuleSubject::ChapterOfKind { .. } | RuleSubject::ExactChapter { .. }
        )
    {
        return Err(error(
            "chapter subjects cannot have chapters; accepted form: Each FS must have exactly one requirements chapter.",
        ));
    }
    Ok(ParsedRule {
        origin,
        anchor,
        subject,
        level,
        polarity,
        relation,
        targets,
        cardinality,
    })
}

fn parse_subject(text: &str, vocab: &RuleVocabulary) -> Result<RuleSubject, RuleParseError> {
    if let Some(kind) = text.strip_prefix("Each ") {
        known_subject(kind, vocab)?;
        return Ok(RuleSubject::Kind(kind.into()));
    }
    if let Some(rest) = text.strip_prefix("The ")
        && let Some((name, kind)) = rest.split_once(" chapter of each ")
    {
        known_subject(kind, vocab)?;
        if !vocab.named_sections {
            return Err(error(format!(
                "named chapter subjects require [id] named_sections = true; accepted form after enabling it: The {name} chapter of each {kind} must cite at least one REQ."
            )));
        }
        validate_named_path(name, vocab, text)?;
        return Ok(RuleSubject::ChapterOfKind {
            kind: kind.into(),
            name: name.into(),
        });
    }
    if text.contains('/') {
        return Err(error(
            "subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one GOAL.",
        ));
    }
    if let Some((declaration, section)) = text.split_once('.')
        && section.contains('*')
    {
        return Err(error(format!(
            "section-component wildcards are not accepted in phase 1; accepted form: {declaration}.requirements must cite at least one REQ."
        )));
    }
    if let Some((declaration, section)) = text.split_once('.')
        && section
            .split('.')
            .any(|part| part.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(error(format!(
            "numbered chapter subjects can detach when headings move; accepted form: {declaration}.requirements must cite at least one REQ."
        )));
    }
    if text.contains('.') && !vocab.named_sections {
        return Err(error(format!(
            "named chapter subjects require [id] named_sections = true; accepted form after enabling it: {text} must cite at least one REQ."
        )));
    }
    for grammar in &vocab.id_grammars {
        if let Ok((id, section)) = parse_id_arg(text, grammar) {
            known_subject(&id.kind, vocab)?;
            return Ok(match section {
                Some(path) => RuleSubject::ExactChapter {
                    declaration: render_id(grammar, &id),
                    path,
                },
                None => RuleSubject::ExactDeclaration(text.into()),
            });
        }
    }
    // A hand-built parser boundary test may provide only scalar vocabulary.
    // Production callers always pass the effective compiled grammar.
    if vocab.id_grammars.is_empty()
        && let Some(kind) = vocab
            .kinds
            .iter()
            .filter(|kind| text.starts_with(&format!("{kind}-")))
            .max_by_key(|kind| kind.len())
    {
        known_subject(kind, vocab)?;
        return Ok(match text.split_once('.') {
            Some((declaration, path)) => RuleSubject::ExactChapter {
                declaration: declaration.into(),
                path: path.into(),
            },
            None => RuleSubject::ExactDeclaration(text.into()),
        });
    }
    Err(error(format!(
        "literal subject \"{text}\" does not match the configured ID grammar; accepted form: FS-login must cite at least one GOAL."
    )))
}

fn validate_named_path(
    path: &str,
    vocab: &RuleVocabulary,
    authored: &str,
) -> Result<(), RuleParseError> {
    if !vocab.named_sections {
        return Err(error(format!(
            "named chapter subjects require [id] named_sections = true; accepted form after enabling it: {authored} must cite at least one REQ."
        )));
    }
    let scalar_valid = !path.is_empty()
        && path.split('.').all(|component| {
            component
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_lowercase)
                && component
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        });
    let grammar_valid = vocab.id_grammars.is_empty()
        || vocab
            .id_grammars
            .iter()
            .any(|grammar| grammar.is_section_path(path));
    if scalar_valid && grammar_valid {
        Ok(())
    } else {
        Err(error(format!(
            "named chapter subject \"{authored}\" does not match the configured section grammar; accepted form: FS-login.requirements must cite at least one REQ."
        )))
    }
}

fn known_subject(kind: &str, vocab: &RuleVocabulary) -> Result<(), RuleParseError> {
    if vocab.kinds.contains(kind) {
        Ok(())
    } else {
        Err(error(format!(
            "unknown kind \"{kind}\"; accepted form: Each FS must cite at least one GOAL."
        )))
    }
}

fn known_target(kind: &str, vocab: &RuleVocabulary) -> Result<(), RuleParseError> {
    if vocab.target_kinds.contains(kind) {
        Ok(())
    } else {
        Err(error(format!(
            "unknown kind \"{kind}\"; accepted form: Each FS must cite at least one GOAL."
        )))
    }
}

fn parse_predicate(
    text: &str,
    polarity: RulePolarity,
    vocab: &RuleVocabulary,
) -> Result<(RuleRelation, RuleTargets, Cardinality), RuleParseError> {
    if polarity == RulePolarity::Prohibiting {
        let rest = text.strip_prefix("cite any ").ok_or_else(|| {
            error(
                "a prohibition must use \"cite any\"; accepted form: Each FS must not cite any AR.",
            )
        })?;
        return Ok((
            RuleRelation::Cite,
            kind_targets(rest, TargetMode::Aggregate, vocab)?,
            Cardinality::NONE,
        ));
    }
    if let Some(rest) = text.strip_prefix("have ") {
        let (card, object, spelling) = count_prefix(rest)?;
        let (name, plural) = if let Some(name) = object.strip_suffix(" chapters") {
            (name, true)
        } else if let Some(name) = object.strip_suffix(" chapter") {
            (name, false)
        } else {
            return Err(error(
                "chapter presence must end in \"chapter\"; accepted form: Each FS must have exactly one requirements chapter.",
            ));
        };
        let expects_plural = match spelling {
            CountSpelling::AtLeastOne | CountSpelling::ExactlyOne => false,
            CountSpelling::AtMost(n) => n != 1,
            CountSpelling::Exactly(_) => true,
        };
        if plural != expects_plural {
            let count = match spelling {
                CountSpelling::AtLeastOne => "at least one".to_string(),
                CountSpelling::ExactlyOne => "exactly one".to_string(),
                CountSpelling::AtMost(n) => format!("at most {n}"),
                CountSpelling::Exactly(n) => format!("exactly {n}"),
            };
            let noun = if expects_plural {
                "chapters"
            } else {
                "chapter"
            };
            return Err(error(format!(
                "chapter count has the wrong singular/plural spelling; accepted form: Each FS must have {count} {name} {noun}."
            )));
        }
        return Ok((
            RuleRelation::HaveChapter,
            RuleTargets::Chapter(name.into()),
            card,
        ));
    }
    if let Some(rest) = text.strip_prefix("cite each ") {
        if let Some(kind) = rest.strip_suffix(" at least once") {
            return Ok((
                RuleRelation::Cite,
                kind_targets(kind, TargetMode::PerTarget, vocab)?,
                Cardinality::AT_LEAST_ONE,
            ));
        }
        if let Some(kind) = rest.strip_suffix(" exactly once") {
            return Ok((
                RuleRelation::Cite,
                kind_targets(kind, TargetMode::PerTarget, vocab)?,
                Cardinality {
                    minimum: Some(1),
                    maximum: Some(1),
                },
            ));
        }
        for marker in [" at most ", " exactly "] {
            if let Some((kind, raw)) = rest.split_once(marker) {
                let n = positive(
                    raw.strip_suffix(" times")
                        .ok_or_else(|| error("per-target counts must end in \"times\"; accepted form: AR-overview.system-overview must cite each AR exactly 2 times."))?,
                )?;
                if marker.contains("exactly") && n == 1 {
                    return Err(error(
                        "numeric \"exactly 1 times\" is not canonical; accepted form: AR-overview.system-overview must cite each AR exactly once.",
                    ));
                }
                let card = if marker.contains("at most") {
                    Cardinality {
                        minimum: None,
                        maximum: Some(n),
                    }
                } else {
                    Cardinality {
                        minimum: Some(n),
                        maximum: Some(n),
                    }
                };
                return Ok((
                    RuleRelation::Cite,
                    kind_targets(kind, TargetMode::PerTarget, vocab)?,
                    card,
                ));
            }
        }
    }
    if let Some(rest) = text.strip_prefix("cite ") {
        if rest.starts_with("no ") {
            return Err(error(
                "\"cite no\" is not accepted; accepted form: Each FS must not cite any AR.",
            ));
        }
        if rest.starts_with("a ") {
            return Err(error(
                "quantifier \"a\" is ambiguous; accepted forms: \"Each FS must cite at least one GOAL.\" or \"Each FS must cite exactly one GOAL.\"",
            ));
        }
        let (card, kinds, _) = count_prefix(rest)?;
        return Ok((
            RuleRelation::Cite,
            kind_targets(kinds, TargetMode::Aggregate, vocab)?,
            card,
        ));
    }
    if let Some(rest) = text.strip_prefix("be cited by ") {
        let (card, kinds, _) = count_prefix(rest)?;
        return Ok((
            RuleRelation::BeCitedBy,
            kind_targets(kinds, TargetMode::Aggregate, vocab)?,
            card,
        ));
    }
    Err(error(
        "verb is not accepted; accepted form: Each FS must cite at least one GOAL.",
    ))
}

fn kind_targets(
    text: &str,
    mode: TargetMode,
    vocab: &RuleVocabulary,
) -> Result<RuleTargets, RuleParseError> {
    let mut values = text.split(" or ").map(str::to_string).collect::<Vec<_>>();
    for target in &values {
        known_target(target.rsplit('/').next().unwrap_or(target), vocab)?;
    }
    values.sort();
    values.dedup();
    Ok(RuleTargets::Kinds { values, mode })
}
