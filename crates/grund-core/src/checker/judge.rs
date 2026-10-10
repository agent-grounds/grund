//! §AR-checker.1.2: `judge`, the relational half of the checker — what one fact
//! says about another. It reads `Schema` only because a rule is written in its
//! vocabulary, and where a verdict depends on bytes presentation decides it
//! compares `Expected` and renders nothing (§AR-checker.1.3).

use std::collections::{BTreeMap, BTreeSet};

use super::agents::check_agents_block_version;
use super::citation_prohibitions::check_citation_prohibitions;
use super::citations::check_citation_obligations;
use super::grounding::check_grounding;
use super::homes::KindHomeIndex;
use super::index::check_kind_indexes;
use super::index_entries::KindIndexEntries;
use super::references::{ReferenceTier, check_citation_resolution};
use super::values::check_values;
use crate::config::{Frame, KindResolution, Rules, Schema};
use crate::grammar::{render_id, render_qualified_id};
use crate::model::{Catalog, CheckReport, Diagnostic, Expected, Id, paired_declarations};
use crate::resolver::{WorkspaceCheckTarget, citation_resolves};

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
/// escaped citations that resolve, kind indexes, unused declarations, grounding
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
    check_agents_block_version(expected, &mut report);

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
    check_citation_resolution(
        findings,
        schema,
        frame,
        targets,
        &resolve,
        ReferenceTier::Configured,
        None,
        &mut report,
    );
    // §AR-checker.2.18 / §FS-values.5: ordinary resolution runs first and the
    // focused pass suppresses comparison at every unresolved or ambiguous site.
    check_values(findings, schema, frame, targets, &mut report);

    // §FS-check.2.3.1 / §AR-checker.2.11: a `<§>`-escaped illustration whose ID
    // resolves to a real declaration is likely a live citation someone bracketed
    // by mistake — the escape silently makes it inert.
    for esc in &findings.escaped_citations {
        if citation_resolves(esc, findings, schema, frame, targets) {
            report.suggestions.push(Diagnostic {
                code: "escaped-citation-resolves",
                path: Some(esc.file.clone()),
                line: Some(esc.line),
                column: Some(esc.column),
                message: format!(
                    "escaped citation {} resolves to a declaration; write {}{} for a live citation, or leave it escaped if it is only an illustration",
                    esc.text.trim(),
                    schema.citation.marker,
                    render_qualified_id(frame.grammar(), esc.namespace.as_deref(), &esc.id)
                ),
                sites: Vec::new(),
                authority: Vec::new(),
            });
        }
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

    check_unused(findings, schema, frame, expected, targets, &mut report);

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
        &mut report,
    );

    // §FS-config.3.9 / §FS-check.3.11 / §FS-check.3.12: citation-direction
    // obligations and prohibitions, when the project declares `[citations]`.
    if rules.citations.declared {
        check_citation_obligations(findings, rules, schema, frame, &mut report);
        check_citation_prohibitions(findings, rules, schema, &mut report);
    }
    report
}

/// §FS-check.4.1: a declaration nothing cites is a warning, not an error —
/// except E2E cases, which are proof artifacts, not citation targets. An
/// index entry is not an inbound citation (§DF-index-not-an-inbound-citation).
/// The warning sits at the first stand-in of the ID's declarations once its stubs
/// are paired by their verdicts (§FS-declarations.stubs.verdict).
fn check_unused(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    expected: &Expected,
    targets: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    report: &mut CheckReport,
) {
    let index_entries = KindIndexEntries::new(findings, schema, frame, &expected.index_targets);
    let mut cited: BTreeSet<&Id> = findings
        .citations
        .iter()
        .filter(|cite| cite.namespace.is_none() && !index_entries.is_index_entry(cite))
        .map(|c| &c.id)
        .collect();
    if let Some(alias) = frame.alias {
        for target in targets.values() {
            cited.extend(
                target
                    .catalog
                    .citations
                    .iter()
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
            && let Some(decl) = paired_declarations(decls).stand_ins().next()
        {
            report.warnings.push(Diagnostic {
                code: "unused",
                path: Some(decl.file.clone()),
                line: Some(decl.line),
                column: None,
                message: format!(
                    "declared but never cited: {}",
                    render_id(frame.grammar(), id)
                ),
                sites: Vec::new(),
                authority: Vec::new(),
            });
        }
    }
}
