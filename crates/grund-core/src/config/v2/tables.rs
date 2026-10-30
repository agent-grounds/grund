//! The tables a v2 header may open (§FS-config-v2.reader.1): each admitted
//! header names one `Table`, a clause the format defines and this grund does not
//! execute is a rollout refusal at its header (§FS-config-v2.rollout), and
//! everything else, v1's sections included, is an unknown section. Each table's
//! keys are listed here once, for the gate and for the refusal that names them
//! (§FS-config-v2.reader.1.keys), and so is where each v1 table's settings went
//! (§FS-config-v2.reader.1.v1-tables).

use crate::config::citations::CitationLevel;
use crate::config::project::{Rung, Strength};
use crate::model::format_list;

/// What the keys under one header are read as.
pub(super) enum Table {
    /// The keys before the first header: the envelope.
    Root,
    /// A parent written explicitly that holds tables and no keys of its own,
    /// such as `[schema.kinds]` (§FS-config-v2.reader.2).
    Parent,
    Schema,
    Sources,
    /// `[schema.sources.extensions]` (§FS-config-v2.schema.sources.extensions).
    Extensions,
    /// `[schema.sources.definitions.<name>]`: the index of its draft.
    Definition(usize),
    /// `[schema.notes.lines]`, `[schema.notes.columns]`, `[schema.leads.words]`:
    /// the strength keys read so far (§FS-config-v2.schema.measures).
    Measure(Measure, Vec<Written>),
    Text,
    /// `[schema.notes.layout]`, with the line of the one strength it takes.
    Layout(Option<usize>),
    /// `[schema.kinds.<NAME>]`: the index of its draft row.
    Row(usize),
    /// `[schema.kinds.<NAME>.fields.<field>]`: the index of its draft field.
    Field(usize),
    /// `[schema.kinds.<NAME>.fields.front-matter]`: the index of its draft field
    /// (§FS-fields.front-matter.config).
    FrontMatter(usize),
    /// `[schema.kinds.<NAME>.fields.front-matter.keys.<key>]`: its field's draft
    /// and the key's index in it (§FS-fields.front-matter.config).
    FrontMatterKey(usize, usize),
    /// `[rules.citations]`.
    Citations,
    /// `[rules.citations.<KIND>]` and the lists read so far, each at its line.
    CitationKind(String, Vec<(CitationLevel, usize)>),
    /// `[rules.citations.grounding]` (`None`) or `[rules.citations.<PLACE>.grounding]`.
    Ladder(Option<String>, Vec<Rung>),
    Resolution,
    Presentation,
    PresentationKind(String),
    PresentationFmt,
    Workspace,
    /// `[workspace.imports.<alias>]`: the index of its import
    /// (§FS-imports.declaration.table).
    Import(usize),
}

/// The measure a measure table is named for.
#[derive(Clone, Copy)]
pub(super) enum Measure {
    Lines,
    Columns,
    Words,
}

/// One strength key of a measure table: its threshold and line.
#[derive(Clone, Copy)]
pub(super) struct Written {
    pub(super) strength: Strength,
    pub(super) value: usize,
    pub(super) line: usize,
}

/// `[rules.citations.<KIND>]`'s keys (§FS-config-v2.rules.citations): every
/// strength, positive and then prohibitive, and the kind's `default`. The
/// strength keys are its first seven, so they are spelled once.
const CITATION_KIND_KEYS: &[&str] = &[
    "must",
    "warn",
    "should",
    "may",
    "must-not",
    "warn-not",
    "should-not",
    "default",
];

/// Every strength key a v2 table can hold, positive and prohibitive
/// (§FS-config-v2.rules.strengths).
const STRENGTH_KEYS: &[&str] = CITATION_KIND_KEYS.split_at(7).0;

/// The positive strengths, which are a measure's and a ladder's keys.
const POSITIVE_STRENGTH_KEYS: &[&str] = STRENGTH_KEYS.split_at(4).0;

/// §FS-config-v2.reader.1.keys: the keys a table admits. The gate admits a key
/// by it and an unknown key's refusal names it, so no key the refusal leaves
/// out is read; `tests_keys.rs` holds that every key it names is.
#[derive(Clone, Copy)]
pub(super) enum Keys {
    /// These keys, in the order the table's own point lists them.
    Closed(&'static [&'static str]),
    /// None: a parent that holds only tables (§FS-config-v2.reader.2).
    OnlyTables,
    /// Any bare key, read as the name this says, which the table judges itself.
    Names(&'static str),
}

