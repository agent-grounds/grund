//! What a fetched declaration owns in a `file` home, its lead included, and whether the
//! home already holds the output there (§FS-fetch.6.1); `fetch_write.rs` writes what it
//! decides.

use super::fetch_write::SnapshotDeclaration;

/// §FS-fetch.6.1: blank by the heading reader's own whitespace test, `trim_start`, so a
/// U+00A0 the reader skips ahead of a heading is blank here as well.
pub(super) fn is_blank(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok_and(|text| text.trim_start().is_empty())
}

/// §FS-fetch.6.1: the end of the line holding the last non-blank character of `text`, its break
/// included; what follows is `text`'s trailing blank lines.
fn content_end(text: &str) -> usize {
    let end = text.trim_end().len();
    let rest = &text[end..];
    match rest.find(['\n', '\r']) {
        Some(at) if rest[at..].starts_with("\r\n") => end + at + 2,
        Some(at) => end + at + 1,
        None => text.len(),
    }
}

/// §FS-fetch.6.1: accepted output as the home reads it, verbatim: the lead ahead of its
/// heading, the declaration from the heading on, and the trailing blank lines that end it.
pub(super) struct Output<'a> {
    bytes: &'a [u8],
    lead: usize,
    content: usize,
}

impl<'a> Output<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        let (lead, content) = std::str::from_utf8(bytes).map_or((0, bytes.len()), |text| {
            (text.len() - text.trim_start().len(), content_end(text))
        });
        Self {
            bytes,
            lead,
            content,
        }
    }

    fn lead(&self) -> &'a [u8] {
        &self.bytes[..self.lead]
    }

    fn declaration(&self) -> &'a [u8] {
        &self.bytes[self.lead..]
    }

    fn trailing(&self) -> &'a [u8] {
        &self.bytes[self.content.max(self.lead)..]
    }
}

/// §FS-fetch.6.1: the bytes of `home` the fetched declaration owns, the output's lead
/// included, and whether they already hold the output, followed by nothing but blank
/// lines up to the next declaration or the end of the file.
pub(super) struct Owned {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) holds: bool,
}

pub(super) fn owned(home: &[u8], existing: &SnapshotDeclaration, output: &Output<'_>) -> Owned {
    let lead = output.lead();
    let starts_line = |at: usize| at == 0 || matches!(home[at - 1], b'\n' | b'\r');
    let lead_start = existing
        .start
        .checked_sub(lead.len())
        .filter(|&at| lead.is_empty() || (home[at..existing.start] == *lead && starts_line(at)));
    let declaration = output.declaration();
    let after = existing.start + declaration.len();
    let holds = lead_start.is_some()
        && home[existing.start..existing.next].starts_with(declaration)
        && is_blank(&home[after..existing.next])
        && (after == existing.next
            || matches!(declaration.last(), Some(b'\n' | b'\r'))
            || matches!(home[after], b'\n' | b'\r'));
    // A trailing blank line the output prints is its own too, so a changed refresh keeps one.
    let trailing = output.trailing();
    let content = std::str::from_utf8(&home[existing.start..existing.next])
        .map_or(existing.end, |text| existing.start + content_end(text));
    let end = if !trailing.is_empty() && home[content..existing.next].starts_with(trailing) {
        existing.end.max(content + trailing.len())
    } else {
        existing.end
    };
    Owned {
        start: lead_start.unwrap_or(existing.start),
        end,
        holds,
    }
}
