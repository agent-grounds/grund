//! Subject recognition shared by rules and catalog filtering (§FS-rules.2,
//! §FS-rules.8).
//!
//! A refused subject comes back as what the parser read rather than as finished
//! text: the production that failed, plus the kind, declaration and path it had
//! read. The rule surfaces answer it with what can be recovered written as a
//! rule subject (§FS-rule-refusals.sentences.rewrite.recovered), and a subject needing named sections with
//! one they make valid (§FS-rule-refusals.sentences.named-sections); a chapter path is refused for the
//! component that failed (§FS-rule-refusals.sentences.chapter-path), and `selectors.rs` renders it as a
//! selector (§FS-rule-refusals.selectors). Both surfaces agree on what failed because only
//! one parser decided it and `SubjectFault::failed` names it for both, and on
//! what a suggestion recovers because only `recovery.rs` builds it.

use super::chapter_paths::{named_literal, split_section, validate_named_path};
use super::forms::{Form, Refusal, refuse};
use super::literal::DeclarationSelector;
use super::modality::{MODALITIES, Split};
use super::recovery::{
    Recovered, Suggestion, as_if_enabled_subject, recover_subject, repair_every,
};
use super::whose::{
    WhoseFault, exact_with_clause, modal_value_append, parse_whose, split_whose, with_clause,
};
use super::{RuleSubject, RuleVocabulary};
use crate::grammar::{parse_id_arg, render_input_id, unnamed_component};
use crate::model::remedies;

/// The reason a subject opening with the fixed word in the wrong case is
/// refused for: the rule's own (§FS-rule-refusals.sentences), which a
/// selector appends to its literal-subject reason (§FS-rule-refusals.selectors).
pub(super) const EACH_IS_CASE_SENSITIVE: &str = "fixed word \"Each\" is case-sensitive";

/// The shared quantifier reason (§FS-rule-refusals.sentences.every).
pub(super) const EVERY_IS_UNSUPPORTED: &str = "quantifier \"every\" is not accepted";

/// What follows a lower-case `each `, where `text` opens with one: the one
/// test both surfaces read the wrong case with (§FS-rule-refusals.selectors).
pub(in crate::rules) fn lower_case_each(text: &str) -> Option<&str> {
    text.strip_prefix("each ")
}

/// Which production of a subject failed (§FS-rule-refusals.sentences).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SubjectFault {
    /// An unsupported quantifier (§FS-rule-refusals.sentences.every).
    Every,
    UnknownKind(String),
    NamedSectionsOff,
    Namespace,
    SectionWildcard,
    NumberedChapter,
    /// The chapter path a `KIND.NAME` or `The NAME chapter of each KIND`
    /// spelling read is off the named-section grammar, refused at a component
    /// of this shape.
    SectionGrammar(Component),
    IdGrammar,
    ChapterQuantified,
    /// A whose clause the grammar does not accept (§FS-rule-refusals.sentences.whose).
    Whose(WhoseFault),
}

/// The shape of the first chapter-path component that is not a named one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Component {
    Numbered,
    Wildcard,
    Malformed,
}

/// How a subject was spelled, which a suggestion keeps (§FS-rule-refusals.selectors).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Spelling {
    Each,
    ChapterOfEach,
    KindName,
    Literal,
}

/// A refused subject: the failed production and what was read before it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SubjectRefusal {
    pub(super) fault: SubjectFault,
    pub(super) spelling: Spelling,
    /// The subject as authored.
    pub(super) text: String,
    /// The kind as typed, or a literal's declaration half.
    pub(super) head: String,
    /// The chapter path read after the head, if any.
    pub(super) path: Option<String>,
    /// What joined `head` and `path` in a `KIND.NAME` or literal spelling.
    pub(super) separator: String,
    /// The modal word the split took as a chapter's name, where it cut one
    /// short (§FS-rule-refusals.sentences.modal-chapter).
    pub(super) modal_chapter: Option<&'static str>,
    /// What followed `Each ` or `of each ` where a quantified subject carried a
    /// whose clause: the text the window of §FS-errors.3.10 names as the legacy
    /// unknown kind. A refusal holding it offers no recovered subject, because
    /// an offer never picks a field or a value (§FS-rule-refusals.sentences.whose).
    pub(super) legacy: Option<String>,
    /// The modal-word append of a value the split cut
    /// (§FS-rule-refusals.sentences.whose).
    pub(super) modal_value: Option<String>,
}

