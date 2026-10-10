//! The grounding pass (§FS-check.3.6), in a file of its own beside
//! `citations.rs` and `sections.rs` (§AR-core-module-layout.1):
//! which `[[kinds]]` row governs each scanned file, what unit that row's
//! `grounding_level` cuts the file into, and which of those units carry no
//! citation to a declared ID.
//!
//! It is the sixth rule to leave `check_with_workspace` as a named function
//! rather than an inline block, and it leaves because it stopped being one: the
//! per-file test became a per-unit one over structure the scanner records
//! (§AR-scanner.2.7.2), with a row lookup in front of it.

use std::collections::BTreeMap;
use std::path::Path;

use super::homes::{DeclarationHome, KindHomeIndex};
use super::obligation_units::ObligationUnit;
use crate::config::{
    DEFAULT_GROUNDING_LEVEL, Frame, Row, Rules, Schema, Strength, any_place_grounded,
    grounding_level_for_kind, homeless_row_grounding, row_grounding, soft_grounding_rungs,
};
use crate::model::{Catalog, CheckReport, Citation, Diagnostic, FileStructure};
use crate::resolver::{WorkspaceCheckTarget, citation_resolves};
use crate::workspace::namespace_is_unverified;

/// One thing a row's `grounding_level` asks for a citation (§FS-check.3.6.2):
/// where it starts, where it ends, the line a finding anchors at, and how the
/// message names it.
struct GroundingUnit {
    start: usize,
    end: usize,
    subject: GroundingSubject,
}

/// How a unit is named in its finding (§FS-check.3.6.3). The home, when the unit
/// sits in a non-citable one, is added by the caller — one place decides it, so
/// every shape reads the same way.
enum GroundingSubject {
    File,
    Section(String),
    DocComment,
}

/// §FS-check.3.6 / §DF-require-grounding: every unit of every governed file must
/// carry a citation to a declared ID — or, in a source file outside a
/// non-citable home, declare one inline (a spec home is grounded in the spec it
/// *is*).
///
/// §FS-workspace.4.3: a citation into an absent optional namespace grounds its unit.
/// It cannot be resolved here, but it resolves in the checkout that has the member
/// — and a unit that lost its grounding to a missing directory would be a finding
/// at the site the run is required to say nothing about.
pub(super) fn check_grounding(
    findings: &Catalog,
    rules: &Rules,
    schema: &Schema,
    frame: Frame<'_>,
    kind_homes: &KindHomeIndex<'_>,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    report: &mut CheckReport,
) {
    if !any_place_grounded(schema, rules, frame.run) {
        return;
    }
    // Two linear passes — citations, then declarations — so the per-unit test
    // below is a lookup in a small per-file list, never a re-scan
    // (§GOAL-fast-feedback).
    let mut cited: BTreeMap<&Path, Vec<usize>> = BTreeMap::new();
    for cite in &findings.citations {
        // §FS-workspace.4.3: unverified grounds the unit — see this function's docs.
        let unverified = cite
            .namespace
            .as_deref()
            .is_some_and(|namespace| namespace_is_unverified(frame.run, namespace));
        if unverified || citation_resolves(cite, findings, schema, frame, workspace) {
            cited
                .entry(cite.file.as_path())
                .or_default()
                .push(cite.line);
        }
    }
    let mut declared: BTreeMap<&Path, Vec<usize>> = BTreeMap::new();
    for decl in findings.declarations.values().flatten() {
        if !decl.is_stub && decl.e2e_case.is_none() {
            declared
                .entry(decl.file.as_path())
                .or_default()
                .push(decl.line);
        }
    }

    for file in &findings.scanned_files {
        let home = kind_homes.unique_decl_home_for_file(file);
        let rungs = governing_rungs(rules, schema, frame, home.as_ref(), file);
        if rungs.is_empty() {
            continue;
        }
        let place = home
            .as_ref()
            .filter(|home| !home.citable)
            .map(|home| home.place());
        let none: &[usize] = &[];
        let cited_lines = cited.get(file.as_path()).map_or(none, Vec::as_slice);
        // §FS-check.3.6.2.3: no inline-declaration escape in a non-citable home — a declaration
        // there is already misplaced (§FS-declarations.checks.misplaced-declaration.3), so the only
        // way to ground a unit is to cite one.
        let declared_lines = match place {
            Some(_) => none,
            None => declared.get(file.as_path()).map_or(none, Vec::as_slice),
        };
        // §FS-config-v2.rules.grounding: rungs run strongest first, and a unit a
        // stronger rung already reported is not reported again by a finer one.
        let mut reported: Vec<(usize, usize)> = Vec::new();
        for (channel, level) in rungs {
            for unit in grounding_units(findings.file_structure.get(file), level) {
                if reported.contains(&(unit.start, unit.end))
                    || cited_lines
                        .iter()
                        .chain(declared_lines)
                        .any(|line| unit.start <= *line && *line <= unit.end)
                {
                    continue;
                }
                reported.push((unit.start, unit.end));
                let target = match channel {
                    Strength::Must => &mut report.errors,
                    Strength::Warn => &mut report.warnings,
                    _ => &mut report.suggestions,
                };
                target.push(Diagnostic {
                    code: "ungrounded",
                    path: Some(file.clone()),
                    line: Some(unit.start),
                    column: None,
                    message: format!(
                        "{}: no {} citation to a declared ID",
                        ungrounded_subject(&unit, place.as_deref()),
                        schema.citation.marker
                    ),
                    sites: Vec::new(),
                    authority: Vec::new(),
                });
            }
        }
    }
}

