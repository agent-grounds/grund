//! Test module: each table's keys against the gate (§FS-config-v2.reader.1.keys).
//! The list a table's refusal names is the one the gate admits by, so every key
//! some table lists is tried at every table, through the line walk alone: a key
//! its table lists is read, and any other is refused as unknown with the hint.

use std::path::Path;

use super::tables::{self, Keys, Opened, Table};
use super::walk::{Reader, walk};

/// One header per table the reader opens, the envelope first, and the parents.
const HEADERS: &[&str] = &[
    "",
    "schema",
    "schema.sources",
    "schema.sources.extensions",
    "schema.sources.definitions",
    "schema.sources.definitions.notes",
    "schema.notes",
    "schema.notes.lines",
    "schema.notes.columns",
    "schema.notes.text",
    "schema.notes.layout",
    "schema.leads",
    "schema.leads.words",
    "schema.kinds",
    "schema.kinds.FS",
    "schema.kinds.FS.fields",
    "schema.kinds.FS.fields.terms",
    "schema.kinds.FS.fields.front-matter",
    "schema.kinds.FS.fields.front-matter.keys",
    "schema.kinds.FS.fields.front-matter.keys.status",
    "rules",
    "rules.citations",
    "rules.citations.FS",
    "rules.citations.grounding",
    "rules.citations.FS.grounding",
    "rules.resolution",
    "presentation",
    "presentation.kinds",
    "presentation.kinds.FS",
    "presentation.fmt",
    "workspace",
];

/// The table `header` opens; a row, a field and a definition list their keys
/// whatever their draft, so the index is any.
fn table(header: &str) -> Table {
    if header.is_empty() {
        return Table::Root;
    }
    match tables::open(header) {
        Opened::Table(table) => table,
        Opened::Row(_) => Table::Row(0),
        Opened::Field(..) => Table::Field(0),
        Opened::FrontMatter(_) => Table::FrontMatter(0),
        Opened::Key(..) => Table::FrontMatterKey(0, 0),
        Opened::Definition(_) => Table::Definition(0),
        Opened::Refused(message) => panic!("[{header}] is refused: {message}"),
    }
}

/// The refusal `key` draws at `header`, or `None` when the walk reads it.
fn walked(header: &str, key: &str) -> Option<String> {
    let mut text = "grund_config_version = 2\n".to_string();
    if !header.is_empty() {
        // A field table names a row written above it (§FS-fields.declare.field).
        if header.starts_with("schema.kinds.FS.") {
            text.push_str("[schema.kinds.FS]\nfiles = [\"requirements.md\"]\n");
        }
        // A key table names its field written above it (§FS-fields.front-matter.config).
        if header.starts_with("schema.kinds.FS.fields.front-matter.keys.") {
            text.push_str("[schema.kinds.FS.fields.front-matter]\n");
        }
        text.push_str(&format!("[{header}]\n"));
    }
    text.push_str(&format!("{key} = \"x\"\n"));
    let mut reader = Reader::new(Path::new("grund.toml"));
    walk(&text, &mut reader).err().map(|err| format!("{err:#}"))
}

/// §FS-config-v2.reader.1.keys: at every table, a key is refused as unknown
/// exactly when the table's keys do not list it, and the refusal then names them.
#[test]
fn every_table_reads_the_keys_it_lists_and_no_other() {
    let mut candidates: Vec<&str> = vec!["bogus"];
    for header in HEADERS {
        if let Keys::Closed(keys) = table(header).keys() {
            for key in keys {
                if !candidates.contains(key) {
                    candidates.push(key);
                }
            }
        }
    }
    let mut failures = Vec::new();
    for header in HEADERS {
        let keys = table(header).keys();
        for key in &candidates {
            let unknown = format!("unknown key `{key}`");
            let refusal = walked(header, key).filter(|err| err.contains(&unknown));
            match (keys.admit(key), refusal) {
                (true, Some(err)) => failures.push(format!("[{header}] lists `{key}`: {err}")),
                (false, None) => failures.push(format!("[{header}] reads `{key}`, unlisted")),
                (false, Some(err)) if !err.ends_with(&keys.hint()) => {
                    failures.push(format!("[{header}] `{key}` names no keys: {err}"));
                }
                _ => {}
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
