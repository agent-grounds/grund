//! A declaration nested inside fetch output is refused at any depth, leaving the tree
//! byte-identical ([§FS-fetch.3.1](../../docs/functional-spec/FS-fetch.md#31-a-nested-declaration-is-refused)). The e2e cases `fetch-nested-*` pin each shape's
//! exact message; these pin what must hold whichever refusal fires first, so a change to
//! the subsection depth limit (#1148) cannot hide or fake the nested-ID refusal.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use support::*;

fn ar_project(name: &str, existing: &str, fetched: &str) -> std::path::PathBuf {
    let root = root(name);
    write(
        &root,
        "grund.toml",
        "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\n\
         slug_pattern = \"[0-9]{3}-[a-z][a-z-]*\"\nnamed_sections = true\n\n\
         [[kinds]]\nkind = \"AR\"\nfile = \"docs/arch.md\"\nfetch = \"scripts/fetch\"\n\n\
         [scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n",
    );
    write(&root, "docs/arch.md", existing);
    executable(
        &root,
        "scripts/fetch",
        &format!("#!/bin/sh\ncat <<'OUT'\n{fetched}\nOUT\n"),
    );
    root
}

/// Two depths below the native H2: refused today by the depth limit alone, and still
/// refused, naming the ID, once that limit no longer applies.
#[test]
fn external_facts_nested_declaration_two_depths_below_is_refused() {
    let root = ar_project(
        "nested-deeper",
        "",
        "## AR-001-thing: Thing\n\n#### AR-002-other: Other",
    );
    let before = tree_with_directories(&root);

    let output = run(&root, &["fetch", "AR-001-thing"]);

    assert_code(&output, 2, "nested two depths below");
    assert_eq!(tree_with_directories(&root), before);
}

/// An ID the home already declares: the tree is green before the fetch, the fetch is
/// refused, and the tree is still green and byte-identical after it.
#[test]
fn external_facts_nested_declaration_never_turns_a_green_tree_red() {
    let root = ar_project(
        "nested-collides",
        "## AR-002-other: Existing\n\nbody\n",
        "## AR-001-thing: Thing\n\n### AR-002-other: Stolen",
    );
    assert_code(&run(&root, &["check", "."]), 0, "green before the fetch");
    let before = tree_with_directories(&root);

    let output = run(&root, &["fetch", "AR-001-thing"]);

    assert_code(&output, 2, "nested ID the home declares");
    assert!(
        stderr(&output).contains("`AR-002-other`"),
        "the refusal names the nested ID: {}",
        stderr(&output)
    );
    assert_eq!(tree_with_directories(&root), before);
    assert_code(
        &run(&root, &["check", "."]),
        0,
        "still green after the fetch",
    );
}
