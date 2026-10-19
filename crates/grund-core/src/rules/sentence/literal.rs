//! A literal declaration subject's spelling and numeric identity (§FS-config.3.2.8).
//! Grammar chooses the numeric component once; rule and catalog selectors use
//! the same matcher, leaving named section paths untouched (§FS-rules.2.1).

use crate::grammar::Grammar;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct DeclarationSelector {
    spelling: String,
    number: Option<(String, u32, String)>,
}

impl DeclarationSelector {
    pub(super) fn parsed(spelling: String, grammar: &Grammar) -> Self {
        let number = grammar.number_span(&spelling).and_then(|span| {
            Some((
                spelling[..span.start].to_string(),
                spelling[span.clone()].parse().ok()?,
                spelling[span.end..].to_string(),
            ))
        });
        Self { spelling, number }
    }

    /// Restrict indexed candidate labels before comparing their numeric value.
    pub(crate) fn prefix(&self) -> &str {
        self.number
            .as_ref()
            .map_or(&self.spelling, |(prefix, ..)| prefix)
    }

    pub(crate) fn matches(&self, declaration: &str) -> bool {
        match &self.number {
            Some((prefix, number, suffix)) => {
                declaration
                    .strip_prefix(prefix)
                    .and_then(|rest| rest.strip_suffix(suffix))
                    .and_then(|digits| digits.parse::<u32>().ok())
                    == Some(*number)
            }
            None => declaration == self.spelling,
        }
    }
}

// Hand-built producer-neutral subjects without a lexical grammar stay exact
// (§AR-rules.2); parsed Markdown subjects use `parsed` above.
impl From<&str> for DeclarationSelector {
    fn from(spelling: &str) -> Self {
        Self {
            spelling: spelling.into(),
            number: None,
        }
    }
}

impl fmt::Display for DeclarationSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.spelling.fmt(f)
    }
}