impl Keys {
    /// Whether the gate hands `key` to the table's reader.
    pub(super) fn admit(self, key: &str) -> bool {
        match self {
            Keys::Closed(keys) => keys.contains(&key),
            Keys::OnlyTables => false,
            Keys::Names(_) => true,
        }
    }

    /// What an unknown key's refusal appends after its text.
    pub(super) fn hint(self) -> String {
        match self {
            Keys::Closed(keys) => format!(" (expected {})", format_list(keys, "or")),
            Keys::OnlyTables => " (it holds only tables)".to_string(),
            Keys::Names(what) => format!(" (expected {what})"),
        }
    }
}

impl Table {
    /// §FS-config-v2.reader.1.keys: the keys this table admits, each list once.
    pub(super) fn keys(&self) -> Keys {
        match self {
            Table::Root => Keys::Closed(&["grund_config_version", "project_name"]),
            Table::Parent => Keys::OnlyTables,
            Table::Schema => Keys::Closed(&[
                "marker",
                "shorthand",
                "id_format",
                "slug_pattern",
                "number_pattern",
                "section_separator",
                "heading_depth",
            ]),
            Table::Sources => {
                Keys::Closed(&["languages", "declarations", "exclude", "respect_gitignore"])
            }
            Table::Extensions => Keys::Names("a suffix"),
            Table::Definition(_) => {
                Keys::Closed(&["class", "comment_prefixes", "doc_comments", "docstrings"])
            }
            Table::Measure(..) | Table::Layout(_) | Table::Ladder(..) => {
                Keys::Closed(POSITIVE_STRENGTH_KEYS)
            }
            Table::Text => Keys::Closed(&["must"]),
            Table::Row(_) => Keys::Closed(&[
                "files",
                "folders",
                "citable",
                "scan",
                "index",
                "id_format",
                "fetch",
                "form",
                "closed",
            ]),
            Table::Field(_) => Keys::Closed(&[
                "title", "presence", "position", "after", "content", "one_of",
            ]),
            // §FS-fields.front-matter.config: the block has no heading to title or place.
            Table::FrontMatter(_) => Keys::Closed(&["presence", "closed"]),
            Table::FrontMatterKey(..) => Keys::Closed(&["presence", "one_of"]),
            Table::Citations => Keys::Closed(&["default"]),
            Table::CitationKind(..) => Keys::Closed(CITATION_KIND_KEYS),
            Table::Resolution => Keys::Names("a kind name"),
            Table::Presentation => {
                Keys::Closed(&["description", "trigger", "conversation", "rules"])
            }
            Table::PresentationKind(_) => Keys::Closed(&["title"]),
            Table::PresentationFmt => Keys::Closed(&["anchors", "links", "exclude"]),
            Table::Workspace => Keys::Closed(&["members", "optional_members", "include_root"]),
            Table::Import(_) => Keys::Closed(&["path", "project"]),
        }
    }
}

/// §FS-config-v2.reader.1.v1-tables: each v1 table, as v1 writes its header, and
/// the v2 tables its settings are written in, in the order its refusal names
/// them. `<KIND>` stands for the header's own kind.
pub(in crate::config) const V1_TABLES: &[(&str, &[&str])] = &[
    (
        "[reference]",
        &[
            "[schema]",
            "[schema.notes.*]",
            "[schema.leads.words]",
            "[rules.citations.grounding]",
            "[presentation]",
        ],
    ),
    ("[id]", &["[schema]"]),
    (
        "[scan]",
        &[
            "[schema.sources]",
            "[schema.sources.extensions]",
            "[schema.sources.definitions.*]",
            "[schema.kinds.code]",
        ],
    ),
    ("[citations]", &["[rules.citations]"]),
    ("[citations.<KIND>]", &["[rules.citations.<KIND>]"]),
    ("[fmt]", &["[presentation.fmt]"]),
    ("[fmt.cross_refs]", &["[presentation.fmt]"]),
    ("[output]", &["run flags, such as --format and --path-base"]),
    ("[[kinds]]", &["[schema.kinds.<NAME>]"]),
];

/// §FS-config-v2.reader.1.v1-tables: where the settings of the v1 table `header`
/// (`[scan]`, `[citations.FS]`, `[[kinds]]`) are written in v2, or `None` for a
/// header that is no v1 table.
pub(in crate::config) fn v1_homes(header: &str) -> Option<Vec<String>> {
    let kind = header
        .strip_prefix("[citations.")
        .and_then(|rest| rest.strip_suffix(']'))
        .filter(|kind| !kind.is_empty() && !kind.contains('.'));
    let row = if kind.is_some() {
        "[citations.<KIND>]"
    } else {
        header
    };
    let (_, homes) = V1_TABLES.iter().find(|(v1, _)| *v1 == row)?;
    let homes = homes.iter().map(|home| match kind {
        Some(kind) => home.replace("<KIND>", kind),
        None => home.to_string(),
    });
    Some(homes.collect())
}

