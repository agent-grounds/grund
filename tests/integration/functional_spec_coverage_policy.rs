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
}

#[derive(Clone, Copy)]
pub(super) struct Exception<'a> {
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
    let leaves = sections
        .iter()
        .filter(|candidate| {
            let descendant_prefix = format!("{candidate}.");
            !sections
                .iter()
                .any(|section| section.starts_with(&descendant_prefix))
        })
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
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
    Exception { id: "FS-check.1.3.10", reason: "rationale for a flag rather than a second, weaker config key" },
    Exception { id: "FS-check.3.3", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.4", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.6.4", reason: "rationale that the rule is a function of tree and config" },
    Exception { id: "FS-check.3.7", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.9", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.16", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.19", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.23", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.3.29.10", reason: "rationale for where the fact becomes knowable" },
    Exception { id: "FS-check.4.6", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.4.7.9", reason: "pointer kept so released changelogs still resolve; the behaviour it named is retired, and the precedence that replaced it is proven at FS-check.3.30.2" },
    Exception { id: "FS-output-shapes.6.1.1", reason: "history of the retired 0.14.0 compatibility form; the shape that replaced it is proven at FS-output-shapes.6.1.2" },
    Exception { id: "FS-check.4.13", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.4.14", reason: "pointer kept so citations written before the move still resolve; the behaviour is proven at the FS-declarations section it names, and this row retires with the pointer" },
    Exception { id: "FS-check.5.1", reason: "finding selection reference, not a behavioral requirement" },
    Exception { id: "FS-cli.1.1", reason: "rationale for the two defaults" },
    Exception { id: "FS-completions.4", reason: "shell installation examples" },
    Exception { id: "FS-config.1.2.2", reason: "rationale for naming no deprecation release" },
    Exception { id: "FS-config.3.4.6.2", reason: "rationale for the key's rename" },
    Exception { id: "FS-config.principle.exceptions", reason: "classifies two shapes as outside the scope relation; each shape's behavior is proven at its own point" },
    Exception { id: "FS-config.principle.install-local", reason: "boundary between committed and install-local state; the axis it names is proven under FS-integrations" },
    Exception { id: "FS-config.requirements.1", reason: "directional config-contract policy, binding the next key rather than describing a released behavior" },
    Exception { id: "FS-config.requirements.3", reason: "directional config-contract policy, binding the next key rather than describing a released behavior" },
    Exception { id: "FS-config.requirements.4", reason: "config-contract policy over every key at once, not a behavioral requirement one run can exhibit" },
    Exception { id: "FS-config.3.4.7.1", reason: "what the scan key is for; its behavior is its children" },
    Exception { id: "FS-config.3.4.8.1", reason: "rationale for asking grounding per place" },
    Exception { id: "FS-config.3.5.10", reason: "rationale for walking a kind home outside include" },
    Exception { id: "FS-cover.5", reason: "coverage interpretation guidance" },
    Exception { id: "FS-declarations.why", reason: "rationale for gathering the declaration checks in one spec" },
    Exception { id: "FS-errors.7", reason: "error-design rationale" },
    Exception { id: "FS-examples.1", reason: "example-suite overview" },
    Exception { id: "FS-fmt.2.4.1.1", reason: "rationale for withholding the shorthand rewrite" },
    Exception { id: "FS-fmt.4", reason: "formatter non-goals" },
    Exception { id: "FS-fmt.7.4.1", reason: "rationale for stating the rule structurally" },
    Exception { id: "FS-fmt.7.5.1", reason: "names what the harness cannot express; its check is outside the gate" },
    Exception { id: "FS-fmt.7.6", reason: "rationale for stating the formatter rules as properties" },
    Exception { id: "FS-id.7", reason: "ID proposal rationale" },
    Exception { id: "FS-id.8", reason: "ID proposal examples" },
    Exception { id: "FS-init.1.1", reason: "initialization rationale" },
    Exception { id: "FS-init.2.3.5.9", reason: "managed-block version history" },
    Exception { id: "FS-init.2.3.6.2", reason: "managed-block version history" },
    Exception { id: "FS-init.2.3.9.1", reason: "rationale for the delimiter shape" },
    Exception { id: "FS-init.6.1", reason: "initialization rationale" },
    Exception { id: "FS-inline-citation-style.4.4.4", reason: "rationale for the two severity levels" },
    Exception { id: "FS-inline-citation-style.6.2", reason: "explicit non-goal" },
    Exception { id: "FS-inline-citation-style.6.3", reason: "explicit non-goal" },
    Exception { id: "FS-inline-citation-style.6.4", reason: "explicit non-goal" },
    Exception { id: "FS-integrations.1.3", reason: "which side assembles the bytes; nothing a user sees turns on it" },
    Exception { id: "FS-integrations.2.2", reason: "what golden coverage can and cannot pin" },
    Exception { id: "FS-integrations.3.5", reason: "integration guidance" },
    Exception { id: "FS-integrations.4.3.6", reason: "rationale for warning where the repository config refuses" },
    Exception { id: "FS-integrations.4.3.13", reason: "agent-guidance block version history" },
    Exception { id: "FS-list.5", reason: "catalog-query rationale" },
    Exception { id: "FS-lsp.5", reason: "LSP non-goals" },
    Exception { id: "FS-refs.5", reason: "reference-query rationale" },
    Exception { id: "FS-show.4.1", reason: "show-query rationale" },
    Exception { id: "FS-workspace.2.2.1.2", reason: "rationale that leaves the empty-directory case undecided" },
    Exception { id: "FS-workspace.2.2.3", reason: "rationale for why the key exists" },
    Exception { id: "FS-workspace.2.2.10.1", reason: "rationale contrasting the glob and absent-member rules" },
    Exception { id: "FS-workspace.6.1.4.1", reason: "rationale: the duplicate error is an unreachable backstop" },
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
    Exception { id: "FS-non-goals.14.1", reason: "explicit non-goal" },
    Exception { id: "FS-check.2.3.3", reason: "planned capability: the gap report" },
    Exception { id: "FS-config.3.7.1", reason: "reserved home for a later markup family" },
    Exception { id: "FS-fmt.6.7.4", reason: "reserved settings for a later markup family" },
    Exception { id: "FS-lsp.1.5", reason: "planned LSP capability" },
    Exception { id: "FS-workspace.8.4.5", reason: "permitted interim fallback, obliging the tool to nothing" },
    Exception { id: "FS-distribution.5", reason: "distribution target description" },
    Exception { id: "FS-init.2.3.4.1", reason: "covered by the byte-exact generated init block" },
    Exception { id: "FS-init.2.3.4.2", reason: "covered by the byte-exact generated init block" },
    Exception { id: "FS-init.2.3.4.6", reason: "covered by the byte-exact generated init block" },
    Exception { id: "FS-init.2.3.4.7", reason: "covered by the byte-exact generated init block" },
    Exception { id: "FS-init.2.3.4.8", reason: "covered by the byte-exact generated init block" },
    Exception { id: "FS-init.2.3.4.9", reason: "covered by the byte-exact generated init block" },
    Exception { id: "FS-init.2.3.4.11", reason: "covered by the byte-exact generated init block" },
    Exception { id: "FS-init.2.3.4.12", reason: "covered by the byte-exact generated init block" },
    Exception { id: "FS-init.2.3.4.13", reason: "covered by the byte-exact generated init block" },
    Exception { id: "FS-init.2.3.4.14", reason: "covered by the byte-exact generated init block" },
    Exception { id: "FS-init.2.3.4.16", reason: "covered by the byte-exact generated init block" },
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
    Exception { id: "FS-cli.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-completions.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-config.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-config-v2.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-cover.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-declarations.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-distribution-candidate.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-distribution.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-errors.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-examples.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-fetch.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-fmt.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-id.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-init.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-init-fixtures.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-inline-citation-style.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-integrations.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-list.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-lsp.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-repository-maintenance.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-non-goals.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-output-shapes.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-refs.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-remote-projects.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-rules.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-show.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-values.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-workspace.terms", reason: "vocabulary, not a behavioral requirement" },
];

#[rustfmt::skip]
pub(super) const TEMPORARY_EXCEPTIONS: &[Exception<'static>] = &[
    Exception { id: "FS-examples.3", reason: "example explanation proof spans the runner and docs" },
    Exception { id: "FS-examples.4", reason: "example maintenance proof spans the runner and docs" },
    Exception { id: "FS-fetch.remote.all", reason: "specified ahead of its release; the Drift piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.check", reason: "specified ahead of its release; the Drift piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.commit", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.findings", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.from", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.install", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-fetch.remote.selector", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-fmt.2.1", reason: "the broad formatter input matrix needs a proof audit" },
    Exception { id: "FS-init.2.3.3", reason: "generated citation-form proof needs a precise mapping audit" },
    Exception { id: "FS-inline-citation-style.3.2", reason: "the multi-case note boundary needs a proof audit" },
    Exception { id: "FS-inline-citation-style.4.3", reason: "the formatter boundary needs a proof audit" },
    Exception { id: "FS-lsp.3", reason: "shared config parity across CLI and LSP needs a proof audit" },
    Exception { id: "FS-remote-projects.bytes.digest", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.bytes.entries", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.bytes.protection", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.checks.remote-missing", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.checks.remote-modified", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.checks.remote-orphan", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.checks.remote-stale", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.declaration.alias", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.declaration.older", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.declaration.registration", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.declaration.table", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.differences.cascade", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.differences.consumer-site", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.differences.ignore", reason: "specified ahead of its release; the Mount piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.differences.integrity", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.differences.paths", reason: "specified ahead of its release; the Drift piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.differences.selectors", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.differences.suppression", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.differences.writes", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.failures.remote-lock-invalid", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.failures.remote-projection-unsupported", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.failures.remote-unreadable", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.ignore", reason: "specified ahead of its release; the Mount piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.lock.agreement", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.lock.digest", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.lock.keys", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.lock.ownership", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.mount.inside", reason: "specified ahead of its release; the Mount piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.mount.local", reason: "specified ahead of its release; the Mount piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.mount.nested", reason: "specified ahead of its release; the Mount piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.mount.rule", reason: "specified ahead of its release; the Mount piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.projection.committed", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.projection.format", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-remote-projects.projection.tree", reason: "specified ahead of its release; the Remotes piece of remote projects proves it" },
    Exception { id: "FS-show.2.3.1.1", reason: "the backward open-boundary walk changes no CLI output, so the slice has no observable form to assert yet" },
    Exception { id: "FS-workspace.8.4.3", reason: "the single-project fallback this names is not what complete_ids_run does today; the wording and the code have to be reconciled first" },
];
