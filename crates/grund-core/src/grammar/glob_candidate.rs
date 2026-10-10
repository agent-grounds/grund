//! The whole-candidate reading of a citation token (§FS-check.1.1.11): a glob
//! operator never ends a token early to leave a citation of its prefix. The
//! regex crate has no lookahead, so the full-ID patterns stop at the first byte
//! outside the grammar; this is the post-match half that looks at what follows
//! and says whether the token was an address or a pattern. Every pass that takes
//! a citation from a line asks it — the scanner's marked and bare paths, the
//! escape pass, and `fmt` — so no two of them can read one token two ways.

use std::ops::Range;

use super::compiled::{Grammar, QUALIFIED_CITATION_PREFIX};
use super::id_format::literal_after_kind_placeholder;

/// How a candidate that starts where a citation could start is read.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CandidateReading {
    /// No operator past what the full-ID pattern matched: the ordinary token.
    Ordinary,
    /// The whole candidate, operator and all, is an address the grammar accepts,
    /// this many bytes long (§FS-check.1.1.11 rules 1 and 2).
    Address(usize),
    /// A pattern this many bytes long, consumed whole (§FS-check.1.1.11 rule 3).
    Pattern(usize),
}

/// The tokens a line's citation passes start from: every full-ID capture (with the
/// offset that rebases it onto the line) and every marked candidate read as a
/// pattern, as the range from its marker to its end (§FS-check.checks.glob-citation).
pub(crate) struct CitationTokens<'a> {
    pub(crate) captures: Vec<(usize, regex::Captures<'a>)>,
    pub(crate) patterns: Vec<Range<usize>>,
}

impl Grammar {
    /// §FS-check.1.1.11: read the candidate at the start of `rest` whole, given the
    /// `prefix_len` bytes the full-ID pattern matched there (`0` for none). A
    /// candidate holding no operator is ordinary; one the grammar accepts whole is
    /// an address; any other is a pattern, provided the operator opens or sits
    /// inside a component of something the grammar was reading — after a matched
    /// prefix, the number-only shorthand's included, or directly after a kind and
    /// the literal its format puts next.
    pub(crate) fn read_candidate(&self, rest: &str, prefix_len: usize) -> CandidateReading {
        let separator = self.source.section_separator.as_str();
        let Some((len, first_operator)) = candidate_extent(rest, separator) else {
            return CandidateReading::Ordinary;
        };
        if len <= prefix_len {
            return CandidateReading::Ordinary;
        }
        if self.citation_whole_re.is_match(&rest[..len]) {
            return CandidateReading::Address(len);
        }
        if prefix_len > 0
            || self.opens_after_kind(&rest[..first_operator])
            || self.shorthand_reads(&rest[..first_operator])
        {
            return CandidateReading::Pattern(len);
        }
        CandidateReading::Ordinary
    }

    /// `read_candidate` for a token that may open with `<alias>/`: the tail after
    /// the alias is read whole, and the lengths stay measured from `rest`.
    pub(crate) fn read_qualified_candidate(
        &self,
        rest: &str,
        prefix_len: usize,
    ) -> CandidateReading {
        let Some(alias) = QUALIFIED_CITATION_PREFIX.find(rest) else {
            return self.read_candidate(rest, prefix_len);
        };
        let at = alias.end();
        match self.read_candidate(&rest[at..], prefix_len.saturating_sub(at)) {
            CandidateReading::Ordinary => CandidateReading::Ordinary,
            CandidateReading::Address(len) => CandidateReading::Address(at + len),
            CandidateReading::Pattern(len) => CandidateReading::Pattern(at + len),
        }
    }

    /// Whether a number-only shorthand reads a prefix of `head`, so an operator
    /// after `FS-001-` or inside `FS-0*1` sits in a component the grammar began
    /// (§FS-check.1.1.11 rule 3); only asked once an operator was found.
    fn shorthand_reads(&self, head: &str) -> bool {
        self.shorthands()
            .any(|shorthand| shorthand.unqualified_prefix_re().is_match(head))
    }

