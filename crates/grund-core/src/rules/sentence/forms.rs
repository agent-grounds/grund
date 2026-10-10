//! The accepted form a refused rule offers (§FS-rules.3.5.4).
//!
//! A refusing production writes no finished text. It returns its reason and
//! what it offers in place of the text it read: that text with the failed
//! production spelled canonically, a two-form refusal's pair, or nothing where
//! the sentence does not say what belongs there. Each caller wraps the form
//! back into the text it read, so a refusal leaves the parser holding the
//! whole sentence as typed with only the failed part replaced
//! (§FS-rules.3.5.4.1). This module reads that sentence again before offering
//! it, and renders the one message both rule surfaces print.

use super::recovery::enabled;
use super::{RuleParseError, RuleVocabulary};

/// What a refusing production offers in place of the text it read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Form {
    /// The text with the failed production replaced (§FS-rules.3.5.4.1).
    One(String),
    /// A two-form refusal's pair, and the word that joins them.
    Two([String; 2], &'static str),
    /// Nothing can be supplied, and whether what is missing is a kind
    /// (§FS-rules.3.5.4.4).
    None { kind: bool },
}

/// A refusal whose form has not been offered yet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Refusal {
    reason: String,
    form: Form,
    /// The form is read with named sections on (§FS-rules.3.5.2).
    enabling: bool,
    /// Guidance that ends the message, form or not (§FS-rules.3.5.1).
    trailer: Option<&'static str>,
}

pub(super) fn refuse(reason: impl Into<String>, form: Form) -> Refusal {
    Refusal {
        reason: reason.into(),
        form,
        enabling: false,
        trailer: None,
    }
}

impl Refusal {
    /// The same refusal, its form wrapped into the text the caller read.
    pub(super) fn within(mut self, wrap: impl Fn(&str) -> String) -> Self {
        self.form = match self.form {
            Form::One(form) => Form::One(wrap(&form)),
            Form::Two([first, second], joiner) => Form::Two([wrap(&first), wrap(&second)], joiner),
            none @ Form::None { .. } => none,
        };
        self
    }

    /// The same refusal, its form read with named sections on where `enabling`.
    pub(super) fn enabling(mut self, enabling: bool) -> Self {
        self.enabling = enabling;
        self
    }

    /// The same refusal, ending in `trailer` whether it offers a form or not.
    pub(super) fn trailing(mut self, trailer: &'static str) -> Self {
        self.trailer = Some(trailer);
        self
    }

    /// The refusal as both rule surfaces print it (§FS-rules.3.5.4), `read`
    /// being the parser a form is read again with. A form is offered only once
    /// it is accepted, and a pair only where both are (§FS-rules.3.5.4.3).
    pub(super) fn offer(
        self,
        vocabulary: &RuleVocabulary,
        read: impl Fn(&str, &RuleVocabulary) -> Option<Refusal>,
    ) -> RuleParseError {
        let sections_on = enabled(vocabulary);
        let mut reread = |form: &str, enabled: bool| match (enabled, &sections_on) {
            (true, Some(sections_on)) => read(form, sections_on),
            _ => read(form, vocabulary),
        };
        let Self {
            reason,
            form,
            enabling,
            trailer,
        } = self;
        let offered = match form {
            Form::One(form) => {
                settle(form, enabling, &mut reread).map(|(form, enabled)| (form, enabled, None))
            }
            Form::Two([first, second], joiner) => {
                settle(first, enabling, &mut reread).and_then(|(first, enabled)| {
                    let (second, also) = settle(second, enabling, &mut reread)?;
                    Ok((first, enabled || also, Some((joiner, second))))
                })
            }
            Form::None { kind } => Err(kind),
        };
        let (first, enabled, second) = match offered {
            Ok(offered) => offered,
            // §FS-rules.3.5.4.4: the reason alone, and the kinds where one is missing.
            Err(kind) => {
                return RuleParseError {
                    message: match trailer {
                        Some(trailer) => format!("{reason}; {trailer}"),
                        None => reason,
                    },
                    unrecovered: kind,
                };
            }
        };
        let forms = [Some(&first), second.as_ref().map(|(_, second)| second)];
        // §FS-rules.3.5.2 step 3: the label is decided on the whole offered sentence.
        let after = enabled
            && forms
                .into_iter()
                .flatten()
                .any(|form| read(form, vocabulary).is_some());
        let (plural, forms) = match second {
            None => ("", first),
            Some((joiner, second)) => ("s", format!("\"{first}\" {joiner} \"{second}\"")),
        };
        let after = if after { " after enabling it" } else { "" };
        let trailer = trailer.map_or(String::new(), |trailer| format!(" {trailer}"));
        RuleParseError {
            message: format!("{reason}; accepted form{plural}{after}: {forms}{trailer}"),
            unrecovered: false,
        }
    }
}

/// The productions a rule sentence has (§FS-rules.3): the fixed word `Each`,
/// the subject, the modality, the verb, the bound, its number, `times`, the
/// chapter `NAME`, the chapter noun, the object kinds and the terminal `.`.
pub(super) const PRODUCTIONS: usize = 11;

/// §FS-rules.3.5.4.3: read `form` again until `read` accepts it, replacing what
/// each pass refuses the way that refusal says. Each pass replaces one refused
/// production, so the passes end within the productions a sentence has; a
/// pass with nothing to put in, more than one candidate, or nothing to change
/// ends them with no form. `enabled` is whether named sections are on for the
/// reading, which a pass may turn on and which is a change of its own.
///
/// `Ok` is the accepted form and whether it was read with named sections on;
/// `Err` is whether what could not be supplied is a kind.
pub(super) fn settle(
    mut form: String,
    mut enabled: bool,
    mut read: impl FnMut(&str, bool) -> Option<Refusal>,
) -> Result<(String, bool), bool> {
    for _ in 0..PRODUCTIONS {
        let Some(refusal) = read(&form, enabled) else {
            return Ok((form, enabled));
        };
        let next = match refusal.form {
            Form::One(next) => next,
            Form::Two(..) => return Err(false),
            Form::None { kind } => return Err(kind),
        };
        if next == form && (enabled || !refusal.enabling) {
            return Err(false);
        }
        enabled |= refusal.enabling;
        form = next;
    }
    Err(false)
}

#[cfg(test)]
#[path = "tests_forms.rs"]
mod tests_forms;
