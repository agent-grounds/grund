//! The controlled-English sentence front end (§FS-rules.2–4, §AR-rules.2).

mod count;
mod selectors;
mod subjects;
mod targets;

use super::RuleAnchor;
use crate::grammar::Grammar;
use count::{CountSpelling, count_prefix, positive};
use std::collections::{BTreeMap, BTreeSet};
use subjects::{parse_subject, split_modality};
use targets::kind_targets;

pub(crate) use selectors::parse_selector;

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

impl RuleVocabulary {
    /// §FS-rules.4.1.1: whether the run holds a workspace vocabulary at all.
    /// False wherever the effective config declares no `[workspace]`, and that
    /// absence is what makes a namespaced object kind unverifiable here rather
    /// than invalid — a scope holding namespaces judged the alias and said no.
    pub(crate) fn workspace_in_scope(&self) -> bool {
        !self.target_namespaces.is_empty()
    }
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
///
/// `Err` is the invalid rule and nothing else. The second half of `Ok` is
/// §FS-rules.4.1's *unverifiable here*: the reason this scope could say neither
/// yes nor no about the object kind's namespace, which is a verdict about that
/// namespace alone. It rides with a parsed rule rather than replacing one, so
/// every other part of the sentence is judged exactly as it is when the object
/// is local, and a caller that learns to ask a new question asks it of both
/// (§FS-rules.4.1.1).
pub(crate) fn parse_rule(
    title: &str,
    origin: String,
    anchor: RuleAnchor,
    vocabulary: &RuleVocabulary,
) -> Result<(ParsedRule, Option<String>), RuleParseError> {
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
    // §FS-rules.3.6: the subject ends at the modality found first in a fixed order.
    let Some((subject_text, level, polarity, predicate)) = split_modality(sentence) else {
        return Err(error(if sentence.contains(" may not ") {
            "modality \"may not\" is not accepted; accepted form: Each FS must not cite any AR."
        } else {
            "rule has no accepted modality; accepted form: Each FS must cite at least one GOAL."
        }));
    };
    let subject = parse_subject(subject_text, vocabulary)?;
    let mut unverifiable = None;
    let (relation, targets, cardinality) =
        parse_predicate(predicate, polarity, vocabulary, &mut unverifiable)?;
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
        unverifiable,
    ))
}

fn parse_predicate(
    text: &str,
    polarity: RulePolarity,
    vocab: &RuleVocabulary,
    unverifiable: &mut Option<String>,
) -> Result<(RuleRelation, RuleTargets, Cardinality), RuleParseError> {
    if polarity == RulePolarity::Prohibiting {
        let rest = text.strip_prefix("cite any ").ok_or_else(|| {
            error(
                "a prohibition must use \"cite any\"; accepted form: Each FS must not cite any AR.",
            )
        })?;
        return Ok((
            RuleRelation::Cite,
            kind_targets(rest, TargetMode::Aggregate, vocab, unverifiable)?,
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
            // §FS-rules.3.5.1: append whitespace guidance to the released refusal prefix.
            return Err(error(
                "chapter name must be a non-empty NAME with no surrounding whitespace; accepted form: Each FS must have exactly one requirements chapter. NAME forbids whitespace anywhere.",
            ));
        }
        let expects_plural = match spelling {
            CountSpelling::AtLeastOne | CountSpelling::ExactlyOne => false,
            CountSpelling::AtMost(n) => n != 1,
            // §FS-rules.3.1: a numeric floor is never one, so it is always plural.
            CountSpelling::AtLeast(_) | CountSpelling::Exactly(_) => true,
        };
        if plural != expects_plural {
            let count = match spelling {
                CountSpelling::AtLeastOne => "at least one".to_string(),
                CountSpelling::ExactlyOne => "exactly one".to_string(),
                CountSpelling::AtLeast(n) => format!("at least {n}"),
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
                kind_targets(kind, TargetMode::PerTarget, vocab, unverifiable)?,
                Cardinality::AT_LEAST_ONE,
            ));
        }
        if let Some(kind) = rest.strip_suffix(" exactly once") {
            return Ok((
                RuleRelation::Cite,
                kind_targets(kind, TargetMode::PerTarget, vocab, unverifiable)?,
                Cardinality {
                    minimum: Some(1),
                    maximum: Some(1),
                },
            ));
        }
        for marker in [" at least ", " at most ", " exactly "] {
            if let Some((kind, raw)) = rest.split_once(marker) {
                let n = positive(
                    raw.strip_suffix(" times")
                        .ok_or_else(|| error("per-target counts must end in \"times\"; accepted form: AR-overview.system-overview must cite each AR exactly 2 times."))?,
                )?;
                // §FS-rules.3: a floor and an exact count of one are spelled
                // `once`, so the numeral is refused for both.
                let card = match marker {
                    " at least " if n == 1 => {
                        return Err(error(
                            "numeric \"at least 1 times\" is not canonical; accepted form: AR-overview.system-overview must cite each AR at least once.",
                        ));
                    }
                    " exactly " if n == 1 => {
                        return Err(error(
                            "numeric \"exactly 1 times\" is not canonical; accepted form: AR-overview.system-overview must cite each AR exactly once.",
                        ));
                    }
                    " at least " => Cardinality {
                        minimum: Some(n),
                        maximum: None,
                    },
                    " at most " => Cardinality {
                        minimum: None,
                        maximum: Some(n),
                    },
                    _ => Cardinality {
                        minimum: Some(n),
                        maximum: Some(n),
                    },
                };
                return Ok((
                    RuleRelation::Cite,
                    kind_targets(kind, TargetMode::PerTarget, vocab, unverifiable)?,
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
            kind_targets(kinds, TargetMode::Aggregate, vocab, unverifiable)?,
            card,
        ));
    }
    if let Some(rest) = text.strip_prefix("be cited by ") {
        let (card, kinds, _) = count_prefix(rest)?;
        return Ok((
            RuleRelation::BeCitedBy,
            kind_targets(kinds, TargetMode::Aggregate, vocab, unverifiable)?,
            card,
        ));
    }
    Err(error(
        "verb is not accepted; accepted form: Each FS must cite at least one GOAL.",
    ))
}