/// The rungs that ask `file` for citations, strongest first: the governing
/// row's effective pair as the `must` rung where it requires grounding, then
/// the v2 rungs below it (§FS-config-v2.rules.grounding). Empty where no row
/// governs the file or nothing grounds it; a v1 config has the pair alone.
fn governing_rungs(
    rules: &Rules,
    schema: &Schema,
    frame: Frame<'_>,
    home: Option<&DeclarationHome<'_>>,
    file: &Path,
) -> Vec<(Strength, usize)> {
    let Some((name, (require, level))) = governing_grounding(rules, schema, frame, home, file)
    else {
        return Vec::new();
    };
    require
        .then_some((Strength::Must, level))
        .into_iter()
        .chain(soft_grounding_rungs(rules, name))
        .collect()
}

/// The row that governs `file`, by name, and its effective
/// `(require_grounding, grounding_level)`, or `None` when no row does
/// (§FS-check.3.6.1). Three predicates, which are §FS-check.3.6's own: a
/// non-citable home governs every scanned file in it, a citable folder home
/// governs the source files in it, and the homeless kind governs the source
/// files no single home claims.
fn governing_grounding<'a>(
    rules: &Rules,
    schema: &'a Schema,
    frame: Frame<'_>,
    home: Option<&DeclarationHome<'_>>,
    file: &Path,
) -> Option<(&'a str, (bool, usize))> {
    let is_markdown = file.extension().and_then(|ext| ext.to_str()) == Some("md");
    let row_of = |kind: &str| {
        schema.rows.iter().find(|row| row.name == kind).map(|row| {
            (
                row.name.as_str(),
                row_grounding(rules, frame.run, &row.name),
            )
        })
    };
    match home {
        Some(home) if !home.citable => row_of(home.kind),
        // A Markdown document is not implementation, so a citable home and the
        // homeless complement both leave it alone; the exception above is a home
        // rather than an extension.
        _ if is_markdown => None,
        Some(home) => row_of(home.kind),
        None => Some((
            schema.complement_name(),
            homeless_row_grounding(schema, rules, frame.run),
        )),
    }
}

