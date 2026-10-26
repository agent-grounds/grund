//! The citation-direction half of the checker (§FS-config.3.9), in a file of its
//! own beside `references.rs` and `sections.rs`
//! (§AR-core-module-layout.1): the obligation pass (§FS-check.3.11) and the
//! two questions it shares with the prohibition pass of
//! `citation_prohibitions.rs` (§FS-check.3.12) — what kind of place a citation
//! sits in, and whether it matches a rule's target. `report.rs` keeps the
//! declaration-shape rules and the diagnostic helpers they share.

use crate::model::remedies;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::obligation_units::{
    ObligationUnit, file_is_obligation_unit, non_citable_kind_names, obligation_units,
};
use crate::config::{
    CitationDisjunction, CitationLevel, CitationTarget, Frame, KindCitationRules, NamespaceMatch,
    Owners, Place, Row, Rules, Schema, citations_table, render_citation_target, row_grounding,
};
use crate::model::{Catalog, CheckReport, Citation, Code, Diagnostic, E2eSpecRef, Id, Switch};
use crate::rules::engine::one_rules_authority;
use crate::scanner::file_home_kind;
use crate::sources::paths_same_location;

/// How a citing kind is named in a finding (§FS-check.3.11, §FS-check.3.12.1): a
/// citable kind by its name, which is the prefix of every ID in it; a
/// non-citable one by its home, which is all a reader of the message could go
/// and look at. `code` keeps its own name — it is the complement of every home
/// there is, its own places included (§FS-config-v2.schema.places.complement).
pub(super) fn citing_side_label(owners: &Owners<'_>, kind: &str, file: &Path) -> String {
    citing_place(owners, kind, file).unwrap_or_else(|| kind.to_string())
}

/// The place a non-citable, non-complement citing kind is named by in a finding
/// about `file`: the place of it that owns the file, one row having several
/// (§FS-config-v2.schema.places.ownership), else its first. `None` for a kind
/// that is named by its name.
pub(super) fn citing_place(owners: &Owners<'_>, kind: &str, file: &Path) -> Option<String> {
    let row = owners
        .schema()
        .rows
        .iter()
        .find(|row| row.name == kind && row.kind.is_none() && !row.is_complement())?;
    owners
        .of(file)
        .filter(|owner| owner.row.name == kind)
        .map(|owner| owner.label())
        .or_else(|| row.places.first().and_then(Place::label))
}

/// The channel a direction finding is reported on.
#[derive(Clone, Copy)]
pub(super) enum DirectionChannel {
    Errors,
    Warnings,
    Suggestions,
}

/// Where the direction passes put each finding, with the citing kind, the level
/// and the entries it is reported under: the report, or the config-to-rule
/// pairing that asks which entry reported what (§FS-rules.6). A finding no
/// entry stands behind passes none.
pub(super) trait DirectionSink {
    fn push(
        &mut self,
        channel: DirectionChannel,
        kind: &str,
        level: CitationLevel,
        entries: &[&CitationDisjunction],
        diagnostic: Diagnostic,
    );

    /// Record that a direction pass found a switch set (§FS-check.1.4.1). Only
    /// the report keeps the record; the pairing has no use for it.
    fn arm(&mut self, _switch: Switch) {}
}

impl DirectionSink for CheckReport {
    fn push(
        &mut self,
        channel: DirectionChannel,
        _: &str,
        _: CitationLevel,
        _: &[&CitationDisjunction],
        diagnostic: Diagnostic,
    ) {
        match channel {
            DirectionChannel::Errors => self.errors.push(diagnostic),
            DirectionChannel::Warnings => self.warnings.push(diagnostic),
            DirectionChannel::Suggestions => self.suggestions.push(diagnostic),
        }
    }

    fn arm(&mut self, switch: Switch) {
        self.armed.arm(switch);
    }
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
    report: &mut impl DirectionSink,
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
    let owners = Owners::new(schema, frame);
    // §FS-declarations.stubs.target.5: a declaration's citations are its edges, those a
    // stub's target lends included; a file's are the ones the walk found in it.
    for cite in findings.edges() {
        if let Some(id) = &cite.enclosing_declaration {
            by_decl.entry(id).or_default().push(cite);
        }
    }
    for cite in &findings.citations {
        let markdown = || frame.compiled.reading(schema, &cite.file).markdown;
        if file_is_obligation_unit(&non_citable, homeless, cite, markdown) {
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
        // §FS-check.1.4.1: past this gate the obligation checks are on, each level's own,
        // and the empty one where its kind can have no unit.
        if can_apply_to_nothing(schema, citing_kind, false).is_some() {
            report.arm(Switch::Obligation);
        }
        if !kind_rules.must.is_empty() || !kind_rules.warn.is_empty() {
            report.arm(Switch::MustDirection);
        }
        if !kind_rules.should.is_empty() {
            report.arm(Switch::ShouldDirection);
        }
        let units = obligation_units(
            citing_kind,
            rules,
            &owners,
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
                Obligor::Table(kind_rules),
            )
        {
            report.push(
                DirectionChannel::Warnings,
                citing_kind,
                CitationLevel::Must,
                &[],
                warning,
            );
        }
        for unit in units {
            for entry in &kind_rules.must {
                if !entry.targets.iter().any(|t| unit.satisfies(t)) {
                    let diagnostic =
                        obligation_diagnostic(Code::MISSING_CITATION, frame, &unit, entry, "must");
                    report.push(
                        DirectionChannel::Errors,
                        citing_kind,
                        CitationLevel::Must,
                        &[entry],
                        diagnostic,
                    );
                }
            }
            // §FS-config-v2.rules.strengths: `warn` is the `must` finding, code
            // and message, on the warning channel.
            for entry in &kind_rules.warn {
                if !entry.targets.iter().any(|t| unit.satisfies(t)) {
                    report.push(
                        DirectionChannel::Warnings,
                        citing_kind,
                        CitationLevel::Warn,
                        &[entry],
                        obligation_diagnostic(Code::MISSING_CITATION, frame, &unit, entry, "must"),
                    );
                }
            }
            for entry in &kind_rules.should {
                if !entry.targets.iter().any(|t| unit.satisfies(t)) {
                    report.push(
                        DirectionChannel::Suggestions,
                        citing_kind,
                        CitationLevel::Should,
                        &[entry],
                        obligation_diagnostic(
                            Code::SUGGESTED_CITATION,
                            frame,
                            &unit,
                            entry,
                            "should",
                        ),
                    );
                }
            }
        }
    }
}

/// Who holds a kind to an obligation (§FS-check.2.2.1): its `[citations]` table,
/// or a `must` / `should` rule by its origin (§FS-check.2.2.1.4).
pub(super) enum Obligor<'a> {
    Table(&'a KindCitationRules),
    Rule(&'a str),
}

/// §FS-check.2.2.1.1 condition 2: the row an obligation on `citing_kind` can find
/// with no unit, a scanned kind with a `folder` home, citable where a rule holds it
/// (§FS-check.2.2.1.2). The warning below and the switch that arms it both ask it
/// (§FS-check.1.4.1), so the two cannot disagree.
pub(super) fn can_apply_to_nothing<'a>(
    schema: &'a Schema,
    citing_kind: &str,
    by_rule: bool,
) -> Option<&'a Row> {
    let row = schema
        .rows
        .iter()
        .find(|row| row.name == citing_kind && !row.is_complement())?;
    let scanned = row.places.first().is_none_or(|place| place.scanned);
    let citable = !by_rule || row.kind.is_some();
    (row.folders().next().is_some() && scanned && citable).then_some(row)
}

