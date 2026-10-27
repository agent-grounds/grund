//! The TOML span tree (§DA-toml-front-matter-reader): the events of
//! `toml_parser`'s parser, read against the source text and folded into the
//! document's top-level keys, each with its byte span and its value, and a
//! contribution TOML refuses kept as a second site of its key rather than an
//! error (§DA-toml-front-matter-reader.2.2). It walks the events in one loop and
//! recurses on nothing, so neither a deep value nor a key of many components
//! can overflow the stack (§REQ-never-crashes.1).

use std::collections::HashMap;

use toml_parser::decoder::ScalarKind;
use toml_parser::parser::{Event, EventKind, RecursionGuard, ValidateWhitespace, parse_document};
use toml_parser::{ParseError, Source};

use super::yaml::{MAX_NESTING, nesting_reason};

/// Bytes of the text the tree was given, never characters
/// (§DA-toml-front-matter-reader.2.3).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TomlSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// One site of a top-level key (§FS-fields.front-matter.keys): its first
/// contribution, spanning the contributions of its first run
/// (§FS-fields.front-matter.coordinates), or a later one TOML refuses, spanning
/// itself.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct TomlKey {
    /// The decoded first component.
    pub(crate) name: String,
    pub(crate) span: TomlSpan,
    /// The value of a `k = …` whose value is a scalar on one line: a string's
    /// decoded text, any other scalar's raw spelling
    /// (§FS-fields.front-matter.values).
    pub(crate) one_line: Option<String>,
}

/// Text the parser or its decoder rejects: the first message in source order,
/// and the byte it did not expect (§FS-fields.front-matter.reading).
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct TomlError {
    pub(crate) at: usize,
    pub(crate) message: String,
}

/// §DA-toml-front-matter-reader.2.4: the guard counts arrays and inline tables,
/// the YAML tree counts the top level too, so the guard stops one below the bound.
const GUARD_DEPTH: u32 = (MAX_NESTING - 1) as u32;

/// The guard's own message, which the shared reason replaces.
const GUARD_MESSAGE: &str = "cannot recurse further; max recursion depth met";

/// Read `text` into its top-level keys in source order, a key TOML refuses to
/// define again listed at each refused site (§DA-toml-front-matter-reader.2.2).
pub(crate) fn parse_toml(text: &str) -> Result<Vec<TomlKey>, TomlError> {
    let source = Source::new(text);
    let tokens = source.lex().into_vec();
    let mut events: Vec<Event> = Vec::with_capacity(tokens.len());
    let mut errors: Vec<ParseError> = Vec::new();
    let mut validated = ValidateWhitespace::new(&mut events, source);
    let mut guarded = RecursionGuard::new(&mut validated, GUARD_DEPTH);
    parse_document(&tokens, &mut guarded, &mut errors);
    let contributions = Walk {
        events: &events,
        at: 0,
        source,
        errors: &mut errors,
    }
    .contributions();
    match first_error(text, &errors) {
        Some(error) => Err(error),
        None => Ok(fold(contributions)),
    }
}

/// §FS-fields.front-matter.reading: the first message in source order, at the
/// start of what it did not expect, the guard's in the shared words.
fn first_error(text: &str, errors: &[ParseError]) -> Option<TomlError> {
    let start = |error: &ParseError| {
        error
            .unexpected()
            .or(error.context())
            .map_or(0, |span| span.start())
    };
    let error = errors.iter().min_by_key(|error| start(error))?;
    let message = match error.description() {
        GUARD_MESSAGE => nesting_reason(),
        description => description.to_string(),
    };
    let mut at = start(error).min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    Some(TomlError { at, message })
}

/// What a contribution writes (§FS-fields.terms): `k = …`, `k.x = …`, `[k]`,
/// `[k.x]`, `[[k]]` or `[[k.x]]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Form {
    Value,
    Dotted,
    Table,
    SubTable,
    Array,
    SubArray,
}

/// One place the document writes a top-level key.
struct Contribution {
    name: String,
    form: Form,
    span: TomlSpan,
    one_line: Option<String>,
}

