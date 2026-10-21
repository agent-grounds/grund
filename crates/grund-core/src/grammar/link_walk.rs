//! The byte walk under the document reading of §FS-citations.wrap: one Markdown
//! line read left to right for the brackets, the destinations, the autolinks and
//! the raw HTML tags every form of §FS-citations.wrap.enclosing.scope is spelled
//! with. It decides no reading itself. `markdown_links.rs` runs it twice, once
//! over the document to settle what each `[` turns out to be and once over the
//! line a caller holds to say where those spans fall in its bytes, so the two
//! runs cannot find different brackets (§AR-system.principle).
//!
//! Not a Markdown parser (§FS-citations.wrap.enclosing.scope): it reads only the
//! extents §FS-citations.wrap.enclosing.shape and §FS-citations.zones name, and
//! reuses the one code-span and destination extent every other pass reads.

use super::never_rewrite::{inline_code_span_end, is_escaped, link_destination_close};

/// A `[` still open: its document-wide `id`, where it stands on the line being
/// walked (`None` when an earlier line of the paragraph opened it), whether it
/// opens an image's description, and whether it may open a definition's label:
/// on a line a definition may start on, at its start behind at most three spaces
/// or the line's container prefixes, and not a footnote's `[^`
/// (§FS-citations.wrap.enclosing.shape).
#[derive(Clone, Copy, Debug)]
pub(super) struct Open {
    pub(super) id: Option<usize>,
    pub(super) at: Option<usize>,
    pub(super) image: bool,
    pub(super) may_define: bool,
}

/// What follows the `]` that closes an [`Open`], which decides its form
/// (§FS-citations.wrap.enclosing.shape).
#[derive(Clone, Copy, Debug)]
pub(super) enum After {
    /// `(` and a destination that closes on the line, `url_start..close`.
    Inline { url_start: usize, close: usize },
    /// `(` and a destination that never closes: the rest of the line is the
    /// destination and no link is made (§FS-citations.zones.link-destination).
    Unclosed,
    /// `[` and a key closed on the line, `start..end`; empty for `[]`.
    Key { start: usize, end: usize },
    /// `:` after a label that opens its line, and a destination and at most a
    /// title on the line; the destination starts at `dest_start` and runs, its
    /// title with it, to the end of the line.
    Definition { dest_start: usize },
    /// Anything else: a shortcut reference or plain brackets.
    Other,
}

/// What the walk reports, in line order.
pub(super) trait Sink {
    /// A `[` opened at `at`; the sink names it, or cannot.
    fn opened(&mut self, at: usize) -> Option<usize>;
    /// `open` closed by the `]` at `at`, followed by `after`.
    fn closed(&mut self, open: Open, at: usize, after: After);
    /// Bytes no pass rewrites: an autolink, or a raw HTML tag
    /// (§FS-citations.zones.link-destination, §FS-citations.zones.authored-markup).
    fn zone(&mut self, start: usize, end: usize);
    /// The content of a raw HTML `<a>` element on this line.
    fn anchor_text(&mut self, start: usize, end: usize);
    /// A backtick run with no closing run left the rest of the line code while
    /// `stack` was open (§FS-citations.zones.inline-code).
    fn unclosed_tick(&mut self, stack: &[Open]);
}

/// The state a line begins in: the `[`s its paragraph still holds open, and
/// whether a raw HTML `<a>` element is open.
#[derive(Clone, Debug, Default)]
pub(super) struct WalkState {
    pub(super) stack: Vec<Open>,
    pub(super) anchor: bool,
}

impl WalkState {
    /// The state as the next line sees it: every opener is carried, not placed.
    pub(super) fn carried(&self) -> Self {
        Self {
            stack: self
                .stack
                .iter()
                .map(|open| Open { at: None, ..*open })
                .collect(),
            anchor: self.anchor,
        }
    }
}

