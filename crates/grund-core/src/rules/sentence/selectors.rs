//! The `list --selector` surface (§FS-rules.8): the selector-only `KIND.NAME`
//! shorthand, and a refusal answered with a selector rather than a rule
//! sentence (§FS-rules.8.1).
//!
//! What failed is decided once, by `parse_subject` and `validate_named_path`;
//! this file only renders that decision. Where the section grammar refused a
//! numbered or wildcard component, it names that component as what failed.
//! Where named sections are off, it asks the same parser again with them on,
//! for the suggestion alone.

use super::subjects::{
    Component, Spelling, SubjectFault, SubjectRefusal, is_named_component, parse_subject, refused,
    split_modality, validate_named_path,
};
use super::{RuleSubject, RuleVocabulary};
use crate::grammar::Grammar;

/// §FS-rules.8.1: the breadcrumb a numbered, wildcard or quantified refusal adds.
const HINT: &str = "hint: grund show --batch --toc expands each selected unit into its sections";

/// A refused selector, as `list` prints it after `error: ` (§FS-rules.8.1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SelectorRefusal {
    reason: String,
    /// The selector to paste back; `None` where no configured kind was recovered.
    suggestion: Option<String>,
    after_enabling: bool,
    hint: bool,
}

impl SelectorRefusal {
    /// The refusal's lines. `known_kinds` is the `known kinds:` line an unknown
    /// `--kind` prints, used where no kind was recovered (§FS-rules.8.1).
    pub(crate) fn render(&self, known_kinds: &str) -> String {
        let hint = if self.hint {
            format!("\n{HINT}")
        } else {
            String::new()
        };
        let Some(suggestion) = &self.suggestion else {
            // §FS-rules.8.1: the breadcrumb follows the kinds where none was recovered.
            return format!("{}\n{known_kinds}{hint}", self.reason);
        };
        let accepted = if self.after_enabling {
            "accepted selector after enabling it"
        } else {
            "accepted selector"
        };
        format!("{}; {accepted}: {suggestion}{hint}", self.reason)
    }
}

/// Parse a `list --selector` value (§FS-rules.2, §FS-rules.8). The selectors
/// it accepts are the subjects plus `KIND.NAME`; only a refusal is new.
pub(crate) fn parse_selector(
    text: &str,
    vocabulary: &RuleVocabulary,
) -> Result<RuleSubject, SelectorRefusal> {
    let refusal = match select(text, vocabulary) {
        Ok(subject) => return Ok(subject),
        Err(refusal) => refusal,
    };
    // §FS-rules.8.1: a pasted rule sentence is answered with its subject, which
    // ends where the rule sentence's does (§FS-rules.3.6).
    let Some((subject, ..)) = split_modality(text) else {
        return Err(answer(&refusal, vocabulary));
    };
    Err(match select(subject, vocabulary) {
        Ok(_) => SelectorRefusal {
            reason: "a rule sentence is not a selector".into(),
            suggestion: Some(subject.into()),
            after_enabling: false,
            hint: false,
        },
        Err(own) => answer(&own, vocabulary),
    })
}

fn select(text: &str, vocabulary: &RuleVocabulary) -> Result<RuleSubject, SubjectRefusal> {
    if vocabulary.kinds.contains(text) {
        return Ok(RuleSubject::Kind(text.into()));
    }
    for separator in &vocabulary.section_separators {
        // §FS-rules.2: `KIND.NAME` means `The NAME chapter of each KIND`.
        if let Some((kind, name)) = text.split_once(separator.as_str())
            && vocabulary.kinds.contains(kind)
        {
            validate_named_path(name, vocabulary).map_err(|fault| {
                refused(fault, Spelling::KindName, text, kind, Some(name), separator)
            })?;
            return Ok(RuleSubject::ChapterOfKind {
                kind: kind.into(),
                name: name.into(),
            });
        }
    }
    // §FS-rules.8.1: the namespace reason `parse_rule` gives before the subject.
    if let Some(head) = text.strip_prefix("Each ")
        && head.starts_with("*/")
    {
        let fault = SubjectFault::Namespace;
        return Err(refused(fault, Spelling::Each, text, head, None, ""));
    }
    parse_subject(text, vocabulary)
}

