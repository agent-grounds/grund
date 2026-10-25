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

use super::forms::{Form, Refusal, refuse};
use super::literal::DeclarationSelector;
use super::recovery::{
    Recovered, Suggestion, as_if_enabled_subject, recover_subject, repair_every,
};
use super::{RuleLevel, RulePolarity, RuleSubject, RuleVocabulary};
use crate::grammar::{named_section_component, parse_id_arg, render_input_id, unnamed_component};

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
                    None => reason,
                }
            }
            _ => legacy,
        };
        // §FS-rule-refusals.sentences.modal-chapter: the released reason stays its prefix.
        match self.modal_chapter {
            Some(name) => format!(
                "{reason} — chapter name \"{name}\" is read as the rule's modality, so rename \
                 the chapter"
            ),
            None => reason,
        }
    }

    /// This refusal of the subject `split` left, naming the modal word the
    /// split took where that word was a chapter subject's name: the subject is
    /// exactly `The` and what follows the modality opens `chapter of each `
    /// (§FS-rule-refusals.sentences.modal-chapter).
    pub(super) fn cut_at(mut self, split: Split<'_>) -> Self {
        let (subject, level, polarity, predicate) = split;
        if subject == "The" && predicate.starts_with("chapter of each ") {
            self.modal_chapter = MODALITIES
                .iter()
                .find(|(_, l, p)| (*l, *p) == (level, polarity))
                .map(|(modality, ..)| modality.trim());
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
            recover_subject(self, vocabulary).map(|recovered| Suggestion {
                text: recovered.rule_subject(),
                after_enabling: false,
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
    }
}

/// A sentence split at its modality: the subject, the modality, the predicate.
pub(super) type Split<'a> = (&'a str, RuleLevel, RulePolarity, &'a str);

/// The accepted modalities, in the order the split looks for them (§FS-rules.3.6).
const MODALITIES: [(&str, RuleLevel, RulePolarity); 4] = [
    (" must not ", RuleLevel::Required, RulePolarity::Prohibiting),
    (
        " should not ",
        RuleLevel::Recommended,
        RulePolarity::Prohibiting,
    ),
    (" must ", RuleLevel::Required, RulePolarity::Positive),
    (" should ", RuleLevel::Recommended, RulePolarity::Positive),
];

/// Split a sentence into its subject, its modality and its predicate. The
/// modality is found in a fixed order rather than at the earliest position
/// (§FS-rules.3.6), and a rule and a pasted selector both split here, so they
/// read the same subject from the same sentence (§FS-rule-refusals.selectors).
pub(super) fn split_modality(sentence: &str) -> Option<Split<'_>> {
    MODALITIES
        .into_iter()
        .find_map(|(modality, level, polarity)| {
            let (subject, predicate) = sentence.split_once(modality)?;
            Some((subject, level, polarity, predicate))
        })
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
    if let Some(kind) = text.strip_prefix("Each ") {
        let each = |fault, head| refused(fault, Spelling::Each, text, head, None, "");
        // §FS-rule-refusals.selectors: refused for its quantifier; `parse_rule` refuses it first.
        if let Some(kind) = kind.strip_prefix("chapter of each ") {
            return Err(each(SubjectFault::ChapterQuantified, kind));
        }
        known_subject(kind, vocab).map_err(|fault| each(fault, kind))?;
        return Ok(RuleSubject::Kind(kind.into()));
    }
    if let Some(rest) = text.strip_prefix("The ")
        && let Some((name, kind)) = rest.split_once(" chapter of each ")
    {
        let chapter = |fault| refused(fault, Spelling::ChapterOfEach, text, kind, Some(name), "");
        known_subject(kind, vocab).map_err(chapter)?;
        if !vocab.named_sections_for(kind) {
            return Err(chapter(SubjectFault::NamedSectionsOff));
        }
        validate_named_path(name, kind, vocab).map_err(chapter)?;
        return Ok(RuleSubject::ChapterOfKind {
            kind: kind.into(),
            name: name.into(),
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

/// The declaration, the separator, and the section path of a literal subject.
fn split_section<'a>(
    text: &'a str,
    vocab: &'a RuleVocabulary,
) -> Option<(&'a str, &'a str, &'a str)> {
    vocab
        .section_separators
        .iter()
        .find_map(|separator| {
            // coordinate-lexer: rule vocabulary, every separator
            text.split_once(separator.as_str())
                .map(|(declaration, section)| (declaration, separator.as_str(), section))
        })
        .or_else(|| {
            vocab
                .section_separators
                .is_empty()
                .then(|| text.split_once('.'))
                .flatten()
                .map(|(declaration, section)| (declaration, ".", section))
        })
}

/// Whether a literal subject's chapter path may be named: named sections are
/// on for the vocabulary, or for the kind of the declaration a configured
/// grammar reads it as (§AR-config.1.5).
fn named_literal(text: &str, vocab: &RuleVocabulary) -> bool {
    vocab.named_sections
        || vocab.id_grammars.iter().any(|grammar| {
            parse_id_arg(text, grammar).is_ok_and(|(id, _)| vocab.named_kinds.contains(&id.kind))
        })
}

/// Accept a chapter path of named components only (§FS-rules.2), read under
/// `kind`'s own project (§AR-config.1.5); a refusal says how its first
/// refused component failed.
pub(super) fn validate_named_path(
    path: &str,
    kind: &str,
    vocab: &RuleVocabulary,
) -> Result<(), SubjectFault> {
    if !vocab.named_sections_for(kind) {
        return Err(SubjectFault::NamedSectionsOff);
    }
    if let Some(component) = path.split('.').find(|part| !named_section_component(part)) {
        let shape = if !component.is_empty() && component.bytes().all(|b| b.is_ascii_digit()) {
            Component::Numbered
        } else if component.contains('*') {
            Component::Wildcard
        } else {
            Component::Malformed
        };
        return Err(SubjectFault::SectionGrammar(shape));
    }
    let grammar_valid = vocab.id_grammars.is_empty()
        || vocab
            .id_grammars
            .iter()
            .any(|grammar| grammar.is_section_path(path));
    if grammar_valid {
        Ok(())
    } else {
        Err(SubjectFault::SectionGrammar(Component::Malformed))
    }
}

fn known_subject(kind: &str, vocab: &RuleVocabulary) -> Result<(), SubjectFault> {
    if vocab.kinds.contains(kind) {
        Ok(())
    } else {
        Err(SubjectFault::UnknownKind(kind.into()))
    }
}
