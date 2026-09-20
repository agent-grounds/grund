//! The controlled-English sentence front end (§FS-rules.2–4, §AR-rules.2).

use super::RuleAnchor;
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

impl Cardinality {
    pub(crate) const AT_LEAST_ONE: Self = Self {
        minimum: Some(1),
        maximum: None,
    };
    pub(crate) const NONE: Self = Self {
        minimum: None,
        maximum: Some(0),
    };
    pub(crate) fn contains(self, count: usize) -> bool {
        self.minimum.is_none_or(|n| count >= n) && self.maximum.is_none_or(|n| count <= n)
    }
    pub(crate) fn wording(self) -> String {
        match (self.minimum, self.maximum) {
            (Some(1), None) => "at least one".into(),
            (Some(n), None) => format!("at least {n}"),
            (None, Some(n)) => format!("at most {n}"),
            (Some(1), Some(1)) => "exactly one".into(),
            (Some(n), Some(m)) if n == m => format!("exactly {n}"),
            _ => "the configured count".into(),
        }
    }
    pub(crate) fn times_wording(self) -> String {
        if self.minimum == Some(1) && self.maximum == Some(1) {
            "exactly once".into()
        } else {
            self.wording()
        }
    }
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

#[derive(Clone, Debug)]
pub(crate) struct RuleVocabulary {
    pub(crate) kinds: BTreeSet<String>,
    pub(crate) target_kinds: BTreeSet<String>,
    pub(crate) named_sections: bool,
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
    let Some((declaration, section)) = text.split_once('.') else {
        let kind = subject_kind(text, vocab)?;
        known_subject(&kind, vocab)?;
        return Ok(RuleSubject::ExactDeclaration(text.into()));
    };
    if section.contains('*') {
        return Err(error(format!(
            "section-component wildcards are not accepted in phase 1; accepted form: {declaration}.requirements must cite at least one REQ."
        )));
    }
    if section
        .split('.')
        .any(|part| part.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(error(format!(
            "numbered chapter subjects can detach when headings move; accepted form: {declaration}.requirements must cite at least one REQ."
        )));
    }
    let kind = subject_kind(declaration, vocab)?;
    known_subject(&kind, vocab)?;
    if !vocab.named_sections {
        return Err(error(format!(
            "named chapter subjects require [id] named_sections = true; accepted form after enabling it: {text} must cite at least one REQ."
        )));
    }
    Ok(RuleSubject::ExactChapter {
        declaration: declaration.into(),
        path: section.into(),
    })
}

fn subject_kind(text: &str, vocab: &RuleVocabulary) -> Result<String, RuleParseError> {
    vocab
        .kinds
        .iter()
        .filter(|k| text.starts_with(k.as_str()) && text.len() > k.len())
        .max_by_key(|k| k.len())
        .cloned()
        .ok_or_else(|| {
            error(format!(
                "unknown kind \"{}\"; accepted form: Each FS must cite at least one GOAL.",
                text.split(['-', '_']).next().unwrap_or(text)
            ))
        })
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
        let (card, object) = count_prefix(rest)?;
        let name = object.strip_suffix(" chapters").or_else(|| object.strip_suffix(" chapter"))
            .ok_or_else(|| error("chapter presence must end in \"chapter\"; accepted form: Each FS must have exactly one requirements chapter."))?;
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
                        .ok_or_else(|| error("per-target counts must end in \"times\""))?,
                )?;
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
        let (card, kinds) = count_prefix(rest)?;
        return Ok((
            RuleRelation::Cite,
            kind_targets(kinds, TargetMode::Aggregate, vocab)?,
            card,
        ));
    }
    if let Some(rest) = text.strip_prefix("be cited by ") {
        let (card, kinds) = count_prefix(rest)?;
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

fn count_prefix(text: &str) -> Result<(Cardinality, &str), RuleParseError> {
    if let Some(rest) = text.strip_prefix("at least one ") {
        return Ok((Cardinality::AT_LEAST_ONE, rest));
    }
    if let Some(rest) = text.strip_prefix("exactly one ") {
        return Ok((
            Cardinality {
                minimum: Some(1),
                maximum: Some(1),
            },
            rest,
        ));
    }
    for prefix in ["at most ", "exactly "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            let (raw, object) = rest
                .split_once(' ')
                .ok_or_else(|| error("count has no object"))?;
            let n = positive(raw)?;
            let card = if prefix == "at most " {
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
            return Ok((card, object));
        }
    }
    Err(error(
        "count is not accepted; accepted form: Each FS must cite at least one GOAL.",
    ))
}

fn positive(raw: &str) -> Result<usize, RuleParseError> {
    raw.parse()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| error("count must be a positive base-10 integer"))
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
