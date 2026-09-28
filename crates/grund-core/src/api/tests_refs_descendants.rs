//! Test module: the `descendants` field of `RefsOpts` (§FS-refs.1, §FS-refs.2)
//! as an embedder sees it. The e2e cases pin the CLI flag; these pin the
//! published option it sets, which is a contract of its own (§AR-bindings.2):
//! it defaults off, it widens the section filter when set, and with it off the
//! same options return what they returned before the field existed.

use std::path::{Path, PathBuf};

use super::*;
use crate::testing::{test_root, write};

/// A tree whose `FS-001-alpha` is cited at `1`, `1.1`, `1.1.1`, the sibling
/// `1.10`, `2`, and bare. The `1.10` site is what tells a string prefix from a
/// component relation (§FS-refs.2).
fn refs_descendants_repo(name: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n",
    );
    write(
        &root.join("docs/FS-001-alpha.md"),
        "# FS-001-alpha: Alpha\n\nLead.\n\n## 1. One\n\nOne.\n\n### 1.1 One one\n\nOne one.\n\n#### 1.1.1 Deep\n\nDeep.\n\n### 1.10 One ten\n\nOne ten.\n\n## 2. Two\n\nTwo.\n",
    );
    write(
        &root.join("docs/FS-002-beta.md"),
        "# FS-002-beta: Beta\n\n§FS-001-alpha.1\n\n§FS-001-alpha.1.1\n\n§FS-001-alpha.1.1.1\n\n§FS-001-alpha.1.10\n\n§FS-001-alpha.2\n\n§FS-001-alpha\n",
    );
    root
}

fn sections(root: &Path, section: Option<&str>, descendants: bool) -> Vec<Option<String>> {
    refs(RefsOpts {
        path: root.to_path_buf(),
        path_provided: true,
        id: "FS-001-alpha".to_string(),
        section: section.map(str::to_string),
        descendants,
    })
    .expect("public refs api")
    .hits
    .into_iter()
    .map(|hit| hit.section)
    .collect()
}

/// §FS-refs.1: the field is off in the constructed default, so an embedder who
/// never names it keeps the exact-coordinate filter §FS-refs.2 has always
/// described. The flag's absence changes no byte, and this is where that
/// starts.
#[test]
fn the_default_options_do_not_widen() {
    assert!(!RefsOpts::default().descendants);
}

/// §FS-refs.2: set, the filter keeps the requested coordinate and everything
/// beneath it at any depth — and not the sibling `1.10`, which a string prefix
/// would have swallowed.
#[test]
fn descendants_widens_the_section_filter_to_the_subtree() {
    let root = refs_descendants_repo("refs_descendants_widens_to_the_subtree");
    assert_eq!(
        sections(&root, Some("1"), true),
        vec![
            Some("1".to_string()),
            Some("1.1".to_string()),
            Some("1.1.1".to_string())
        ]
    );
    assert_eq!(
        sections(&root, Some("1.1"), true),
        vec![Some("1.1".to_string()), Some("1.1.1".to_string())]
    );
}

/// §FS-refs.2: unset, the same options return exactly what they return today —
/// the exact coordinate, and nothing under it. Read beside the case above,
/// this is the one that says the widening is opt-in rather than a new default.
#[test]
fn the_same_options_without_the_field_return_the_exact_coordinate() {
    let root = refs_descendants_repo("refs_descendants_off_is_todays_answer");
    assert_eq!(
        sections(&root, Some("1"), false),
        vec![Some("1".to_string())]
    );
    assert_eq!(
        sections(&root, Some("1.1"), false),
        vec![Some("1.1".to_string())]
    );
}

/// §FS-refs.4: a query that carries no section has nothing to widen, so the
/// field is a no-op there rather than an error — the whole-ID list, bare-ID
/// citation included, either way.
#[test]
fn with_no_section_the_field_changes_nothing() {
    let root = refs_descendants_repo("refs_descendants_no_section_noop");
    assert_eq!(
        sections(&root, None, true),
        sections(&root, None, false),
        "the wider question is the same question when there is no section"
    );
    assert_eq!(sections(&root, None, true).len(), 6);
}
