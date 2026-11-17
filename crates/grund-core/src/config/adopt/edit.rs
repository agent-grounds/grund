//! §AR-config.10.2: the candidate is the file's own text with three edits, `languages`, the lines
//! of adopted entries and the tables of definitions nothing names any more
//! (§FS-config-adopt-builtins.4). Every line span is one the version-2 reader recorded in the load
//! that built the model, so the edit parses no key itself; the one rule it applies to a line is
//! the reader's own comment rule, to find the comments it removes (§FS-config-adopt-builtins.7).

use std::collections::{BTreeMap, BTreeSet};

use super::plan::{Decision, Plan};
use super::record::AdoptComment;
use crate::config::sources::SourceModel;
use crate::config::v1::strip_comment;
use crate::config::v2::Spans;

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
    let mut languages_line = None;
    if languages != listed
        && let Some(span) = key_span(spans, SOURCES, "languages")
    {
        languages_line = Some((span.first, span.last));
        // §FS-config-adopt-builtins.4.1: the comment after the value stays on the one line.
        for line in span.first..span.last {
            removed.insert(line, format!("after [{SOURCES}] languages"));
        }
        if span.first < span.last {
            removed.insert(span.last, String::new());
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
        match languages_line {
            Some((first, last)) if line == first => {
                candidate.push_str(&languages_value(raw, lines[last - 1], &languages));
                continue;
            }
            _ => {}
        }
        if !removed.contains_key(&line) {
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
    let mut out: Vec<String> = Vec::new();
    for name in listed {
        let entries: Vec<_> = plan
            .entries
            .iter()
            .filter(|entry| entry.definition.name == *name)
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
        if (keeps || rows.is_empty()) && !out.contains(name) {
            out.push(name.clone());
        }
        for row in rows {
            if !listed.iter().any(|known| known == row) && !out.iter().any(|known| known == row) {
                out.push(row.to_string());
            }
        }
    }
    out
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
/// in file order. A line with no place is one whose comment stays or that holds none: the last line
/// of `languages`, rewritten with its comment (§FS-config-adopt-builtins.4.1), or a blank.
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

/// §FS-config-adopt-builtins.4.1: the `languages` line, its key as written, `names` as a one-line
/// array, and whatever followed the value on its last line, a comment included.
fn languages_value(first: &str, last: &str, names: &[String]) -> String {
    let code = strip_comment(body(first));
    let eq = code.find('=').map_or(code.len(), |at| at + 1);
    let gap = code[eq..].len() - code[eq..].trim_start().len();
    let key = &code[..eq + gap];
    let array: Vec<String> = names.iter().map(|name| format!("\"{name}\"")).collect();
    let tail_line = body(last);
    let tail = &tail_line[strip_comment(tail_line).trim_end().len()..];
    let ending = &last[body(last).len()..];
    format!("{key}[{}]{tail}{ending}", array.join(", "))
}

/// The span the reader recorded for `key` in `table`.
fn key_span<'a>(
    spans: &'a Spans,
    table: &str,
    key: &str,
) -> Option<&'a crate::config::v2::KeySpan> {
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
