//! The v2 line walk (§FS-config-v2.reader): one pass over the file, each header
//! opening the table `tables.rs` names and each key read by its concern's file
//! as it is met, so the first refusal is the first one in the file. What every
//! table shares is kept here: explicit headers and the line each was first
//! written at (§FS-config-v2.reader.2), each table's keys (§FS-config-v2.reader.1),
//! and the close that runs a table's cross-key checks once its keys are all read.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

use super::fields::FieldDraft;
use super::schema::RowDraft;
use super::sources::{self, SourcesDraft};
use super::strings::JoinedLines;
use super::tables::{self, Opened, Table};
use super::{defaults, fields, front_matter, measures, presentation, rules, schema};
use crate::config::fields::FRONT_MATTER;
use crate::config::kind::KindResolution;
use crate::config::project::{CommaWithoutElement, Project, StringOverBreak};
use crate::config::record::ConfigLocation;
use crate::config::v1::{bail_config, string_close, strip_comment};
use crate::grammar::is_escaped;

/// The reader's state: the project lowered so far, and what only the whole
/// file can judge, kept with the line it was written at (§FS-config-v2.reader.4).
pub(super) struct Reader<'a> {
    pub(super) path: &'a Path,
    pub(super) project: Project,
    /// The `[schema.kinds.<NAME>]` rows, in authored order.
    pub(super) rows: Vec<RowDraft>,
    /// The `[schema.kinds.<NAME>.fields.<field>]` tables, in header order
    /// (§FS-fields.declare.field).
    pub(super) fields: Vec<FieldDraft>,
    /// `[schema.sources]` and its tables as written (§FS-config-v2.schema.sources).
    pub(super) sources: SourcesDraft,
    /// The explicit `[rules.resolution]` entries.
    pub(super) resolution: Vec<(String, KindResolution, usize)>,
    /// The place each row ladder names, at its header.
    pub(super) row_ladders: Vec<(String, usize)>,
    /// The kind each `[presentation.kinds.<NAME>]` names, at its header.
    pub(super) presentation_kinds: Vec<(String, usize)>,
    /// The lines the value being read was written across (§FS-config-v2.reader.6.window).
    pub(super) value: JoinedLines,
    headers: BTreeMap<String, usize>,
    header: String,
    keys: BTreeMap<String, usize>,
    table: Table,
}

impl<'a> Reader<'a> {
    pub(super) fn new(path: &'a Path) -> Self {
        Self {
            path,
            project: defaults::default_project(),
            rows: Vec::new(),
            fields: Vec::new(),
            sources: SourcesDraft::default(),
            resolution: Vec::new(),
            row_ladders: Vec::new(),
            presentation_kinds: Vec::new(),
            value: JoinedLines::default(),
            headers: BTreeMap::new(),
            header: String::new(),
            keys: BTreeMap::new(),
            table: Table::Root,
        }
    }

    /// A located refusal (§FS-config.4.3).
    pub(super) fn fail<T>(&self, line: usize, message: String) -> Result<T> {
        bail_config(self.path, line, message)
    }

    pub(super) fn at(&self, line: usize) -> Option<ConfigLocation> {
        Some(ConfigLocation {
            path: self.path.to_path_buf(),
            line,
        })
    }

    /// §FS-config-v2.reader.2: open the table an explicit header names.
    fn open(&mut self, name: String, line: usize) -> Result<()> {
        if let Some(first) = self.headers.get(&name) {
            return self.fail(
                line,
                format!("repeated table `[{name}]` (first written at line {first})"),
            );
        }
        self.headers.insert(name.clone(), line);
        let table = match tables::open(&name) {
            Opened::Refused(message) => return self.fail(line, message),
            Opened::Row(row) => {
                self.rows.push(RowDraft::new(row, line));
                Table::Row(self.rows.len() - 1)
            }
            Opened::Field(row, field) => {
                Table::Field(fields::open_field(self, &row, &field, line)?)
            }
            Opened::FrontMatter(row) => {
                Table::FrontMatter(fields::open_field(self, &row, FRONT_MATTER, line)?)
            }
            Opened::Key(row, key) => {
                let (field, key) = front_matter::open_key_table(self, &row, &key, line)?;
                Table::FrontMatterKey(field, key)
            }
            Opened::Definition(name) => {
                Table::Definition(sources::open_definition(self, &name, line)?)
            }
            Opened::Table(table) => table,
        };
        let citations = &mut self.project.rules.citations;
        match &table {
            Table::Citations => citations.declared = true,
            Table::CitationKind(kind, _) => {
                citations.declared = true;
                citations.per_kind.entry(kind.clone()).or_default();
                citations.written.0.push(kind.clone());
            }
            Table::Ladder(Some(place), _) => self.row_ladders.push((place.clone(), line)),
            Table::PresentationKind(kind) => self.presentation_kinds.push((kind.clone(), line)),
            Table::Workspace => {
                self.project.workspace.declared = true;
                self.project.workspace.section_source = self.at(line);
            }
            _ => {}
        }
        self.table = table;
        self.header = name;
        self.keys.clear();
        Ok(())
    }