/// Walk one line from `state`, reporting to `sink` and leaving in `state` what
/// is still open at its end; `defines` where a definition may start on the line.
/// Brackets balance and code spans and destinations are passed over whole, as
/// §FS-citations.wrap.enclosing.shape reads them.
pub(super) fn walk_line(line: &str, defines: bool, state: &mut WalkState, sink: &mut impl Sink) {
    let bytes = line.as_bytes();
    let mut anchor_from = state.anchor.then_some(0);
    let mut i = 0;
    while i < bytes.len() {
        if is_escaped(bytes, i) {
            i += 1;
            continue;
        }
        match bytes[i] {
            b'`' => match inline_code_span_end(bytes, i) {
                Some(end) => {
                    i = end;
                    continue;
                }
                None => {
                    sink.unclosed_tick(&state.stack);
                    break;
                }
            },
            b'<' => {
                if let Some(end) = html_comment_end(bytes, i) {
                    i = end;
                    continue;
                }
                if let Some(end) = autolink_end(bytes, i) {
                    sink.zone(i, end);
                    i = end;
                    continue;
                }
                if let Some((end, anchor)) = html_tag(bytes, i) {
                    sink.zone(i, end);
                    match anchor {
                        Some(true) if anchor_from.is_none() => anchor_from = Some(end),
                        Some(false) => {
                            if let Some(start) = anchor_from.take() {
                                sink.anchor_text(start, i);
                            }
                        }
                        _ => {}
                    }
                    i = end;
                    continue;
                }
            }
            b'[' => {
                let image = i > 0 && bytes[i - 1] == b'!' && !is_escaped(bytes, i - 1);
                let may_define = defines
                    && !image
                    && behind_container_prefixes(&bytes[..i])
                    && bytes.get(i + 1) != Some(&b'^');
                state.stack.push(Open {
                    id: sink.opened(i),
                    at: Some(i),
                    image,
                    may_define,
                });
            }
            b']' => {
                let open = state.stack.pop();
                let after = after_close(bytes, i, open);
                if let Some(open) = open {
                    sink.closed(open, i, after);
                }
                i = match after {
                    After::Inline { close, .. } => close + 1,
                    After::Key { end, .. } => end + 1,
                    After::Unclosed | After::Definition { .. } => bytes.len(),
                    After::Other => i + 1,
                };
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    if let Some(start) = anchor_from {
        sink.anchor_text(start, bytes.len());
    }
    state.anchor = anchor_from.is_some();
}

/// §FS-citations.wrap.enclosing.shape: the form what follows the `]` at `close`
/// gives. A destination is read with the one extent of
/// §FS-citations.zones.link-destination whether or not a `[` opened the text,
/// as every other reading of a destination is; a key only after a text.
fn after_close(bytes: &[u8], close: usize, open: Option<Open>) -> After {
    match bytes.get(close + 1) {
        Some(b'(') => match link_destination_close(bytes, close + 2) {
            Some(end) => After::Inline {
                url_start: close + 2,
                close: end,
            },
            None => After::Unclosed,
        },
        Some(b'[') if open.is_some() => match key_close(bytes, close + 2) {
            Some(end) => After::Key {
                start: close + 2,
                end,
            },
            None => After::Other,
        },
        Some(b':') if open.is_some_and(|open| open.may_define) => {
            let mut dest_start = close + 2;
            while bytes
                .get(dest_start)
                .is_some_and(|b| *b == b' ' || *b == b'\t')
            {
                dest_start += 1;
            }
            if definition_tail(bytes, dest_start) {
                After::Definition { dest_start }
            } else {
                After::Other
            }
        }
        _ => After::Other,
    }
}

/// §FS-citations.wrap.enclosing.shape: whether `prefix`, the bytes before a `[`
/// on its line, is where a definition's label may start: at most three spaces,
/// after any block-quote `>` markers and list markers with their indent.
fn behind_container_prefixes(prefix: &[u8]) -> bool {
    let mut i = 0;
    loop {
        let indent = prefix[i..].iter().take_while(|b| **b == b' ').count();
        if i + indent == prefix.len() {
            return indent <= 3;
        }
        let marker = i + indent;
        let after = match prefix[marker] {
            b'>' if indent <= 3 => marker + 1,
            b'-' | b'*' | b'+' => marker + 1,
            b'0'..=b'9' => {
                let digits = prefix[marker..]
                    .iter()
                    .take_while(|b| b.is_ascii_digit())
                    .count();
                match prefix.get(marker + digits) {
                    Some(b'.' | b')') if digits <= 9 => marker + digits + 1,
                    _ => return false,
                }
            }
            _ => return false,
        };
        let block_quote = prefix[marker] == b'>';
        match prefix.get(after) {
            Some(b' ' | b'\t') => i = after + 1,
            None if block_quote => return true,
            _ if block_quote => i = after,
            _ => return false,
        }
    }
}

/// §FS-citations.wrap.enclosing.shape: whether the bytes from `start` to the end
/// of the line are a definition's destination and at most a title after it, set
/// off by whitespace and closed on the line. Anything else leaves the line prose,
/// and so does a destination the line does not hold.
fn definition_tail(bytes: &[u8], start: usize) -> bool {
    let Some(end) = definition_destination_end(bytes, start) else {
        return false;
    };
    let rest = &bytes[end..];
    let title = rest.trim_ascii_start();
    if title.is_empty() {
        return true;
    }
    let close = match title[0] {
        b'"' => b'"',
        b'\'' => b'\'',
        b'(' => b')',
        _ => return false,
    };
    if title.len() == rest.len() {
        return false;
    }
    let at = end + rest.len() - title.len();
    (at + 1..bytes.len())
        .filter(|&j| !is_escaped(bytes, j))
        .find(|&j| bytes[j] == close || (close == b')' && bytes[j] == b'('))
        .is_some_and(|j| bytes[j] == close && bytes[j + 1..].trim_ascii().is_empty())
}

/// Past a definition's destination opened at `start`: `<…>` closed on the line
/// with no `<` inside, or a run with no whitespace whose parentheses balance.
fn definition_destination_end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) == Some(&b'<') {
        return (start + 1..bytes.len())
            .filter(|&j| !is_escaped(bytes, j))
            .find(|&j| matches!(bytes[j], b'<' | b'>'))
            .filter(|&j| bytes[j] == b'>')
            .map(|j| j + 1);
    }
    let mut depth = 0usize;
    let mut j = start;
    while j < bytes.len() && !bytes[j].is_ascii_whitespace() {
        if !is_escaped(bytes, j) {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => depth = depth.checked_sub(1)?,
                _ => {}
            }
        }
        j += 1;
    }
    (j > start && depth == 0).then_some(j)
}