/// A cursor over the events, decoding every key and scalar it passes so the
/// decoder's refusals join the parser's.
struct Walk<'w, 'i> {
    events: &'w [Event],
    at: usize,
    source: Source<'i>,
    errors: &'w mut Vec<ParseError>,
}

impl Walk<'_, '_> {
    fn peek(&self) -> Option<Event> {
        self.events.get(self.at).copied()
    }

    fn next(&mut self) -> Option<Event> {
        let event = self.peek()?;
        self.at += 1;
        Some(event)
    }

    fn next_if(&mut self, kinds: &[EventKind]) -> Option<Event> {
        self.peek()
            .filter(|event| kinds.contains(&event.kind()))
            .inspect(|_| self.at += 1)
    }

    fn skip_whitespace(&mut self) {
        while self.next_if(&[EventKind::Whitespace]).is_some() {}
    }

    /// Every contribution in source order. A key-value below a header fills
    /// that header's table, so it extends the header's span instead.
    fn contributions(mut self) -> Vec<Contribution> {
        let mut out: Vec<Contribution> = Vec::new();
        let mut header: Option<usize> = None;
        while let Some(event) = self.next() {
            let start = event.span().start();
            match event.kind() {
                EventKind::StdTableOpen | EventKind::ArrayTableOpen => {
                    let array = event.kind() == EventKind::ArrayTableOpen;
                    self.skip_whitespace();
                    let Some(first) = self.next_if(&[EventKind::SimpleKey]) else {
                        continue;
                    };
                    let (name, dotted) = self.key(first);
                    let close = [EventKind::StdTableClose, EventKind::ArrayTableClose];
                    let end = self
                        .next_if(&close)
                        .map_or(first.span().end(), |close| close.span().end());
                    let form = match (array, dotted) {
                        (false, false) => Form::Table,
                        (false, true) => Form::SubTable,
                        (true, false) => Form::Array,
                        (true, true) => Form::SubArray,
                    };
                    header = Some(out.len());
                    out.push(Contribution {
                        name,
                        form,
                        span: TomlSpan { start, end },
                        one_line: None,
                    });
                }
                EventKind::SimpleKey => {
                    let (name, dotted) = self.key(event);
                    let (end, one_line) = self.value(event.span().end());
                    if let Some(index) = header {
                        out[index].span.end = end;
                        continue;
                    }
                    let form = if dotted { Form::Dotted } else { Form::Value };
                    out.push(Contribution {
                        name,
                        form,
                        span: TomlSpan { start, end },
                        one_line: one_line.filter(|_| !dotted),
                    });
                }
                EventKind::Scalar => self.scalar(event),
                _ => {}
            }
        }
        out
    }

    /// A key from its first component, `first`: that component decoded, and
    /// whether more follow. Only the first is kept, so a key of many components
    /// costs one pass and no depth (§FS-fields.front-matter.keys).
    fn key(&mut self, first: Event) -> (String, bool) {
        let mut name = String::new();
        if let Some(raw) = self.source.get(first) {
            raw.decode_key(&mut name, self.errors);
        }
        let mut dotted = false;
        while let Some(event) = self.next_if(&[
            EventKind::SimpleKey,
            EventKind::KeySep,
            EventKind::Whitespace,
        ]) {
            if event.kind() == EventKind::SimpleKey {
                dotted = true;
                if let Some(raw) = self.source.get(event) {
                    raw.decode_key(&mut (), self.errors);
                }
            }
        }
        (name, dotted)
    }

    /// The value after a key's `=`: where it ends, `after` where nothing is
    /// read, and its one-line text where it is a scalar on one line.
    fn value(&mut self, after: usize) -> (usize, Option<String>) {
        if self.next_if(&[EventKind::KeyValSep]).is_none() {
            return (after, None);
        }
        self.skip_whitespace();
        let open = [EventKind::ArrayOpen, EventKind::InlineTableOpen];
        if let Some(event) = self.next_if(&[EventKind::Scalar]) {
            let Some(raw) = self.source.get(event) else {
                return (event.span().end(), None);
            };
            let mut decoded = String::new();
            let kind = raw.decode_scalar(&mut decoded, self.errors);
            let written = raw.as_str();
            let one_line = match kind {
                _ if written.contains('\n') => None,
                ScalarKind::String => Some(decoded),
                _ => Some(written.to_string()),
            };
            (event.span().end(), one_line)
        } else if let Some(event) = self.next_if(&open) {
            (self.close(event.span().end()), None)
        } else {
            (after, None)
        }
    }

