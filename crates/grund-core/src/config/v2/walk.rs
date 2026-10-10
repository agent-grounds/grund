//! The v2 line walk (§FS-config-v2.reader): one pass over the file, each header
//! opening the table `tables.rs` names and each key read by its concern's file
//! as it is met, so the first refusal is the first one in the file. What every
//! table shares is kept here: explicit headers and the line each was first
//! written at (§FS-config-v2.reader.2), each table's keys (§FS-config-v2.reader.1),
//! and the close that runs a table's cross-key checks once its keys are all read.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

use super::schema::RowDraft;
use super::tables::{self, Opened, Table};
use super::{defaults, measures, presentation, rules, schema};
use crate::config::kind::KindResolution;
use crate::config::project::Project;
use crate::config::record::ConfigLocation;
use crate::config::v1::{bail_config, parse_string, strip_comment};

/// The reader's state: the project lowered so far, and what only the whole
/// file can judge, kept with the line it was written at (§FS-config-v2.reader.4).
pub(super) struct Reader<'a> {
    pub(super) path: &'a Path,
    pub(super) project: Project,
    /// The `[schema.kinds.<NAME>]` rows, in authored order.
    pub(super) rows: Vec<RowDraft>,
    /// Where `grund_config_version` was written: the anchor of an omitted
    /// default this grund cannot execute (§FS-config-v2.defaults.2).
    pub(super) version_line: usize,
    /// Whether `[schema.sources] languages` was written.
    pub(super) languages: bool,
    /// The explicit `[rules.resolution]` entries.
    pub(super) resolution: Vec<(String, KindResolution, usize)>,
    /// The place each row ladder names, at its header.
    pub(super) row_ladders: Vec<(String, usize)>,
    /// The kind each `[presentation.kinds.<NAME>]` names, at its header.
    pub(super) presentation_kinds: Vec<(String, usize)>,
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
            version_line: 1,
            languages: false,
            resolution: Vec::new(),
            row_ladders: Vec::new(),
            presentation_kinds: Vec::new(),
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
            Opened::Table(table) => table,
        };
        let citations = &mut self.project.rules.citations;
        match &table {
            Table::Citations => citations.declared = true,
            Table::CitationKind(kind, _) => {
                citations.declared = true;
                citations.per_kind.entry(kind.clone()).or_default();
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
    /// table does not admit it.
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
        let admitted = bare(key) && self.read_key(&mut table, key, value, line)?;
        self.table = table;
        if !admitted {
            return self.fail(line, format!("unknown key `{key}` {at}"));
        }
        Ok(())
    }

    fn read_key(&mut self, table: &mut Table, key: &str, value: &str, line: usize) -> Result<bool> {
        match table {
            Table::Root => self.envelope_key(key, value, line),
            Table::Parent => Ok(false),
            Table::Schema => schema::schema_key(self, key, value, line),
            Table::Sources => schema::sources_key(self, key, value, line),
            Table::Row(index) => schema::row_key(self, *index, key, value, line),
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
            // `selects` read this key's value; the reader keeps where it was.
            "grund_config_version" => self.version_line = line,
            "project_name" => {
                self.project.name = Some(parse_string(self.path, line, value)?);
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
            _ => Ok(()),
        }
    }
}

/// §FS-config-v2.reader: walk `text` into `reader`, header by header.
pub(super) fn walk(text: &str, reader: &mut Reader) -> Result<()> {
    for (idx, raw) in text.lines().enumerate() {
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
        reader.entry(key.trim(), value.trim(), line_no)?;
    }
    reader.close()
}

/// The dotted name a header line writes. v2 writes every table under its own
/// bare header, never an array of tables (§FS-config-v2.reader.2).
fn header_name(reader: &Reader, line: usize, text: &str) -> Result<String> {
    if let Some(inner) = text.strip_prefix("[[") {
        let inner = inner.trim_end_matches(']').trim();
        return reader.fail(
            line,
            format!("unknown config section `[[{inner}]]` (v2 writes no arrays of tables)"),
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
