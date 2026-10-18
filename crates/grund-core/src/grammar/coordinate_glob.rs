//! Coordinate globs (§FS-coordinate-globs): a pattern read once into tokens over the
//! operator table the citation guard reads (§FS-coordinate-globs.grammar), and every
//! refusal with its column and the rewrite it offers (§FS-coordinate-globs.errors.reasons).
//! Where a pattern's ID atom ends depends on the project's outer separator
//! (§FS-coordinate-globs.grammar.components), so the separator-dependent checks and the
//! matcher take it as an argument and live beside this file, in `coordinate_glob_read.rs`
//! and `coordinate_glob_match.rs`; the body's tokenizer is `coordinate_glob_tokens.rs`.

use std::fmt;

use super::coordinate_glob_tokens::Tokenizer;
use super::glob_candidate::GlobOperator;

/// Which loaded projects a pattern reads (§FS-coordinate-globs.scope).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GlobQualifier {
    /// No qualifier: the current project only.
    Current,
    /// `*/`: every loaded project.
    Every,
    /// `alias/`: the one loaded project with this exact alias.
    Alias(String),
}

/// One refused pattern, rendered as the line after `error: `
/// (§FS-coordinate-globs.errors.shape).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GlobRefusal {
    pub(crate) pattern: String,
    /// The one-based byte column in the pattern as typed.
    pub(crate) column: usize,
    pub(crate) reason: String,
    /// Each accepted glob, already parsed again (§FS-rule-refusals.requirements.typed).
    pub(crate) accepted: Vec<String>,
}

impl fmt::Display for GlobRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid coordinate glob \"{}\" at column {}: {}",
            self.pattern, self.column, self.reason
        )?;
        match self.accepted.as_slice() {
            [] => Ok(()),
            [one] => write!(f, "; accepted glob: \"{one}\""),
            many => {
                let quoted: Vec<String> = many.iter().map(|glob| format!("\"{glob}\"")).collect();
                write!(f, "; accepted globs: {}", quoted.join(" or "))
            }
        }
    }
}

/// One token of a pattern's body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Token {
    /// A character matched as itself; an escaped one is never a separator.
    Literal { ch: char, escaped: bool },
    /// An unescaped `.` outside a class.
    Dot,
    /// `?`.
    Any,
    /// A run of unescaped `*`, this many long.
    Stars(usize),
    /// A class: inclusive code-point ranges, and whether it is negated.
    Class {
        negated: bool,
        members: Vec<(char, char)>,
    },
    /// `{`, with each alternative's first token and the group's `}`.
    Open {
        alternatives: Vec<usize>,
        close: usize,
    },
    /// `,` inside a group, with the group's `}`.
    Comma { close: usize },
    /// `}` closing a group.
    Close,
}

/// A token and the bytes of the pattern it was read from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Spanned {
    pub(super) token: Token,
    pub(super) start: usize,
    pub(super) end: usize,
}

/// A pattern the grammar accepts (§FS-coordinate-globs.grammar).
#[derive(Clone, Debug)]
pub(crate) struct CoordinateGlob {
    pub(super) text: String,
    qualifier: GlobQualifier,
    /// Where the body starts, past the qualifier and its `/`.
    pub(super) body: usize,
    pub(super) tokens: Vec<Spanned>,
}

impl CoordinateGlob {
    /// Read `text`, making every refusal decided before any config is read
    /// (§FS-coordinate-globs.errors.shape): the qualifier, the brackets and
    /// escapes, and — for a pattern that no outer separator but `.` could split
    /// differently — the components as a `.`-separated project reads them.
    pub(crate) fn parse(text: &str) -> Result<Self, GlobRefusal> {
        Self::launch(text, true)
    }

    pub(super) fn launch(text: &str, offer: bool) -> Result<Self, GlobRefusal> {
        let glob = Self::read(text, offer)?;
        if !glob.may_split_elsewhere() {
            glob.check_components(".", offer)?;
        }
        Ok(glob)
    }

    fn read(text: &str, offer: bool) -> Result<Self, GlobRefusal> {
        let body = last_unescaped_slash(text).map_or(0, |slash| slash + 1);
        let qualifier = match body {
            0 => GlobQualifier::Current,
            _ => read_qualifier(text, body - 1, offer)?,
        };
        let tokens = Tokenizer::new(text, body, offer).run()?;
        Ok(Self {
            text: text.to_string(),
            qualifier,
            body,
            tokens,
        })
    }

