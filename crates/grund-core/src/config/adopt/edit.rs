//! §AR-config.10.2: the candidate is the file's own text with three edits, `languages`, the lines
//! of adopted entries and the tables of definitions nothing names any more
//! (§FS-config-adopt-builtins.4). Every line span is one the version-2 reader recorded in the load
//! that built the model, so the edit parses no key itself. The rules it applies to a line are the
//! reader's own: its comment rule, to find the comments it removes (§FS-config-adopt-builtins.7),
//! and, on a `languages` value written across lines, where a string ends (§FS-config.3.11).

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::Path;

use super::plan::{Decision, Plan};
use super::record::AdoptComment;
use crate::config::sources::SourceModel;
use crate::config::v1::{parse_string, string_close, strip_comment};
use crate::config::v2::{KeySpan, Spans};

const SOURCES: &str = "schema.sources";
const EXTENSIONS: &str = "schema.sources.extensions";
const DEFINITIONS: &str = "schema.sources.definitions.";

/// The candidate and what writing it costs.
pub(crate) struct Edited {
    pub(crate) candidate: String,
    /// The candidate's `languages`, in order (§FS-config-adopt-builtins.4.1).
    pub(crate) languages: Vec<String>,
    /// The comments on lines the edit removes, in file order (§FS-config-adopt-builtins.7).
    pub(crate) comments: Vec<AdoptComment>,
}

/// §FS-config-adopt-builtins.4: `text` with `plan`'s three edits, at the places `spans` recorded.
pub(crate) fn edit(text: &str, spans: &Spans, model: &SourceModel, plan: &Plan) -> Edited {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let listed: Vec<String> = model.admitted.iter().map(|l| l.name.clone()).collect();
    let languages = renamed(&listed, plan);
    // Each removed line, with the place a comment on it is listed under.
    let mut removed: BTreeMap<usize, String> = BTreeMap::new();
    // Each line written anew, the text that takes its place.
    let mut rewritten: BTreeMap<usize, String> = BTreeMap::new();
    if languages != listed
        && let Some(span) = key_span(spans, SOURCES, "languages")
    {
        // §FS-config-adopt-builtins.4.1: a value on one line is written back on one line.
        if span.first == span.last {
            let raw = lines[span.first - 1];
            rewritten.insert(span.first, languages_value(raw, &languages));
        } else {
            let renaming = Renaming {
                listed: &listed,
                plan,
            };
            rename_across_lines(&lines, span, &renaming, &mut rewritten, &mut removed);
        }
    }
    let tables = removed_tables(model, plan);
    for table in &tables {
        remove_table(&lines, spans, table, &mut removed);
    }
    // §FS-config-adopt-builtins.4.2: each adopted suffix's entry goes with its line.
    let adopted: BTreeSet<&str> = adopted_suffixes(plan).collect();
    for span in spans.keys.iter().filter(|span| span.table == EXTENSIONS) {
        let suffix = unquoted(&span.key);
        if !adopted.contains(suffix) {
            continue;
        }
        let above = format!("above [{EXTENSIONS}] {suffix}");
        for line in attached(&lines, span.first) {
            removed.insert(line, above.clone());
        }
        for line in span.first..=span.last {
            removed.insert(line, format!("after [{EXTENSIONS}] {suffix}"));
        }
    }
    let comments = listed_comments(&lines, &removed);
    let mut candidate = String::with_capacity(text.len());
    for (at, raw) in lines.iter().enumerate() {
        let line = at + 1;
        if let Some(text) = rewritten.get(&line) {
            candidate.push_str(text);
        } else if !removed.contains_key(&line) {
            candidate.push_str(raw);
        }
    }
    Edited {
        candidate,
        languages,
        comments,
    }
}

/// §FS-config-adopt-builtins.4.1: `listed` with each fully adopted definition replaced by the rows
/// its suffixes were adopted into, and each partly kept one followed by them, never twice.
pub(crate) fn renamed(listed: &[String], plan: &Plan) -> Vec<String> {
    let renaming = Renaming { listed, plan };
    let mut out: Vec<String> = Vec::new();
    for name in listed {
        let becomes = renaming.rename(name, &out);
        out.extend(becomes);
    }
    out
}

/// §FS-config-adopt-builtins.4.1: what each name of `languages` becomes, asked in the list's
/// order, so the whole list and a value rewritten name by name agree.
struct Renaming<'a> {
    listed: &'a [String],
    plan: &'a Plan,
}

