//! The controlled-English sentence front end (§FS-rules.2–4, §AR-rules.2).

mod count;
mod forms;
mod predicate;
mod recovery;
mod selectors;
mod subjects;
mod targets;

use super::RuleAnchor;
use crate::grammar::Grammar;
use forms::{Form, Refusal, refuse};
use predicate::parse_predicate;
use std::collections::{BTreeMap, BTreeSet};
use subjects::{Spelling, SubjectFault, parse_subject, refused, split_modality};

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
    /// The refusal offers no form and what could not be supplied is a kind, so
    /// `check --rule` follows it with the `known kinds:` line
    /// (§FS-rules.3.5.4.4).
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
/// before it is offered (§FS-rules.3.5.4).
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

/// Read a sentence, or refuse it with what its refusing production offers in
/// place of what it read, wrapped back into the whole sentence
/// (§FS-rules.3.5.4.1).
fn read(
    title: &str,
    origin: String,
    anchor: RuleAnchor,
    vocabulary: &RuleVocabulary,
) -> Result<(ParsedRule, Option<String>), Refusal> {
    if let Some(refusal) = documented(title) {
        return Err(refusal);
    }
    // §FS-rules.3.5.4.5: the terminal `.` is appended, and `Each` capitalized.
    if !title.ends_with('.') {
        return Err(refuse(
            "rule must end with \".\"",
            Form::One(format!("{title}.")),
        ));
    }
    if let Some(rest) = title.strip_prefix("each ") {
        return Err(refuse(
            "fixed word \"Each\" is case-sensitive",
            Form::One(format!("Each {rest}")),
        ));
    }
    // §FS-rules.3.5.4.4: a path names no kind to put in its place.
    if title.starts_with("Each file in ") {
        return Err(refuse(
            "path subjects are not accepted in phase 1",
            Form::None { kind: true },
        ));
    }
    let sentence = &title[..title.len() - 1];
    // §FS-rules.3.6: the subject ends at the modality found first in a fixed order.
    let split = split_modality(sentence);
    if let Some(refusal) = quantified(title, split.map(|(subject, ..)| subject), vocabulary) {
        return Err(refusal);
    }
    let Some((subject_text, level, polarity, predicate)) = split else {
        return Err(match sentence.split_once(" may not ") {
            // §FS-rules.3.5.4.5: `must not`, the rest as typed.
            Some((before, after)) => refuse(
                "modality \"may not\" is not accepted",
                Form::One(format!("{before} must not {after}.")),
            ),
            // §FS-rules.3.5.4.4: the sentence does not say which modality it meant.
            None => refuse("rule has no accepted modality", Form::None { kind: false }),
        });
    };
    // What follows the subject and what precedes the predicate, both as typed.
    let after_subject = &title[subject_text.len()..];
    let before_predicate = &sentence[..sentence.len() - predicate.len()];
    let subject = parse_subject(subject_text, vocabulary).map_err(|refusal| {
        let refusal = refusal.rule_refusal(vocabulary);
        // §FS-rules.3.5.4.2: a subject holding a clause of its own is two rules.
        if holds_clause(subject_text) {
            return refusal.offering_none(true);
        }
        refusal.within(|subject| format!("{subject}{after_subject}"))
    })?;
    let mut unverifiable = None;
    let (relation, targets, cardinality) =
        parse_predicate(predicate, polarity, vocabulary, &mut unverifiable).map_err(|refusal| {
            refusal.within(|predicate| format!("{before_predicate}{predicate}."))
        })?;
    if relation == RuleRelation::HaveChapter {
        // §FS-rules.3.5.4.5: a chapter subject becomes its declaration, or `Each KIND`.
        let declaration = match &subject {
            RuleSubject::ChapterOfKind { kind, .. } => Some(format!("Each {kind}")),
            RuleSubject::ExactChapter { declaration, .. } => Some(declaration.clone()),
            RuleSubject::Kind(_) | RuleSubject::ExactDeclaration(_) => None,
        };
        if let Some(declaration) = declaration {
            return Err(refuse(
                "chapter subjects cannot have chapters",
                Form::One(format!("{declaration}{after_subject}")),
            ));
        }
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

/// §FS-rules.3.5: three documented sentences are refused for their own
/// production whatever the vocabulary reads of the rest, and offer themselves
/// with that production replaced (§FS-rules.3.5.4.5).
fn documented(title: &str) -> Option<Refusal> {
    let form = String::from;
    Some(match title {
        "Each FS must cite no AR." => refuse(
            "\"cite no\" is not accepted",
            Form::One(form("Each FS must not cite any AR.")),
        ),
        "Each FS must cite a GOAL." => refuse(
            "quantifier \"a\" is ambiguous",
            Form::Two(
                [
                    form("Each FS must cite at least one GOAL."),
                    form("Each FS must cite exactly one GOAL."),
                ],
                "or",
            ),
        ),
        "Each FS must cite at least one GOAL and must not cite any AR." => refuse(
            "conjunctions are not accepted",
            Form::Two(
                [
                    form("Each FS must cite at least one GOAL."),
                    form("Each FS must not cite any AR."),
                ],
                "and",
            ),
        ),
        _ => return None,
    })
}

/// §FS-rules.12: a namespaced or chapter-quantified subject is refused for its
/// own production before the modality is sought. Its form is the subject
/// rebuilt as §FS-rules.3.5.4.2 says, and there is none where no modality
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
    let after_subject = &title[subject.len()..];
    Some(refusal.within(|subject| format!("{subject}{after_subject}")))
}
