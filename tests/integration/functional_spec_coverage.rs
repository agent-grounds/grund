//! Contract for the exact-leaf functional-spec coverage gate
//! (§AR-goal-measurement.1). The production scanner owns the catalog; this
//! integration target owns the cross-tree evidence and exception policy.

use grund_core::{Findings, scan};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Eq, PartialEq)]
enum CoverageProblem {
    UncoveredLeaf(String),
    DuplicateException(String),
    InvalidException(String),
    CoveredException(String),
    EmptyReason(String),
}

#[derive(Clone, Copy)]
struct Exception<'a> {
    id: &'a str,
    reason: &'a str,
}

/// Compare the production catalog with exact test evidence and the two reviewed
/// exception tables (§AR-goal-measurement.1). An exception is valid only while
/// its point is an uncited leaf; that makes proof retire debt automatically.
fn functional_spec_coverage_problems(
    catalog: &Findings,
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

fn functional_spec_sections(catalog: &Findings) -> BTreeSet<String> {
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
const PERMANENT_EXCEPTIONS: &[Exception<'static>] = &[
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
    Exception { id: "FS-non-goals.1", reason: "explicit non-goal" },
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
    Exception { id: "FS-check.6.1", reason: "planned watch-mode capability" },
    Exception { id: "FS-check.6.2", reason: "planned watch-mode capability" },
    Exception { id: "FS-check.6.3", reason: "planned watch-mode capability" },
    Exception { id: "FS-check.6.4", reason: "planned watch-mode capability" },
    Exception { id: "FS-config.3.7.1", reason: "reserved home for a later markup family" },
    Exception { id: "FS-config.5.2", reason: "no second schema version exists whose meaning could be re-read" },
    Exception { id: "FS-fmt.6.7.4", reason: "reserved settings for a later markup family" },
    Exception { id: "FS-lsp.1.5", reason: "planned LSP capability" },
    Exception { id: "FS-lsp.2.3", reason: "reserved LSP capability" },
    Exception { id: "FS-output-shapes.6.1.2", reason: "planned 0.16.0 failed-query shape" },
    Exception { id: "FS-workspace.8.4.5", reason: "permitted interim fallback, obliging the tool to nothing" },
    Exception { id: "FS-distribution.2", reason: "distribution target description" },
    Exception { id: "FS-distribution.3.2", reason: "packaging target" },
    Exception { id: "FS-distribution.3.3", reason: "packaging target" },
    Exception { id: "FS-distribution.4.11", reason: "planned full-ecosystem release" },
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
    Exception { id: "FS-cover.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-declarations.terms", reason: "vocabulary, not a behavioral requirement" },
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
    Exception { id: "FS-non-goals.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-output-shapes.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-refs.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-rules.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-show.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-values.terms", reason: "vocabulary, not a behavioral requirement" },
    Exception { id: "FS-workspace.terms", reason: "vocabulary, not a behavioral requirement" },
];

#[rustfmt::skip]
const TEMPORARY_EXCEPTIONS: &[Exception<'static>] = &[
    Exception { id: "FS-examples.3", reason: "example explanation proof spans the runner and docs" },
    Exception { id: "FS-examples.4", reason: "example maintenance proof spans the runner and docs" },
    Exception { id: "FS-fmt.2.1", reason: "the broad formatter input matrix needs a proof audit" },
    Exception { id: "FS-init.2.3.3", reason: "generated citation-form proof needs a precise mapping audit" },
    Exception { id: "FS-inline-citation-style.3.2", reason: "the multi-case note boundary needs a proof audit" },
    Exception { id: "FS-inline-citation-style.4.3", reason: "the formatter boundary needs a proof audit" },
    Exception { id: "FS-lsp.3", reason: "shared config parity across CLI and LSP needs a proof audit" },
    Exception { id: "FS-show.2.3.1.1", reason: "the backward open-boundary walk changes no CLI output, so the slice has no observable form to assert yet" },
    Exception { id: "FS-workspace.8.4.3", reason: "the single-project fallback this names is not what complete_ids_run does today; the wording and the code have to be reconciled first" },
];

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repository root")
}

fn is_live_test_source(root: &Path, file: &Path) -> bool {
    let Ok(relative) = file.strip_prefix(root) else {
        return false;
    };
    let path = relative.to_string_lossy().replace('\\', "/");
    if path == "tests/integration/functional_spec_coverage.rs" {
        return false;
    }
    if path.starts_with("tests/integration/") {
        return true;
    }
    if !path.starts_with("crates/") || !path.ends_with(".rs") {
        return false;
    }
    if path.contains("/tests/") {
        return true;
    }
    path.contains("/src/")
        && relative
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("tests_"))
}

