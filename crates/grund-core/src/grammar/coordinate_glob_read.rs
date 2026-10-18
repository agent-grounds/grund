//! A coordinate glob read under one project's outer separator
//! (§FS-coordinate-globs.grammar.components): the component checks that need the
//! separator (§FS-coordinate-globs.errors.reasons), and the reading the matcher in
//! `coordinate_glob_match.rs` steps through (§FS-coordinate-globs.matching). Both
//! walk the token list as an automaton whose states are positions, never as the
//! product of the brace groups, so a pattern is answered in time bounded by its length
//! and the coordinate's (§FS-coordinate-globs.matching.bounded).
//!
//! Where the ID atom ends is a fact of each alternative: it ends at the first
//! unescaped separator when the alternative holds one, else at its first unescaped
//! `.`. A walk under a separator other than `.` therefore guesses at the start
//! whether the alternative it is on holds one, and a guess the alternative
//! contradicts is a dead path.

use std::collections::BTreeMap;

use super::coordinate_glob::{CoordinateGlob, GlobRefusal, GlobStage, Spanned, Token, offered};

/// How a walk reads the pattern's dots.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Mode {
    /// The outer separator is `.`: every unescaped dot separates.
    Dotted,
    /// Guessed to hold the separator, still before it: dots are literal.
    BeforeSeparator,
    /// Past the separator: dots separate, and the separator is literal.
    AfterSeparator,
    /// Guessed to hold no separator: the first dot ends the ID atom.
    NoSeparator,
}

impl Mode {
    pub(super) fn starts(separator: &str) -> &'static [Mode] {
        if separator == "." {
            &[Mode::Dotted]
        } else {
            &[Mode::BeforeSeparator, Mode::NoSeparator]
        }
    }
}

/// What the token at one position is, under one mode.
pub(super) enum Reading<'g> {
    /// It ends a component; the walk goes on at `next` in `mode`.
    Separator {
        next: usize,
        mode: Mode,
    },
    /// A literal dot inside the ID atom.
    LiteralDot,
    Token(&'g Token),
    /// The guess this path made is contradicted.
    Dead,
}

/// The pattern's tokens with each separator occurrence found for one project.
pub(super) struct Read<'g> {
    tokens: &'g [Spanned],
    /// Where the walk resumes past a separator occurrence starting here.
    separator_at: Vec<Option<usize>>,
    pub(super) starts: &'static [Mode],
}

impl<'g> Read<'g> {
    pub(super) fn new(glob: &'g CoordinateGlob, separator: &str) -> Self {
        let tokens = glob.tokens.as_slice();
        let separator_at = (0..tokens.len())
            .map(|pos| match separator {
                "." => None,
                _ => occurrence(tokens, pos, separator),
            })
            .collect();
        Self {
            tokens,
            separator_at,
            starts: Mode::starts(separator),
        }
    }

    pub(super) fn end(&self) -> usize {
        self.tokens.len()
    }

    pub(super) fn reading(&self, pos: usize, mode: Mode) -> Reading<'g> {
        if let Some(next) = self.separator_at[pos] {
            match mode {
                Mode::BeforeSeparator => {
                    return Reading::Separator {
                        next,
                        mode: Mode::AfterSeparator,
                    };
                }
                Mode::NoSeparator => return Reading::Dead,
                Mode::Dotted | Mode::AfterSeparator => {}
            }
        }
        match &self.tokens[pos].token {
            Token::Dot if mode == Mode::BeforeSeparator => Reading::LiteralDot,
            Token::Dot => Reading::Separator {
                next: pos + 1,
                mode,
            },
            token => Reading::Token(token),
        }
    }

    /// Where group syntax at `pos` lets the walk go without reading anything.
    pub(super) fn jumps(&self, pos: usize) -> Option<Vec<usize>> {
        match &self.tokens[pos].token {
            Token::Open { alternatives, .. } => Some(alternatives.clone()),
            Token::Comma { close } => Some(vec![close + 1]),
            Token::Close => Some(vec![pos + 1]),
            _ => None,
        }
    }
}

/// The token after an unescaped run of literals at `pos` spelling `separator`.
fn occurrence(tokens: &[Spanned], pos: usize, separator: &str) -> Option<usize> {
    let mut rest = separator;
    let mut at = pos;
    while !rest.is_empty() {
        let Token::Literal { ch, escaped: false } = tokens.get(at)?.token else {
            return None;
        };
        rest = rest.strip_prefix(ch)?;
        at += 1;
    }
    Some(at)
}

/// What the component being read holds so far, for the checks a component close makes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Component {
    /// Nothing yet, opened by the dot at this token, if a dot opened it.
    Empty(Option<usize>),
    /// Exactly one star run, at this token.
    Run(usize),
    /// Anything else, with the first run of two or more stars in it.
    Mixed(Option<usize>),
}

/// A component check that failed: the token it points at, and how.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Problem {
    /// An empty component, closed at this byte, opened and closed by these tokens.
    Empty {
        at: usize,
        opener: Option<usize>,
        closer: Option<usize>,
    },
    /// A whole component of three or more stars.
    LongRun(usize),
    /// A run of two or more stars inside a component.
    PartRun(usize),
}

