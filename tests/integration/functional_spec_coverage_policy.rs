//! Exact-leaf coverage policy and reviewed exception tables (§AR-goal-measurement.1).

use grund_core::Catalog;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Eq, PartialEq)]
pub(super) enum CoverageProblem {
    UncoveredLeaf(String),
    DuplicateException(String),
    InvalidException(String),
    CoveredException(String),
    EmptyReason(String),
    DuplicateLeadPromise(String),
    InvalidLeadPromise(String),
    UncitedLeadPromise(String),
}

#[derive(Clone, Copy)]
pub(super) struct Exception<'a> {
    pub(super) id: &'a str,
    pub(super) reason: &'a str,
}

/// A parent whose lead holds a promise none of its children holds
/// (§REQ-spec-content.grain), with the promise in a few words.
#[derive(Clone, Copy)]
pub(super) struct LeadPromise<'a> {
    pub(super) id: &'a str,
    pub(super) reason: &'a str,
}

/// Compare the production catalog with exact test evidence and the two reviewed
/// exception tables (§AR-goal-measurement.1). An exception is valid only while
/// its point is an uncited leaf; that makes proof retire debt automatically.
pub(super) fn functional_spec_coverage_problems(
    catalog: &Catalog,
    evidence: &BTreeSet<&str>,
    permanent: &[Exception<'_>],
    temporary: &[Exception<'_>],
) -> Vec<CoverageProblem> {
    let sections = functional_spec_sections(catalog);
    let leaves = leaves_of(&sections);
    let exceptions = permanent.iter().chain(temporary);
    let mut occurrences = BTreeMap::<&str, usize>::new();
    for exception in exceptions.clone() {
        *occurrences.entry(exception.id).or_default() += 1;
    }

    let mut problems = occurrences
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(id, _)| CoverageProblem::DuplicateException((*id).to_string()))
        .collect::<Vec<_>>();
    for exception in exceptions.clone() {
        if !leaves.contains(exception.id) {
            problems.push(CoverageProblem::InvalidException(exception.id.to_string()));
        }
    }
    for exception in exceptions.clone() {
        if exception.reason.trim().is_empty() {
            problems.push(CoverageProblem::EmptyReason(exception.id.to_string()));
        }
    }
    for exception in exceptions.clone() {
        if leaves.contains(exception.id) && evidence.contains(exception.id) {
            problems.push(CoverageProblem::CoveredException(exception.id.to_string()));
        }
    }

    let excepted = exceptions
        .map(|exception| exception.id)
        .collect::<BTreeSet<_>>();
    problems.extend(
        leaves
            .difference(evidence)
            .filter(|id| !excepted.contains(**id))
            .map(|id| CoverageProblem::UncoveredLeaf((*id).to_string())),
    );
    problems
}

/// Hold each listed parent's lead to exact evidence (§AR-goal-measurement.1): a
/// citation of a child does not prove what only the parent's lead promises. An
/// entry that names no section, or a leaf the rule above holds already, is invalid.
pub(super) fn lead_promise_problems(
    catalog: &Catalog,
    evidence: &BTreeSet<&str>,
    promises: &[LeadPromise<'_>],
) -> Vec<CoverageProblem> {
    let sections = functional_spec_sections(catalog);
    let leaves = leaves_of(&sections);
    let mut listed = BTreeSet::new();
    let mut problems = Vec::new();
    for promise in promises {
        let id = promise.id.to_string();
        if !listed.insert(promise.id) {
            problems.push(CoverageProblem::DuplicateLeadPromise(id.clone()));
        } else if !sections.contains(promise.id) || leaves.contains(promise.id) {
            problems.push(CoverageProblem::InvalidLeadPromise(id.clone()));
        } else if !evidence.contains(promise.id) {
            problems.push(CoverageProblem::UncitedLeadPromise(id.clone()));
        }
        if promise.reason.trim().is_empty() {
            problems.push(CoverageProblem::EmptyReason(id));
        }
    }
    problems
}

/// The sections with no recorded descendant.
fn leaves_of(sections: &BTreeSet<String>) -> BTreeSet<&str> {
    sections
        .iter()
        .filter(|candidate| {
            let descendant_prefix = format!("{candidate}.");
            !sections
                .iter()
                .any(|section| section.starts_with(&descendant_prefix))
        })
        .map(String::as_str)
        .collect()
}

fn functional_spec_sections(catalog: &Catalog) -> BTreeSet<String> {
    let mut sections = BTreeSet::new();
    for (id, declarations) in &catalog.declarations {
        if id.kind != "FS" {
            continue;
        }
        let base = id
            .slug
            .as_deref()
            .map(|slug| format!("FS-{slug}"))
            .or_else(|| id.num.map(|number| format!("FS-{number}")))
            .expect("an FS declaration has the configured identifier component");
        for declaration in declarations {
            sections.extend(
                declaration
                    .sections
                    .keys()
                    .map(|section| format!("{base}.{section}")),
            );
        }
    }
    sections
}

#[rustfmt::skip]
pub(super) const PERMANENT_EXCEPTIONS: &[Exception<'static>] = &[
    Exception { id: "FS-check.3.3", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.4", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.7", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.9", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.16", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.19", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.23", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.4.6", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.4.7.9", reason: "pointer kept so released changelogs still resolve; the behaviour it named is retired, and the precedence that replaced it is proven at FS-check.3.30" },
    Exception { id: "FS-scan.path.warning.cost", reason: "pointer kept so released changelogs still resolve; the warning it named was retired in 0.19.0 by #621, and the answers it announced are proven at FS-scan.path.show, FS-scan.path.list and FS-scan.path.id" },
    Exception { id: "FS-scan.path.warning.release", reason: "pointer kept so released changelogs still resolve; the warning it named was retired in 0.19.0 by #621, and the answers it announced are proven at FS-scan.path.show, FS-scan.path.list and FS-scan.path.id" },
    Exception { id: "FS-scan.path.warning.shape", reason: "pointer kept so released changelogs still resolve; the warning it named was retired in 0.19.0 by #621, and the answers it announced are proven at FS-scan.path.show, FS-scan.path.list and FS-scan.path.id" },
    Exception { id: "FS-scan.projects.narrowed.alias", reason: "pointer kept so released changelogs still resolve; the refusal it annotated was retired in 0.19.0 by #621, and the alias resolving instead is proven at FS-scan.projects.narrowed" },
    Exception { id: "FS-check.4.13", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.4.14", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-rules.3.5.1", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-rule-refusals section it names, and this row retires with the pointer" },
    Exception { id: "FS-rules.3.5.2", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-rule-refusals section it names, and this row retires with the pointer" },
    Exception { id: "FS-rules.3.5.3", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-rule-refusals section it names, and this row retires with the pointer" },
    Exception { id: "FS-rules.3.5.4.1", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-rule-refusals section it names, and this row retires with the pointer" },
    Exception { id: "FS-rules.3.5.4.2", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-rule-refusals section it names, and this row retires with the pointer" },
    Exception { id: "FS-rules.3.5.4.3", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-rule-refusals section it names, and this row retires with the pointer" },
    Exception { id: "FS-rules.3.5.4.4", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-rule-refusals section it names, and this row retires with the pointer" },
    Exception { id: "FS-rules.3.5.4.5", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-rule-refusals section it names, and this row retires with the pointer" },
    Exception { id: "FS-rules.3.5.4.6", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-rule-refusals section it names, and this row retires with the pointer" },
    Exception { id: "FS-rules.3.5.5", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-rule-refusals section it names, and this row retires with the pointer" },
    Exception { id: "FS-declarations.checks.broken-stub.1", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-declarations.checks.broken-stub.2", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-declarations.checks.broken-stub.3", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-declarations.checks.duplicate.1", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-declarations.checks.duplicate.2", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-declarations.checks.duplicate.3", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.5", reason: "finding selection reference, not a behavioral requirement" },
    Exception { id: "FS-config.principle.install-local", reason: "boundary between committed and install-local state; the axis it names is proven under FS-integrations" },
    Exception { id: "FS-config.requirements.1", reason: "directional config-contract policy, binding the next key rather than describing a released behavior" },
    Exception { id: "FS-config.requirements.3", reason: "directional config-contract policy, binding the next key rather than describing a released behavior" },
    Exception { id: "FS-config.requirements.4", reason: "config-contract policy over every key at once, not a behavioral requirement one run can exhibit" },
    Exception { id: "FS-config.3.5.10", reason: "pointer kept so citations written before the move still resolve; the rationale it named is the one FS-scan.scope.homes cites, and this row retires with the pointer" },
    Exception { id: "FS-cover.5", reason: "coverage interpretation guidance" },
    Exception { id: "FS-declarations.why", reason: "rationale for gathering the declaration checks in one spec" },
    Exception { id: "FS-errors.7", reason: "error-design rationale" },
    Exception { id: "FS-examples.1", reason: "example-suite overview" },
    Exception { id: "FS-fmt.4", reason: "formatter non-goals" },
    Exception { id: "FS-id.7", reason: "ID proposal rationale" },
    Exception { id: "FS-id.8", reason: "ID proposal examples" },
    Exception { id: "FS-inline-citation-style.5.6", reason: "pointer kept so citations written before the whole-block comparison still resolve; the version rule it now names is proven at FS-init.2.3.7, and this row retires with the pointer" },
    Exception { id: "FS-inline-citation-style.6.2", reason: "explicit non-goal" },
    Exception { id: "FS-inline-citation-style.6.3", reason: "explicit non-goal" },
    Exception { id: "FS-inline-citation-style.6.4", reason: "explicit non-goal" },
    Exception { id: "FS-integrations.3.5", reason: "integration guidance" },
    Exception { id: "FS-list.5", reason: "catalog-query rationale" },
    Exception { id: "FS-lsp.5", reason: "LSP non-goals" },
    Exception { id: "FS-refs.5", reason: "reference-query rationale" },
    Exception { id: "FS-show.4", reason: "show-query rationale" },
    Exception { id: "FS-non-goals.2", reason: "explicit non-goal" },
    Exception { id: "FS-non-goals.4", reason: "explicit non-goal" },
    Exception { id: "FS-non-goals.5", reason: "explicit non-goal" },
    Exception { id: "FS-non-goals.6", reason: "explicit non-goal" },
    Exception { id: "FS-non-goals.7", reason: "explicit non-goal" },
    Exception { id: "FS-non-goals.8", reason: "explicit non-goal" },
    Exception { id: "FS-non-goals.10", reason: "explicit non-goal" },
    Exception { id: "FS-non-goals.11", reason: "explicit non-goal" },
    Exception { id: "FS-non-goals.12.1", reason: "explicit non-goal" },
    Exception { id: "FS-non-goals.12.2", reason: "explicit non-goal" },
    Exception { id: "FS-non-goals.14", reason: "explicit non-goal" },
    Exception { id: "FS-distribution.5", reason: "distribution target description" },
    Exception { id: "FS-terms.terms.1", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-terms.terms.2", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-terms.terms.3", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-terms.terms.4", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-terms.terms.5", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-terms.terms.6", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-terms.terms.7", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-terms.terms.8", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-terms.terms.9", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-check.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-citations.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-coordinate-globs.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-scan.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-cli.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-completions.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-config.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-config-v2.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-config-explain.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-cover.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-declarations.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-distribution-candidate.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-distribution.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-distribution.requirements.next-operation", reason: "directional binding-inventory policy, binding the next operation rather than describing a released behavior" },
    Exception { id: "FS-errors.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-examples.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-examples.requirements.verdict", reason: "a property of the shared e2e runner over every manifest; a case cannot exhibit its own refusal, and the runner's tests prove FS-examples.5.1, 5.3 and 5.4" },
    Exception { id: "FS-fetch.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-fields.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-fmt.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-fmt.requirements.next-class", reason: "directional formatter policy, binding the next rewrite class rather than describing a released behavior" },
    Exception { id: "FS-id.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-imports.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-init.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-init.requirements.next-agent", reason: "directional init policy, binding the next supported agent rather than describing a released behavior" },
    Exception { id: "FS-init-fixtures.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-inline-citation-style.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-integrations.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-languages.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-list.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-lsp.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-lsp.requirements.inputs", reason: "directional editor-surface policy, binding the next answer's inputs rather than describing a released behavior" },
    Exception { id: "FS-repository-maintenance.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-non-goals.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-output-shapes.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-refs.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-remote-projects.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-rule-refusals.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-rule-refusals.requirements.next-refusal", reason: "directional refusal policy, binding the next refusal rather than describing a released behavior" },
    Exception { id: "FS-rules.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-rules.requirements.addition", reason: "directional rule-language policy, binding the next family rather than describing a released behavior" },
    Exception { id: "FS-section-lock.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-show.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-workspace.terms", reason: "vocabulary, not a behavioral requirement" },
];

#[rustfmt::skip]
pub(super) const TEMPORARY_EXCEPTIONS: &[Exception<'static>] = &[
    Exception { id: "FS-config-v2.requirements.ahead", reason: "specified ahead of its release; each refused clause's owner proves it, and config-v2-unsupported-clause-refused pins the refusal meanwhile" },
    Exception { id: "FS-distribution.requirements.ecosystems", reason: "npm and PyPI publication is owed to RM-distribution; the cargo half is the release workflow, which no e2e case runs" },
    Exception { id: "FS-distribution.requirements.versions", reason: "the proof is split: cli-version shows the -dev half, test_check_release_ramps.py the release gate, and the shared tag version is the release workflow's" },
    Exception { id: "FS-examples.3", reason: "example explanation proof spans the runner and docs" },
    Exception { id: "FS-examples.4", reason: "example maintenance proof spans the runner and docs" },
    Exception { id: "FS-fetch.remote.all", reason: "specified ahead of its release; the Drift piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.check", reason: "specified ahead of its release; the Drift piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.commit", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.findings", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.from", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.install", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.selector", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-inline-citation-style.3.2", reason: "the multi-case note boundary needs a proof audit" },
    Exception { id: "FS-lsp.3", reason: "shared config parity across CLI and LSP needs a proof audit" },
    Exception { id: "FS-remote-projects.bytes", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.checks.remote-missing", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.checks.remote-modified", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.checks.remote-orphan", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.checks.remote-stale", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.differences", reason: "specified ahead of its release; the Remotes and Drift pieces of remote projects prove it" },
    Exception { id: "FS-remote-projects.failures.remote-lock-invalid", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.failures.remote-projection-unsupported", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.failures.remote-unreadable", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.lock", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.mount.local", reason: "specified ahead of its release; the Mount piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.projection", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-rules.requirements.phase-one", reason: "each phase-1 absence is owed to its own change, which proves it" },
    Exception { id: "FS-show.2.3.1.1", reason: "the backward open-boundary walk changes no CLI output, so the slice has no observable form to assert yet" },
    Exception { id: "FS-workspace.8.4.3", reason: "the single-project fallback this names is not what complete_ids_run does today; the wording and the code have to be reconciled first" },
    Exception { id: "FS-workspace.requirements.ahead", reason: "specified ahead of its release; the Mount and Remotes pieces of remote projects prove it, and the relaxed standalone mode is owed to its own change" },
];

// Parents whose leads keep a promise of their own; the gate demands a test citing each exactly.
// Sorted by coordinate; the fold pass appends a record's `lead-promises` here.
#[rustfmt::skip]
pub(super) const LEAD_PROMISES: &[LeadPromise<'static>] = &[
    LeadPromise { id: "FS-check.1.3", reason: "--full cancels include and the scan = false prune and nothing else; reads a superset, each file once; in-scope report unchanged, outside only 3.14, 3.24 and section-outside-declaration; unused warning kept, widening include is what counts the edge; each workspace project widened against its own scope; empty-scan caution kept; a flag, never a key" },
    LeadPromise { id: "FS-check.3.5", reason: "which entrypoints are verified, legacy and malformed blocks, the whole-block comparison and its two exemptions, one stale finding per section with its noun and source, and the workspace entry detail" },
    LeadPromise { id: "FS-check.3.6", reason: "a finding's location and message" },
    LeadPromise { id: "FS-check.3.13", reason: "a docstring's content is judged; at most one shorthand finding per site, in three forms; the ambiguous list is sorted and complete" },
    LeadPromise { id: "FS-check.3.17", reason: "a bare marked citation of a covered ID that the index holds no entry for, and that the next `grund fmt --write` would wrap, is `unlinked-index-entry`, once per ID at the first such citation in file order; nothing else in the index is reported there" },
    LeadPromise { id: "FS-cli.3", reason: "`--format json` opts into the FS-errors.5 shapes on the FS-errors.1 streams (folded 3.1), and a passed `--format` outranks the v1 `[output] format` key of a command that honours it" },
    LeadPromise { id: "FS-cochange-recipe.snapshots", reason: "isolated exact trees, symlink and cycle refusals, complete queries, the refusal list and rename handling, stated in its own body beside its one child snapshots.scope (pre-existing; tests/e2e/cochange_refusals.py, cochange_snapshot_edges.py and cochange_fixture.py already cite the parent exactly)" },
    LeadPromise { id: "FS-config.1", reason: "a bare file name starts discovery at the working directory, so `grund <command> NAME` and `grund <command> ./NAME` are one run" },
    LeadPromise { id: "FS-config-v2.reader.1", reason: "tables and keys are closed; the unknown-section, unknown-key and duplicate-key refusals at their line; every v1 table but `[workspace]`, and every discarded alias, is unknown in v2" },
    LeadPromise { id: "FS-config-v2.reader.4", reason: "every value keeps its line, and rows keep the authored header order" },
    LeadPromise { id: "FS-config-v2.reader.5", reason: "an array may span lines as one value at its key's line; one open at end of file is `unclosed array`" },
    LeadPromise { id: "FS-declarations.checks.duplicate-section", reason: "only headings of one declaration's body collide, compared on canonical paths; a front-matter key that is a section, written twice in a kind with a `front-matter` field; a stub's headings judged in its target whether or not `[scan] include` reaches it, once; a duplicate in every heading-level mode, with section-heading-level judging only the first claimant" },
    LeadPromise { id: "FS-declarations.line", reason: "the ID is the whole token the effective grammar reads, `format` literals such as `:` and every `slug_pattern` character included, in every declaration position, and every command reads that complete ID" },
    LeadPromise { id: "FS-declarations.stubs.pairing", reason: "stubs to one target stand for its declarations once, and stubs to different files that each declare the ID make it a duplicate" },
    LeadPromise { id: "FS-declarations.stubs.target", reason: "the target is read on the text a save would write, a buffer open under any spelling or with no file behind it included" },
    LeadPromise { id: "FS-distribution.4.6", reason: "no hook, CI job or release step asks a change for a changelog entry, nothing waits in the tree for a release, and the one record a change writes for one is a notice on its decision" },
    LeadPromise { id: "FS-fmt.2.4", reason: "only a whole token is rewritten; a typed trigger expands under both policies; a qualified shorthand follows the aliased project; the declaration set is scanned on first use, over the whole project" },
    LeadPromise { id: "FS-fmt.2.5", reason: "a suppressed scope changes no finding; check, refs, show and cover report what they would without it; a shorthand there stays a check error fmt does not clear" },
    LeadPromise { id: "FS-fmt.6", reason: "a marked citation in a file read as Markdown becomes a link to its declaration; check, show and refs resolve the citation, never the URL (FS-non-goals.1)" },
    LeadPromise { id: "FS-fmt.6.2", reason: "the link destination does not depend on how the file argument was spelled" },
    LeadPromise { id: "FS-fmt.6.3", reason: "a link to a heading the same run expands takes the anchor of the text the run leaves; a marked off-grammar catalog citation is wrapped like any other" },
    LeadPromise { id: "FS-fmt.7", reason: "check's findings are not bound to what fmt can rewrite, save the one closed exception of FS-citations.rewrite-exemption" },
    LeadPromise { id: "FS-id.1", reason: "an unknown or non-citable `<KIND>` is a CLI-level error, exit 2, with the `known kinds:` line, both on stderr (folded from FS-id.1.1; FS-id.1 keeps FS-id.1.2, a flag)" },
    LeadPromise { id: "FS-imports.writes", reason: "writers walk only the consumer's projects, and `grund fetch` of an ID in an import is refused before its integration runs, exit 2" },
    LeadPromise { id: "FS-init.1", reason: "`<path>` may come before or after the flags (folded from 1.1)" },
    LeadPromise { id: "FS-init.2.4", reason: "`grund.toml` is written only where neither discovery location holds a config; a `.agents/grund.toml` repository gets no bare twin (folded from 2.4.1); an existing config is never overwritten, not even with `--force`, and is reported `exists ` (folded from 2.4.2)" },
    LeadPromise { id: "FS-init.4", reason: "a `--dry-run` preview exits `0` with paths pending (folded from 4.3)" },
    LeadPromise { id: "FS-integrations.4", reason: "`--write` prints no artifact, reports on stderr and exits 0, touches only the four things grund owns (outside them only a created WezTerm config's scaffold), and is idempotent and reversible" },
    LeadPromise { id: "FS-integrations.4.1", reason: "block fence and comment token per host file, one-block replacement and its refusals (exit 2), WezTerm scaffold and unwired-config note, resolver write and chmod failure" },
    LeadPromise { id: "FS-integrations.5", reason: "the detection plan's object and its per-client fields in order, the plan on stdout and a CLI-level error on stderr, and the guide's detect/diff/confirm workflow (from folded 5.2)" },
    LeadPromise { id: "FS-integrations.6", reason: "with an explicit client, output depends only on the binary version and the flags; detection also on the variables of 2 and, for JSON, the fixed installation targets" },
    LeadPromise { id: "FS-list.2", reason: "which rows exist: one per declaration the scan finds, value-source entries and configured E2E cases included; one per citable section in size mode and wherever --glob or --selector selects one; a value root is metadata on its declaration's row; a fetched kind is listed from its committed snapshot without running the fetcher, so a missing snapshot is no row; a cited-but-undeclared ID has no row; a paired stub has no row of its own, its ID listed at the target's declarations; a broken stub is a row at its own site, noted with its target; one flagged line per independent duplicate home; a retained off-grammar declaration is a normal row, rendered as written" },
    LeadPromise { id: "FS-list.3.4", reason: "the size rows: one per declaration and citable section site, lead and full body measured; a paired stub has no row of its own, its rows are the target's as the scanned tree prints them, measured, and in JSON with stub false and defines null; a target that declares the ID twice gives two measured duplicate rows, never one unmeasured row at the stub; the unmeasured broken-stub row is exactly a stub check reports broken; measuring a home outside the scan does not scan it, so check's lead warning judges no site there; duplicate declaration and section sites are separate, marked, site-local rows, never merged and none the winner, and show still refuses the coordinate; JSON rows are FS-output-shapes.5.2's, with section null on a declaration row" },
    LeadPromise { id: "FS-lsp.1.3", reason: "a citation's definition is the heading of what it names; a value binding lands on its component or root, a named citation on its named heading (from the folded 1.3.5)" },
    LeadPromise { id: "FS-non-goals.12", reason: "no surface over the engine but the FS-distribution bindings and the LSP server; any third kind (in-engine scripting, a per-editor wrapper, a marketplace plugin) is out of scope" },
    LeadPromise { id: "FS-output-shapes.4", reason: "the show object's key order, `path` and `line` where the declaration or selected section starts, every slice printing it with `--brief` narrowing only `body` (pre-existing); an E2E case prints its manifest object; a value-source declaration or component, or a front-matter coordinate, prints its exact slice, its start and a null anchor (folded from 4.2, 4.3, 4.4; child 4.1 stays)" },
    LeadPromise { id: "FS-output-shapes.5", reason: "list's own row, --summary and --total objects (pre-existing: the lead holds list's shapes beside its refs, size and cover children)" },
    LeadPromise { id: "FS-output-shapes.6", reason: "a launch-time failure is raw `error:` text on stderr under every format (pre-existing: the lead holds this beside child 6.1)" },
    LeadPromise { id: "FS-refs.1", reason: "the inputs, every flag and its meaning, --descendants a no-op without a section (unchanged role; the e2e cases already cite FS-refs.1)" },
    LeadPromise { id: "FS-refs.2", reason: "the citation set, value bindings included (from the folded 2.2, via FS-values.3.2), stub-target lending, declaration-local citations, --descendants by whole components, stdout order" },
    LeadPromise { id: "FS-refs.2.1", reason: "the empty answer, exit 0, the typo notes with and without a path, and the API `note` field of both notes" },
    LeadPromise { id: "FS-refs.4", reason: "the exit statuses and the text and JSON refusals of ID mode and --from" },
    LeadPromise { id: "FS-refs.from", reason: "the source resolves as `grund <coordinate> <path>`, lists only the path's tree (a source whose home lies outside it lists nothing), once each, sorted" },
    LeadPromise { id: "FS-refs.globs", reason: "matching by FS-coordinate-globs, each site once in ID mode's order and shapes" },
    LeadPromise { id: "FS-rule-refusals.sentences.rewrite", reason: "the subject, modality, count and object kinds that did not fail are kept as typed and only the failed production is spelled canonically (from the folded replaced)" },
    LeadPromise { id: "FS-rules.2", reason: "a target pinned at the rule's own member by its whole alias path, nested or not, matches what the bare kind matches, with the bare kind's findings (from 7.9)" },
    LeadPromise { id: "FS-rules.5.1", reason: "a resolved citation of a numbered section, or of a name under a number, counts toward its nearest enclosing unit (from 7.8 and 7.12)" },
    LeadPromise { id: "FS-scan.path", reason: "every command but `init` discovers its configuration from its `<path>`" },
    LeadPromise { id: "FS-scan.scope", reason: "`exclude` skips directory names at any depth, one path component each (version 1 `[scan] exclude`)" },
    LeadPromise { id: "FS-scan.scope.extensions", reason: "an extension makes a file readable, not declarable" },
    LeadPromise { id: "FS-scan.scope.homes", reason: "every kind home is scanned whether or not `include` names it; an absent home is silent; `scan = false` lists without scanning" },
    LeadPromise { id: "FS-scan.scope.links", reason: "file links followed anywhere; directory links fenced at the canonical project root, scan roots included; workspace ownership boundary" },
    LeadPromise { id: "FS-schema.4", reason: "a node no direction rule names shows `—` in the table and sits in the graph formats' `no direction rule names these` box (from 4.2)" },
    LeadPromise { id: "FS-schema.5", reason: "a self-edge is one edge to its own node, written `<KIND> (itself)` in the table, and no configuration is refused or reordered for a cycle (from 5.3)" },
    LeadPromise { id: "FS-show.2.1", reason: "a declaration with no lead prints nothing and exits 0 (folded 2.1.4)" },
    LeadPromise { id: "FS-show.2.1.2", reason: "an empty lead or map, and a selected section's map (folded 2.1.2.2, 2.1.2.3)" },
    LeadPromise { id: "FS-show.2.2", reason: "the default prints a section's heading and its lead (moved 2.1.5)" },
    LeadPromise { id: "FS-show.2.3", reason: "section selection, the doc-comment forms, several declarations per comment and a stub read as its target (folded 2.3.3, 2.3.5-2.3.7)" },
    LeadPromise { id: "FS-show.2.4", reason: "the slices over a manifest, which has no sections (folded 2.4.1)" },
    LeadPromise { id: "FS-terms.terms", reason: "the Terms chapter's form (one definition per word, the narrowed form, the lean line, its order and completeness) and the review of a change that touches shared-term prose" },
    LeadPromise { id: "FS-values.2.4", reason: "an exact tag where no marked root can stand (plain, named, empty or malformed numbered heading, outside a declaration, nested or overlapping marks, inside a whole-declaration value) is invalid-value-declaration at the tag" },
    LeadPromise { id: "FS-workspace.6.1", reason: "a short leaf name does not resolve on its own, and check at the outermost root names the candidate paths (from 6.1.1, owned by FS-check.3.8)" },
];
