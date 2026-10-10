//! §AR-config.1: the project a `grund.toml` describes, one record per concern
//! inside an envelope (§FS-config.concerns). A reader lowers a file into this
//! and nothing above config learns which version spelled it (§AR-config.2):
//! the records carry meanings, never spellings.
//!
//! Every validated value carries the `ConfigLocation` it was written at, so the
//! one validation pass after lowering anchors an error where the reader used to
//! (§AR-config.4). The rows of the schema, and the views over them, are
//! `rows.rs`; the invocation's facts are `run.rs`; what is derived once is
//! `compiled.rs`.

use std::collections::BTreeMap;

use super::citations::CitationRules;
use super::kind::KindResolution;
use super::point_sizes::LeadSizeWarning;
use super::record::{ConfigLocation, ShorthandPolicy};
use super::rows::{Nesting, Row};

/// The `[citations]` table under its concern's name (§AR-config.1.2): the
/// direction rules, unchanged in shape from the record `Config` carried.
pub type Citations = CitationRules;

/// §AR-config.1.1: one project — the envelope (`name`, `version`, `workspace`)
/// read before any concern, and the three concerns it holds.
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub name: Option<String>,
    /// Where `project_name` was written (§FS-config.4.3).
    pub name_source: Option<ConfigLocation>,
    /// The format version that spelled the file: `1` for every file the v1
    /// reader reads (§AR-config.3.2). Nothing above config reads it.
    pub version: u32,
    pub workspace: Members,
    pub schema: Schema,
    pub rules: Rules,
    pub presentation: Presentation,
}

/// §AR-config.1.1: the `[workspace]` block, each list with the line it was
/// written at — the envelope's say over which projects there are.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Members {
    pub declared: bool,
    /// Where the `[workspace]` header was written (§FS-workspace.6.1.3).
    pub section_source: Option<ConfigLocation>,
    pub members: Vec<String>,
    pub members_source: Option<ConfigLocation>,
    pub optional_members: Vec<String>,
    pub optional_members_source: Option<ConfigLocation>,
    pub include_root: bool,
    pub include_root_source: Option<ConfigLocation>,
}

/// §AR-config.1.2: what exists — the citation and ID grammar, what is read,
/// how notes are written, and the rows that say where things live.
#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    pub citation: CitationSyntax,
    pub ids: IdGrammar,
    pub sources: Sources,
    pub notes: NoteStyle,
    pub leads: Option<LeadSizeWarning>,
    /// The v2 lead thresholds `leads` cannot hold: `must`, `should` and `may`
    /// on `[schema.leads.words]` (§FS-config-v2.schema.measures).
    pub lead_thresholds: Vec<Threshold>,
    pub rows: Vec<Row>,
    pub nesting: Nesting,
}

/// §AR-config.1.2: how a citation is written (§FS-config.3.1).
#[derive(Clone, Debug, PartialEq)]
pub struct CitationSyntax {
    pub marker: String,
    pub strict: bool,
    pub shorthand: ShorthandPolicy,
}

/// §AR-config.1.2: the ID grammar's inputs (§FS-config.3.2).
#[derive(Clone, Debug, PartialEq)]
pub struct IdGrammar {
    pub format: String,
    pub section_separator: String,
    pub number_pattern: String,
    pub slug_pattern: String,
    pub named_sections: bool,
    pub section_heading_levels: String,
}

/// §AR-config.1.2: what the walk reads (§FS-config.3.5).
#[derive(Clone, Debug, PartialEq)]
pub struct Sources {
    pub include: Option<Vec<String>>,
    pub exclude: Vec<String>,
    pub extensions: Vec<String>,
    pub comment_prefixes: Vec<String>,
    pub docstring_python: bool,
    pub respect_gitignore: bool,
}

/// §AR-config.1.2: how an inline note is written (§FS-inline-citation-style).
/// The two line budgets carry their lines, which the budget rule anchors at.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteStyle {
    pub inline_style: String,
    pub suggested_lines: usize,
    pub suggested_lines_source: Option<ConfigLocation>,
    pub max_lines: usize,
    pub max_lines_source: Option<ConfigLocation>,
    pub max_columns: usize,
    pub layout: String,
    pub layout_check: String,
    pub warn_on_suggested: bool,
    /// The v2 line thresholds the fields above cannot hold: a `should` budget,
    /// and a `warn` budget beside a tighter guidance one
    /// (§FS-config-v2.schema.measures). Empty for every v1 file.
    pub extra_lines: Vec<Threshold>,
    /// The v2 column thresholds below the `must` cap (§FS-config-v2.schema.measures).
    pub extra_columns: Vec<Threshold>,
}