/// Render a refused subject as a selector refusal (§FS-rules.8.1).
fn answer(refusal: &SubjectRefusal, vocabulary: &RuleVocabulary) -> SelectorRefusal {
    // §FS-rules.8.1: a numbered or wildcard component is named as what failed.
    let fault = match &refusal.fault {
        SubjectFault::SectionGrammar(Component::Numbered) => SubjectFault::NumberedChapter,
        SubjectFault::SectionGrammar(Component::Wildcard) => SubjectFault::SectionWildcard,
        fault => fault.clone(),
    };
    if fault == SubjectFault::NamedSectionsOff {
        return answer_as_if_enabled(refusal, vocabulary);
    }
    SelectorRefusal {
        reason: fault.reason(&refusal.text),
        // §FS-rules.8.1: the breadcrumb follows from the reason alone.
        hint: matches!(
            fault,
            SubjectFault::NumberedChapter
                | SubjectFault::SectionWildcard
                | SubjectFault::ChapterQuantified
        ),
        suggestion: suggestion(refusal, vocabulary),
        after_enabling: false,
    }
}

/// §FS-rules.8.1: a selector refused for needing named sections keeps that
/// reason, and is suggested what the same selector is suggested with them on.
fn answer_as_if_enabled(refusal: &SubjectRefusal, vocabulary: &RuleVocabulary) -> SelectorRefusal {
    let suggestion =
        enabled(vocabulary).and_then(|enabled| match select(&refusal.text, &enabled) {
            Ok(_) => Some(refusal.text.clone()),
            Err(own) => suggestion(&own, &enabled),
        });
    SelectorRefusal {
        reason: refusal.fault.reason(&refusal.text),
        // The label says whether pasting it back needs named sections on.
        after_enabling: suggestion
            .as_deref()
            .is_some_and(|selector| select(selector, vocabulary).is_err()),
        suggestion,
        hint: false,
    }
}

/// The vocabulary with named sections on and every grammar compiled that way,
/// or `None` where a grammar cannot be (§FS-rules.8.1).
fn enabled(vocabulary: &RuleVocabulary) -> Option<RuleVocabulary> {
    let id_grammars = vocabulary
        .id_grammars
        .iter()
        .map(Grammar::with_named_sections)
        .collect::<anyhow::Result<_>>()
        .ok()?;
    Some(RuleVocabulary {
        named_sections: true,
        id_grammars,
        ..vocabulary.clone()
    })
}

/// §FS-rules.8.1's four steps: the selector built from what was typed and the
/// configured kinds, or `None` where no configured kind can be recovered.
fn suggestion(refusal: &SubjectRefusal, vocabulary: &RuleVocabulary) -> Option<String> {
    let head = refusal
        .head
        .rsplit_once('/')
        .map_or(refusal.head.as_str(), |(_, local)| local);
    let kind = recovered_kind(head, vocabulary)?;
    let names = named_prefix(refusal.path.as_deref(), vocabulary);
    let base = match refusal.spelling {
        Spelling::Each => return Some(format!("Each {kind}")),
        Spelling::ChapterOfEach if names.is_empty() => return Some(format!("Each {kind}")),
        Spelling::ChapterOfEach => return Some(format!("The {names} chapter of each {kind}")),
        Spelling::Literal if is_declaration(head, vocabulary) => head,
        Spelling::KindName | Spelling::Literal => kind,
    };
    Some(if names.is_empty() {
        base.into()
    } else {
        format!("{base}{}{names}", refusal.separator)
    })
}

/// Step 1: the configured kind the text names or starts with, longest first.
fn recovered_kind<'a>(head: &str, vocabulary: &'a RuleVocabulary) -> Option<&'a str> {
    vocabulary
        .kinds
        .iter()
        .filter(|kind| {
            head.strip_prefix(kind.as_str())
                .is_some_and(|rest| !rest.starts_with(|c: char| c.is_ascii_alphanumeric()))
        })
        .max_by_key(|kind| kind.len())
        .map(String::as_str)
}

/// Step 2: whether a declaration was written as a valid ID.
fn is_declaration(head: &str, vocabulary: &RuleVocabulary) -> bool {
    matches!(
        parse_subject(head, vocabulary),
        Ok(RuleSubject::ExactDeclaration(_))
    )
}

/// Step 3: the longest run of named components before the first refused one.
fn named_prefix(path: Option<&str>, vocabulary: &RuleVocabulary) -> String {
    let Some(path) = path else {
        return String::new();
    };
    let components = path.split('.').collect::<Vec<_>>();
    let named = components
        .iter()
        .take_while(|component| is_named_component(component))
        .count();
    (1..=named)
        .rev()
        .map(|length| components[..length].join("."))
        .find(|prefix| validate_named_path(prefix, vocabulary).is_ok())
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "tests_selectors.rs"]
mod tests_selectors;
