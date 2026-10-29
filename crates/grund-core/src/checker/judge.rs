//! §AR-checker.1.2: `judge`, the relational half of the checker — what one fact
//! says about another. It reads `Schema` only because a rule is written in its
//! vocabulary, and where a verdict depends on bytes presentation decides it
//! compares `Expected` and renders nothing (§AR-checker.1.3).

use crate::model::remedies;
use std::collections::{BTreeMap, BTreeSet};

use super::agents::check_agents_block_version;
use super::citation_prohibitions::check_citation_prohibitions;
use super::citations::check_citation_obligations;
use super::grounding::check_grounding;
use super::homes::KindHomeIndex;
use super::index::check_kind_indexes;
use super::index_entries::KindIndexEntries;
use super::references::{ReferenceTier, check_citation_resolution};
use super::section_lock::check_section_locks;
use super::snapshots::arm_missing_snapshot;
use super::values::check_values;
use crate::config::{Frame, KindResolution, Rules, Schema};
use crate::grammar::render_catalog_id;
use crate::model::{
    Catalog, CheckReport, Code, Diagnostic, Expected, Id, paired_declarations, sort_path_key,
};
use crate::resolver::{WorkspaceCheckTarget, judge_section_locks, resolved_coordinate_target};

/// The workspace a project is judged in (§FS-workspace.4): every loaded
/// project as a citation resolves against it, and each one's rules by alias.
/// The rules ride beside the targets because a target is records the resolver
/// holds, and a qualified citation's `resolve` policy is a rule of its target's
/// (§FS-check.4.12, §AR-config.5).
pub(crate) struct CheckWorkspace<'a> {
    pub(crate) targets: &'a BTreeMap<String, WorkspaceCheckTarget<'a>>,
    pub(crate) rules: BTreeMap<&'a str, &'a Rules>,
}

impl<'a> CheckWorkspace<'a> {
    /// The workspace of `targets`, each alias's rules looked up in `rules`.
    pub(crate) fn new(
        targets: &'a BTreeMap<String, WorkspaceCheckTarget<'a>>,
        rules: impl IntoIterator<Item = (&'a str, &'a Rules)>,
    ) -> Self {
        Self {
            targets,
            rules: rules.into_iter().collect(),
        }
    }

    /// A project judged alone: no target beyond its own.
    pub(crate) fn alone(targets: &'a BTreeMap<String, WorkspaceCheckTarget<'a>>) -> Self {
        Self::new(targets, [])
    }
}

