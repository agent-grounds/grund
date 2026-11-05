//! §FS-config-migrate.3.1.7.1: the way through a fetched `value_chapter`'s refusal is a real one.
//!
//! The refusal's case and the repair's two cases run in the full pass. What the pass cannot see is
//! whether the tree the repair's check case reads is the one its write case leaves, so this ties
//! the two together.
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
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