    /// §FS-config-v2.reader.1: one key, refused when repeated or when its
    /// table does not admit it, which is when the table's keys do not list it
    /// and the refusal names the ones they do (§FS-config-v2.reader.1.keys).
    fn entry(&mut self, key: &str, value: &str, line: usize) -> Result<()> {
        let at = tables::label(&self.header);
        if let Some(first) = self.keys.get(key) {
            return self.fail(
                line,
                format!("duplicate key `{key}` {at} (first written at line {first})"),
            );
        }
        self.keys.insert(key.to_string(), line);
        let mut table = std::mem::replace(&mut self.table, Table::Root);
        let keys = table.keys();
        // §FS-config-v2.schema.sources.extensions: a suffix key may be quoted, so a
        // malformed one reaches the suffix check rather than the bare-key one.
        let quoted = key.len() >= 2 && key.starts_with('"') && key.ends_with('"');
        let admitted = if matches!(table, Table::Extensions) && quoted {
            let key = self.string(line, key)?;
            self.read_key(&mut table, &key, value, line)?
        } else {
            bare(key) && keys.admit(key) && self.read_key(&mut table, key, value, line)?
        };
        self.table = table;
        if !admitted {
            let hint = keys.hint();
            return self.fail(line, format!("unknown key `{key}` {at}{hint}"));
        }
        Ok(())
    }

    /// Reached only for a key `table.keys()` admits, so no arm's `Ok(false)` is a second gate.
    fn read_key(&mut self, table: &mut Table, key: &str, value: &str, line: usize) -> Result<bool> {
        match table {
            Table::Root => self.envelope_key(key, value, line),
            Table::Parent => Ok(false),
            Table::Schema => schema::schema_key(self, key, value, line),
            Table::Sources => sources::sources_key(self, key, value, line),
            Table::Extensions => sources::extension_key(self, key, value, line),
            Table::Definition(index) => sources::definition_key(self, *index, key, value, line),
            Table::Row(index) => schema::row_key(self, *index, key, value, line),
            Table::Field(index) => fields::field_key(self, *index, key, value, line),
            Table::FrontMatter(index) => {
                front_matter::front_matter_key(self, *index, key, value, line)
            }
            Table::FrontMatterKey(field, index) => {
                front_matter::key_key(self, *field, *index, key, value, line)
            }
            Table::Measure(_, written) => measures::measure_key(self, written, key, value, line),
            Table::Text => measures::text_key(self, key, value, line),
            Table::Layout(first) => measures::layout_key(self, first, key, value, line),
            Table::Citations => rules::citations_key(self, key, value, line),
            Table::CitationKind(kind, lists) => {
                rules::citation_kind_key(self, kind, lists, key, value, line)
            }
            Table::Ladder(_, rungs) => rules::ladder_key(self, rungs, key, value, line),
            Table::Resolution => rules::resolution_key(self, key, value, line),
            Table::Presentation => presentation::presentation_key(self, key, value, line),
            Table::PresentationKind(kind) => presentation::kind_key(self, kind, key, value, line),
            Table::PresentationFmt => presentation::fmt_key(self, key, value, line),
            Table::Workspace => presentation::workspace_key(self, key, value, line),
        }
    }

