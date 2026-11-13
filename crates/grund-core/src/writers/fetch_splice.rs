//! Where fetch's output goes in its home (§FS-fetch.4): the range the scan records for the
//! requested declaration, the candidate built by putting the output there or at its place in ID
//! order, and the line a second splice would change (§FS-fetch.admission.5). Lines are counted as
//! the scan's file record counts them, so every number here is one the scan would print.

use crate::model::FileRecord;

/// The bytes and lines of a home the requested declaration owns, `first..=last` 1-based.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Owned {
    pub(super) first: usize,
    pub(super) last: usize,
    pub(super) start: usize,
    pub(super) end: usize,
}

impl Owned {
    /// §FS-fetch.4: from the declaration's heading line to the end of the last line of its body
    /// the scan records that holds text, before that line's ending, so the blank lines that part
    /// it from what follows are the home's and stay where they are.
    pub(super) fn of(record: &FileRecord, line: usize, body_end: usize) -> Self {
        let last = (line..=body_end.max(line))
            .rev()
            .find(|at| record.line(*at).is_some_and(|text| !text.trim().is_empty()))
            .unwrap_or(line);
        let start = record.line_start(line).unwrap_or(record.text().len());
        let end = record
            .line_start(last)
            .map_or(start, |at| at + record.line(last).map_or(0, str::len));
        Self {
            first: line,
            last,
            start,
            end,
        }
    }

    /// The range an insertion owns: nothing, at byte `at`, which is line `line`.
    pub(super) fn empty(at: usize, line: usize) -> Self {
        Self {
            first: line,
            last: line.saturating_sub(1),
            start: at,
            end: at,
        }
    }
}

/// §FS-fetch.4: the home with `output` in place of `owned`. One line ending after the range goes
/// with the output when the output brings its own, and as many blank lines as the output ends
/// with, so the separator before the next sibling is the home's however the output ends.
pub(super) fn replace(original: &str, owned: &Owned, output: &str) -> String {
    let mut rest = &original[owned.end..];
    if output.ends_with('\n') {
        rest = strip_line_ending(rest);
        for _ in 0..trailing_blank_lines(output) {
            match blank_line_len(rest) {
                Some(len) => rest = &rest[len..],
                None => break,
            }
        }
    }
    let mut candidate = String::with_capacity(original.len() + output.len());
    candidate.push_str(&original[..owned.start]);
    candidate.push_str(output);
    candidate.push_str(rest);
    candidate
}

/// §FS-fetch.4: the home with `output` inserted at byte `at`, a line start, parted from what
/// comes before and after it by one blank line in the home's own line ending. Also the byte the
/// output starts at in the candidate.
pub(super) fn insert(original: &str, at: usize, output: &str) -> (String, usize) {
    let ending = if original.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let (before, after) = original.split_at(at);
    let mut candidate = String::with_capacity(original.len() + output.len() + 4);
    candidate.push_str(before);
    if !candidate.is_empty() && !candidate.ends_with("\n\n") && !candidate.ends_with("\r\n\r\n") {
        candidate.push_str(ending);
    }
    let start = candidate.len();
    candidate.push_str(output);
    if !after.is_empty()
        && !after.starts_with('\n')
        && !after.starts_with("\r\n")
        && !candidate.ends_with("\n\n")
        && !candidate.ends_with("\r\n\r\n")
    {
        candidate.push_str(ending);
    }
    candidate.push_str(after);
    (candidate, start)
}

/// The 1-based line, as `str::lines` counts them, of the first byte where `a` and `b` differ.
pub(super) fn first_differing_line(a: &str, b: &str) -> usize {
    let at = a
        .bytes()
        .zip(b.bytes())
        .position(|(x, y)| x != y)
        .unwrap_or(a.len().min(b.len()));
    a[..at].matches('\n').count() + 1
}

fn strip_line_ending(text: &str) -> &str {
    text.strip_prefix("\r\n")
        .or_else(|| text.strip_prefix('\n'))
        .unwrap_or(text)
}

/// The length of the blank line `text` starts with, its ending included.
fn blank_line_len(text: &str) -> Option<usize> {
    let end = text.find('\n')?;
    text[..end].trim().is_empty().then_some(end + 1)
}

/// How many blank lines follow the line that holds `output`'s last text.
fn trailing_blank_lines(output: &str) -> usize {
    let Some(last) = output.rfind(|c: char| !c.is_whitespace()) else {
        return 0;
    };
    match output[last..].find('\n') {
        Some(newline) => output[last + newline + 1..].matches('\n').count(),
        None => 0,
    }
}
