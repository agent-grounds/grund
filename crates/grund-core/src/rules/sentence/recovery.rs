//! What a refused subject still names, for both surfaces that answer a refusal
//! with something to paste back (§FS-rule-refusals.selectors, §FS-rule-refusals.sentences.named-sections,
//! §FS-rule-refusals.sentences.rewrite.recovered).
//!
//! §FS-rule-refusals.selectors's four steps recover a configured kind, a declaration written
//! as a valid ID, and the longest run of named chapter components from what was
//! typed. They return those parts rather than finished text: `list --selector`
//! spells them as a selector and the rule surfaces spell them as a rule
//! subject, so what can be recovered is decided once. Where named sections are
//! off, both surfaces ask their own parser again with them on.

use super::chapter_paths::validate_named_path;
use super::subjects::{Spelling, SubjectFault, SubjectRefusal, parse_subject};
use super::{RuleSubject, RuleVocabulary};
use crate::grammar::{Grammar, named_section_component, split_alias_path};
use std::collections::BTreeSet;

/// What §FS-rule-refusals.selectors's four steps recovered from a refused subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Recovered {
    /// How the subject was spelled, which a suggestion keeps (step 4).
    spelling: Spelling,
    /// The configured kind the text names or starts with (step 1).
    kind: String,
    /// The declaration, kept where a literal wrote it as a valid ID (step 2).
    declaration: Option<String>,
    /// The longest run of named components before the first refused one, empty
    /// where none survives (step 3).
    names: String,
    /// What joined the head and the chapter path as typed.
    separator: String,
    /// Whether step 3 dropped a typed component that names something, which a
    /// selector may and a rule may not (§FS-rule-refusals.sentences.rewrite.same-units).
    dropped: bool,
}

impl Recovered {
    /// The parts as a selector (§FS-rule-refusals.selectors): the typed spelling, falling
    /// back to `KIND[.NAME…]` or `Each KIND` only where nothing of it survives.
    pub(super) fn selector(&self) -> String {
        let Self { kind, names, .. } = self;
        let base = match self.spelling {
            Spelling::Each => return format!("Each {kind}"),
            Spelling::ChapterOfEach if names.is_empty() => return format!("Each {kind}"),
            Spelling::ChapterOfEach => return format!("The {names} chapter of each {kind}"),
            Spelling::KindName | Spelling::Literal => self.declaration.as_deref().unwrap_or(kind),
        };
        self.with_names(base)
    }

    /// The parts as a rule subject (§FS-rule-refusals.sentences.rewrite.recovered): what the selector would
    /// be, with the selector-only `KIND.NAME[.NAME…]` written
    /// `The NAME[.NAME…] chapter of each KIND` and a bare `KIND` `Each KIND`.
    /// `None` where step 3 dropped a typed component, because that subject would
    /// judge a parent chapter, the declaration or every declaration of the kind
    /// rather than what was typed (§FS-rule-refusals.sentences.rewrite.same-units).
    pub(super) fn rule_subject(&self) -> Option<String> {
        let Self { kind, names, .. } = self;
        if self.dropped {
            return None;
        }
        Some(match &self.declaration {
            Some(declaration) => self.with_names(declaration),
            None if names.is_empty() => format!("Each {kind}"),
            None => format!("The {names} chapter of each {kind}"),
        })
    }

    fn with_names(&self, base: &str) -> String {
        if self.names.is_empty() {
            base.into()
        } else {
            format!("{base}{}{}", self.separator, self.names)
        }
    }
}

/// §FS-rule-refusals.selectors: recover a faithful selector, or offer nothing
/// where either its kind or its typed scope cannot be supplied.
pub(super) fn recover(refusal: &SubjectRefusal, vocabulary: &RuleVocabulary) -> Option<Recovered> {
    recover_subject(refusal, vocabulary).ok()
}