/// §AR-checker.1.2: the managed-block comparison, resolution, explicit values,
/// escaped citations that resolve, kind indexes, unused declarations, unclosed
/// wrappers, grounding
/// and citation directions, each pass in the order the single driver ran it.
/// Unsorted: the merge sorts (§AR-checker.1.4).
pub(crate) fn judge(
    rules: &Rules,
    schema: &Schema,
    catalog: &Catalog,
    expected: &Expected,
    frame: Frame<'_>,
    workspace: &CheckWorkspace<'_>,
) -> CheckReport {
    let mut report = CheckReport::default();
    let findings = catalog;
    let targets = workspace.targets;
    // §FS-check.3.5: managed agent-entrypoint blocks that are out of date (or
    // newer than this binary), or whose generated sections have drifted from
    // `Expected`'s bytes (§AR-checker.2.7), are check errors.
    check_agents_block_version(expected, frame, &mut report);

    // §FS-check.3.1 / §FS-check.3.2 / §FS-check.3.8 / §FS-check.3.13: the
    // reference-resolution family, in `references.rs` (§AR-checker.2.13,
    // §FS-check.3.14) because `check --full` reruns it outside `[scan] include`.

    // §FS-check.4.12: a kind's `resolve` policy is its own project's rule.
    let resolve = |namespace: Option<&str>, kind: &str| -> Option<KindResolution> {
        let rules = match namespace {
            Some(alias) => *workspace.rules.get(alias)?,
            None => rules,
        };
        rules.resolution.get(kind).copied()
    };
    // §FS-check.1.4.1: the warning is on where the pass below can choose it.
    arm_missing_snapshot(schema, targets, &resolve, &mut report.armed);
    // §FS-section-lock: one judgement of the locks, against `targets` by alias
    // (§FS-section-lock.file.members), that the lock rules report and a missing
    // section reads where its text went (§FS-section-lock.checks.missing-section).
    let locks = judge_section_locks(findings, schema, frame, targets);
    check_citation_resolution(
        findings,
        schema,
        frame,
        targets,
        &resolve,
        &locks,
        ReferenceTier::Configured,
        None,
        &mut report,
    );
    // §AR-checker.2.18 / §FS-values.5: ordinary resolution runs first and the
    // focused pass suppresses comparison at every unresolved or ambiguous site.
    check_values(findings, schema, frame, targets, &mut report);
    // §FS-section-lock.checks: the lock against the tree, in `section_lock.rs`.
    check_section_locks(&locks, frame, &mut report);

    // §FS-check.2.3.1 / §AR-checker.2.11: a `<§>`-escaped illustration whose coordinate
    // resolves is likely a live citation bracketed by mistake; its live form keeps the
    // section, in the separator of the project that reads it (§FS-check.2.3.1.3).
    for esc in &findings.escaped_citations {
        // §FS-config.3.2.8: the suggested live form names the selected declaration.
        let Some(target) = resolved_coordinate_target(esc, findings, schema, frame, targets) else {
            continue;
        };
        let section = esc
            .section
            .as_deref()
            .map(|section| format!("{}{section}", target.schema.ids.section_separator))
            .unwrap_or_default();
        report.suggestions.push(Diagnostic {
            code: Code::ESCAPED_CITATION_RESOLVES,
            path: Some(esc.file.clone()),
            line: Some(esc.line),
            column: Some(esc.column),
            message: format!(
                "escaped citation {} resolves to a declaration; {}",
                esc.text.trim(),
                remedies::escaped_citation_resolves(format!(
                    "{}{}{section}",
                    schema.citation.marker,
                    super::support::qualify(
                        esc.namespace.as_deref(),
                        &render_catalog_id(
                            target.frame().grammar(),
                            &target.catalog.declarations,
                            &esc.id
                        )
                    )
                ))
            ),
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }

    // §FS-check.3.18 / §FS-check.3.17: a kind's index must list every declaration
    // in its folder, as a full link. In `index.rs` — one file per
    // invariant family, the arrangement §AR-checker.2.15's section rules already use.
    check_kind_indexes(
        findings,
        schema,
        frame,
        &expected.index_targets,
        &mut report,
    );

    // One read of every index for the unused rule, which tells an entry apart
    // (§AR-checker.2.6).
    let index_entries = KindIndexEntries::new(findings, schema, frame, &expected.index_targets);
    check_unused(findings, frame, &index_entries, targets, &mut report);

    // §FS-init.2.3.5.11: a scoped `AGENTS.md` no row's `files` lists is not a
    // unit, so neither pass below holds it to its directory's rules.
    let mut units = CheckReport::default();

    // §FS-check.3.6 / §DF-require-grounding: the grounding pass, per `[[kinds]]`
    // row and per unit, in `grounding.rs` (§AR-checker.2.8).
    let kind_homes = KindHomeIndex::new(schema, frame);
    check_grounding(
        findings,
        rules,
        schema,
        frame,
        &kind_homes,
        targets,
        &mut units,
    );

    // §FS-config.3.9 / §FS-check.3.11 / §FS-check.3.12: citation-direction
    // obligations and prohibitions, when the project declares `[citations]`.
    if rules.citations.declared {
        check_citation_obligations(findings, rules, schema, frame, &mut units);
        check_citation_prohibitions(findings, rules, schema, frame, &mut units);
    }
    let unit = |diagnostic: &Diagnostic| {
        diagnostic
            .path
            .as_ref()
            .is_none_or(|path| !expected.non_units.contains(path))
    };
    report.errors.extend(units.errors.into_iter().filter(unit));
    report
        .warnings
        .extend(units.warnings.into_iter().filter(unit));
    report
        .suggestions
        .extend(units.suggestions.into_iter().filter(unit));
    report.armed.absorb(&mut units.armed);
    report
}

/// §FS-check.4.1: a declaration nothing cites is a warning, not an error —
/// except E2E cases, which are proof artifacts, not citation targets. An
/// index entry is not an inbound citation (§DF-index-not-an-inbound-citation).
/// The warning sits at the first of the ID's declarations once its stubs are
/// paired by their verdicts (§AR-scanner.4.6), at its line in a stub's target
/// wherever that target sits (§FS-declarations.stubs.pairing.4), and a citation a
/// stub's target lends is use (§FS-refs.2, §FS-declarations.stubs.target.5).
fn check_unused(
    findings: &Catalog,
    frame: Frame<'_>,
    index_entries: &KindIndexEntries,
    targets: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    report: &mut CheckReport,
) {
    // §FS-check.4.1.2: retain the discounted sites for the warning's explanation.
    let mut excluded: BTreeMap<&Id, Vec<_>> = BTreeMap::new();
    let mut cited = BTreeSet::new();
    for citation in findings.edges().filter(|cite| cite.namespace.is_none()) {
        if index_entries.is_index_entry(citation) {
            excluded.entry(&citation.id).or_default().push(citation);
        } else {
            cited.insert(&citation.id);
        }
    }
    if let Some(alias) = frame.alias {
        for target in targets.values() {
            cited.extend(
                target
                    .catalog
                    .edges()
                    .filter(|cite| cite.namespace.as_deref() == Some(alias))
                    .map(|cite| &cite.id),
            );
        }
    }
    for (id, decls) in &findings.declarations {
        if id.kind == "E2E" {
            continue;
        }
        if !cited.contains(id)
            && let Some(decl) = paired_declarations(decls).iter().next().map(|p| p.record)
        {
            report.warnings.push(Diagnostic {
                code: Code::UNUSED,
                path: Some(decl.file.clone()),
                line: Some(decl.line),
                column: None,
                message: {
                    let mut message = format!(
                        "declared but never cited: {}",
                        render_catalog_id(frame.grammar(), &findings.declarations, id)
                    );
                    // §FS-check.4.1.2 / §FS-errors.3.1: bounded, deterministic sites,
                    // spelled against the same report base as the finding's anchor.
                    if let Some(sites) = excluded.get_mut(id) {
                        sites.sort_by_key(|cite| {
                            (sort_path_key(&cite.file), cite.line, cite.column)
                        });
                        let first = sites[0];
                        let more = if sites.len() > 1 {
                            format!(" and {} more", sites.len() - 1)
                        } else {
                            String::new()
                        };
                        message.push_str(&format!(
                            " (its index entry at {}:{}{more} does not count as a use)",
                            frame.display_path(&first.file),
                            first.line
                        ));
                    }
                    message.push_str(&format!("; {}", remedies::unused()));
                    message
                },
                sites: Vec::new(),
                authority: Vec::new(),
            });
        }
    }
}
