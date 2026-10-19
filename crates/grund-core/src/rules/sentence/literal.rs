//! A literal declaration subject's spelling and numeric identity (§FS-config.3.2.8).
//! Grammar chooses the numeric component once; rule and catalog selectors use
//! the same matcher, leaving named section paths untouched (§FS-rules.2.1).

use crate::grammar::Grammar;
use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Debug)]
pub(crate) struct DeclarationSelector {
    spelling: String,
    number: Option<(String, u32, String)>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum SelectorIdentity<'a> {
    Exact(&'a str),
    Numeric(&'a str, u32, &'a str),
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

    /// Matching and semantic grouping share numeric identity, independently of
    /// display spelling (§FS-config.3.2.8, §FS-rules.6).
    fn identity(&self) -> SelectorIdentity<'_> {
        match &self.number {
            Some((prefix, number, suffix)) => SelectorIdentity::Numeric(prefix, *number, suffix),
            None => SelectorIdentity::Exact(&self.spelling),
        }
    }

    /// Restrict indexed candidate labels before comparing their numeric value.
    pub(crate) fn prefix(&self) -> &str {
        match self.identity() {
            SelectorIdentity::Numeric(prefix, ..) => prefix,
            SelectorIdentity::Exact(spelling) => spelling,
        }
    }

    pub(crate) fn matches(&self, declaration: &str) -> bool {
        match self.identity() {
            SelectorIdentity::Numeric(prefix, number, suffix) => {
                declaration
                    .strip_prefix(prefix)
                    .and_then(|rest| rest.strip_suffix(suffix))
                    .and_then(|digits| digits.parse::<u32>().ok())
                    == Some(number)
            }
            SelectorIdentity::Exact(spelling) => declaration == spelling,
        }
    }
}

impl PartialEq for DeclarationSelector {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}

impl Eq for DeclarationSelector {}

impl Ord for DeclarationSelector {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identity().cmp(&other.identity())
    }
}

impl PartialOrd for DeclarationSelector {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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

#[cfg(test)]
#[path = "tests_literal.rs"]
mod tests_literal;