fn repository_evidence(root: &Path, catalog: &Findings) -> BTreeSet<String> {
    // §AR-goal-measurement.1: source paths and their filter share the scanner's canonical root.
    let root = root.canonicalize().expect("canonical evidence root");
    let mut evidence = BTreeSet::new();
    let cases = fs::read_dir(root.join("tests/e2e/cases")).expect("read e2e cases");
    for case in cases {
        let case = case.expect("read e2e case entry").path();
        if !case.is_dir() {
            continue;
        }
        let refs = fs::read_to_string(case.join("spec.refs")).expect("read e2e spec.refs");
        evidence.extend(
            refs.lines()
                .map(str::trim)
                .filter(|reference| reference.starts_with("FS-") && !reference.is_empty())
                .map(str::to_string),
        );
    }
    evidence.extend(
        catalog
            .citations
            .iter()
            .filter(|citation| {
                citation.namespace.is_none()
                    && citation.has_marker
                    && citation.id.kind == "FS"
                    && is_live_test_source(&root, &citation.file)
            })
            .filter_map(|citation| {
                let slug = citation.id.slug.as_deref()?;
                let section = citation.section.as_deref()?;
                Some(format!("FS-{slug}.{section}"))
            }),
    );
    evidence
}

#[test]
fn every_behavioral_functional_spec_leaf_has_exact_test_evidence() {
    let root = repository_root();
    let catalog = scan(&root).expect("scan repository with the production scanner");
    let evidence = repository_evidence(&root, &catalog);
    let borrowed = evidence.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let problems = functional_spec_coverage_problems(
        &catalog,
        &borrowed,
        PERMANENT_EXCEPTIONS,
        TEMPORARY_EXCEPTIONS,
    );
    assert!(
        problems.is_empty(),
        "functional-spec coverage policy failed:\n{problems:#?}"
    );
}

struct Fixture {
    root: PathBuf,
}

/// The name a fixture gives its root (§AR-ci.10.1). `clock` is the wall-clock
/// reading the name is allowed to see, taken as an argument rather than read
/// here so that a test can hold two names to one reading without a clock shim:
/// what the identity may not do is rest on it. The process-wide serial is what
/// carries the identity, as the `crates/grund-cli/tests` fixtures do: `libtest`
/// runs this binary's cases as threads of one process, so the process id
/// distinguishes nothing inside it and the reading only records when the tree
/// was written.
fn fixture_root_name(clock: SystemTime) -> String {
    static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);
    let serial = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let nonce = clock
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    format!(
        "grund-functional-spec-coverage-{}-{nonce}-{serial}",
        std::process::id()
    )
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(fixture_root_name(SystemTime::now()));
        Self::claim(root).expect("claim fixture root")
    }

    /// Take `root` for this fixture's exclusive use and write the synthetic
    /// repository into it (§AR-ci.10.2). A root that already exists is some
    /// other fixture's, so the error is reported rather than the tree shared.
    /// `create_dir` on the root is the claim: it fails with `AlreadyExists`
    /// instead of adopting a tree, and nothing is cleared in order to take it.
    fn claim(root: PathBuf) -> io::Result<Self> {
        fs::create_dir(&root)?;
        fs::create_dir(root.join("docs"))?;
        fs::write(
            root.join("grund.toml"),
            r#"grund_config_version = 1
project_name = "coverage-fixture"

[reference]
marker = "§"
strict = true

[id]
format = "{kind}-{slug}"
section_separator = "."
named_sections = true
section_heading_levels = "strict"
slug_pattern = "[a-z][a-z0-9-]*"

[[kinds]]
kind = "FS"
folder = "docs"
title = "Behavior"

[scan]
extensions = ["md", "rs"]
"#,
        )?;
        fs::write(
            root.join("docs/spec.md"),
            concat!(
                "# FS-coverage: Coverage fixture\n\n",
                "## 1. Numeric parent\n\n",
                "### 1.1 Numeric child\n\n",
                "## named: Named parent\n\n",
                "### named.child: Named child\n\n",
                "```markdown\n",
                "#### named.child.fenced: Not a section\n",
                "```\n\n",
                "# FS-second: Second declaration\n\n",
                "## 1. Its own section\n",
            ),
        )?;
        Ok(Self { root })
    }

    fn scan(&self) -> Findings {
        scan(&self.root).expect("scan synthetic functional spec")
    }
}

/// A fixture removes only the tree it created (§AR-ci.10.2): the claim above is
/// what makes that true here, because a `Fixture` exists only where `create_dir`
/// on this root succeeded.
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn declaration<'a>(catalog: &'a Findings, slug: &str) -> &'a grund_core::Declaration {
    catalog
        .declarations
        .iter()
        .find(|(id, _)| id.kind == "FS" && id.slug.as_deref() == Some(slug))
        .and_then(|(_, declarations)| declarations.first())
        .unwrap_or_else(|| panic!("missing FS-{slug}"))
}