impl SubjectFault {
    /// What a refusal names as failed: a chapter path the section grammar
    /// refused at a numbered or wildcard component is refused for that
    /// component, whichever spelling reached it (§FS-rule-refusals.sentences.chapter-path,
    /// §FS-rule-refusals.selectors).
    pub(super) fn failed(&self) -> Self {
        match self {
            Self::SectionGrammar(Component::Numbered) => Self::NumberedChapter,
            Self::SectionGrammar(Component::Wildcard) => Self::SectionWildcard,
            fault => fault.clone(),
        }
    }

    /// The released reason a refusal opens with (§FS-rule-refusals.sentences).
    pub(super) fn reason(&self, text: &str) -> String {
        match self {
            Self::Every => EVERY_IS_UNSUPPORTED.into(),
            Self::UnknownKind(kind) => format!("unknown kind \"{kind}\""),
            Self::NamedSectionsOff => {
                "named chapter subjects require [id] named_sections = true".into()
            }
            Self::Namespace => "subject namespaces must be local in phase 1".into(),
            Self::SectionWildcard => {
                "section-component wildcards are not accepted in phase 1".into()
            }
            Self::NumberedChapter => {
                "numbered chapter subjects can detach when headings move".into()
            }
            Self::SectionGrammar(_) => format!(
                "named chapter subject \"{text}\" does not match the configured section grammar"
            ),
            Self::IdGrammar => {
                format!("literal subject \"{text}\" does not match the configured ID grammar")
            }
            Self::ChapterQuantified => {
                "chapter-quantified subjects are not accepted in phase 1".into()
            }
            Self::Whose(fault) => fault.reason(),
        }
    }
}

impl SubjectRefusal {
    /// The reason this refusal opens with on both surfaces: the released one
    /// for the component that failed, or, where what it named as a kind is
    /// configured but declares no IDs, §FS-errors.2.5.1's alone in its place
    /// (§FS-errors.3.8.1).
    pub(super) fn reason(&self, vocabulary: &RuleVocabulary) -> String {
        let fault = self.fault.failed();
        let legacy = fault.reason(&self.text);
        let reason = match &fault {
            SubjectFault::UnknownKind(kind) => vocabulary.kind_refusal(legacy, kind),
            // §FS-errors.3.8.1: a selector spelled `KIND` or `KIND.NAME`.
            SubjectFault::IdGrammar if self.spelling == Spelling::Literal => {
                let reason = vocabulary.kind_refusal(legacy, &self.head);
                // §FS-rule-refusals.selectors: `each FS` adds the rule's reason; nothing moves.
                match lower_case_each(&self.text) {
                    Some(_) => format!("{reason} — {EACH_IS_CASE_SENSITIVE}"),
                    // §FS-rule-refusals.sentences.whose: an exact ID with a clause appends why.
                    None if exact_with_clause(&self.text, vocabulary) => format!(
                        "{reason} — a whose clause narrows only Each KIND and The NAME chapter \
                         of each KIND"
                    ),
                    None => reason,
                }
            }
            _ => legacy,
        };
        // §FS-rule-refusals.sentences.whose: a value the split cut names the modal word.
        let reason = match &self.modal_value {
            Some(append) => format!("{reason}{append}"),
            None => reason,
        };
        // §FS-rule-refusals.sentences.modal-chapter: the released reason stays its prefix.
        match self.modal_chapter {
            Some(name) => format!(
                "{reason} — chapter name \"{name}\" is read as the rule's modality, so {}",
                remedies::invalid_rule_rename()
            ),
            None => reason,
        }
    }

