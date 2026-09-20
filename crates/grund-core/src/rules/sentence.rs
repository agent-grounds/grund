//! The controlled-English sentence front end (§FS-rules.2–4, §AR-rules.2).

mod count;
mod subjects;
mod targets;

use super::RuleAnchor;
use crate::grammar::Grammar;
use count::{CountSpelling, count_prefix, positive};
use std::collections::{BTreeMap, BTreeSet};
use subjects::parse_subject;
use targets::kind_targets;

pub(crate) use subjects::parse_selector;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum RuleSubject {
    Kind(String),
    ExactDeclaration(String),
    ChapterOfKind {
        kind: String,
        name: String,
    },
    ExactChapter {
        declaration: String,
        path: String,
        separator: String,
    },
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
    /// Citable target kinds by full workspace alias. Bare targets continue to
    /// use `target_kinds`; qualified targets must resolve in their namespace.
    pub(crate) target_namespaces: BTreeMap<String, BTreeSet<String>>,
    pub(crate) named_sections: bool,
    /// Effective lexical grammars for the catalogs this vocabulary can select.
    /// The sentence front end sees syntax, never scan facts (§AR-rules.1).
    pub(crate) id_grammars: Vec<Grammar>,
    pub(crate) section_separators: Vec<String>,
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
        if name.is_empty() || name.trim() != name || name.chars().any(char::is_whitespace) {
            return Err(error(
                "chapter name must be a non-empty NAME with no surrounding whitespace; accepted form: Each FS must have exactly one requirements chapter.",
            ));
        }
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
