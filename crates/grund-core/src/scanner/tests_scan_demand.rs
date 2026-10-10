//! Test module: the scanner reads what grounding needs it to record as
//! `ScanDemand` (§AR-scanner.2.7.3) — no structure for a level-1 tree, and for
//! a level-2 row its own files alone.

use crate::config::load_config;
use crate::testing::{canonical_test_path, scan_findings, test_root, write};

/// A tree with a runbook home and a source file outside every home.
fn tree(name: &str, reference: &str) -> std::path::PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n[reference]\n{reference}\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\n\
             [[kinds]]\nkind = \"runbook\"\nfolder = \"runbooks\"\ncitable = false\n\
             grounding_level = 2\n"
        ),
    );
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\n## 1. One\n");
    write(&root.join("runbooks/deploy.md"), "# Deploy\n\n## Steps\n");
    write(&root.join("src/lib.rs"), "/// A doc comment.\nfn f() {}\n");
    root
}

/// §AR-scanner.2.7.3: a level-1 tree records no structure at all.
#[test]
fn a_level_one_tree_records_no_structure() {
    let root = test_root("a_level_one_tree_records_no_structure");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nrequire_grounding = true\n",
    );
    write(
        &root.join("docs/functional-spec/FS-a.md"),
        "# FS-a: A\n\n## 1. One\n",
    );
    write(&root.join("src/lib.rs"), "/// A doc comment.\nfn f() {}\n");
    let config = load_config(&root).expect("the config loads");
    let findings = scan_findings(&config, &root);
    assert!(findings.file_structure.is_empty());
}

/// §AR-scanner.2.7.3: a level-2 row records the structure of its own files and
/// of no other row's.
#[test]
fn a_level_two_row_records_only_its_own_files() {
    let root = tree(
        "a_level_two_row_records_only_its_own_files",
        "require_grounding = true",
    );
    let config = load_config(&root).expect("the config loads");
    let findings = scan_findings(&config, &root);
    let recorded = findings
        .file_structure
        .keys()
        .map(|path| canonical_test_path(path))
        .collect::<Vec<_>>();
    assert_eq!(
        recorded,
        vec![canonical_test_path(&root.join("runbooks/deploy.md"))]
    );
}
