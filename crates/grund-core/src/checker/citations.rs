//! The citation-direction half of the checker (§FS-config.3.9), in a file of its
//! own beside `references.rs` and `sections.rs`
//! (§AR-core-module-layout.1): the obligation pass (§FS-check.3.11) and the
//! two questions it shares with the prohibition pass of
//! `citation_prohibitions.rs` (§FS-check.3.12) — what kind of place a citation
//! sits in, and whether it matches a rule's target. `report.rs` keeps the
//! declaration-shape rules and the diagnostic helpers they share.

use std::collections::BTreeMap;
use std::path::Path;

use super::obligation_units::{
    ObligationUnit, file_is_obligation_unit, non_citable_kind_names, obligation_units,
};
use crate::config::{
    CitationDisjunction, CitationTarget, Frame, KindCitationRules, NamespaceMatch, Row, Rules,
    Schema, render_citation_target, row_grounding,
};
use crate::model::{
    Catalog, CheckReport, Citation, Diagnostic, E2eSpecRef, Id, paths_same_location,
};
use crate::scanner::file_home_kind;

/// How a citing kind is named in a finding (§FS-check.3.11, §FS-check.3.12.1): a
/// citable kind by its name, which is the prefix of every ID in it; a
/// non-citable one by its home, which is all a reader of the message could go
/// and look at. `code` keeps its own name — it is the one non-citable kind with
/// no place, being the complement of every home there is.
pub(super) fn citing_side_label(schema: &Schema, kind: &str) -> String {
    schema
        .rows
        .iter()
        .find(|row| row.name == kind && row.kind.is_none())
        .and_then(Row::place_label)
        .unwrap_or_else(|| kind.to_string())
}

/// §AR-checker.2.9 / §FS-check.3.11: every top-level declaration of a citing
/// kind with a `must` / `should` obligation must carry, in its body, a citation
/// satisfying each obligation entry. `must` misses are `missing-citation`
/// errors; `should` misses are `suggested-citation` suggestions.
///
/// Why the citation indexes are built up front: the per-declaration and per-case
/// rescans they replace were O(kinds × declarations × citations) and dominated
/// `grund check` on a large tree.
pub(super) fn check_citation_obligations(
    findings: &Catalog,
    rules: &Rules,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    // Index every citation once, up front, so each citing kind's obligation pass
    // is a map lookup rather than a fresh O(citations) scan per declaration
    // (§AR-benchmarks).
    let mut by_decl: BTreeMap<&Id, Vec<&Citation>> = BTreeMap::new();
    let mut by_file: BTreeMap<(&str, &Path), Vec<&Citation>> = BTreeMap::new();
    // Resolved once, not per citation: the per-file question below is asked of
    // every citation in the tree, and answering it by scanning `[[kinds]]` each
    // time would make the pass O(citations × kinds) for no gain (§AR-benchmarks).
    let non_citable = non_citable_kind_names(schema);
    let homeless = schema.complement_name();
    for cite in &findings.citations {
        if let Some(id) = &cite.enclosing_declaration {
            by_decl.entry(id).or_default().push(cite);
        }
        if file_is_obligation_unit(&non_citable, homeless, cite) {
            by_file
                .entry((cite.source_kind.as_str(), cite.file.as_path()))
                .or_default()
                .push(cite);
        }
    }
    // Bucket the (usually zero — fixture trees are carved out of `[scan]`)
    // citations that live under an E2E case directory to their nearest case, so
    // the E2E obligation never `starts_with`-scans every citation per case.
    let mut e2e_by_case: BTreeMap<&Path, Vec<&Citation>> = BTreeMap::new();
    for decls in findings.declarations.values() {
        for decl in decls {
            if decl.e2e_case.is_some() {
                e2e_by_case.entry(decl.file.as_path()).or_default();
            }
        }
    }
    if !e2e_by_case.is_empty() {
        for cite in &findings.citations {
            for ancestor in cite.file.ancestors() {
                if let Some(bucket) = e2e_by_case.get_mut(ancestor) {
                    bucket.push(cite);
                    break;
                }
            }
        }
    }

    for (citing_kind, kind_rules) in &rules.citations.per_kind {
        if kind_rules.must.is_empty() && kind_rules.warn.is_empty() && kind_rules.should.is_empty()
        {
            continue;
        }
        let units = obligation_units(
            citing_kind,
            rules,
            schema,
            findings,
            &by_decl,
            &by_file,
            &e2e_by_case,
        );
        // §FS-check.2.2.1.1: a walked folder with real non-entry content must not
        // silently pass when this obligation has no unit to evaluate.
        if units.is_empty()
            && let Some(warning) = empty_citation_obligation_warning(
                rules,
                schema,
                frame,
                findings,
                citing_kind,
                kind_rules,
            )
        {
            report.warnings.push(warning);
        }
        for unit in units {
            for entry in &kind_rules.must {
                if !entry.targets.iter().any(|t| unit.satisfies(t)) {
                    report.errors.push(obligation_diagnostic(
                        "missing-citation",
                        frame,
                        &unit,
                        entry,
                        "must",
                    ));
                }
            }
            // §FS-config-v2.rules.strengths: `warn` is the `must` finding, code
            // and message, on the warning channel.
            for entry in &kind_rules.warn {
                if !entry.targets.iter().any(|t| unit.satisfies(t)) {
                    report.warnings.push(obligation_diagnostic(
                        "missing-citation",
                        frame,
                        &unit,
                        entry,
                        "must",
                    ));
                }
            }
            for entry in &kind_rules.should {
                if !entry.targets.iter().any(|t| unit.satisfies(t)) {
                    report.suggestions.push(obligation_diagnostic(
                        "suggested-citation",
                        frame,
                        &unit,
                        entry,
                        "should",
                    ));
                }
            }
        }
    }
}

