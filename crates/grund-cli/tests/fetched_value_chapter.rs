//! Issue #1156's cases: a `value_chapter` on a row that also sets `fetch` is refused in words of
//! its own, and the way through those words name is a real one (§FS-config-migrate.3.1.7.1).
//!
//! The refusal's case is held out of the full pass until the fix lands, and implement removes the
//! hold. The repair's two cases stay in the full pass and are green before the fix on purpose: the
//! fix changes no behaviour they read, and they are what makes the new words true.
#![allow(dead_code)]
#[path = "support/case_runner.rs"]
mod case_runner;

use std::fs;
use std::path::{Path, PathBuf};

use case_runner::CaseKind::E2e;
use case_runner::{assert_every_case_passed, run_case};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
#[ignore = "red until #1156 gives a fetched row's value_chapter refusal its own words; implement removes this"]
fn fetched_value_chapter_refusal_contract() {
    let root = repo_root();
    let case = root.join("tests/e2e/cases/config-migrate-refuse-value-chapter-fetched");
    assert_every_case_passed(
        "fetched value_chapter refusal",
        &[run_case(&root, &case, E2e)],
    );
}

/// The check case proves a wrong literal still fails on the migrated tree only while its fixture
/// is the tree the write case leaves, so the two cannot drift apart.
#[test]
fn fetched_value_chapter_repair_checks_the_tree_it_wrote() {
    let cases = repo_root().join("tests/e2e/cases");
    let written = cases.join("config-migrate-fetched-marked-value-root-write/expected.repo");
    let checked = cases.join("config-migrate-fetched-marked-value-root-check/repo");
    for file in ["grund.toml", "docs/arch.md", "scripts/fetch"] {
        let read = |tree: &Path| fs::read(tree.join(file)).expect("read the fixture");
        assert_eq!(
            read(&written),
            read(&checked),
            "{file}: the check case must run on the tree the write case leaves"
        );
    }
}