/// The unescaped `]` that closes a key opened just before `from`, on the line.
fn key_close(bytes: &[u8], from: usize) -> Option<usize> {
    (from..bytes.len()).find(|&j| bytes[j] == b']' && !is_escaped(bytes, j))
}

/// Past the `-->` of an HTML comment opened at `at`, or the line's end: a
/// comment opens no element (§FS-citations.wrap.enclosing.scope).
fn html_comment_end(bytes: &[u8], at: usize) -> Option<usize> {
    if !bytes[at..].starts_with(b"<!--") {
        return None;
    }
    let rest = &bytes[at + 4..];
    Some(
        rest.windows(3)
            .position(|window| window == b"-->")
            .map_or(bytes.len(), |found| at + 4 + found + 3),
    )
}

/// Past the `>` of an autolink opened at `at`: `<scheme:…>` or `<user@host>`,
/// with no space, `<` or `>` inside (§FS-citations.zones.link-destination).
fn autolink_end(bytes: &[u8], at: usize) -> Option<usize> {
    let body_end = (at + 1..bytes.len())
        .find(|&j| matches!(bytes[j], b'>' | b'<' | b' ' | b'\t'))
        .filter(|&j| bytes[j] == b'>')?;
    let body = &bytes[at + 1..body_end];
    let scheme_len = body
        .iter()
        .position(|b| *b == b':')
        .filter(|&len| (2..=32).contains(&len))
        .filter(|&len| {
            body[0].is_ascii_alphabetic()
                && body[1..len]
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'.' | b'-'))
        });
    let email = body
        .iter()
        .position(|b| *b == b'@')
        .is_some_and(|at_sign| at_sign > 0 && at_sign + 1 < body.len() && !body.contains(&b':'));
    (scheme_len.is_some() || email).then_some(body_end + 1)
}

/// A raw HTML tag opened at `at`: past its `>`, the first outside a quoted
/// attribute value on the line, and whether it opens (`Some(true)`) or closes
/// (`Some(false)`) an `<a>` element, its name matched without regard to case
/// (§FS-citations.wrap.enclosing.scope, §FS-citations.zones.authored-markup).
fn html_tag(bytes: &[u8], at: usize) -> Option<(usize, Option<bool>)> {
    let closing = bytes.get(at + 1) == Some(&b'/');
    let name_start = at + 1 + usize::from(closing);
    if !bytes.get(name_start)?.is_ascii_alphabetic() {
        return None;
    }
    let mut name_end = name_start;
    while bytes
        .get(name_end)
        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'-')
    {
        name_end += 1;
    }
    if !matches!(bytes.get(name_end), Some(b' ' | b'\t' | b'>' | b'/') | None) {
        return None;
    }
    let mut quote = None;
    let mut j = name_end;
    let end = loop {
        let byte = *bytes.get(j)?;
        match (quote, byte) {
            (None, b'"' | b'\'') => quote = Some(byte),
            (Some(open), _) if byte == open => quote = None,
            (None, b'>') => break j + 1,
            _ => {}
        }
        j += 1;
    };
    let anchor = if !bytes[name_start..name_end].eq_ignore_ascii_case(b"a") {
        None
    } else if closing {
        Some(false)
    } else {
        (bytes[end - 2] != b'/').then_some(true)
    };
    Some((end, anchor))
}
