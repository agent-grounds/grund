//! §AR-config.1.5: what is derived from a `Project` once — the ID grammar, and
//! the demand that tells the scanner what to record. `compile` is the one place
//! either is built, so a grammar cannot disagree with the schema it came from.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;

use super::file_reading::FileReading;
use super::grounding::finest_grounding_level_for_kind;
use super::project::{Project, Schema, Sources};
use super::record::DEFAULT_GROUNDING_LEVEL;
use super::rows::Row;
use crate::grammar::{Grammar, GrammarKind, LexicalSettings, require_kinds};

/// §AR-config.1.5: `compile(&Project)`.
#[derive(Clone)]
pub struct Compiled {
    pub grammar: Grammar,
    pub demand: ScanDemand,
    /// A v2 project's grammar per distinct host list of its admitted languages
    /// (§FS-languages.prefixes): a file reads through its own language's hosts.
    /// Empty for v1, whose one list `grammar` was built over.
    pub(crate) hosts: Vec<(Vec<String>, Grammar)>,
}

/// §AR-config.6.1: what the scanner must record beyond the catalog — the rows
/// whose effective `grounding_level` is finer than the file, the complement's
/// included, so the scanner records per-file structure for their files alone
/// (§AR-scanner.2.7.3). A level-1 tree — every config written before the keys
/// existed — names no row and pays nothing (§GOAL-fast-feedback).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScanDemand {
    structure_rows: BTreeSet<String>,
}

impl ScanDemand {
    /// §AR-config.6.1: whether no row asks for structure — the one-field answer
    /// that excuses every file of a level-1 tree from the per-file lookup.
    pub fn is_empty(&self) -> bool {
        self.structure_rows.is_empty()
    }

    /// §AR-config.6.1: whether the files of the row named `row` record their
    /// grounding structure — the homeless kind's name for a file no single home
    /// claims (§AR-scanner.2.7.1).
    pub fn records_structure(&self, row: &str) -> bool {
        self.structure_rows.contains(row)
    }
}

impl Compiled {
    /// The lexical settings of `project` under this grammar — what a scan and a
    /// rendered note sentence read, without the façade (§AR-config.5).
    pub(crate) fn lexical<'a>(&'a self, schema: &'a Schema) -> LexicalSettings<'a> {
        let (citation, sources, notes) = (&schema.citation, &schema.sources, &schema.notes);
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

    /// How `path` is read under this project (§FS-config-v2.schema.sources): the
    /// one resolver every consumer asks for a file's language, hosts and gate.
    pub(crate) fn reading<'a>(&'a self, schema: &'a Schema, path: &'a Path) -> FileReading<'a> {
        FileReading::of(self, schema, path)
    }

    /// The grammar a file of a language with these `comment_prefixes` reads with:
    /// the one built over them, or the project's where v1 built only that one.
    pub(crate) fn grammar_for(&self, comment_prefixes: &[String]) -> &Grammar {
        self.hosts
            .iter()
            .find(|(hosts, _)| hosts == comment_prefixes)
            .map_or(&self.grammar, |(_, grammar)| grammar)
    }
}

/// §AR-config.1.5: compile the grammar and the scan demand of `project`.
pub(crate) fn compile(project: &Project) -> Result<Compiled> {
    let ids = &project.schema.ids;
    let kinds = grammar_kinds(&project.schema.rows);
    // §FS-config-v2.defaults.1: only v1 refuses a table with no citable kind, and
    // not the empty one `kinds = []` declares (§FS-config.3.4.4.5).
    if project.version != 2 && !project.schema.rows.is_empty() {
        require_kinds(&kinds)?;
    }
    let grammar = Grammar::build(
        &ids.format,
        &kinds,
        &ids.number_pattern,
        &ids.slug_pattern,
        &ids.section_separator,
        ids.named_sections,
        &project.schema.sources.comment_prefixes,
    )?;
    Ok(Compiled {
        hosts: host_grammars(&grammar, &project.schema.sources)?,
        grammar,
        demand: scan_demand(project),
    })
}

/// §FS-languages.prefixes: a v2 file has no global prefix list, so each distinct
/// host list an admitted language reads through gets its own grammar, built once
/// here. A v1 project has one list and needs none.
pub(super) fn host_grammars(
    grammar: &Grammar,
    sources: &Sources,
) -> Result<Vec<(Vec<String>, Grammar)>> {
    let mut hosts: Vec<(Vec<String>, Grammar)> = Vec::new();
    let Some(model) = &sources.model else {
        return Ok(hosts);
    };
    for language in &model.admitted {
        let prefixes = &language.comment_prefixes;
        if !hosts.iter().any(|(built, _)| built == prefixes) {
            hosts.push((prefixes.clone(), grammar.with_hosts(prefixes)?));
        }
    }
    Ok(hosts)
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

/// §AR-config.6.1: every row, and the complement under its name, whose
/// effective level is finer than the file — read through the one
/// `grounding_level_for_kind` the checker cuts units with (§AR-checker.2.8), so
/// what the scanner records and what the checker cuts stay one rule. A v2 rung
/// below `must` finer than the file asks too (§FS-config-v2.rules.grounding).
pub(super) fn scan_demand(project: &Project) -> ScanDemand {
    let (schema, rules) = (&project.schema, &project.rules);
    let structure_rows = row_names(schema)
        .filter(|row| finest_grounding_level_for_kind(schema, rules, row) > DEFAULT_GROUNDING_LEVEL)
        .map(str::to_string)
        .collect();
    ScanDemand { structure_rows }
}

/// Every row name a file can belong to: each row, and the complement's name
/// where no row is the complement (§FS-config.3.9.2).
fn row_names(schema: &Schema) -> impl Iterator<Item = &str> {
    schema
        .rows
        .iter()
        .map(|row| row.name.as_str())
        .chain(std::iter::once(schema.complement_name()))
}
