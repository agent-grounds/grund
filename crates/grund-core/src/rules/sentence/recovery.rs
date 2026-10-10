//! What a refused subject still names, for both surfaces that answer a refusal
//! with something to paste back (§FS-rules.8.1, §FS-rules.3.5.2,
//! §FS-rules.3.5.4.2).
//!
//! §FS-rules.8.1's four steps recover a configured kind, a declaration written
//! as a valid ID, and the longest run of named chapter components from what was
//! typed. They return those parts rather than finished text: `list --selector`
//! spells them as a selector and the rule surfaces spell them as a rule
//! subject, so what can be recovered is decided once. Where named sections are
//! off, both surfaces ask their own parser again with them on.

use super::subjects::{
    Spelling, SubjectRefusal, is_named_component, parse_subject, validate_named_path,
};
use super::{RuleSubject, RuleVocabulary};
use crate::grammar::Grammar;

/// What §FS-rules.8.1's four steps recovered from a refused subject.
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
}

impl Recovered {
    /// The parts as a selector (§FS-rules.8.1): the typed spelling, falling
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

    /// The parts as a rule subject (§FS-rules.3.5.4.2): what the selector would
    /// be, with the selector-only `KIND.NAME[.NAME…]` written
    /// `The NAME[.NAME…] chapter of each KIND` and a bare `KIND` `Each KIND`.
    pub(super) fn rule_subject(&self) -> String {
        let Self { kind, names, .. } = self;
        match &self.declaration {
            Some(declaration) => self.with_names(declaration),
            None if names.is_empty() => format!("Each {kind}"),
            None => format!("The {names} chapter of each {kind}"),
        }
    }

    fn with_names(&self, base: &str) -> String {
        if self.names.is_empty() {
            base.into()
        } else {
            format!("{base}{}{}", self.separator, self.names)
        }
    }
}

/// §FS-rules.8.1's four steps over a refusal, or `None` where no configured
/// kind can be recovered.
pub(super) fn recover(refusal: &SubjectRefusal, vocabulary: &RuleVocabulary) -> Option<Recovered> {
    let head = refusal
        .head
        .rsplit_once('/')
        .map_or(refusal.head.as_str(), |(_, local)| local);
    let kind = recovered_kind(head, vocabulary)?;
    let declaration = (refusal.spelling == Spelling::Literal && is_declaration(head, vocabulary))
        .then(|| head.to_string());
    Some(Recovered {
        spelling: refusal.spelling,
        kind: kind.into(),
        declaration,
        names: named_prefix(refusal.path.as_deref(), vocabulary),
        separator: refusal.separator.clone(),
    })
}

/// A suggestion for a subject refused for needing named sections, and whether
/// pasting it back needs them on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Suggestion {
    pub(super) text: String,
    pub(super) after_enabling: bool,
}

/// §FS-rules.8.1 and §FS-rules.3.5.2: a subject refused for needing named
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
    let enabled = enabled(vocabulary)?;
    let text = match parse(text, &enabled) {
        Ok(_) => text.to_string(),
        Err(own) => render(&recover(&own, &enabled)?),
    };
    Some(Suggestion {
        after_enabling: parse(&text, vocabulary).is_err(),
        text,
    })
}

/// The vocabulary with named sections on and every grammar compiled that way,
/// or `None` where a grammar cannot be (§FS-rules.8.1). A form suggested after
/// enabling them is read again with it (§FS-rules.3.5.4.3).
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
#[path = "tests_recovery.rs"]
mod tests_recovery;
