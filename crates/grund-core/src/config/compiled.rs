//! §AR-config.1.5: what is derived from a `Project` once — the ID grammar, and
//! the demand that tells the scanner what to record. `compile` is the one place
//! either is built, so a grammar cannot disagree with the schema it came from.

use anyhow::Result;

use super::project::Project;
use super::record::DEFAULT_GROUNDING_LEVEL;
use super::rows::Row;
use crate::grammar::{Grammar, GrammarKind, LexicalSettings};

/// §AR-config.1.5: `compile(&Project)`.
#[derive(Clone)]
pub struct Compiled {
    pub grammar: Grammar,
    pub demand: ScanDemand,
}

/// §AR-config.1.5: what the scanner must record beyond the catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScanDemand {
    /// Whether any row's effective `grounding_level` is finer than the file, so
    /// the scanner records per-file structure (§AR-scanner.2.7). A level-1
    /// tree — every config written before the keys existed — pays nothing
    /// (§GOAL-fast-feedback).
    pub grounding_units: bool,
}

impl Compiled {
    /// The lexical settings of `project` under this grammar — what a scan and a
    /// rendered note sentence read, without the façade (§AR-config.5).
    pub(crate) fn lexical<'a>(&'a self, project: &'a Project) -> LexicalSettings<'a> {
        let (citation, sources, notes) = (
            &project.schema.citation,
            &project.schema.sources,
            &project.schema.notes,
        );
        LexicalSettings {
            grammar: &self.grammar,
            marker: &citation.marker,
            strict: citation.strict,
            comment_prefixes: &sources.comment_prefixes,
            docstring_python: sources.docstring_python,
            inline_style: &notes.inline_style,
            inline_note_layout: &notes.layout,
            inline_note_layout_check: &notes.layout_check,
        }
    }
}

/// §AR-config.1.5: compile the grammar and the scan demand of `project`.
pub(crate) fn compile(project: &Project) -> Result<Compiled> {
    let ids = &project.schema.ids;
    let grammar = Grammar::build(
        &ids.format,
        &grammar_kinds(&project.schema.rows),
        &ids.number_pattern,
        &ids.slug_pattern,
        &ids.section_separator,
        ids.named_sections,
        &project.schema.sources.comment_prefixes,
    )?;
    Ok(Compiled {
        grammar,
        demand: ScanDemand {
            grounding_units: grounding_units(project),
        },
    })
}

/// The citable rows as the ID grammar reads them (§FS-config.3.4): the name
/// that prefixes every ID of the kind, and the row's `format` override where it
/// carries one (§FS-config.3.2). A non-citable row declares no IDs, so it
/// enters no pattern, and deciding that is config's (§AR-system.2.1).
fn grammar_kinds(rows: &[Row]) -> Vec<GrammarKind> {
    rows.iter()
        .filter_map(|row| {
            row.kind.as_ref().map(|kind| GrammarKind {
                name: row.name.clone(),
                format: kind.id_format.clone(),
            })
        })
        .collect()
}

/// §FS-config.3.4.8: whether the complement or any row resolves to a level
/// finer than the file, each row's own level over the `[reference]` default.
fn grounding_units(project: &Project) -> bool {
    let grounding = &project.rules.grounding;
    let level = |name: &str| {
        grounding
            .kinds
            .get(name)
            .and_then(|kind| kind.level)
            .unwrap_or(grounding.level)
    };
    let complement_level = project
        .schema
        .rows
        .iter()
        .find(|row| row.is_complement())
        .map_or(grounding.level, |row| level(&row.name));
    complement_level > DEFAULT_GROUNDING_LEVEL
        || project
            .schema
            .rows
            .iter()
            .any(|row| level(&row.name) > DEFAULT_GROUNDING_LEVEL)
}
