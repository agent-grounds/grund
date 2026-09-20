//! Subject-selector recognition shared by rules and catalog filtering
//! (§FS-rules.2, §FS-rules.8).

use super::{RuleParseError, RuleSubject, RuleVocabulary, error};
use crate::grammar::{parse_id_arg, render_id};

pub(crate) fn parse_selector(
    text: &str,
    vocabulary: &RuleVocabulary,
) -> Result<RuleSubject, RuleParseError> {
    if vocabulary.kinds.contains(text) {
        return Ok(RuleSubject::Kind(text.into()));
    }
    for separator in &vocabulary.section_separators {
        if let Some((kind, name)) = text.split_once(separator)
            && vocabulary.kinds.contains(kind)
        {
            validate_named_path(name, vocabulary, text)?;
            return Ok(RuleSubject::ChapterOfKind {
                kind: kind.into(),
                name: name.into(),
            });
        }
    }
    parse_subject(text, vocabulary)
}

pub(super) fn parse_subject(
    text: &str,
    vocab: &RuleVocabulary,
) -> Result<RuleSubject, RuleParseError> {
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
    if let Some((declaration, section)) = split_section(text, vocab)
        && section.contains('*')
    {
        return Err(error(format!(
            "section-component wildcards are not accepted in phase 1; accepted form: {declaration}.requirements must cite at least one REQ."
        )));
    }
    if let Some((declaration, section)) = split_section(text, vocab)
        && section
            .split('.')
            .any(|part| part.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(error(format!(
            "numbered chapter subjects can detach when headings move; accepted form: {declaration}.requirements must cite at least one REQ."
        )));
    }
    if split_section(text, vocab).is_some() && !vocab.named_sections {
        return Err(error(format!(
            "named chapter subjects require [id] named_sections = true; accepted form after enabling it: {text} must cite at least one REQ."
        )));
    }
    for (index, grammar) in vocab.id_grammars.iter().enumerate() {
        if let Ok((id, section)) = parse_id_arg(text, grammar) {
            known_subject(&id.kind, vocab)?;
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
        known_subject(kind, vocab)?;
        return Ok(match split_section(text, vocab) {
            Some((declaration, path)) => RuleSubject::ExactChapter {
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
    Err(error(format!(
        "literal subject \"{text}\" does not match the configured ID grammar; accepted form: FS-login must cite at least one GOAL."
    )))
}

fn split_section<'a>(text: &'a str, vocab: &RuleVocabulary) -> Option<(&'a str, &'a str)> {
    vocab
        .section_separators
        .iter()
        .find_map(|separator| text.split_once(separator))
        .or_else(|| {
            vocab
                .section_separators
                .is_empty()
                .then(|| text.split_once('.'))
                .flatten()
        })
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