impl Renaming<'_> {
    /// The names `name` becomes after `before`, the list so far: a fully adopted definition the
    /// rows its suffixes were adopted into, a partly kept one itself and then those rows, any
    /// other name itself, and nothing that is listed already or earlier by the same edit.
    fn rename(&self, name: &str, before: &[String]) -> Vec<String> {
        let entries: Vec<_> = self
            .plan
            .entries
            .iter()
            .filter(|entry| entry.definition.name == name)
            .collect();
        let mut rows: Vec<&str> = Vec::new();
        for entry in &entries {
            if let Decision::Adopted(row) = &entry.decision
                && !rows.contains(&row.name.as_str())
            {
                rows.push(&row.name);
            }
        }
        let keeps = entries
            .iter()
            .any(|entry| matches!(entry.decision, Decision::Kept(_)));
        let mut out: Vec<String> = Vec::new();
        if (keeps || rows.is_empty()) && !before.iter().any(|known| known == name) {
            out.push(name.to_string());
        }
        for row in rows {
            let listed = self.listed.iter().chain(before).any(|known| known == row);
            if !listed && !out.iter().any(|known| known == row) {
                out.push(row.to_string());
            }
        }
        out
    }
}

/// §FS-config-adopt-builtins.4.1: a `languages` value written across lines, at `span`, keeps its
/// lines, and with them every comment: each name is rewritten where it stands, and a line goes
/// only when the list drops every name on it and nothing else stands on it, its comment then
/// listed (§FS-config-adopt-builtins.7).
fn rename_across_lines(
    lines: &[&str],
    span: &KeySpan,
    renaming: &Renaming<'_>,
    rewritten: &mut BTreeMap<usize, String>,
    removed: &mut BTreeMap<usize, String>,
) {
    let mut out: Vec<String> = Vec::new();
    for line in span.first..=span.last {
        let raw = lines[line - 1];
        match renamed_line(raw, line == span.first, renaming, &mut out) {
            LineEdit::Kept => {}
            LineEdit::Rewritten(text) => {
                rewritten.insert(line, text);
            }
            LineEdit::Removed => {
                removed.insert(line, format!("after [{SOURCES}] languages"));
            }
        }
    }
}

/// What the edit does to one line of a `languages` value written across lines.
enum LineEdit {
    Kept,
    Rewritten(String),
    Removed,
}

/// §FS-config-adopt-builtins.4.1: one line of a `languages` value written across lines, each name
/// on it renamed after `out`, the list so far, which it extends. The text before the first name
/// and after the last stays, the comment included; names that stay keep their bytes, and the
/// names on a line one of them changes on are joined by `, `.
fn renamed_line(
    raw: &str,
    key_line: bool,
    renaming: &Renaming<'_>,
    out: &mut Vec<String>,
) -> LineEdit {
    let text = body(raw);
    let code = strip_comment(text);
    // The key's own line holds the key, quoted or bare, before its `=`.
    let from = match key_line {
        true => code.find('=').map_or(code.len(), |at| at + 1),
        false => 0,
    };
    let names = written_strings(code, from);
    let (Some(first), Some(last)) = (names.first(), names.last()) else {
        return LineEdit::Kept;
    };
    let (lead, trail) = (&code[..first.start], &code[last.end..]);
    let mut changed = false;
    let mut written: Vec<String> = Vec::new();
    for range in &names {
        let as_written = &code[range.clone()];
        // A loaded file's names all read; one that does not is left as it is.
        let Ok(name) = parse_string(Path::new(""), 0, as_written) else {
            written.push(as_written.to_string());
            continue;
        };
        let becomes = renaming.rename(&name, out);
        out.extend(becomes.iter().cloned());
        if becomes.len() == 1 && becomes[0] == name {
            written.push(as_written.to_string());
            continue;
        }
        changed = true;
        written.extend(becomes.iter().map(|row| format!("\"{row}\"")));
    }
    if !changed {
        return LineEdit::Kept;
    }
    let after = &raw[code.len()..];
    if !written.is_empty() {
        return LineEdit::Rewritten(format!("{lead}{}{trail}{after}", written.join(", ")));
    }
    // No name is left on the line, so neither is the comma that followed the last.
    let trail = trail.trim_start().strip_prefix(',').unwrap_or(trail);
    if lead.trim().is_empty() && trail.trim().is_empty() {
        return LineEdit::Removed;
    }
    LineEdit::Rewritten(format!("{lead}{trail}{after}"))
}

/// Each basic string in `code` from byte `from`, opening quote to closing quote, where the one
/// close every reader shares says it ends (§FS-config.3.11). Outside a string, in an array, every
/// `"` opens one.
fn written_strings(code: &str, from: usize) -> Vec<Range<usize>> {
    let bytes = code.as_bytes();
    let mut strings = Vec::new();
    let mut at = from;
    while at < bytes.len() {
        if bytes[at] == b'"' {
            let Some(close) = string_close(bytes, at + 1) else {
                break;
            };
            strings.push(at..close + 1);
            at = close + 1;
        } else {
            at += 1;
        }
    }
    strings
}

/// Each adopted suffix, in entry order.
fn adopted_suffixes(plan: &Plan) -> impl Iterator<Item = &str> {
    plan.entries
        .iter()
        .filter(|entry| matches!(entry.decision, Decision::Adopted(_)))
        .map(|entry| entry.suffix.as_str())
}