/// §FS-config-v2.rules.strengths: one strength, which names a finding's
/// channel. A prohibition is a citation level (`CitationLevel`), never this.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Strength {
    Must,
    Warn,
    Should,
    May,
}

impl Strength {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Must => "must",
            Self::Warn => "warn",
            Self::Should => "should",
            Self::May => "may",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "must" => Some(Self::Must),
            "warn" => Some(Self::Warn),
            "should" => Some(Self::Should),
            "may" => Some(Self::May),
            _ => None,
        }
    }
}

/// §FS-config-v2.schema.measures: one strength key of a measure table and the
/// threshold it was written with, at its line.
#[derive(Clone, Debug, PartialEq)]
pub struct Threshold {
    pub strength: Strength,
    pub value: usize,
    pub source: Option<ConfigLocation>,
}

/// §FS-config-v2.rules.grounding: one rung of a grounding ladder — a strength
/// and a unit, `1` for the file and `2..=6` for a heading level.
#[derive(Clone, Debug, PartialEq)]
pub struct Rung {
    pub strength: Strength,
    pub level: usize,
    pub source: Option<ConfigLocation>,
}

/// §AR-config.1.2: how things relate — the citation directions, the grounding
/// obligation, and how strongly an external kind must resolve.
#[derive(Clone, Debug, PartialEq)]
pub struct Rules {
    pub citations: Citations,
    pub grounding: Grounding,
    /// `[[kinds]] resolve` per kind name, the fetched kind's default included
    /// (§FS-config.3.4.10).
    pub resolution: BTreeMap<String, KindResolution>,
}

/// §AR-config.1.2: the grounding pair (§FS-config.3.4.8): the `[reference]`
/// default, and what each row says over it.
#[derive(Clone, Debug, PartialEq)]
pub struct Grounding {
    pub require: bool,
    pub level: usize,
    /// Where `[reference] grounding_level` was written (§FS-config.3.4.8.5).
    pub level_source: Option<ConfigLocation>,
    pub kinds: BTreeMap<String, KindGrounding>,
    /// The project's v2 ladder as written, `None` in v1 and where v2 wrote none
    /// (§FS-config-v2.rules.grounding). Its `must` rung is also `require` and
    /// `level` above, the pair v1 resolves (§FS-config-v2.mapping).
    pub ladder: Option<Vec<Rung>>,
}

/// One row's grounding keys, as written, each with its line (§FS-config.3.4.8).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KindGrounding {
    pub require: Option<bool>,
    pub require_source: Option<ConfigLocation>,
    pub level: Option<usize>,
    pub level_source: Option<ConfigLocation>,
    /// The row's v2 ladder, which replaces the project's whole
    /// (§FS-config-v2.rules.grounding); `None` for every v1 row.
    pub ladder: Option<Vec<Rung>>,
}

/// §AR-config.1.2: what bytes get written — the formatter, the titles, the
/// identity line, the agent surfaces and the report.
#[derive(Clone, Debug, PartialEq)]
pub struct Presentation {
    pub fmt: FmtPresentation,
    /// `[[kinds]] title` per kind name (§FS-config.3.4).
    pub kinds: BTreeMap<String, String>,
    pub description: Option<String>,
    pub conversation: Option<String>,
    pub trigger: String,
    pub output: OutputPresentation,
}

/// `[fmt]` and `[fmt.cross_refs]` (§FS-config.3.7, §FS-config.3.10).
#[derive(Clone, Debug, PartialEq)]
pub struct FmtPresentation {
    pub exclude: Vec<String>,
    pub cross_refs_enabled: bool,
    pub anchor_format: String,
}

/// `[output]` (§FS-config.3.6). `color` is reserved and read by nothing, and a
/// lossless lowering keeps it all the same (§AR-config.3.1).
#[derive(Clone, Debug, PartialEq)]
pub struct OutputPresentation {
    pub format: String,
    pub relative_paths: bool,
    pub color: Option<String>,
}
