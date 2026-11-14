//! Test module: a numbered or enabled named heading after a nested declaration, back inside the
//! enclosing body, is recorded on the innermost declaration whose body holds it — never on the
//! nested declaration the scan read last, and never dropped from every map
//! (§FS-declarations.section.map.nested, over the body span of §FS-declarations.body). Issue
//! #1359: the scan offered the heading to the nested declaration alone, which refused it.

use crate::checker::check_findings;
use crate::model::{Catalog, Id};
use crate::testing::{error_codes, legacy_fs_folder_config, scan_findings, test_root, write};

fn id(number: u32, slug: &str) -> Id {
    Id {
        kind: "FS".to_string(),
        num: Some(number),
        slug: Some(slug.to_string()),
    }
}

fn section_paths(findings: &Catalog, number: u32, slug: &str) -> Vec<String> {
    findings.declarations[&id(number, slug)][0]
        .sections
        .keys()
        .cloned()
        .collect()
}

fn scan(name: &str, text: &str) -> (crate::config::Config, Catalog) {
    let root = test_root(name);
    write(&root.join("docs/functional-spec/FS-001-ci.md"), text);
    let mut config = legacy_fs_folder_config(root.clone());
    config.named_sections = true;
    config.rebuild_grammar().expect("rebuild named grammar");
    let findings = scan_findings(&config, &root);
    (config, findings)
}

/// The report's shape, numbered and named: both headings after `FS-002-matrix` are coordinates of
/// `FS-001-ci`, the nested declaration has none, and nothing is reported outside a body.
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn a_heading_after_a_nested_declaration_is_the_enclosing_declarations() {
    let (config, findings) = scan(
        "section_after_nested_declaration",
        "# FS-001-ci: CI\n\nThe CI lead.\n\n## FS-002-matrix: Matrix\n\nThe matrix body.\n\n\
         ## 1. Pull-request workflows\n\nThe workflows body.\n\n\
         ## pr: Named workflows\n\nThe named body.\n",
    );
    assert_eq!(section_paths(&findings, 1, "ci"), ["1", "pr"]);
    assert_eq!(section_paths(&findings, 2, "matrix"), Vec::<String>::new());
    assert!(
        findings.section_headings_outside_declarations.is_empty(),
        "both headings lie inside FS-001-ci's body"
    );
    let report = check_findings(&findings, &config);
    assert!(
        !error_codes(&report)
            .iter()
            .any(|code| code == "section-outside-declaration"),
        "{:?}",
        error_codes(&report)
    );
}

/// Bodies nested two deep, and a nested declaration inside a section: each heading goes to the
/// innermost declaration whose body holds it.
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn the_innermost_enclosing_body_owns_the_heading() {
    let (_, findings) = scan(
        "section_after_nested_declaration_innermost",
        "# FS-001-ci: CI\n\nThe CI lead.\n\n## 1. Overview\n\nThe overview.\n\n\
         ### FS-002-deep: Deep\n\nThe deep body.\n\n### 1.2 Second\n\nThe second body.\n\n\
         ## FS-003-outer: Outer\n\nThe outer body.\n\n### FS-004-inner: Inner\n\nThe inner.\n\n\
         ### 1. Back in outer\n\nBack in outer.\n\n## 2. Back in ci\n\nBack in ci.\n",
    );
    assert_eq!(section_paths(&findings, 1, "ci"), ["1", "1.2", "2"]);
    assert_eq!(section_paths(&findings, 2, "deep"), Vec::<String>::new());
    assert_eq!(section_paths(&findings, 3, "outer"), ["1"]);
    assert_eq!(section_paths(&findings, 4, "inner"), Vec::<String>::new());
}

/// `## 1.` before the nested declaration and `## 1.` after it are two claimants of `FS-001-ci.1`:
/// the first holds the map, the second is on the duplicate list, and check reports it.
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn a_path_repeated_across_a_nested_declaration_is_a_duplicate() {
    let (config, findings) = scan(
        "section_after_nested_declaration_duplicate",
        "# FS-001-ci: CI\n\nThe CI lead.\n\n## 1. Before\n\nBefore.\n\n\
         ## FS-002-matrix: Matrix\n\nThe matrix body.\n\n## 1. After\n\nAfter.\n",
    );
    let ci = &findings.declarations[&id(1, "ci")][0];
    assert_eq!(ci.sections.keys().collect::<Vec<_>>(), ["1"]);
    assert_eq!(
        ci.duplicate_sections
            .iter()
            .map(|(path, info)| (path.as_str(), info.line))
            .collect::<Vec<_>>(),
        [("1", 13)]
    );
    let report = check_findings(&findings, &config);
    assert!(
        error_codes(&report)
            .iter()
            .any(|code| code == "duplicate-section"),
        "{:?}",
        error_codes(&report)
    );
}