/// §FS-config-adopt-builtins.4.2: the tables the edit removes, by header: the extensions table
/// when no entry is left in it, and each definition every entry naming it was adopted from.
fn removed_tables(model: &SourceModel, plan: &Plan) -> Vec<String> {
    let adopted: BTreeSet<&str> = adopted_suffixes(plan).collect();
    let mut tables = Vec::new();
    if !adopted.is_empty()
        && model
            .extensions
            .iter()
            .all(|(suffix, _)| adopted.contains(suffix.as_str()))
    {
        tables.push(EXTENSIONS.to_string());
    }
    for definition in &model.definitions {
        let mut naming = model
            .extensions
            .iter()
            .filter(|(_, name)| *name == definition.name)
            .peekable();
        if naming.peek().is_some() && naming.all(|(suffix, _)| adopted.contains(suffix.as_str())) {
            tables.push(format!("{DEFINITIONS}{}", definition.name));
        }
    }
    tables
}

/// §FS-config-adopt-builtins.4.2: `table` goes with its header, every line up to the next header
/// and the blank lines between. A comment run directly above the next header is that header's,
/// so it stays; one directly above this header is this one's, so it goes. A table that runs to
/// the end of the file takes the blank lines above it too, so the file does not end in them.
fn remove_table(lines: &[&str], spans: &Spans, table: &str, removed: &mut BTreeMap<usize, String>) {
    let Some(at) = spans
        .headers
        .iter()
        .position(|(name, _)| unquoted_header(name) == table)
    else {
        return;
    };
    let header = spans.headers[at].1;
    let next = spans.headers.get(at + 1).map(|(_, line)| *line);
    let end = match next {
        Some(next) => attached(lines, next).next().unwrap_or(next) - 1,
        None => lines.len(),
    };
    for line in attached(lines, header) {
        removed.insert(line, format!("above [{table}]"));
    }
    removed.insert(header, format!("after [{table}]"));
    for line in header + 1..=end {
        removed.insert(line, format!("inside [{table}]"));
    }
    if next.is_none() {
        let mut line = attached(lines, header).next().unwrap_or(header);
        while line > 1 && blank(lines[line - 2]) {
            line -= 1;
            removed.insert(line, String::new());
        }
    }
}

/// The attached comment run directly above `line` (§FS-config-migrate.terms), first to last.
fn attached<'a>(lines: &'a [&str], line: usize) -> impl Iterator<Item = usize> + 'a {
    let mut first = line;
    while first > 1 && comment_only(lines[first - 2]) {
        first -= 1;
    }
    first..line
}

/// §FS-config-adopt-builtins.7: each run of removed lines that carry a comment and share a place,
/// in file order. A line with no place is a blank, which holds none.
fn listed_comments(lines: &[&str], removed: &BTreeMap<usize, String>) -> Vec<AdoptComment> {
    let mut comments: Vec<AdoptComment> = Vec::new();
    for (line, place) in removed {
        if place.is_empty() || !has_comment(lines[line - 1]) {
            continue;
        }
        match comments.last_mut() {
            Some(run) if run.last + 1 == *line && run.place == *place => run.last = *line,
            _ => comments.push(AdoptComment {
                first: *line,
                last: *line,
                place: place.clone(),
            }),
        }
    }
    comments
}

/// §FS-config-adopt-builtins.4.1: the one-line `languages` line, its key as written, `names` as a
/// one-line array, and whatever followed the value, a comment included.
fn languages_value(raw: &str, names: &[String]) -> String {
    let line = body(raw);
    let code = strip_comment(line);
    let eq = code.find('=').map_or(code.len(), |at| at + 1);
    let gap = code[eq..].len() - code[eq..].trim_start().len();
    let key = &code[..eq + gap];
    let array: Vec<String> = names.iter().map(|name| format!("\"{name}\"")).collect();
    let tail = &line[code.trim_end().len()..];
    let ending = &raw[line.len()..];
    format!("{key}[{}]{tail}{ending}", array.join(", "))
}

/// The span the reader recorded for `key` in `table`.
fn key_span<'a>(spans: &'a Spans, table: &str, key: &str) -> Option<&'a KeySpan> {
    spans
        .keys
        .iter()
        .find(|span| span.table == table && unquoted(&span.key) == key)
}

/// A key as the table reads it: written bare or quoted.
fn unquoted(key: &str) -> &str {
    key.strip_prefix('"')
        .and_then(|key| key.strip_suffix('"'))
        .unwrap_or(key)
}

/// A definition's header with its name unquoted.
fn unquoted_header(name: &str) -> String {
    match name.strip_prefix(DEFINITIONS) {
        Some(definition) => format!("{DEFINITIONS}{}", unquoted(definition)),
        None => name.to_string(),
    }
}

/// A line without its ending.
fn body(raw: &str) -> &str {
    raw.trim_end_matches(['\n', '\r'])
}

fn blank(raw: &str) -> bool {
    body(raw).trim().is_empty()
}

/// A line that holds a comment and nothing else.
fn comment_only(raw: &str) -> bool {
    body(raw).trim_start().starts_with('#')
}

/// A line that carries a comment, whole or trailing, by the reader's own comment rule.
fn has_comment(raw: &str) -> bool {
    let line = body(raw);
    strip_comment(line).len() < line.len()
}