impl CoordinateGlob {
    /// The component checks under `separator` (§FS-coordinate-globs.grammar.globstar,
    /// §FS-coordinate-globs.errors.reasons): no alternative may hold an empty component or
    /// a star run that is not a whole `**`. Of several, the leftmost is reported, with
    /// each rewrite `offer` accepts.
    pub(super) fn check_components(
        &self,
        separator: &str,
        offer: Option<GlobStage<'_>>,
    ) -> Result<(), GlobRefusal> {
        let read = Read::new(self, separator);
        let problem = walk_components(&read, self.text.len());
        let Some(problem) = problem else {
            return Ok(());
        };
        let span = |token: usize| &self.tokens[token];
        let splice = |token: usize, with: &str| {
            let spanned = span(token);
            format!(
                "{}{with}{}",
                &self.text[..spanned.start],
                &self.text[spanned.end..]
            )
        };
        let (at, reason, rewrites) = match problem {
            Problem::Empty { at, opener, closer } => {
                let dot = opener
                    .or(closer)
                    .filter(|&token| span(token).token == Token::Dot);
                (
                    at,
                    "empty component",
                    dot.map(|dot| splice(dot, "")).into_iter().collect(),
                )
            }
            Problem::LongRun(run) => (span(run).start, STAR_REASON, vec![splice(run, "**")]),
            Problem::PartRun(run) => {
                let star = splice(run, "*");
                let mut rewrites = vec![star.clone()];
                if span(run).end == self.text.len() {
                    rewrites.push(format!("{star}{{,.**}}"));
                }
                (span(run).start, STAR_REASON, rewrites)
            }
        };
        Err(self.refusal(at, reason, offered(offer, rewrites)))
    }

    /// The ID atom's literal text before its first operator, a `{` included, as the
    /// note quotes it (§FS-coordinate-globs.errors.note).
    pub(crate) fn literal_prefix(&self, separator: &str) -> String {
        let read = Read::new(self, separator);
        let mode = if read.separator_at.iter().any(Option::is_some) {
            Mode::BeforeSeparator
        } else {
            read.starts[read.starts.len() - 1]
        };
        let mut prefix = String::new();
        for pos in 0..read.end() {
            match read.reading(pos, mode) {
                Reading::LiteralDot => prefix.push('.'),
                Reading::Token(Token::Literal { ch, .. }) => prefix.push(*ch),
                _ => break,
            }
        }
        prefix
    }
}

const STAR_REASON: &str = "\"**\" must be a whole component";

/// Walk every alternative at once, positions in order, and return the leftmost
/// problem any alternative a mode reads whole has.
fn walk_components(read: &Read<'_>, text_len: usize) -> Option<Problem> {
    type States = BTreeMap<(Mode, Component), Option<Problem>>;
    let column = |problem: &Problem| match *problem {
        Problem::Empty { at, .. } => at,
        Problem::LongRun(run) | Problem::PartRun(run) => read.tokens[run].start,
    };
    let keep = |old: Option<Problem>, new: Option<Problem>| match (old, new) {
        (Some(old), Some(new)) => Some(if column(&new) < column(&old) {
            new
        } else {
            old
        }),
        (old, new) => old.or(new),
    };
    let close = |component: Component, at: usize, closer: Option<usize>| match component {
        Component::Empty(opener) => Some(Problem::Empty { at, opener, closer }),
        Component::Run(run) if read.tokens[run].token != Token::Stars(2) => {
            Some(Problem::LongRun(run))
        }
        Component::Mixed(Some(run)) => Some(Problem::PartRun(run)),
        _ => None,
    };
    let mut states: Vec<States> = vec![States::new(); read.end() + 1];
    for &mode in read.starts {
        states[0].insert((mode, Component::Empty(None)), None);
    }
    let mut found: Option<Option<Problem>> = None;
    for pos in 0..=read.end() {
        let here = std::mem::take(&mut states[pos]);
        for ((mode, component), problem) in here {
            let push = |states: &mut Vec<States>, next: usize, key, problem| {
                let slot = states[next].entry(key).or_insert(problem);
                *slot = keep(*slot, problem);
            };
            if pos == read.end() {
                if mode != Mode::BeforeSeparator {
                    let last = keep(problem, close(component, text_len, None));
                    found = Some(found.map_or(last, |seen| keep(seen, last)));
                }
                continue;
            }
            if let Some(targets) = read.jumps(pos) {
                for next in targets {
                    push(&mut states, next, (mode, component), problem);
                }
                continue;
            }
            let start = read.tokens[pos].start;
            match read.reading(pos, mode) {
                Reading::Dead => {}
                Reading::Separator { next, mode: after } => {
                    let problem = keep(problem, close(component, start, Some(pos)));
                    let opener = (read.tokens[pos].token == Token::Dot).then_some(pos);
                    push(
                        &mut states,
                        next,
                        (after, Component::Empty(opener)),
                        problem,
                    );
                }
                reading => {
                    let run = matches!(reading, Reading::Token(Token::Stars(n)) if *n >= 2);
                    let component = match (component, run) {
                        (Component::Empty(_), true) => Component::Run(pos),
                        (Component::Empty(_), false) => Component::Mixed(None),
                        (Component::Run(first), _) => Component::Mixed(Some(first)),
                        (Component::Mixed(None), true) => Component::Mixed(Some(pos)),
                        (mixed @ Component::Mixed(_), _) => mixed,
                    };
                    push(&mut states, pos + 1, (mode, component), problem);
                }
            }
        }
    }
    found.flatten()
}
