//! A chapter path in a subject (§FS-rules.2, §FS-rules.2.1): where a literal
//! subject's declaration ends and its path begins, and whether a path of named
//! components is one the kind's own project can address (§AR-config.1.5).

use super::RuleVocabulary;
use super::subjects::{Component, SubjectFault};
use crate::grammar::{named_section_component, parse_id_arg};

/// The declaration, the separator, and the section path of a literal subject.
pub(super) fn split_section<'a>(
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
pub(super) fn named_literal(text: &str, vocab: &RuleVocabulary) -> bool {
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
