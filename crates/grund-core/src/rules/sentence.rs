//! The controlled-English sentence front end (§FS-rules.2–4, §AR-rules.2).

mod conjunction;
mod count;
mod forms;
mod predicate;
mod recovery;
mod selectors;
mod stages;
mod subjects;
mod targets;

use super::RuleAnchor;
use crate::config::{KindTable, one_grammar_window};
use crate::grammar::Grammar;
use stages::read;
use std::collections::{BTreeMap, BTreeSet};

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

impl RuleSubject {
    /// The section path this subject selects in a declaration of `kind`
    /// rendered as `declaration`, or `None` where it selects no chapter there.
    /// A chapter-of-kind `NAME` is the whole path, never its last component
    /// (§FS-rules.2.1), so it is looked up exactly as an exact coordinate's path
    /// is, and `list` and `list --size` read a subject the same way (§FS-rules.8).
    pub(crate) fn chapter_path(&self, kind: &str, declaration: &str) -> Option<&str> {
        match self {
            Self::ChapterOfKind { kind: wanted, name } if wanted == kind => Some(name),
            Self::ExactChapter {
                declaration: wanted,
                path,
                ..
            } if wanted == declaration => Some(path),
            _ => None,
        }
    }
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
    /// The kind table an object is judged in (§AR-config.1.5): the
    /// project whose config reads the rule as `""`, for a bare `KIND`, and each
    /// project the rule's run loaded by alias, for `alias/KIND` and `*/KIND`.
    pub(crate) objects: KindTable,
    pub(crate) named_sections: bool,
    /// Kinds whose own project turns named sections on, where the vocabulary
    /// spans projects that set it differently (§AR-config.1.5).
    pub(crate) named_kinds: BTreeSet<String>,
    /// Configured kinds that declare no IDs, each with the reason it is refused
    /// for (§FS-errors.2.5.1); the bare object kinds share the subjects' scope.
    pub(crate) non_citable: BTreeMap<String, String>,
    /// Effective lexical grammars for the catalogs this vocabulary can select.
    /// The sentence front end sees syntax, never scan facts (§AR-rules.1).
    pub(crate) id_grammars: Vec<Grammar>,
    pub(crate) section_separators: Vec<String>,
}

impl RuleVocabulary {
    /// §AR-config.1.5: whether a named chapter of `kind` is
    /// addressable, read under the project that configures it.
    pub(crate) fn named_sections_for(&self, kind: &str) -> bool {
        self.named_sections || self.named_kinds.contains(kind)
    }

    /// §FS-errors.3.8: `legacy`, a refusal that names `kind`, with
    /// §FS-errors.2.5.1's reason appended where `kind` is configured but not
    /// citable here, until the window closes.
    pub(crate) fn kind_refusal(&self, legacy: String, kind: &str) -> String {
        match self.non_citable.get(kind) {
            Some(reason) => one_grammar_window(&legacy, reason),
            None => legacy,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RuleParseError {
    pub(crate) message: String,
    /// The refusal offers no form and what could not be supplied is a kind, so
    /// `check --rule` follows it with the `known kinds:` line
    /// (§FS-rule-refusals.sentences.rewrite.no-form).
    pub(crate) unrecovered: bool,
}

impl std::fmt::Display for RuleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}
impl std::error::Error for RuleParseError {}

/// Parse exactly the five released families (§FS-rules.3). A refusal offers
/// the sentence as typed with the failed production replaced, read again
/// before it is offered (§FS-rule-refusals.sentences.rewrite).
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
    read(title, origin, anchor, vocabulary).map_err(|refusal| {
        refusal.offer(vocabulary, |form, vocabulary| {
            let anchor = RuleAnchor {
                path: String::new(),
                line: 0,
                column: None,
            };
            read(form, String::new(), anchor, vocabulary).err()
        })
    })
}

#[cfg(test)]
#[path = "sentence/tests_order.rs"]
mod tests_order;
