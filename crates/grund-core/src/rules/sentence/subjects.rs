//! Subject recognition shared by rules and catalog filtering (§FS-rules.2,
//! §FS-rules.8).
//!
//! A refused subject comes back as what the parser read rather than as finished
//! text: the production that failed, plus the kind, declaration and path it had
//! read. The rule surfaces answer it with what can be recovered written as a
//! rule subject (§FS-rules.3.5.4.2), and a subject needing named sections with
//! one they make valid (§FS-rules.3.5.2); a chapter path is refused for the
//! component that failed (§FS-rules.3.5.3), and `selectors.rs` renders it as a
//! selector (§FS-rules.8.1). Both surfaces agree on what failed because only
//! one parser decided it and `SubjectFault::failed` names it for both, and on
//! what a suggestion recovers because only `recovery.rs` builds it.

use super::forms::{Form, Refusal, refuse};
use super::recovery::{Recovered, Suggestion, as_if_enabled, recover};
use super::{RuleLevel, RulePolarity, RuleSubject, RuleVocabulary};
use crate::grammar::{parse_id_arg, render_id};

/// Which production of a subject failed (§FS-rules.3.5).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SubjectFault {
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

/// How a subject was spelled, which a suggestion keeps (§FS-rules.8.1).
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
}

impl SubjectFault {
    /// What a refusal names as failed: a chapter path the section grammar
    /// refused at a numbered or wildcard component is refused for that
    /// component, whichever spelling reached it (§FS-rules.3.5.3,
    /// §FS-rules.8.1).
    pub(super) fn failed(&self) -> Self {
        match self {
            Self::SectionGrammar(Component::Numbered) => Self::NumberedChapter,
            Self::SectionGrammar(Component::Wildcard) => Self::SectionWildcard,
            fault => fault.clone(),
        }
    }

    /// The released reason a refusal opens with (§FS-rules.3.5).
    pub(super) fn reason(&self, text: &str) -> String {
        match self {
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
    /// The rule surfaces' refusal (§FS-rules.3.5): the reason for the component
    /// that failed (§FS-rules.3.5.3), then a form whose subject is rebuilt from
    /// what can be recovered (§FS-rules.3.5.4.2), or, where it needs named
    /// sections, the subject §FS-rules.3.5.2 suggests, read with them on. Where
    /// no configured kind is recovered there is no form, and what is missing is
    /// a kind.
    pub(super) fn rule_refusal(&self, vocabulary: &RuleVocabulary) -> Refusal {
        let suggested = if self.fault == SubjectFault::NamedSectionsOff {
            as_if_enabled(
                &self.text,
                vocabulary,
                parse_subject,
                Recovered::rule_subject,
            )
        } else {
            recover(self, vocabulary).map(|recovered| Suggestion {
                text: recovered.rule_subject(),
                after_enabling: false,
            })
        };
        let reason = self.fault.failed().reason(&self.text);
        match suggested {
            Some(suggested) => {
                refuse(reason, Form::One(suggested.text)).enabling(suggested.after_enabling)
            }
            None => refuse(reason, Form::None { kind: true }),
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
    }
}

/// Split a sentence into its subject, its modality and its predicate. The
/// modality is found in a fixed order rather than at the earliest position
/// (§FS-rules.3.6), and a rule and a pasted selector both split here, so they
/// read the same subject from the same sentence (§FS-rules.8.1).
pub(super) fn split_modality(sentence: &str) -> Option<(&str, RuleLevel, RulePolarity, &str)> {
    [
        (" must not ", RuleLevel::Required, RulePolarity::Prohibiting),
        (
            " should not ",
            RuleLevel::Recommended,
            RulePolarity::Prohibiting,
        ),
        (" must ", RuleLevel::Required, RulePolarity::Positive),
        (" should ", RuleLevel::Recommended, RulePolarity::Positive),
    ]
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
    if let Some(kind) = text.strip_prefix("Each ") {
        let each = |fault, head| refused(fault, Spelling::Each, text, head, None, "");
        // §FS-rules.8.1: refused for its quantifier; `parse_rule` refuses it first.
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
        if !vocab.named_sections {
            return Err(chapter(SubjectFault::NamedSectionsOff));
        }
        validate_named_path(name, vocab).map_err(chapter)?;
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
    // §FS-rules.3.5.3: the first empty or all-digit component decides; empty is no number.
    if let Some(section) = path
        && let Some(part) = section
            .split('.')
            .find(|part| part.is_empty() || part.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(literal(if part.is_empty() {
            SubjectFault::SectionGrammar(Component::Malformed)
        } else {
            SubjectFault::NumberedChapter
        }));
    }
    if path.is_some() && !vocab.named_sections {
        return Err(literal(SubjectFault::NamedSectionsOff));
    }
    for (index, grammar) in vocab.id_grammars.iter().enumerate() {
        if let Ok((id, section)) = parse_id_arg(text, grammar) {
            known_subject(&id.kind, vocab).map_err(literal)?;
            return Ok(match section {
                Some(path) => RuleSubject::ExactChapter {
                    declaration: render_id(grammar, &id),
                    path,
                    separator: vocab
                        .section_separators
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| ".".into()),
                },
                None => RuleSubject::ExactDeclaration(text.into()),
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

/// Whether one chapter-path component is a named one (§FS-rules.2).
pub(super) fn is_named_component(component: &str) -> bool {
    component
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_lowercase)
        && component
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// Accept a chapter path of named components only (§FS-rules.2); a refusal
/// says how its first refused component failed.
pub(super) fn validate_named_path(path: &str, vocab: &RuleVocabulary) -> Result<(), SubjectFault> {
    if !vocab.named_sections {
        return Err(SubjectFault::NamedSectionsOff);
    }
    if let Some(component) = path.split('.').find(|part| !is_named_component(part)) {
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