/// §FS-rule-refusals.sentences.rewrite.recovered: derive scope once for both
/// surfaces. An error says whether a kind is missing, so a rule with a known
/// kind but no faithful subject gets no misleading known-kinds footer.
pub(super) fn recover_subject(
    refusal: &SubjectRefusal,
    vocabulary: &RuleVocabulary,
) -> Result<Recovered, bool> {
    // §FS-workspace.1: recovery of refused text, which no grammar admits, so the lenient split.
    let (_, head) = split_alias_path(&refusal.head);
    let kind = recovered_kind(head, vocabulary).ok_or(true)?;
    if refusal.fault == SubjectFault::ChapterQuantified {
        return Err(false);
    }
    // §FS-rule-refusals.sentences.rewrite.recovered: a chapter left in the head is scope,
    // not extra text to discard after recognizing the kind.
    if head.contains(char::is_whitespace) {
        return reordered_scope(refusal, kind, vocabulary).ok_or(false);
    }
    if refusal.spelling == Spelling::Each {
        // §FS-rule-refusals.sentences.rewrite.recovered: removing the refused `Each`
        // must keep a valid literal's identity, including its configured chapter separator.
        let literal = match parse_subject(head, vocabulary) {
            Ok(RuleSubject::ExactDeclaration(declaration)) => {
                Some((declaration, String::new(), String::new()))
            }
            Ok(RuleSubject::ExactChapter {
                declaration,
                path,
                separator,
            }) => Some((declaration, path, separator)),
            Err(own)
                if own.fault == SubjectFault::NamedSectionsOff
                    && !vocabulary.named_sections_for(kind) =>
            {
                return Err(false);
            }
            Err(own)
                if own.spelling == Spelling::Literal
                    && own.path.is_some()
                    && is_declaration(&own.head, vocabulary) =>
            {
                // §FS-rule-refusals.sentences.rewrite.recovered: reuse the literal's parsed
                // head and path so a refused suffix cannot discard its declaration or named prefix.
                if !vocabulary.named_sections_for(kind) {
                    return Err(false);
                }
                return recover_subject(&own, vocabulary);
            }
            _ => None,
        };
        if let Some((declaration, names, separator)) = literal {
            return Ok(Recovered {
                spelling: Spelling::Literal,
                kind: kind.into(),
                declaration: Some(declaration.to_string()),
                names,
                separator,
                dropped: false,
            });
        }
        // §FS-rule-refusals.sentences.rewrite.recovered: a refused component keeps its
        // valid prefix, even when `Each` left the path inside the head.
        if let Some(path) = vocabulary
            .section_separators
            .iter()
            .find_map(|separator| head.strip_prefix(kind)?.strip_prefix(separator.as_str()))
        {
            if !vocabulary.named_sections_for(kind) {
                return Err(false);
            }
            let (names, dropped) = named_prefix(Some(path), kind, vocabulary);
            return Ok(Recovered {
                spelling: Spelling::ChapterOfEach,
                kind: kind.into(),
                declaration: None,
                names,
                separator: String::new(),
                dropped,
            });
        }
    }
    let declaration = (refusal.spelling == Spelling::Literal && is_declaration(head, vocabulary))
        .then(|| head.to_string());
    let (names, dropped) = named_prefix(refusal.path.as_deref(), kind, vocabulary);
    Ok(Recovered {
        spelling: refusal.spelling,
        kind: kind.into(),
        declaration,
        names,
        separator: refusal.separator.clone(),
        dropped,
    })
}

/// §FS-rule-refusals.sentences.rewrite.recovered: recognize only typed, complete
/// named paths in alternative subject word orders. The ordinary refused-path
/// prefix policy remains in `named_prefix`; an ambiguous word order cannot
/// borrow its whole-declaration fallback.
fn reordered_scope(
    refusal: &SubjectRefusal,
    kind: &str,
    vocabulary: &RuleVocabulary,
) -> Option<Recovered> {
    let text = match refusal.spelling {
        Spelling::Each => refusal.text.strip_prefix("Each ")?,
        Spelling::Literal => &refusal.text,
        _ => return None,
    };
    let (_, text) = split_alias_path(text);
    let (base, tail) = text.split_once(' ')?;
    let names = tail.strip_suffix(" chapter").unwrap_or(tail);
    validate_named_path(names, kind, vocabulary).ok()?;
    if base == kind {
        return Some(Recovered {
            spelling: Spelling::ChapterOfEach,
            kind: kind.into(),
            declaration: None,
            names: names.into(),
            separator: String::new(),
            dropped: false,
        });
    }
    if refusal.spelling != Spelling::Literal || !is_declaration(base, vocabulary) {
        return None;
    }
    // §FS-rule-refusals.sentences.rewrite.recovered: the configured grammar must accept
    // exactly one literal chapter spelling; no declaration or separator is guessed.
    let separators: BTreeSet<_> = vocabulary
        .section_separators
        .iter()
        .map(String::as_str)
        .chain(vocabulary.section_separators.is_empty().then_some("."))
        .collect();
    let mut candidates = separators.into_iter().filter_map(|separator| {
        match parse_subject(&format!("{base}{separator}{names}"), vocabulary).ok()? {
            RuleSubject::ExactChapter {
                declaration,
                path,
                separator,
            } => Some(Recovered {
                spelling: Spelling::Literal,
                kind: kind.into(),
                declaration: Some(declaration.to_string()),
                names: path,
                separator,
                dropped: false,
            }),
            _ => None,
        }
    });
    let recovered = candidates.next()?;
    candidates.next().is_none().then_some(recovered)
}