/// The units one file offers at `level` (§FS-check.3.6.2). Each level contains
/// the one below it, so the file is always a unit and nothing passes vacuously
/// for lacking structure; above level 1 the recorded structure adds the heading
/// subtrees of a Markdown file, or the doc-comment blocks of a source file.
fn grounding_units(structure: Option<&FileStructure>, level: usize) -> Vec<GroundingUnit> {
    let end = structure.map_or(usize::MAX, |structure| structure.total_lines.max(1));
    let mut units = vec![GroundingUnit {
        start: 1,
        end,
        subject: GroundingSubject::File,
    }];
    let Some(structure) = structure.filter(|_| level > DEFAULT_GROUNDING_LEVEL) else {
        return units;
    };
    for (index, heading) in structure.headings.iter().enumerate() {
        if heading.level < 2 || heading.level > level {
            continue;
        }
        // The subtree runs to the line before the next heading at the same or a
        // higher level, which is what makes a parent satisfied by any descendant
        // and a leaf answerable for itself.
        let next = structure.headings[index + 1..]
            .iter()
            .find(|later| later.level <= heading.level)
            .map(|later| later.line - 1)
            .unwrap_or(structure.total_lines);
        units.push(GroundingUnit {
            start: heading.line,
            end: next.max(heading.line),
            subject: GroundingSubject::Section(format!(
                "{} {}",
                "#".repeat(heading.level),
                heading.text
            )),
        });
    }
    for block in &structure.doc_comments {
        // §FS-check.3.6.2.2: level 2 reaches the unindented blocks — the parse-free
        // stand-in for a top-level item — and any higher level reaches them all.
        if block.indented && level == 2 {
            continue;
        }
        units.push(GroundingUnit {
            start: block.start,
            end: block.end,
            subject: GroundingSubject::DocComment,
        });
    }
    units
}

/// The line spans one file offers as units at `level` (§FS-check.3.11.3) — the
/// same cut §FS-check.3.6.2 makes for grounding, because *whether* a place's
/// files must cite and *what* they must cite are asked of the same thing.
fn grounding_unit_spans(structure: Option<&FileStructure>, level: usize) -> Vec<(usize, usize)> {
    grounding_units(structure, level)
        .into_iter()
        .map(|unit| (unit.start, unit.end))
        .collect()
}

/// The obligation units of one citing kind that has no declarations to attach
/// an obligation to — the homeless kind and every non-citable one
/// (§FS-check.3.11). Their unit is the file, cut by the row's
/// `grounding_level` exactly as the grounding pass cuts it, so *whether* a
/// place's files must cite and *what* they must cite are asked of one thing. A
/// unit carrying no citation is not a unit: obligations constrain what a file
/// cites, never whether it cites at all. At level 1 — every configuration
/// written before the key existed — this is one unit per file at line 1.
pub(super) fn file_obligation_units<'a>(
    citing_kind: &str,
    rules: &Rules,
    schema: &Schema,
    findings: &'a Catalog,
    by_file: &BTreeMap<(&'a str, &'a Path), Vec<&'a Citation>>,
) -> Vec<ObligationUnit<'a>> {
    let place = schema
        .rows
        .iter()
        .find(|row| row.name == citing_kind)
        .and_then(Row::place_label);
    let level = grounding_level_for_kind(schema, rules, citing_kind);
    by_file
        .iter()
        .filter(|((kind, _), _)| *kind == citing_kind)
        .flat_map(|((_, file), citations)| {
            grounding_unit_spans(findings.file_structure.get(*file), level)
                .into_iter()
                .filter_map(|(start, end)| {
                    let inside: Vec<&Citation> = citations
                        .iter()
                        .copied()
                        .filter(|cite| start <= cite.line && cite.line <= end)
                        .collect();
                    (!inside.is_empty()).then(|| ObligationUnit {
                        id: None,
                        place: place.clone(),
                        path: file.to_path_buf(),
                        line: start,
                        citations: inside,
                        e2e_spec_refs: Vec::new(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// How an ungrounded unit is named (§FS-check.3.6.3): the unit, then the
/// non-citable home it sits in when it sits in one. A section unit only ever
/// arises inside such a home, since that is the only place Markdown is governed.
fn ungrounded_subject(unit: &GroundingUnit, place: Option<&str>) -> String {
    let subject = match (&unit.subject, place) {
        (GroundingSubject::File, None) => "ungrounded source file".to_string(),
        (GroundingSubject::File, Some(_)) => "ungrounded file".to_string(),
        (GroundingSubject::Section(heading), _) => format!("ungrounded section `{heading}`"),
        (GroundingSubject::DocComment, _) => "ungrounded doc-comment".to_string(),
    };
    match place {
        Some(place) => format!("{subject} in kind home {place}"),
        None => subject,
    }
}
