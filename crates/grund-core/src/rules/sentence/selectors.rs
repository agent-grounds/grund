//! The `list --selector` surface (§FS-rules.8): the selector-only `KIND.NAME`
//! shorthand, and a refusal answered with a selector rather than a rule
//! sentence (§FS-rules.8.1).
//!
//! What failed is decided once, by `parse_subject` and `validate_named_path`,
//! and what a suggestion can recover once, by `recovery.rs`; this file only
//! renders those decisions. Where the section grammar refused a numbered or
//! wildcard component, it names that component as what failed. Where named
//! sections are off, it asks the same parser again with them on, for the
//! suggestion alone.

use super::recovery::{Recovered, as_if_enabled, recover};
use super::subjects::{
    Component, Spelling, SubjectFault, SubjectRefusal, parse_subject, refused, split_modality,
    validate_named_path,
};
use super::{RuleSubject, RuleVocabulary};

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
        suggestion: recover(refusal, vocabulary).map(|recovered| recovered.selector()),
        after_enabling: false,
    }
}

/// §FS-rules.8.1: a selector refused for needing named sections keeps that
/// reason, and is suggested what the same selector is suggested with them on.
fn answer_as_if_enabled(refusal: &SubjectRefusal, vocabulary: &RuleVocabulary) -> SelectorRefusal {
    let suggested = as_if_enabled(&refusal.text, vocabulary, select, Recovered::selector);
    SelectorRefusal {
        reason: refusal.fault.reason(&refusal.text),
        // The label says whether pasting it back needs named sections on.
        after_enabling: suggested
            .as_ref()
            .is_some_and(|suggested| suggested.after_enabling),
        suggestion: suggested.map(|suggested| suggested.text),
        hint: false,
    }
}

#[cfg(test)]
#[path = "tests_selectors.rs"]
mod tests_selectors;