fn no_exceptions() -> &'static [Exception<'static>] {
    &[]
}

#[test]
fn production_scan_owns_numeric_named_fenced_and_declaration_boundaries() {
    let catalog = Fixture::new().scan();
    let coverage = declaration(&catalog, "coverage");
    assert_eq!(
        coverage
            .sections
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["1", "1.1", "named", "named.child"]
    );
    assert_eq!(
        declaration(&catalog, "second")
            .sections
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["1"]
    );
}

#[test]
fn parent_evidence_does_not_cover_a_new_child_leaf() {
    let catalog = Fixture::new().scan();
    let evidence = BTreeSet::from(["FS-coverage.1", "FS-coverage.named.child", "FS-second.1"]);
    assert_eq!(
        functional_spec_coverage_problems(&catalog, &evidence, no_exceptions(), no_exceptions()),
        [CoverageProblem::UncoveredLeaf("FS-coverage.1.1".into())]
    );
}

#[test]
fn exact_child_evidence_covers_the_child_leaf() {
    let catalog = Fixture::new().scan();
    let evidence = BTreeSet::from(["FS-coverage.1.1", "FS-coverage.named.child", "FS-second.1"]);
    assert!(
        functional_spec_coverage_problems(&catalog, &evidence, no_exceptions(), no_exceptions())
            .is_empty()
    );
}

#[test]
fn exception_tables_reject_duplicates_invalid_non_leaves_and_empty_reasons() {
    let catalog = Fixture::new().scan();
    let permanent = [
        Exception {
            id: "FS-coverage.1.1",
            reason: "reviewed prose",
        },
        Exception {
            id: "FS-coverage.missing",
            reason: "not in the catalog",
        },
        Exception {
            id: "FS-coverage.1",
            reason: "a parent is not a leaf",
        },
    ];
    let temporary = [
        Exception {
            id: "FS-coverage.1.1",
            reason: "temporary debt",
        },
        Exception {
            id: "FS-coverage.named.child",
            reason: "",
        },
    ];
    assert_eq!(
        functional_spec_coverage_problems(
            &catalog,
            &BTreeSet::from(["FS-second.1"]),
            &permanent,
            &temporary
        ),
        [
            CoverageProblem::DuplicateException("FS-coverage.1.1".into()),
            CoverageProblem::InvalidException("FS-coverage.missing".into()),
            CoverageProblem::InvalidException("FS-coverage.1".into()),
            CoverageProblem::EmptyReason("FS-coverage.named.child".into()),
        ]
    );
}

#[test]
fn newly_covered_temporary_entry_must_be_retired() {
    let catalog = Fixture::new().scan();
    let temporary = [Exception {
        id: "FS-coverage.1.1",
        reason: "proof is pending",
    }];
    let evidence = BTreeSet::from(["FS-coverage.1.1", "FS-coverage.named.child", "FS-second.1"]);
    assert_eq!(
        functional_spec_coverage_problems(&catalog, &evidence, no_exceptions(), &temporary),
        [CoverageProblem::CoveredException("FS-coverage.1.1".into())]
    );
}

#[test]
fn repository_evidence_counts_sources_and_excludes_its_own_synthetic_proofs() {
    let fixture = Fixture::new();
    let config_path = fixture.root.join("grund.toml");
    let config = fs::read_to_string(&config_path).expect("read fixture config");
    fs::write(
        config_path,
        format!("{config}\ninclude = [\"docs\", \"crates\", \"tests\"]\n"),
    )
    .expect("include every evidence root in the scan");
    for (path, contents) in [
        (
            "crates/sample/src/nested/tests_behavior.rs",
            "// \u{a7}FS-proof.unit\n",
        ),
        (
            "crates/sample/tests/behavior.rs",
            "// \u{a7}FS-proof.crate\n",
        ),
        (
            "tests/integration/behavior.rs",
            "// \u{a7}FS-proof.integration\n",
        ),
        (
            "tests/integration/functional_spec_coverage.rs",
            "// \u{a7}FS-proof.gate\nconst INVENTORY: &str = \"FS-proof.inventory\";\n",
        ),
        (
            "crates/sample/src/implementation.rs",
            "// \u{a7}FS-proof.production\n",
        ),
        (
            "tests/e2e/cases/synthetic/repo/proof.rs",
            "// \u{a7}FS-proof.synthetic\n",
        ),
        (
            "tests/e2e/cases/synthetic/spec.refs",
            "FS-coverage.1\nFS-proof.manifest\n",
        ),
        (
            "tests/integration/inventory.md",
            "FS-proof.inventory\n<\u{a7}>FS-proof.escaped\n",
        ),
    ] {
        let file = fixture.root.join(path);
        fs::create_dir_all(file.parent().unwrap()).expect("create evidence directory");
        fs::write(file, contents).expect("write evidence fixture");
    }
    let root = fixture.root.join("tests/integration/../..");
    let catalog = scan(&root).expect("scan evidence fixture through a lexical root");
    assert!(
        catalog.citations.iter().any(|citation| {
            citation
                .file
                .ends_with("tests/integration/functional_spec_coverage.rs")
                && citation.section.as_deref() == Some("gate")
        }),
        "self-exclusion must discard a citation actually returned by the scanner"
    );
    assert_eq!(
        repository_evidence(&root, &catalog),
        BTreeSet::from(
            [
                "FS-coverage.1",
                "FS-proof.unit",
                "FS-proof.crate",
                "FS-proof.integration",
                "FS-proof.manifest",
            ]
            .map(str::to_string)
        ),
        "only exact manifests and live citations in the approved sources count"
    );
}