    /// The end of the array or inline table just opened, its nested values
    /// counted in a loop rather than recursed into.
    fn close(&mut self, mut end: usize) -> usize {
        let mut depth = 1usize;
        while let Some(event) = self.next() {
            end = event.span().end();
            match event.kind() {
                EventKind::ArrayOpen | EventKind::InlineTableOpen => depth += 1,
                EventKind::ArrayClose | EventKind::InlineTableClose => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                EventKind::SimpleKey => {
                    self.key(event);
                }
                EventKind::Scalar => self.scalar(event),
                _ => {}
            }
        }
        end
    }

    /// A scalar no key's value is read from, decoded for its refusals alone.
    fn scalar(&mut self, event: Event) {
        if let Some(raw) = self.source.get(event) {
            let _ = raw.decode_scalar(&mut (), self.errors);
        }
    }
}

/// What a key's allowed contributions have made it so far
/// (§FS-fields.front-matter.keys).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    /// `k = …`.
    Value,
    /// A table its dotted keys define.
    Dotted,
    /// A table its own header defines.
    Table,
    /// A table only a header below it has opened.
    Implicit,
    /// An array of tables.
    Array,
}

impl State {
    fn of(form: Form) -> Self {
        match form {
            Form::Value => State::Value,
            Form::Dotted => State::Dotted,
            Form::Table => State::Table,
            Form::SubTable | Form::SubArray => State::Implicit,
            Form::Array => State::Array,
        }
    }

    /// §FS-fields.front-matter.keys: what a later contribution leaves the key,
    /// `None` where TOML refuses it because of what the key already is.
    fn then(self, form: Form) -> Option<Self> {
        let below = matches!(form, Form::SubTable | Form::SubArray);
        match (self, form) {
            (State::Dotted, Form::Dotted) => Some(self),
            (State::Implicit, Form::Table) => Some(State::Table),
            (State::Array, Form::Array) => Some(self),
            (State::Dotted | State::Table | State::Implicit | State::Array, _) if below => {
                Some(self)
            }
            _ => None,
        }
    }
}

/// §FS-fields.front-matter.keys, §FS-fields.front-matter.coordinates: each key at
/// its first contribution, its span running to the last contribution of its
/// first run, and each refused contribution as a site of its own. A refused one
/// ends the first run, so no slice holds another site of its key.
fn fold(contributions: Vec<Contribution>) -> Vec<TomlKey> {
    let mut keys: Vec<(TomlKey, State)> = Vec::new();
    let mut refused: Vec<TomlKey> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    // The key whose first run no other contribution has interrupted yet.
    let mut open: Option<usize> = None;
    for contribution in contributions {
        let Some(&at) = index.get(&contribution.name) else {
            index.insert(contribution.name.clone(), keys.len());
            open = Some(keys.len());
            let state = State::of(contribution.form);
            keys.push((site(contribution), state));
            continue;
        };
        let (key, state) = &mut keys[at];
        match state.then(contribution.form) {
            Some(next) => {
                *state = next;
                if open == Some(at) {
                    key.span.end = contribution.span.end;
                } else {
                    open = None;
                }
            }
            None => {
                open = None;
                refused.push(site(contribution));
            }
        }
    }
    let mut sites: Vec<TomlKey> = keys.into_iter().map(|(key, _)| key).collect();
    sites.extend(refused);
    sites.sort_by_key(|key| key.span.start);
    sites
}

fn site(contribution: Contribution) -> TomlKey {
    TomlKey {
        name: contribution.name,
        span: contribution.span,
        one_line: contribution.one_line,
    }
}