/// §FS-check.2.2.1.1: identify the one run-level warning for a walked folder whose
/// explicit obligation has real scanned content but no ordinary obligation unit.
/// The warning is derived from the same normalized home classifier the scanner
/// uses for citation-source attribution, so explicit paths and symlink spellings
/// stay inside the same home boundary.
fn empty_citation_obligation_warning(
    rules: &Rules,
    schema: &Schema,
    frame: Frame<'_>,
    findings: &Catalog,
    citing_kind: &str,
    kind_rules: &KindCitationRules,
) -> Option<Diagnostic> {
    let row = schema.rows.iter().find(|row| row.name == citing_kind)?;
    let folder = row.folder()?;
    let scanned = row.places.first().is_none_or(|place| place.scanned);
    if !scanned
        || !findings.scanned_files.iter().any(|file| {
            file_home_kind(file, schema, frame).as_deref() == Some(citing_kind)
                && !kind_entry_file(file, frame, row)
        })
    {
        return None;
    }

    let level = if !kind_rules.must.is_empty() {
        "must"
    } else if !kind_rules.warn.is_empty() {
        "warn"
    } else {
        "should"
    };
    let place = format!("{folder}/");
    // §FS-config-v2.rules.citations: v2 names the table, and the row key, its own way.
    let v2 = frame.version == 2;
    let table = if v2 { "rules.citations" } else { "citations" };
    let message = if row.kind.is_some() {
        format!(
            "[{table}.{citing_kind}] {level} applies to nothing — {place} declares no {citing_kind} ID; did you mean `citable = false`?"
        )
    } else {
        // §FS-check.2.2.1.2: the row-key half is advice, so it is given only where it is
        // still advice — a row already grounding (§FS-config.3.4.8) has made that
        // setting, and this run is already reporting what it caught (§FS-check.3.6).
        let tail = if row_grounding(rules, frame.run, &row.name).0 {
            String::new()
        } else if v2 {
            format!(
                "; set `must = \"file\"` in [rules.citations.{citing_kind}.grounding] to make that an error"
            )
        } else {
            format!("; set require_grounding = true on the {place} row to make that an error")
        };
        format!(
            "[{table}.{citing_kind}] {level} applies to nothing — no scanned file in {place} carries a citation{tail}"
        )
    };
    Some(Diagnostic {
        code: "empty-citation-obligation",
        path: None,
        line: None,
        column: None,
        message,
        sites: Vec::new(),
        authority: Vec::new(),
    })
}

/// Whether `file` is the entry file excluded from a folder's content count:
/// the effective citable index, or literal `README.md` for a non-citable home
/// (§FS-check.2.2.1.1). `index = false` naturally has no entry path.
fn kind_entry_file(file: &Path, frame: Frame<'_>, row: &Row) -> bool {
    let entry = if row.kind.is_some() {
        row.index_path()
    } else {
        row.folder()
            .map(|folder| Path::new(folder).join("README.md"))
    };
    entry.is_some_and(|entry| paths_same_location(file, &frame.root().join(entry)))
}

fn obligation_diagnostic(
    code: &'static str,
    frame: Frame<'_>,
    unit: &ObligationUnit<'_>,
    entry: &CitationDisjunction,
    verb_level: &str,
) -> Diagnostic {
    Diagnostic {
        code,
        path: Some(unit.path.clone()),
        line: Some(unit.line),
        column: None,
        message: format!(
            "{} {verb_level} cite {} (citation direction)",
            unit.subject(frame.grammar()),
            render_target_phrase(entry)
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// Whether a citation matches a rule target: same cited kind, and a namespace
/// qualifier that covers the citation's namespace (§FS-config.3.9.3).
pub(super) fn citation_matches_target(cite: &Citation, target: &CitationTarget) -> bool {
    if cite.id.kind != target.kind {
        return false;
    }
    match &target.namespace {
        NamespaceMatch::Any => true,
        NamespaceMatch::Local => cite.namespace.is_none(),
        NamespaceMatch::Alias(alias) => cite.namespace.as_deref() == Some(alias.as_str()),
    }
}

pub(super) fn e2e_spec_ref_matches_target(spec_ref: &E2eSpecRef, target: &CitationTarget) -> bool {
    if spec_ref.kind != target.kind {
        return false;
    }
    match &target.namespace {
        NamespaceMatch::Any => true,
        NamespaceMatch::Local => spec_ref.namespace.is_none(),
        NamespaceMatch::Alias(alias) => spec_ref.namespace.as_deref() == Some(alias.as_str()),
    }
}

/// Render a disjunction as a human phrase for a finding message: kinds joined by
/// " or " (§FS-init.2.3.5.4 uses the same phrasing in the agent entrypoint).
fn render_target_phrase(entry: &CitationDisjunction) -> String {
    entry
        .targets
        .iter()
        .map(render_citation_target)
        .collect::<Vec<_>>()
        .join(" or ")
}