    /// This refusal of the subject `split` left, naming the modal word the
    /// split took where that word was a chapter subject's name: the subject is
    /// exactly `The` and what follows the modality opens `chapter of each `
    /// (§FS-rule-refusals.sentences.modal-chapter).
    ///
    /// A whose clause's value the split cut short is named the same way, where a
    /// `one_of` entry of its field runs past the modality
    /// (§FS-rule-refusals.sentences.whose).
    pub(super) fn cut_at(mut self, split: Split<'_>) -> Self {
        let (subject, level, polarity, predicate) = split;
        let modality = MODALITIES
            .iter()
            .find(|(_, l, p)| (*l, *p) == (level, polarity))
            .map(|(modality, ..)| *modality);
        if subject == "The" && predicate.starts_with("chapter of each ") {
            self.modal_chapter = modality.map(str::trim);
        }
        if let (SubjectFault::Whose(fault), Some(modality)) = (&self.fault, modality)
            && let Some((value, one_of)) = fault.cut_value()
        {
            self.modal_value = modal_value_append(value, one_of, modality);
        }
        self
    }

    /// The rule surfaces' refusal (§FS-rule-refusals.sentences): the reason for the component
    /// that failed (§FS-rule-refusals.sentences.chapter-path), then a form whose subject is rebuilt from
    /// what can be recovered (§FS-rule-refusals.sentences.rewrite.recovered), or, where it needs named
    /// sections, the subject §FS-rule-refusals.sentences.named-sections suggests, read with them on. Where
    /// no faithful subject is recovered there is no form; a missing kind is
    /// distinguished from scope that cannot safely be rewritten.
    pub(super) fn rule_refusal(&self, vocabulary: &RuleVocabulary) -> Refusal {
        // §FS-rule-refusals.sentences.whose: only a value joined by ` or ` is offered forms,
        // one subject per value; a kind that is missing is the one thing a form cannot supply.
        if let Some(legacy) = &self.legacy {
            let form = match &self.fault {
                SubjectFault::Whose(fault) => fault
                    .or_forms(&self.text)
                    .map_or(Form::None { kind: false }, |forms| Form::Many(forms, "and")),
                SubjectFault::UnknownKind(_) => Form::None { kind: true },
                _ => Form::None { kind: false },
            };
            return refuse(self.reason(vocabulary), form).windowed(legacy);
        }
        // §FS-rule-refusals.sentences.every: replace only the quantifier; the offer is reparsed.
        if let Some(text) = repair_every(self) {
            return refuse(self.reason(vocabulary), Form::One(text));
        }
        let suggested = if self.fault == SubjectFault::NamedSectionsOff {
            as_if_enabled_subject(
                &self.text,
                vocabulary,
                parse_subject,
                Recovered::rule_subject,
            )
        } else {
            // §FS-rule-refusals.sentences.rewrite.same-units: a rebuild that drops a typed
            // component judges other units, so the kind was recovered and no form is.
            recover_subject(self, vocabulary).and_then(|recovered| {
                Ok(Suggestion {
                    text: recovered.rule_subject().ok_or(false)?,
                    after_enabling: false,
                })
            })
        };
        let reason = self.reason(vocabulary);
        match suggested {
            Ok(suggested) => {
                refuse(reason, Form::One(suggested.text)).enabling(suggested.after_enabling)
            }
            Err(kind) => refuse(reason, Form::None { kind }),
        }
    }
}

/// Build a refusal from what a production read.
pub(super) fn refused(
    fault: SubjectFault,
    spelling: Spelling,
    text: &str,
    head: &str,
    path: Option<&str>,
    separator: &str,
) -> SubjectRefusal {
    SubjectRefusal {
        fault,
        spelling,
        text: text.into(),
        head: head.into(),
        path: path.map(Into::into),
        separator: separator.into(),
        modal_chapter: None,
        legacy: None,
        modal_value: None,
    }
}

