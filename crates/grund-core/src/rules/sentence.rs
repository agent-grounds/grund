//! The controlled-English sentence front end (§FS-rules.2–4, §AR-rules.2).

mod chapter_paths;
mod conjunction;
mod count;
mod forms;
mod literal;
mod modality;
mod places;
mod predicate;
mod recovery;
mod selectors;
mod stages;
mod subjects;
mod targets;
mod whose;

use super::RuleAnchor;
use crate::config::{KindField, KindTable, Quote};
use crate::grammar::Grammar;
use literal::DeclarationSelector;
use stages::read;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) use places::SubjectKinds;

pub(crate) use selectors::parse_selector;
#[cfg(test)]
pub(super) use subjects::lower_case_each;

/// A whose clause (§FS-rules.2.2): the declarations whose scalar field at
/// `field`, a whole section path, has `value` as its display name. It is part of
/// the subject, so of a rule's semantic equality (§AR-rules.2, §FS-rules.6).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Whose {
    pub(crate) field: String,
    pub(crate) value: String,
}

impl Whose {
    /// Whether a declaration whose field reads `value` is selected: `None` is a
    /// declaration with no section at the field, which is outside the rule.
    pub(crate) fn admits(&self, value: Option<&str>) -> bool {
        value == Some(self.value.as_str())
    }
}

impl std::fmt::Display for Whose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " whose {} is {}", self.field, self.value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum RuleSubject {
    /// `Each KIND`, with the whose clause it may end in (§FS-rules.2.2).
    Kind(String, Option<Whose>),
    ExactDeclaration(DeclarationSelector),
    ChapterOfKind {
        kind: String,
        name: String,
        whose: Option<Whose>,
    },
    ExactChapter {
        declaration: DeclarationSelector,
        path: String,
        separator: String,
    },
}

impl RuleSubject {
    /// The one subject matcher (§FS-rules.2, §FS-rules.8): whether this subject
    /// selects, in a declaration of `kind` rendered as `declaration`, the
    /// declaration itself (`section` is `None`) or its chapter at `section`.
    /// `list`, `list --size` and literal resolution all ask it, so a subject
    /// selects one set of units wherever it is read.
    /// Numeric components compare by value while report labels keep their
    /// authored digits (§FS-config.3.2.8).
    ///
    /// `value` is the display name of the declaration's field at
    /// [`Self::whose_field`], read by the caller with the one field reader
    /// (§FS-rules.2.2): the matcher reads scalar inputs, never scanner records
    /// (§AR-rules.1).
    pub(crate) fn selects(
        &self,
        kind: &str,
        declaration: &str,
        section: Option<&str>,
        value: Option<&str>,
    ) -> bool {
        match section {
            None => match self {
                Self::Kind(wanted, whose) => {
                    wanted == kind && whose.as_ref().is_none_or(|whose| whose.admits(value))
                }
                Self::ExactDeclaration(wanted) => wanted.matches(declaration),
                _ => false,
            },
            Some(section) => self.chapter_path(kind, declaration, value) == Some(section),
        }
    }

    /// The field a whose clause reads (§FS-rules.2.2), whose display name a
    /// caller passes to [`Self::selects`] and [`Self::chapter_path`].
    pub(crate) fn whose_field(&self) -> Option<&str> {
        self.whose().map(|whose| whose.field.as_str())
    }

    /// The whose clause a quantified subject ends in (§FS-rules.2.2).
    pub(crate) fn whose(&self) -> Option<&Whose> {
        match self {
            Self::Kind(_, whose) | Self::ChapterOfKind { whose, .. } => whose.as_ref(),
            Self::ExactDeclaration(_) | Self::ExactChapter { .. } => None,
        }
    }

    /// The literal a subject spelled by name writes, which must name exactly
    /// one unit (§FS-rules.4, §FS-rules.8); `None` for a quantified subject.
    pub(crate) fn literal(&self) -> Option<String> {
        match self {
            Self::ExactDeclaration(literal) => Some(literal.to_string()),
            Self::ExactChapter {
                declaration,
                path,
                separator,
            } => Some(format!("{declaration}{separator}{path}")),
            Self::Kind(..) | Self::ChapterOfKind { .. } => None,
        }
    }

    /// The section path this subject selects in a declaration of `kind`
    /// rendered as `declaration`, or `None` where it selects no chapter there.
    /// A chapter-of-kind `NAME` is the whole path, never its last component
    /// (§FS-rules.2.1), so it is looked up exactly as an exact coordinate's path
    /// is, and `list` and `list --size` read a subject the same way (§FS-rules.8).
    /// A declaration whose field at the whose clause, read as `value`, does not
    /// hold its value is outside the rule and has none (§FS-rules.2.2).
    pub(crate) fn chapter_path(
        &self,
        kind: &str,
        declaration: &str,
        value: Option<&str>,
    ) -> Option<&str> {
        match self {
            Self::ChapterOfKind {
                kind: wanted,
                name,
                whose,
            } if wanted == kind && whose.as_ref().is_none_or(|whose| whose.admits(value)) => {
                Some(name)
            }
            Self::ExactChapter {
                declaration: wanted,
                path,
                ..
            } if wanted.matches(declaration) => Some(path),
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
    /// Configured kinds that declare no IDs, each with the entry it is refused
    /// by (§FS-errors.2.5.1); the bare object kinds share the subjects' scope.
    pub(crate) non_citable: SubjectKinds,
    /// Effective lexical grammars for the catalogs this vocabulary can select.
    /// The sentence front end sees syntax, never scan facts (§AR-rules.1).
    pub(crate) id_grammars: Vec<Grammar>,
    pub(crate) section_separators: Vec<String>,
    /// Each subject kind's declared fields, read from the kind table under the
    /// project that configures it (§FS-rules.2.2, §AR-config.1.5); `None` for a
    /// kind of a version-1 config, which declares no scalar field.
    pub(crate) fields: BTreeMap<String, Option<Vec<KindField>>>,
}

impl RuleVocabulary {
    /// §AR-config.1.5: whether a named chapter of `kind` is
    /// addressable, read under the project that configures it.
    pub(crate) fn named_sections_for(&self, kind: &str) -> bool {
        self.named_sections || self.named_kinds.contains(kind)
    }

    /// §FS-errors.3.8.1: `legacy`, a refusal that names `kind`, or, where
    /// `kind` is configured but not citable here, §FS-errors.2.5.1's reason
    /// alone in its place.
    pub(crate) fn kind_refusal(&self, legacy: String, kind: &str) -> String {
        match self.non_citable.entries.get(kind) {
            Some(entry) => entry.non_citable_reason(Quote::Double),
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
        let mut error = refusal.offer(vocabulary, |form, vocabulary| {
            let anchor = RuleAnchor {
                path: String::new(),
                line: 0,
                column: None,
            };
            read(form, String::new(), anchor, vocabulary).err()
        });
        // §FS-rule-refusals.sentences.rewrite.no-form.place: supplement only the
        // original place refusal, after all earlier stages and shared reasons.
        if let Some(guidance) = vocabulary.non_citable.guidance(title, vocabulary) {
            error.message.push_str(&guidance);
        }
        error
    })
}

#[cfg(test)]
#[path = "sentence/tests_order.rs"]
mod tests_order;