    pub(crate) fn qualifier(&self) -> &GlobQualifier {
        &self.qualifier
    }

    /// Refuse a pattern that opens with the citation marker
    /// (§FS-coordinate-globs.errors.marker).
    pub(crate) fn check_marker(&self, marker: &str) -> Result<(), GlobRefusal> {
        let body = &self.text[self.body..];
        let Some(rest) = body.strip_prefix(marker).filter(|_| !marker.is_empty()) else {
            return Ok(());
        };
        let rewrite = format!("{}{rest}", &self.text[..self.body]);
        Err(self.refusal(
            self.body,
            "coordinate globs do not take a citation marker",
            offered([rewrite], None),
        ))
    }

    /// The refusal of a qualifier naming no loaded project (§FS-coordinate-globs.errors.reasons).
    pub(crate) fn unknown_alias(&self, alias: &str, known: &[&str]) -> GlobRefusal {
        let mut reason = format!("unknown project alias \"{alias}\"");
        if !known.is_empty() {
            reason.push_str(&format!("\nknown aliases: {}", known.join(", ")));
        }
        self.refusal(0, &reason, Vec::new())
    }

    /// The pattern with its body's first bytes spelled `spelling`, where they are
    /// `spelling` with ASCII case ignored: the note's suggestion
    /// (§FS-coordinate-globs.errors.note).
    pub(crate) fn respelled(&self, spelling: &str) -> Option<String> {
        let body = &self.text[self.body..];
        let typed = body.get(..spelling.len())?;
        typed.eq_ignore_ascii_case(spelling).then(|| {
            format!(
                "{}{spelling}{}",
                &self.text[..self.body],
                &body[spelling.len()..]
            )
        })
    }

    /// Whether the body holds a character a project could take as its outer
    /// separator — an unescaped literal outside a class that is not alphanumeric,
    /// `-` or `_` — so a `.`-separated reading of it may not be the project's.
    fn may_split_elsewhere(&self) -> bool {
        self.tokens.iter().any(|spanned| {
            matches!(spanned.token, Token::Literal { ch, escaped: false }
                if !(ch.is_alphanumeric() || ch == '-' || ch == '_'))
        })
    }

    pub(super) fn refusal(&self, at: usize, reason: &str, accepted: Vec<String>) -> GlobRefusal {
        refusal(&self.text, at, reason, accepted)
    }
}

/// Keep only the rewrites that parse again, under `separator` when one is named
/// (§FS-coordinate-globs.errors.shape).
pub(super) fn offered(
    rewrites: impl IntoIterator<Item = String>,
    separator: Option<&str>,
) -> Vec<String> {
    rewrites
        .into_iter()
        .filter(|rewrite| {
            CoordinateGlob::launch(rewrite, false).is_ok_and(|glob| {
                separator.is_none_or(|separator| glob.check_components(separator, false).is_ok())
            })
        })
        .collect()
}

pub(super) fn refusal(text: &str, at: usize, reason: &str, accepted: Vec<String>) -> GlobRefusal {
    GlobRefusal {
        pattern: text.to_string(),
        column: at + 1,
        reason: reason.to_string(),
        accepted,
    }
}

fn last_unescaped_slash(text: &str) -> Option<usize> {
    let mut last = None;
    let mut chars = text.char_indices();
    while let Some((at, ch)) = chars.next() {
        match ch {
            '\\' => {
                chars.next();
            }
            '/' => last = Some(at),
            _ => {}
        }
    }
    last
}

/// `*` or one exact alias, unescaped (§FS-coordinate-globs.grammar.components).
fn read_qualifier(text: &str, slash: usize, offer: bool) -> Result<GlobQualifier, GlobRefusal> {
    let raw = &text[..slash];
    if raw == "*" {
        return Ok(GlobQualifier::Every);
    }
    let mut alias = String::new();
    let mut chars = raw.char_indices();
    while let Some((at, ch)) = chars.next() {
        if ch == '\\' {
            alias.extend(chars.next().map(|(_, next)| next));
        } else if GlobOperator::of(ch).is_some() {
            let rewrites = offer.then(|| format!("*/{}", &text[slash + 1..]));
            return Err(refusal(
                text,
                at,
                "a project qualifier is one alias or \"*\"",
                offered(rewrites, None),
            ));
        } else {
            alias.push(ch);
        }
    }
    Ok(GlobQualifier::Alias(alias))
}
