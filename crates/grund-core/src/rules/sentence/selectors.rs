//! The `list --selector` surface (§FS-rules.8): the selector-only `KIND.NAME`
//! shorthand, and a refusal answered with a selector rather than a rule
//! sentence (§FS-rule-refusals.selectors).
//!
//! What failed is decided once, by `parse_subject` and `validate_named_path`
//! and named by `SubjectFault::failed`, and what a suggestion can recover once,
//! by `recovery.rs`; this file only renders those decisions. Where named
//! sections are off, it asks the same parser again with them on, for the
//! suggestion alone.

use super::chapter_paths::validate_named_path;
use super::forms::{Form, refuse, settle};
use super::modality::split_modality;
use super::recovery::{Recovered, as_if_enabled, recover, repair_every};
use super::subjects::{Spelling, SubjectFault, SubjectRefusal, parse_subject, refused};
use super::{RulePolarity, RuleSubject, RuleVocabulary};

/// §FS-rule-refusals.selectors: the breadcrumb a numbered, wildcard or quantified refusal adds.
const HINT: &str = "hint: grund show --batch --toc expands each selected unit into its sections";

/// A refused selector, as `list` prints it after `error: ` (§FS-rule-refusals.selectors).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SelectorRefusal {
    reason: String,
    /// The selector to paste back; `None` where no configured kind was recovered.
    suggestion: Option<String>,
    after_enabling: bool,
    hint: bool,
    /// Whether `known kinds:` follows where no selector is offered: not after a
    /// whose clause refused under a configured kind (§FS-errors.3.10).
    kinds: bool,
}

impl SelectorRefusal {
    /// The refusal's lines. `known_kinds` is the `known kinds:` line an unknown
    /// `--kind` prints, used where no kind was recovered (§FS-rule-refusals.selectors).
    pub(crate) fn render(&self, known_kinds: &str) -> String {
        let hint = if self.hint {
            format!("\n{HINT}")
        } else {
            String::new()
        };
        let Some(suggestion) = &self.suggestion else {
            if !self.kinds {
                return format!("{}{hint}", self.reason);
            }
            // §FS-rule-refusals.selectors: the breadcrumb follows the kinds where none was
            // recovered.
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
    // §FS-rule-refusals.selectors: a pasted rule sentence is answered with its subject, which
    // ends where the rule sentence's does (§FS-rules.3.6).
    let Some(split @ (subject, ..)) = split_modality(text) else {
        return Err(answer(&refusal, vocabulary));
    };
    Err(match select(subject, vocabulary) {
        Ok(_) => SelectorRefusal {
            reason: "a rule sentence is not a selector".into(),
            suggestion: Some(subject.into()),
            after_enabling: false,
            hint: false,
            kinds: true,
        },
        // §FS-rule-refusals.sentences.modal-chapter: a pasted sentence, or a two-word modal
        // name, is cut where the rule is and named as it is; a bare subject is not.
        Err(own) if split.2 == RulePolarity::Prohibiting || split_modality(split.3).is_some() => {
            answer(&own.cut_at(split), vocabulary)
        }
        Err(own) => answer(&own, vocabulary),
    })
}

fn select(text: &str, vocabulary: &RuleVocabulary) -> Result<RuleSubject, SubjectRefusal> {
    if vocabulary.kinds.contains(text) {
        return Ok(RuleSubject::Kind(text.into(), None));
    }
    for separator in &vocabulary.section_separators {
        // §FS-rules.2: `KIND.NAME` means `The NAME chapter of each KIND`.
        // coordinate-lexer: rule vocabulary `KIND.NAME`, not an ID coordinate
        if let Some((kind, name)) = text.split_once(separator.as_str())
            && vocabulary.kinds.contains(kind)
        {
            validate_named_path(name, kind, vocabulary).map_err(|fault| {
                refused(fault, Spelling::KindName, text, kind, Some(name), separator)
            })?;
            return Ok(RuleSubject::ChapterOfKind {
                kind: kind.into(),
                name: name.into(),
                whose: None,
            });
        }
    }
    // §FS-rule-refusals.selectors: the namespace reason `parse_rule` gives before the subject.
    if let Some(head) = text.strip_prefix("Each ")
        && head.starts_with("*/")
    {
        let fault = SubjectFault::Namespace;
        return Err(refused(fault, Spelling::Each, text, head, None, ""));
    }
    parse_subject(text, vocabulary)
}

/// Render a refused subject as a selector refusal (§FS-rule-refusals.selectors).
fn answer(refusal: &SubjectRefusal, vocabulary: &RuleVocabulary) -> SelectorRefusal {
    // §FS-rule-refusals.selectors: a subject with a whose clause is offered no selector;
    // `known kinds:` follows only a kind that is not configured (§FS-errors.3.10).
    if refusal.clause {
        return SelectorRefusal {
            reason: refusal.reason(vocabulary),
            suggestion: None,
            after_enabling: false,
            hint: false,
            kinds: matches!(refusal.fault, SubjectFault::UnknownKind(_)),
        };
    }
    // §FS-rule-refusals.sentences.every: reparse the token-only repair on this same surface.
    if let Some(text) = repair_every(refusal) {
        return SelectorRefusal {
            reason: refusal.reason(vocabulary),
            suggestion: settle_every(text, vocabulary),
            after_enabling: false,
            hint: false,
            kinds: true,
        };
    }
    // §FS-rule-refusals.selectors: a numbered or wildcard component is named as what failed.
    let fault = refusal.fault.failed();
    if fault == SubjectFault::NamedSectionsOff {
        return answer_as_if_enabled(refusal, vocabulary);
    }
    SelectorRefusal {
        reason: refusal.reason(vocabulary),
        // §FS-rule-refusals.selectors: the breadcrumb follows from the reason alone.
        hint: matches!(
            fault,
            SubjectFault::NumberedChapter
                | SubjectFault::SectionWildcard
                | SubjectFault::ChapterQuantified
        ),
        suggestion: recover(refusal, vocabulary).map(|recovered| recovered.selector()),
        after_enabling: false,
        kinds: true,
    }
}

/// Bound selector repairs with the ordinary reparse machinery
/// (§FS-rule-refusals.sentences.every). The quantifier reason cannot offer an
/// enabled-only selector; other subject faults keep their existing recovery.
fn settle_every(text: String, vocabulary: &RuleVocabulary) -> Option<String> {
    settle(text, false, |text, _| {
        let next = select(text, vocabulary).err()?;
        let suggestion = repair_every(&next).or_else(|| {
            let offered = answer(&next, vocabulary);
            offered.suggestion.filter(|_| !offered.after_enabling)
        });
        let form = suggestion.map_or(Form::None { kind: false }, Form::One);
        Some(refuse(next.reason(vocabulary), form))
    })
    .ok()
    .map(|(text, _)| text)
}

/// §FS-rule-refusals.selectors: a selector refused for needing named sections keeps that
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
        kinds: true,
    }
}

#[cfg(test)]
#[path = "tests_selectors.rs"]
mod tests_selectors;