    /// Whether `head` is a configured kind, alone or followed by exactly the
    /// literal its effective format puts after `{kind}` — `FS-` in `FS-*`.
    fn opens_after_kind(&self, head: &str) -> bool {
        self.source.kinds.iter().any(|kind| {
            let Some(tail) = head.strip_prefix(kind.name.as_str()) else {
                return false;
            };
            let format = kind.format.as_deref().unwrap_or(&self.source.format);
            tail.is_empty() || literal_after_kind_placeholder(format) == Some(tail)
        })
    }
}

/// §FS-check.1.1.11 for a qualified tail no loaded grammar reads
/// (§FS-workspace.5.2): with no format to accept it whole, an operator past the
/// `prefix_len` bytes the loose shape read, after anything at all, is a pattern.
pub(crate) fn read_loose_candidate(rest: &str, prefix_len: usize) -> CandidateReading {
    match candidate_extent(rest, ".") {
        Some((len, first_operator))
            if len > prefix_len && (prefix_len > 0 || first_operator > 0) =>
        {
            CandidateReading::Pattern(len)
        }
        _ => CandidateReading::Ordinary,
    }
}

/// The candidate's length and where its first operator sits, or `None` when it
/// holds no operator. The candidate runs over address characters, separators and
/// operators; a `*` or `?` run after a complete component that leads to nothing
/// more is punctuation and ends it, `[^` is a footnote, and a trailing separator
/// is sentence punctuation (§FS-check.1.1.11).
fn candidate_extent(rest: &str, separator: &str) -> Option<(usize, usize)> {
    let mut at = 0;
    let mut first_operator = None;
    while let Some(ch) = rest[at..].chars().next() {
        if is_address_char(ch) {
            at += ch.len_utf8();
            continue;
        }
        if rest[at..].starts_with(separator) {
            at += separator.len();
            continue;
        }
        let end = match ch {
            '*' | '?' => {
                let run = rest[at..]
                    .find(|ch: char| ch != '*' && ch != '?')
                    .unwrap_or(rest.len() - at);
                let after_component = rest[..at]
                    .chars()
                    .next_back()
                    .is_some_and(|prev| prev.is_ascii_alphanumeric() || prev == '_');
                if after_component && !leads_on(&rest[at + run..], separator) {
                    break;
                }
                at + run
            }
            '[' | '{' if opens_group(&rest[at..]) => {
                let close = if ch == '[' { ']' } else { '}' };
                rest[at..]
                    .find(|c: char| c == close || c.is_whitespace())
                    .filter(|offset| rest[at + offset..].starts_with(close))
                    .map_or(at + 1, |offset| at + offset + 1)
            }
            _ => break,
        };
        first_operator.get_or_insert(at);
        at = end;
    }
    let first_operator = first_operator?;
    while at > first_operator && rest[..at].ends_with(separator) {
        at -= separator.len();
    }
    Some((at, first_operator))
}

fn is_address_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
}

/// A `[` class opens on an address character or `!`, never `[^`; a `{`
/// alternation on an address character or `,`.
fn opens_group(text: &str) -> bool {
    let mut chars = text.chars();
    let open = chars.next();
    let next = chars.next();
    match (open, next) {
        (Some('['), Some(next)) => is_address_char(next) || next == '!',
        (Some('{'), Some(next)) => is_address_char(next) || next == ',',
        _ => false,
    }
}

/// Whether what follows a `*` or `?` run continues the candidate: an address
/// character, another operator, or a separator leading to either.
fn leads_on(after: &str, separator: &str) -> bool {
    let opens = |text: &str| {
        text.chars()
            .next()
            .is_some_and(|ch| is_address_char(ch) || ch == '*' || ch == '?')
            || opens_group(text)
    };
    opens(after) || after.strip_prefix(separator).is_some_and(opens)
}