    /// The envelope (§FS-config.concerns): the version and the project's name.
    fn envelope_key(&mut self, key: &str, value: &str, line: usize) -> Result<bool> {
        match key {
            // `selects` read this key's value.
            "grund_config_version" => {}
            "project_name" => {
                self.project.name = Some(self.string(line, value)?);
                self.project.name_source = self.at(line);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// The checks a table can only make once all its keys are read.
    fn close(&mut self) -> Result<()> {
        match std::mem::replace(&mut self.table, Table::Root) {
            Table::Measure(measure, written) => measures::close_measure(self, measure, &written),
            Table::CitationKind(kind, lists) => rules::close_citation_kind(self, &kind, &lists),
            Table::Ladder(place, rungs) => rules::close_ladder(self, place, rungs),
            Table::Row(index) => schema::close_row(self, index),
            _ => Ok(()),
        }
    }
}

/// §FS-config-v2.reader: walk `text` into `reader`, header by header.
pub(super) fn walk(text: &str, reader: &mut Reader) -> Result<()> {
    let mut lines = text.lines().enumerate();
    while let Some((idx, raw)) = lines.next() {
        let line_no = idx + 1;
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            reader.close()?;
            let name = header_name(reader, line_no, line)?;
            reader.open(name, line_no)?;
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return reader.fail(line_no, "expected `key = value`".to_string());
        };
        // §FS-config-v2.reader.5: an array written across lines is one value, read
        // at the line of its key.
        let joined = joined_value(value, line_no, &mut lines);
        let Some(value) = joined.value else {
            let open = joined.open.map(OpenLine::parenthetical).unwrap_or_default();
            return reader.fail(line_no, format!("unclosed array{open}"));
        };
        // §FS-config-v2.reader.5.strings.window: read as before, and warned for.
        if let Some(open) = joined.open
            && let Some(at) = reader.at(line_no)
        {
            reader
                .project
                .strings_over_breaks
                .push(StringOverBreak { at, open });
        }
        reader.value = JoinedLines::new(line_no, joined.starts);
        reader.entry(key.trim(), &value, line_no)?;
        // §FS-config-v2.reader.5.commas.window: warned for once its elements load.
        if let Some(comma) = joined.commas
            && let Some(at) = reader.at(line_no)
        {
            let commas = CommaWithoutElement { at, line: comma };
            reader.project.commas_without_elements.push(commas);
        }
    }
    reader.close()
}

/// The value a `key = value` line starts, as [`joined_value`] reads it.
pub(crate) struct Joined {
    /// The value, or `None` when the array is still open at the end of the text.
    pub(crate) value: Option<String>,
    /// The array's open line, the first that ends inside a string
    /// (§FS-config-v2.reader.5.strings); `None` when every string closes on its line.
    pub(crate) open: Option<OpenLine>,
    /// The line of the first comma with no element before it in the run taken off before
    /// the `]` (§FS-config-v2.reader.5.commas), 1-based; `None` when there is none.
    pub(crate) commas: Option<usize>,
    /// Where in `value` each line after the key's starts, in order, so a place in the value
    /// names the line it was written on (§FS-config-v2.reader.6.window).
    pub(crate) starts: Vec<usize>,
}

/// A line of an array that ends inside a string (§FS-config-v2.reader.5.strings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OpenLine {
    /// The line, 1-based.
    pub(crate) line: usize,
    /// Whether the string open at its end opened with `"""`, which version 2 does
    /// not read (§FS-config-v2.reader.5.strings.multi-line).
    pub(crate) multi_line: bool,
}

impl OpenLine {
    /// The parenthetical `unclosed array` appends for this open line
    /// (§FS-config-v2.reader.5, §FS-config-v2.reader.5.strings.multi-line).
    pub(crate) fn parenthetical(self) -> String {
        let line = self.line;
        if self.multi_line {
            format!(" (a multi-line string opens on line {line}, which version 2 does not read)")
        } else {
            format!(" (a string is still open at the end of line {line})")
        }
    }
}

/// The value a `key = value` line starts, read as the reader reads it
/// (§FS-config-v2.reader.5): an array written across lines takes the lines it
/// spans from `lines`, joined into one, and loses the run of commas before its `]`,
/// reporting the first with no element before it (§FS-config-v2.reader.5.commas.window). `line` is the key's line, 1-based, and `lines` yields each line
/// after it with its 0-based index, as `str::lines().enumerate()` does.
/// The ancestor climb reads `[workspace] members` through this too, so a value the
/// config loads is never one the climb cannot read (§FS-workspace.6.1.7.5).
///
/// Through grund 0.21.x a string still open at a line's end runs on into the next
/// line, joined with the break read as one space, as the readers before it read it
/// (§FS-config-v2.reader.5.strings.window); the first line that ends so is reported
/// beside the value, which is what the warning and the refusal name.
pub(crate) fn joined_value<'a>(
    value: &str,
    line: usize,
    lines: &mut impl Iterator<Item = (usize, &'a str)>,
) -> Joined {
    let mut value = value.trim().to_string();
    let mut starts = Vec::new();
    if !value.starts_with('[') {
        return Joined {
            value: Some(value),
            open: None,
            commas: None,
            starts,
        };
    }
    let mut brackets = Brackets::default();
    brackets.read(&value);
    let mut open = brackets.open_line(line);
    while brackets.depth > 0 {
        let Some((idx, next)) = lines.next() else {
            return Joined {
                value: None,
                open,
                commas: None,
                starts,
            };
        };
        let next = strip_comment(next).trim();
        value.push(' ');
        starts.push(value.len());
        value.push_str(next);
        brackets.read(next);
        open = open.or(brackets.open_line(idx + 1));
    }
    let (value, comma) = array_value(&value);
    // §FS-config-v2.reader.5.commas.window: the line the comma stands on, not the key's.
    let commas = comma.map(|at| line + starts.partition_point(|&start| start <= at));
    Joined {
        value: Some(value),
        open,
        commas,
        starts,
    }
}

/// The `[` an array's text still holds open outside its strings, and the string
/// open at the end of the text read so far. A `"` outside a string opens one unless
/// it is part of a `\"` escape, and the string closes where `string_close` says, the
/// one close `strip_comment`, `split_outside_strings` and `is_one_basic_string`
/// read strings by (§FS-config.3.11).
#[derive(Default)]
struct Brackets {
    depth: i32,
    /// `Some(multi_line)` while inside a string: whether it opened with `"""`.
    string: Option<bool>,
}

impl Brackets {
    /// Read the next piece of the array's text. The pieces are joined with a space,
    /// so no `\"` escape reaches across two of them.
    fn read(&mut self, piece: &str) {
        let bytes = piece.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if self.string.is_some() {
                let Some(close) = string_close(bytes, i) else {
                    return;
                };
                self.string = None;
                i = close + 1;
                continue;
            }
            match bytes[i] {
                // §FS-config-v2.reader.5.strings.multi-line: the third quote of
                // a `"""` opens what the first would have in TOML.
                b'"' if !is_escaped(bytes, i) => {
                    self.string = Some(i >= 2 && bytes[i - 2..i] == *b"\"\"");
                }
                b'[' => self.depth += 1,
                b']' => self.depth -= 1,
                _ => {}
            }
            i += 1;
        }
    }

