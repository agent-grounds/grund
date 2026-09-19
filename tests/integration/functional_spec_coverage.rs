//! Contract for the exact-leaf functional-spec coverage gate
//! (§AR-goal-measurement.1). The production scanner owns the catalog; this
//! integration target owns the cross-tree evidence and exception policy.

use grund_core::{Findings, scan};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
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
#[allow(dead_code)] // Read once the detector replaces the specification scaffold below.
struct Exception<'a> {
    id: &'a str,
    reason: &'a str,
}

/// Specification scaffold: implementation replaces the empty result with the
/// detector used by the repository gate. Keeping the seam here makes the first
/// test run fail on the missing coverage decision rather than on compilation.
fn functional_spec_coverage_problems(
    _catalog: &Findings,
    _evidence: &BTreeSet<&str>,
    _permanent: &[Exception<'_>],
    _temporary: &[Exception<'_>],
) -> Vec<CoverageProblem> {
    Vec::new()
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
