//! Contract for the exact-leaf functional-spec coverage gate
//! (§AR-goal-measurement.1). The production scanner owns the catalog; this
//! integration target owns the cross-tree evidence and exception policy.

use grund_core::{Findings, scan};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
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
    Exception { id: "FS-check.3.6.4", reason: "rationale that the rule is a function of tree and config" },
    Exception { id: "FS-check.4.8.10", reason: "rationale for where the fact becomes knowable" },
    Exception { id: "FS-check.5.1", reason: "finding selection reference, not a behavioral requirement" },
    Exception { id: "FS-cli.1.1", reason: "rationale for the two defaults" },
    Exception { id: "FS-completions.4", reason: "shell installation examples" },
    Exception { id: "FS-config.1.2.2", reason: "rationale for naming no deprecation release" },
    Exception { id: "FS-config.3.4.6.2", reason: "rationale for the key's rename" },
    Exception { id: "FS-config.3.4.7.1", reason: "what the scan key is for; its behavior is its children" },
    Exception { id: "FS-config.3.4.8.1", reason: "rationale for asking grounding per place" },
    Exception { id: "FS-config.3.5.10", reason: "rationale for walking a kind home outside include" },
    Exception { id: "FS-cover.5", reason: "coverage interpretation guidance" },
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
    Exception { id: "FS-non-goals.9", reason: "explicit non-goal" },
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
    Exception { id: "FS-output-shapes.6.1.2", reason: "planned 0.15.0 failed-query shape" },
    Exception { id: "FS-workspace.8.4.5", reason: "permitted interim fallback, obliging the tool to nothing" },
    Exception { id: "FS-distribution.2", reason: "distribution target description" },
    Exception { id: "FS-distribution.3.2", reason: "packaging target" },
    Exception { id: "FS-distribution.3.3", reason: "packaging target" },
    Exception { id: "FS-distribution.4.1", reason: "release-process target" },
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

impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "grund-functional-spec-coverage-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("docs")).expect("create fixture docs");
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
        )
        .expect("write fixture config");
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
        )
        .expect("write fixture spec");
        Self { root }
    }

    fn scan(&self) -> Findings {
        scan(&self.root).expect("scan synthetic functional spec")
    }
}

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