    /// `line`, when the text read so far ends inside a string
    /// (§FS-config-v2.reader.5.strings).
    fn open_line(&self, line: usize) -> Option<OpenLine> {
        self.string.map(|multi_line| OpenLine { line, multi_line })
    }
}

/// An array value with the run of adjacent commas before its `]` taken off, as the
/// readers before grund 0.21.0 took it off (§FS-config-v2.reader.5.commas.window), and
/// where in `value` the first comma of that run with no element before it stands: one
/// whose text before it, whitespace skipped, ends in `[` or `,`
/// (§FS-config-v2.reader.5.commas). Only a run that holds such a comma differs from the
/// one trailing comma TOML allows.
fn array_value(value: &str) -> (String, Option<usize>) {
    match value.strip_suffix(']') {
        Some(inner) if value.starts_with('[') && inner.trim_end().ends_with(',') => {
            let kept = inner.trim_end().trim_end_matches(',');
            let run = inner.trim_end().len() - kept.len();
            let first = if kept.trim_end().ends_with(['[', ',']) {
                Some(kept.len())
            } else {
                (run > 1).then_some(kept.len() + 1)
            };
            (format!("{kept}]"), first)
        }
        _ => (value.to_string(), None),
    }
}

/// The dotted name a header line writes. v2 writes every table under its own
/// bare header, never an array of tables (§FS-config-v2.reader.2).
fn header_name(reader: &Reader, line: usize, text: &str) -> Result<String> {
    if let Some(inner) = text.strip_prefix("[[") {
        let inner = inner.trim_end_matches(']').trim();
        // §FS-config-v2.reader.1.v1-tables: `[[kinds]]` names the row it becomes.
        let hint = tables::v1_hint(&format!("[[{inner}]]"));
        return reader.fail(
            line,
            format!("unknown config section `[[{inner}]]` (v2 writes no arrays of tables){hint}"),
        );
    }
    let Some(inner) = text
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    else {
        return reader.fail(line, "expected `[table]`".to_string());
    };
    let segments: Vec<&str> = inner.split('.').map(str::trim).collect();
    if !segments.iter().all(|segment| bare(segment)) {
        return reader.fail(
            line,
            format!(
                "unsupported table header `[{inner}]` (a v2 header is bare names joined by `.`)"
            ),
        );
    }
    Ok(segments.join("."))
}

/// A bare TOML key: letters, digits, `_` and `-`.
fn bare(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}