/// A suggestion for a subject refused for needing named sections, and whether
/// pasting it back needs them on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Suggestion {
    pub(super) text: String,
    pub(super) after_enabling: bool,
}

/// Replace only the unsupported subject quantifier (§FS-rule-refusals.sentences.every).
/// Callers reparse the result before offering it, preserving unknown kinds and
/// complete paths for the ordinary recovery stages rather than guessing here.
pub(super) fn repair_every(refusal: &SubjectRefusal) -> Option<String> {
    if refusal.fault != SubjectFault::Every {
        return None;
    }
    match refusal.spelling {
        Spelling::Each => Some(format!("Each {}", refusal.head)),
        Spelling::ChapterOfEach => Some(format!(
            "The {} chapter of each {}",
            refusal.path.as_deref()?,
            refusal.head
        )),
        _ => None,
    }
}

/// §FS-rule-refusals.selectors and §FS-rule-refusals.sentences.named-sections: a subject refused for needing named
/// sections is asked of the surface's own `parse` again with them on. Where
/// that accepts it, it is suggested as typed; otherwise `render` spells what
/// the second refusal recovers. The label is decided by parsing the suggestion
/// as configured. `None` where nothing is recovered, or where a configured
/// grammar cannot be compiled with named sections on.
pub(super) fn as_if_enabled<T>(
    text: &str,
    vocabulary: &RuleVocabulary,
    parse: impl Fn(&str, &RuleVocabulary) -> Result<T, SubjectRefusal>,
    render: impl Fn(&Recovered) -> String,
) -> Option<Suggestion> {
    as_if_enabled_subject(text, vocabulary, parse, |recovered| Some(render(recovered))).ok()
}

/// §FS-rule-refusals.sentences.rewrite.recovered: preserve the shared recovery's
/// missing-kind distinction when a rule retries with named sections enabled.
/// Where `render` spells nothing, the kind was recovered and the typed scope
/// was not (§FS-rule-refusals.sentences.rewrite.same-units).
pub(super) fn as_if_enabled_subject<T>(
    text: &str,
    vocabulary: &RuleVocabulary,
    parse: impl Fn(&str, &RuleVocabulary) -> Result<T, SubjectRefusal>,
    render: impl Fn(&Recovered) -> Option<String>,
) -> Result<Suggestion, bool> {
    let enabled = enabled(vocabulary).ok_or(true)?;
    let text = match parse(text, &enabled) {
        Ok(_) => text.to_string(),
        Err(own) => render(&recover_subject(&own, &enabled)?).ok_or(false)?,
    };
    Ok(Suggestion {
        after_enabling: parse(&text, vocabulary).is_err(),
        text,
    })
}

/// The vocabulary with named sections on and every grammar compiled that way,
/// or `None` where a grammar cannot be (§FS-rule-refusals.selectors). A form suggested after
/// enabling them is read again with it (§FS-rule-refusals.sentences.rewrite.reparsed).
pub(super) fn enabled(vocabulary: &RuleVocabulary) -> Option<RuleVocabulary> {
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

/// Step 3: the longest run of named components before the first refused one,
/// and whether that drops a typed component other than one empty one at the
/// end, which names nothing (§FS-rule-refusals.sentences.rewrite.same-units).
fn named_prefix(path: Option<&str>, kind: &str, vocabulary: &RuleVocabulary) -> (String, bool) {
    let Some(path) = path else {
        return (String::new(), false);
    };
    let components = path.split('.').collect::<Vec<_>>();
    let named = components
        .iter()
        .take_while(|component| named_section_component(component))
        .count();
    let kept = (1..=named)
        .rev()
        .find(|&length| {
            validate_named_path(&components[..length].join("."), kind, vocabulary).is_ok()
        })
        .unwrap_or(0);
    let dropped = !matches!(&components[kept..], [] | [""]);
    (components[..kept].join("."), dropped)
}

#[cfg(test)]
#[path = "tests_recovery.rs"]
mod tests_recovery;