pub(super) fn parse_subject(
    text: &str,
    vocab: &RuleVocabulary,
) -> Result<RuleSubject, SubjectRefusal> {
    // §FS-rule-refusals.sentences.every: both surfaces refuse these before literal-ID parsing.
    if let Some(kind) = text.strip_prefix("Every ") {
        return Err(refused(
            SubjectFault::Every,
            Spelling::Each,
            text,
            kind,
            None,
            "",
        ));
    }
    if let Some(rest) = text.strip_prefix("The ")
        && let Some((name, kind)) = rest.split_once(" chapter of every ")
    {
        return Err(refused(
            SubjectFault::Every,
            Spelling::ChapterOfEach,
            text,
            kind,
            Some(name),
            "",
        ));
    }
    if let Some(after_each) = text.strip_prefix("Each ") {
        // §FS-rule-refusals.selectors: refused for its quantifier; `parse_rule` refuses it first.
        if let Some(kind) = after_each.strip_prefix("chapter of each ") {
            let fault = SubjectFault::ChapterQuantified;
            return Err(refused(fault, Spelling::Each, text, kind, None, ""));
        }
        let (kind, clause) = split_whose(after_each, vocab);
        let each = |fault| {
            let refusal = refused(fault, Spelling::Each, text, kind, None, "");
            with_clause(refusal, after_each, clause)
        };
        known_subject(kind, vocab).map_err(each)?;
        let whose = clause
            .map(|clause| parse_whose(kind, clause, vocab))
            .transpose()
            .map_err(|fault| each(SubjectFault::Whose(fault)))?;
        return Ok(RuleSubject::Kind(kind.into(), whose));
    }
    if let Some(rest) = text.strip_prefix("The ")
        && let Some((name, after_each)) = rest.split_once(" chapter of each ")
    {
        let (kind, clause) = split_whose(after_each, vocab);
        let chapter = |fault| {
            let refusal = refused(fault, Spelling::ChapterOfEach, text, kind, Some(name), "");
            with_clause(refusal, after_each, clause)
        };
        known_subject(kind, vocab).map_err(chapter)?;
        if !vocab.named_sections_for(kind) {
            return Err(chapter(SubjectFault::NamedSectionsOff));
        }
        validate_named_path(name, kind, vocab).map_err(chapter)?;
        let whose = clause
            .map(|clause| parse_whose(kind, clause, vocab))
            .transpose()
            .map_err(|fault| chapter(SubjectFault::Whose(fault)))?;
        return Ok(RuleSubject::ChapterOfKind {
            kind: kind.into(),
            name: name.into(),
            whose,
        });
    }
    let split = split_section(text, vocab);
    let (head, separator, path) = split.map_or((text, "", None), |(declaration, sep, section)| {
        (declaration, sep, Some(section))
    });
    let literal = |fault| refused(fault, Spelling::Literal, text, head, path, separator);
    if text.contains('/') {
        return Err(literal(SubjectFault::Namespace));
    }
    if let Some(section) = path
        && section.contains('*')
    {
        return Err(literal(SubjectFault::SectionWildcard));
    }
    // §FS-rule-refusals.sentences.chapter-path: the first empty or all-digit component decides;
    // empty is no number.
    if let Some(section) = path
        && let Some(part) = unnamed_component(section)
    {
        return Err(literal(if part.is_empty() {
            SubjectFault::SectionGrammar(Component::Malformed)
        } else {
            SubjectFault::NumberedChapter
        }));
    }
    if path.is_some() && !named_literal(text, vocab) {
        return Err(literal(SubjectFault::NamedSectionsOff));
    }
    for (index, grammar) in vocab.id_grammars.iter().enumerate() {
        if let Ok((id, section)) = parse_id_arg(text, grammar) {
            known_subject(&id.kind, vocab).map_err(literal)?;
            let declaration =
                DeclarationSelector::parsed(render_input_id(grammar, text, &id), grammar);
            return Ok(match section {
                Some(path) => RuleSubject::ExactChapter {
                    declaration,
                    path,
                    separator: vocab
                        .section_separators
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| ".".into()),
                },
                None => RuleSubject::ExactDeclaration(declaration),
            });
        }
    }
    if vocab.id_grammars.is_empty()
        && let Some(kind) = vocab
            .kinds
            .iter()
            .filter(|kind| text.starts_with(&format!("{kind}-")))
            .max_by_key(|kind| kind.len())
    {
        known_subject(kind, vocab).map_err(literal)?;
        return Ok(match split {
            Some((declaration, _, path)) => RuleSubject::ExactChapter {
                declaration: declaration.into(),
                path: path.into(),
                separator: vocab
                    .section_separators
                    .first()
                    .cloned()
                    .unwrap_or_else(|| ".".into()),
            },
            None => RuleSubject::ExactDeclaration(text.into()),
        });
    }
    Err(literal(SubjectFault::IdGrammar))
}

fn known_subject(kind: &str, vocab: &RuleVocabulary) -> Result<(), SubjectFault> {
    if vocab.kinds.contains(kind) {
        Ok(())
    } else {
        Err(SubjectFault::UnknownKind(kind.into()))
    }
}
