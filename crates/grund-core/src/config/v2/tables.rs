//! The tables a v2 header may open (§FS-config-v2.reader.1): each admitted
//! header names one `Table`, a clause the format defines and this grund does not
//! execute is a rollout refusal at its header (§FS-config-v2.rollout), and
//! everything else, v1's sections included, is an unknown section.

use crate::config::citations::CitationLevel;
use crate::config::project::{Rung, Strength};

/// What the keys under one header are read as.
pub(super) enum Table {
    /// The keys before the first header: the envelope.
    Root,
    /// A parent written explicitly that holds tables and no keys of its own,
    /// such as `[schema.kinds]` (§FS-config-v2.reader.2).
    Parent,
    Schema,
    Sources,
    /// `[schema.notes.lines]`, `[schema.notes.columns]`, `[schema.leads.words]`:
    /// the strength keys read so far (§FS-config-v2.schema.measures).
    Measure(Measure, Vec<Written>),
    Text,
    /// `[schema.notes.layout]`, with the line of the one strength it takes.
    Layout(Option<usize>),
    /// `[schema.kinds.<NAME>]`: the index of its draft row.
    Row(usize),
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

/// Every strength key a v2 table can hold, positive and prohibitive
/// (§FS-config-v2.rules.strengths).
const STRENGTH_KEYS: [&str; 7] = [
    "must",
    "warn",
    "should",
    "may",
    "must-not",
    "warn-not",
    "should-not",
];

/// What a header opens, or why it is refused.
pub(super) enum Opened {
    Table(Table),
    /// A row header; the reader makes its draft.
    Row(String),
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
    let table = match segments.as_slice() {
        ["schema"] => Table::Schema,
        ["schema", "sources"] => Table::Sources,
        // §FS-config-v2.rollout: #456's language tables.
        ["schema", "sources", "extensions"] | ["schema", "sources", "definitions", ..] => {
            return Opened::Refused(not_yet(&format!("[{name}]")));
        }
        ["schema", "notes" | "leads" | "kinds"] | ["rules"] | ["presentation", "kinds"] => {
            Table::Parent
        }
        ["schema", "notes", "lines"] => Table::Measure(Measure::Lines, Vec::new()),
        ["schema", "notes", "columns"] => Table::Measure(Measure::Columns, Vec::new()),
        ["schema", "notes", "text"] => Table::Text,
        ["schema", "notes", "layout"] => Table::Layout(None),
        ["schema", "leads", "words"] => Table::Measure(Measure::Words, Vec::new()),
        ["schema", "kinds", row] => return Opened::Row(row.to_string()),
        // §FS-config-v2.schema.fields: #458's fields.
        ["schema", "kinds", _, "fields", ..] => {
            return Opened::Refused(not_yet(&format!("[{name}]")));
        }
        ["rules", "citations"] => Table::Citations,
        ["rules", "citations", "grounding"] => Table::Ladder(None, Vec::new()),
        ["rules", "citations", kind] => Table::CitationKind(kind.to_string(), Vec::new()),
        ["rules", "citations", place, "grounding"] => {
            Table::Ladder(Some(place.to_string()), Vec::new())
        }
        ["rules", "resolution"] => Table::Resolution,
        ["presentation"] => Table::Presentation,
        ["presentation", "kinds", kind] => Table::PresentationKind(kind.to_string()),
        ["presentation", "fmt"] => Table::PresentationFmt,
        ["workspace"] => Table::Workspace,
        _ => return Opened::Refused(format!("unknown config section `{name}`")),
    };
    Opened::Table(table)
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