/// §FS-check.2.2.1.1: identify the one run-level warning for a walked folder whose
/// explicit obligation has real scanned content but no ordinary obligation unit.
/// The warning is derived from the same normalized home classifier the scanner
/// uses for citation-source attribution, so explicit paths and symlink spellings
/// stay inside the same home boundary. The table and a rule ask it alike, so the
/// two never disagree about whether a folder is empty (§FS-check.2.2.1.4).
pub(super) fn empty_citation_obligation_warning(
    rules: &Rules,
    schema: &Schema,
    frame: Frame<'_>,
    findings: &Catalog,
    citing_kind: &str,
    obligor: Obligor<'_>,
) -> Option<Diagnostic> {
    let row = can_apply_to_nothing(schema, citing_kind, matches!(obligor, Obligor::Rule(_)))?;
    if !findings.scanned_files.iter().any(|file| {
        file_home_kind(file, schema, frame).as_deref() == Some(citing_kind)
            && !kind_entry_file(file, frame, row)
    }) {
        return None;
    }

    // §FS-config-v2.schema.places: every folder of the row, named together.
    let folders: Vec<String> = row.folders().map(|folder| format!("{folder}/")).collect();
    let place = folders.join(", ");
    let kind_rules = match obligor {
        Obligor::Table(kind_rules) => kind_rules,
        // §FS-check.2.2.1.2: a rule's subject is citable, so only the declaration
        // form reaches it, named by the rule and ending at the fact.
        Obligor::Rule(origin) => {
            return Some(Diagnostic {
                code: Code::EMPTY_CITATION_OBLIGATION,
                path: None,
                line: None,
                column: None,
                message: format!(
                    "{origin} applies to nothing — {place} declares no {citing_kind} ID"
                ),
                sites: Vec::new(),
                // §FS-check.2.2.1.4: the rule's own line, kept by `--only-rule` for `--rule`.
                authority: one_rules_authority(origin),
            });
        }
    };
    let level = if !kind_rules.must.is_empty() {
        "must"
    } else if !kind_rules.warn.is_empty() {
        "warn"
    } else {
        "should"
    };
    // §FS-config-v2.rules.citations: v2 names the table, and the row key, its own way.
    let table = citations_table(frame.version);
    let message = if row.kind.is_some() {
        format!(
            "[{table}.{citing_kind}] {level} applies to nothing — {place} declares no {citing_kind} ID; {}",
            remedies::empty_citation_obligation_citable(frame.version)
        )
    } else {
        // §FS-check.2.2.1.2: the row-key half is advice, so it is given only where it is
        // still advice — a row already grounding (§FS-config.3.4.8) has made that
        // setting, and this run is already reporting what it caught (§FS-check.3.6).
        let tail = if row_grounding(rules, frame.run, &row.name).0 {
            String::new()
        } else {
            // §FS-errors.5.5.1: the version's side of the row's grounding case.
            format!(
                "; {}",
                remedies::empty_citation_obligation_grounding(frame.version, citing_kind, &place)
            )
        };
        format!(
            "[{table}.{citing_kind}] {level} applies to nothing — no scanned file in {place} carries a citation{tail}"
        )
    };
    Some(Diagnostic {
        code: Code::EMPTY_CITATION_OBLIGATION,
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
    // §FS-config-v2.schema.places.ownership: each folder has its own entry file.
    let entries: Vec<PathBuf> = if row.kind.is_some() {
        row.index_paths()
            .into_iter()
            .map(|(_, index)| index)
            .collect()
    } else {
        row.folders()
            .map(|folder| Path::new(folder).join("README.md"))
            .collect()
    };
    entries
        .iter()
        .any(|entry| paths_same_location(file, &frame.root().join(entry)))
}

fn obligation_diagnostic(
    code: Code,
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