/// §FS-config-v2.reader.1.v1-tables: what the refusal of the table `header`
/// appends, which is nothing for a table that is no v1 spelling.
pub(super) fn v1_hint(header: &str) -> String {
    v1_homes(header).map_or_else(String::new, |homes| {
        format!(" (a v1 table; in v2: {})", homes.join(", "))
    })
}

/// One header: its path, where a `<…>` segment matches any one name; for a table
/// of names, the placeholder its bare keys stand for; and what it opens from the
/// names its `<…>` segments matched.
type Header = (&'static str, Option<&'static str>, fn(&[&str]) -> Opened);

/// §FS-config-v2.reader.1: every header the v2 reader opens, in the order `open`
/// tries them. `admitted` walks the same list, so `config explain` names exactly
/// the tables the reader opens (§FS-config-explain.3).
const HEADERS: &[Header] = &[
    ("schema", None, |_| Opened::Table(Table::Schema)),
    ("schema.sources", None, |_| Opened::Table(Table::Sources)),
    ("schema.sources.extensions", Some("<suffix>"), |_| {
        Opened::Table(Table::Extensions)
    }),
    ("schema.sources.definitions", None, |_| {
        Opened::Table(Table::Parent)
    }),
    ("schema.sources.definitions.<name>", None, |names| {
        Opened::Definition(names[0].to_string())
    }),
    ("schema.notes", None, |_| Opened::Table(Table::Parent)),
    ("schema.notes.lines", None, |_| {
        Opened::Table(Table::Measure(Measure::Lines, Vec::new()))
    }),
    ("schema.notes.columns", None, |_| {
        Opened::Table(Table::Measure(Measure::Columns, Vec::new()))
    }),
    ("schema.notes.text", None, |_| Opened::Table(Table::Text)),
    ("schema.notes.layout", None, |_| {
        Opened::Table(Table::Layout(None))
    }),
    ("schema.leads", None, |_| Opened::Table(Table::Parent)),
    ("schema.leads.words", None, |_| {
        Opened::Table(Table::Measure(Measure::Words, Vec::new()))
    }),
    ("schema.kinds", None, |_| Opened::Table(Table::Parent)),
    ("schema.kinds.<NAME>", None, |names| {
        Opened::Row(names[0].to_string())
    }),
    // §FS-config-v2.schema.fields, §FS-fields.declare.field.
    ("schema.kinds.<NAME>.fields", None, |_| {
        Opened::Table(Table::Parent)
    }),
    // §FS-fields.front-matter.config: the one field name with keys of its own,
    // tried before any field name.
    ("schema.kinds.<NAME>.fields.front-matter", None, |names| {
        Opened::FrontMatter(names[0].to_string())
    }),
    ("schema.kinds.<NAME>.fields.front-matter.keys", None, |_| {
        Opened::Table(Table::Parent)
    }),
    (
        "schema.kinds.<NAME>.fields.front-matter.keys.<key>",
        None,
        |names| Opened::Key(names[0].to_string(), names[1].to_string()),
    ),
    ("schema.kinds.<NAME>.fields.<field>", None, |names| {
        Opened::Field(names[0].to_string(), names[1].to_string())
    }),
    ("rules", None, |_| Opened::Table(Table::Parent)),
    ("rules.citations", None, |_| Opened::Table(Table::Citations)),
    ("rules.citations.grounding", None, |_| {
        Opened::Table(Table::Ladder(None, Vec::new()))
    }),
    ("rules.citations.<KIND>", None, |names| {
        Opened::Table(Table::CitationKind(names[0].to_string(), Vec::new()))
    }),
    ("rules.citations.<KIND>.grounding", None, |names| {
        Opened::Table(Table::Ladder(Some(names[0].to_string()), Vec::new()))
    }),
    ("rules.resolution", Some("<KIND>"), |_| {
        Opened::Table(Table::Resolution)
    }),
    ("presentation", None, |_| Opened::Table(Table::Presentation)),
    ("presentation.kinds", None, |_| Opened::Table(Table::Parent)),
    ("presentation.kinds.<NAME>", None, |names| {
        Opened::Table(Table::PresentationKind(names[0].to_string()))
    }),
    ("presentation.fmt", None, |_| {
        Opened::Table(Table::PresentationFmt)
    }),
    ("workspace", None, |_| Opened::Table(Table::Workspace)),
    // §FS-imports.declaration.table, §FS-imports.declaration.alias.
    ("workspace.imports", None, |_| Opened::Table(Table::Parent)),
    ("workspace.imports.<alias>", None, |names| {
        Opened::Import(names[0].to_string())
    }),
];

