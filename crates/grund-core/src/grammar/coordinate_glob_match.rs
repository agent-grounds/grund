//! The matcher of a coordinate glob under one project's separator
//! (§FS-coordinate-globs.matching): an automaton over the positions of the reading
//! `coordinate_glob_read.rs` gives, stepped one coordinate symbol at a time, so a
//! match costs time bounded by the pattern's length and the coordinate's
//! (§FS-coordinate-globs.matching.bounded).

use super::coordinate_glob::{CoordinateGlob, Token};
use super::coordinate_glob_read::{Mode, Read, Reading};

impl CoordinateGlob {
    /// The pattern's matcher under `separator` (§FS-coordinate-globs.matching).
    pub(crate) fn matcher(&self, separator: &str) -> GlobMatcher<'_> {
        GlobMatcher {
            read: Read::new(self, separator),
        }
    }
}

/// One symbol of a coordinate: a character of a component, or the boundary between two.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Symbol {
    Char(char),
    Separator,
}

/// A position of the matcher's automaton.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct State {
    pos: usize,
    mode: Mode,
    /// Inside a `**`, reading whole components.
    globstar: bool,
    /// A `**` matched zero components, so the separator after it reads nothing.
    skip: bool,
}

/// A pattern compiled for one project's separator (§FS-coordinate-globs.matching).
pub(crate) struct GlobMatcher<'g> {
    read: Read<'g>,
}

impl GlobMatcher<'_> {
    /// Whether the coordinate `id` with `section` (a `.`-joined path) matches:
    /// anchored, case-sensitive and by code point (§FS-coordinate-globs.matching.anchored).
    pub(crate) fn matches(&self, id: &str, section: Option<&str>) -> bool {
        let components = section.into_iter().flat_map(|section| section.split('.'));
        let symbols = id
            .chars()
            .map(Symbol::Char)
            .chain(components.flat_map(|component| {
                std::iter::once(Symbol::Separator).chain(component.chars().map(Symbol::Char))
            }));
        let mut current = States::new(self.read.end());
        for &mode in self.read.starts {
            self.close_over(&mut current, start(0, mode));
        }
        for symbol in symbols {
            let mut next = States::new(self.read.end());
            for state in &current.list {
                if let Some(target) = self.step(*state, symbol) {
                    self.close_over(&mut next, target);
                }
            }
            if next.list.is_empty() {
                return false;
            }
            current = next;
        }
        let current = current.list;
        current.iter().any(|state| {
            state.pos == self.read.end()
                && !state.globstar
                && !state.skip
                && state.mode != Mode::BeforeSeparator
        })
    }

    fn step(&self, state: State, symbol: Symbol) -> Option<State> {
        if state.globstar {
            return Some(state);
        }
        if state.skip || state.pos == self.read.end() {
            return None;
        }
        let ch = match symbol {
            Symbol::Char(ch) => Some(ch),
            Symbol::Separator => None,
        };
        let advance = start(state.pos + 1, state.mode);
        match self.read.reading(state.pos, state.mode) {
            Reading::Separator { next, mode } => ch.is_none().then_some(start(next, mode)),
            Reading::LiteralDot => (ch == Some('.')).then_some(advance),
            Reading::Token(Token::Literal { ch: want, .. }) => {
                (ch == Some(*want)).then_some(advance)
            }
            Reading::Token(Token::Any) => ch.map(|_| advance),
            Reading::Token(Token::Class { negated, members }) => ch
                .filter(|&ch| {
                    members
                        .iter()
                        .any(|&(low, high)| (low..=high).contains(&ch))
                        != *negated
                })
                .map(|_| advance),
            Reading::Token(Token::Stars(1)) => ch.map(|_| state),
            _ => None,
        }
    }

    /// Add `state` and every state reachable from it without reading a symbol.
    fn close_over(&self, states: &mut States, state: State) {
        let mut pending = vec![state];
        while let Some(state) = pending.pop() {
            if !states.insert(state) {
                continue;
            }
            let State {
                pos,
                mode,
                globstar,
                skip,
            } = state;
            if globstar {
                pending.push(start(pos + 1, mode));
                continue;
            }
            if pos == self.read.end() {
                continue;
            }
            if let Some(targets) = self.read.jumps(pos) {
                pending.extend(targets.into_iter().map(|next| State { pos: next, ..state }));
                continue;
            }
            match self.read.reading(pos, mode) {
                Reading::Separator { next, mode } if skip => pending.push(start(next, mode)),
                Reading::Token(Token::Stars(run)) if !skip => {
                    if *run >= 2 {
                        pending.push(State {
                            globstar: true,
                            ..state
                        });
                        pending.push(State {
                            pos: pos + 1,
                            skip: true,
                            ..state
                        });
                    } else {
                        pending.push(start(pos + 1, mode));
                    }
                }
                _ => {}
            }
        }
    }
}

/// A set of automaton states: the list in insertion order, and a bitmap for membership.
struct States {
    list: Vec<State>,
    seen: Vec<bool>,
}

impl States {
    fn new(end: usize) -> Self {
        Self {
            list: Vec::new(),
            seen: vec![false; (end + 1) * 16],
        }
    }

    fn insert(&mut self, state: State) -> bool {
        let index = ((state.pos * 4 + state.mode as usize) * 2 + usize::from(state.globstar)) * 2
            + usize::from(state.skip);
        let fresh = !std::mem::replace(&mut self.seen[index], true);
        if fresh {
            self.list.push(state);
        }
        fresh
    }
}

fn start(pos: usize, mode: Mode) -> State {
    State {
        pos,
        mode,
        globstar: false,
        skip: false,
    }
}