/// A root for the cases below, which have to know a path before a fixture takes
/// it. Named the way §AR-ci.10.1 asks — a serial beside the thread — so the
/// cases that pin the fixture's identity cannot be bitten by the defect they
/// pin.
fn held_root(case: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "grund-functional-spec-coverage-held-{}-{:?}-{}-{case}",
        std::process::id(),
        std::thread::current().id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

/// Two fixtures whose clock readings land in one tick still name two
/// directories (§AR-ci.10.1). Freezing the reading is what makes this a test
/// rather than a race: on a platform whose `CLOCK_REALTIME` resolution is
/// 1000ns this is the ordinary case, not a contrived one.
#[test]
fn two_fixture_roots_named_from_one_clock_reading_differ() {
    let tick = SystemTime::now();
    assert_ne!(
        fixture_root_name(tick),
        fixture_root_name(tick),
        "two fixtures that read the clock inside one tick name the same \
         directory, so the first to finish removes the tree the second is \
         still scanning"
    );
}

/// The same reading taken from two threads (§AR-ci.10.1). This is the shape the
/// gate actually fails in: `libtest` runs this binary's six fixtures as threads
/// of one process, so the process id is shared and the reading is the whole of
/// what is left to tell them apart.
#[test]
fn two_threads_naming_a_root_from_one_clock_reading_differ() {
    let tick = SystemTime::now();
    let here = fixture_root_name(tick);
    let there = std::thread::spawn(move || fixture_root_name(tick))
        .join()
        .expect("name a fixture root on another thread");
    assert_ne!(
        here, there,
        "two test threads that read the clock inside one tick name the same \
         directory, which is the collision the macOS leg of the matrix fails on"
    );
}

/// A root a fixture did not create is not its root (§AR-ci.10.2). `create_dir`
/// on the root makes a name already taken an `AlreadyExists` the case reports,
/// where `create_dir_all` accepts the existing tree and two fixtures share one.
#[test]
fn a_fixture_refuses_a_root_that_already_exists() {
    let held = held_root("refuses-an-existing-root");
    fs::create_dir_all(held.join("docs")).expect("pre-create the root a colliding fixture takes");

    let outcome = Fixture::claim(held.clone());
    let refused = outcome.as_ref().err().map(io::Error::kind);
    drop(outcome);
    let _ = fs::remove_dir_all(&held);

    assert_eq!(
        refused,
        Some(io::ErrorKind::AlreadyExists),
        "claiming a root that already exists succeeded, so two fixtures naming \
         one directory share a tree and the first Drop removes it under the \
         other's scan"
    );
}

/// A refused claim leaves what it found alone (§AR-ci.10.2). Clearing a root in
/// order to take it moves the deletion earlier rather than removing it, and
/// leftover hygiene is what makes that look reasonable, so it is worth pinning
/// beside the refusal rather than inside it.
#[test]
fn a_refused_claim_does_not_touch_the_tree_it_found() {
    let held = held_root("leaves-the-tree-it-found");
    fs::create_dir_all(held.join("docs")).expect("pre-create the held root");
    let holders = held.join("docs/spec.md");
    let written = "written by the fixture that holds this root\n";
    fs::write(&holders, written).expect("write the holder's file");

    let outcome = Fixture::claim(held.clone());
    let survived = fs::read_to_string(&holders).ok();
    drop(outcome);
    let _ = fs::remove_dir_all(&held);

    assert_eq!(
        survived.as_deref(),
        Some(written),
        "a fixture that took a root it did not create overwrote the holder's \
         file, so the collision destroys the other fixture's tree before any \
         Drop runs"
    );
}