/// §FS-config-explain.3: each table the v2 reader admits, `[header]` or `""`
/// for the top level, with the keys its gate admits.
pub(in crate::config) fn admitted() -> Vec<(String, Vec<String>)> {
    let mut tables = vec![(String::new(), keys_of(&Table::Root, None))];
    for (header, placeholder, opens) in HEADERS {
        let names: Vec<&str> = header.split('.').filter(|s| s.starts_with('<')).collect();
        let table = match opens(&names) {
            Opened::Table(table) => table,
            Opened::Row(_) => Table::Row(0),
            Opened::Field(..) => Table::Field(0),
            Opened::FrontMatter(_) => Table::FrontMatter(0),
            Opened::Key(..) => Table::FrontMatterKey(0, 0),
            Opened::Definition(_) => Table::Definition(0),
            Opened::Import(_) => Table::Import(0),
            Opened::Refused(why) => unreachable!("[{header}] is refused: {why}"),
        };
        tables.push((format!("[{header}]"), keys_of(&table, *placeholder)));
    }
    tables
}

/// The keys `table`'s gate admits, a table of names answering as `placeholder`.
fn keys_of(table: &Table, placeholder: Option<&str>) -> Vec<String> {
    match table.keys() {
        Keys::Closed(keys) => keys.iter().map(|key| key.to_string()).collect(),
        Keys::OnlyTables => Vec::new(),
        Keys::Names(_) => placeholder.into_iter().map(str::to_string).collect(),
    }
}

/// What a header opens, or why it is refused.
pub(super) enum Opened {
    Table(Table),
    /// A row header; the reader makes its draft.
    Row(String),
    /// A field header, its kind and its name; the reader makes its draft.
    Field(String, String),
    /// The front-matter field's header and its kind (§FS-fields.front-matter.config).
    FrontMatter(String),
    /// A front-matter key's header, its kind and the key (§FS-fields.front-matter.config).
    Key(String, String),
    /// A language definition's header and its name; the reader makes its draft.
    Definition(String),
    /// An import's header and its alias; the reader makes its record
    /// (§FS-imports.declaration.table).
    Import(String),
    Refused(String),
}

/// §FS-config-v2.reader.1: the table `name` opens.
pub(super) fn open(name: &str) -> Opened {
    let segments: Vec<&str> = name.split('.').collect();
    // §FS-config-v2.reader.3: a strength key is never also a table.
    if let Some((parent, last)) = name.rsplit_once('.')
        && STRENGTH_KEYS.contains(&last)
        && takes_strength_keys(parent)
    {
        return Opened::Refused(format!(
            "`{last}` is a strength key in [{parent}], not a table"
        ));
    }
    for (header, _, opens) in HEADERS {
        if let Some(names) = matched(header, &segments) {
            return opens(&names);
        }
    }
    // §FS-config-v2.reader.1.v1-tables: a v1 table names where it went.
    let hint = v1_hint(&format!("[{name}]"));
    Opened::Refused(format!("unknown config section `{name}`{hint}"))
}

/// The names `segments` gives the `<…>` segments of `header`, or `None` where
/// they spell another header.
fn matched<'a>(header: &str, segments: &[&'a str]) -> Option<Vec<&'a str>> {
    let pattern: Vec<&str> = header.split('.').collect();
    if pattern.len() != segments.len() {
        return None;
    }
    let mut names = Vec::new();
    for (want, got) in pattern.iter().zip(segments) {
        if want.starts_with('<') {
            names.push(*got);
        } else if want != got {
            return None;
        }
    }
    Some(names)
}

/// Whether the table `name` is one whose keys are strengths: a measure, a
/// ladder, or a kind's citation lists.
fn takes_strength_keys(name: &str) -> bool {
    matches!(
        name.split('.').collect::<Vec<_>>().as_slice(),
        ["schema", "notes", "lines" | "columns" | "text" | "layout"]
            | ["schema", "leads", "words"]
            | ["rules", "citations", _]
            | ["rules", "citations", _, "grounding"]
    )
}

/// §FS-config-v2.rollout: the refusal of a clause the format defines and this
/// grund does not execute yet.
pub(super) fn not_yet(clause: &str) -> String {
    format!("`{clause}` is part of the v2 format but not supported by this grund yet")
}

/// Where a key sits, in a message: `in [schema]`, or `at the top level`.
pub(super) fn label(header: &str) -> String {
    if header.is_empty() {
        "at the top level".to_string()
    } else {
        format!("in [{header}]")
    }
}
