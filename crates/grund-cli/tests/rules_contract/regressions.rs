//! Round-two regressions for chapter identity, canonical facts, selector
//! resolution and namespace grammar (§FS-rules.2, §FS-rules.5.1, §FS-rules.8).

use super::support::{assert_run, run, scratch, text, write};
use std::fs;

#[test]
fn chapter_handles_and_display_names_have_distinct_meanings() {
    let root = scratch("handle-versus-display");
    write(
        &root,
        "docs/fs/FS-demo.md",
        "# FS-demo: Renamed display\n\n## requirements: Acceptance criteria\n\nNo citation.\n",
    );
    let output = run(
        &root,
        &[
            "check",
            ".",
            "--rule",
            "Each FS must have exactly one requirements chapter.",
            "--only",
            "chapter-cardinality",
        ],
    );
    assert_run(
        &output,
        1,
        "docs/fs/FS-demo.md:1: error: FS-demo has 0 requirements chapters; --rule requires exactly one\n",
        "",
    );
    let output = run(&root, &["list", ".", "--selector", "FS.requirements"]);
    assert!(text(&output.stdout).contains("Acceptance criteria"));

    write(
        &root,
        "docs/fs/FS-demo.md",
        "# FS-demo: Renamed handle\n\n## needs: requirements\n\nNo citation.\n",
    );
    let output = run(&root, &["check", ".", "--only", "missing-citation"]);
    assert_run(&output, 0, "success\n", "");
    let output = run(&root, &["list", ".", "--selector", "FS.requirements"]);
    assert_run(&output, 0, "", "");
    let output = run(
        &root,
        &[
            "check",
            ".",
            "--rule",
            "Each FS must have exactly one requirements chapter.",
            "--only",
            "chapter-cardinality",
        ],
    );
    assert_run(&output, 0, "success\n", "");
}

#[test]
fn healthy_stub_and_inline_home_mint_one_rule_node() {
    let root = scratch("canonical-inline-home");
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    write(
        &root,
        "grund.toml",
        &config.replace("include = [\"docs\"]", "include = [\"docs\", \"src\"]"),
    );
    write(
        &root,
        "docs/ar/AR-inline.md",
        "# AR-inline: [src/lib.rs](../../src/lib.rs)\n",
    );
    write(
        &root,
        "src/lib.rs",
        "/// AR-inline: Inline architecture\n///\n/// Canonical inline body.\npub struct Inline;\n",
    );
    let output = run(&root, &["check", ".", "--only", "citation-cardinality"]);
    let stdout = text(&output.stdout);
    assert_eq!(
        stdout.matches("cites AR-inline 0 times").count(),
        1,
        "{stdout}"
    );
}

#[test]
fn resolved_section_citations_count_by_kind_but_missing_sections_make_no_edge() {
    let root = scratch("section-citation-identity");
    write(
        &root,
        "docs/req/REQ-demo.md",
        "# REQ-demo: Requirement\n\n## evidence: Evidence\n\nProof.\n",
    );
    write(
        &root,
        "docs/fs/FS-demo.md",
        "# FS-demo: Section target\n\n## requirements: Requirements\n\nCites \u{a7}REQ-demo.evidence.\n",
    );
    let output = run(&root, &["check", ".", "--only", "missing-citation"]);
    assert_run(&output, 0, "success\n", "");

    write(
        &root,
        "docs/fs/FS-demo.md",
        "# FS-demo: Missing section target\n\n## requirements: Requirements\n\nCites \u{a7}REQ-demo.missing.\n",
    );
    let output = run(&root, &["check", ".", "--only", "missing-citation"]);
    assert_run(
        &output,
        1,
        "docs/fs/FS-demo.md:3: error: FS-demo.requirements must cite REQ (RULE-requirements)\n",
        "",
    );
}

#[test]
fn exact_chapter_selectors_refuse_missing_and_ambiguous_coordinates_in_all_modes() {
    let root = scratch("exact-selector-resolution");
    for extra in [&[][..], &["--summary"][..], &["--size=words"][..]] {
        let mut args = vec!["list", ".", "--selector", "FS-demo.missing"];
        args.extend_from_slice(extra);
        let output = run(&root, &args);
        assert_eq!(output.status.code(), Some(2));
        assert!(text(&output.stderr).contains("literal subject FS-demo.missing does not resolve"));
    }
    write(
        &root,
        "docs/fs/FS-demo.md",
        "# FS-demo: Duplicate chapter\n\n## requirements: First\n\nOne.\n\n## requirements: Second\n\nTwo.\n",
    );
    for extra in [&[][..], &["--summary"][..], &["--size=words"][..]] {
        let mut args = vec!["list", ".", "--selector", "FS-demo.requirements"];
        args.extend_from_slice(extra);
        let output = run(&root, &args);
        assert_eq!(output.status.code(), Some(2));
        assert!(text(&output.stderr).contains("literal subject FS-demo.requirements is ambiguous"));
    }
}

#[test]
fn configured_separator_is_used_by_rules_and_normal_list_rows() {
    let root = scratch("rule-section-separator");
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    write(
        &root,
        "grund.toml",
        &config.replace(
            "format = \"{kind}-{slug}\"",
            "format = \"{kind}-{slug}\"\nsection_separator = \"#\"",
        ),
    );
    write(
        &root,
        "docs/rules/RULE-overview.md",
        "# RULE-overview: AR-overview#system-overview must cite each AR exactly once.\n\nBecause \u{a7}GOAL-rules.\n",
    );
    write(
        &root,
        "docs/fs/FS-demo.md",
        "# FS-demo: Custom separator\n\n## needs: requirements\n\nNo citation.\n",
    );
    let output = run(&root, &["list", ".", "--selector", "FS-demo#needs"]);
    assert!(text(&output.stdout).starts_with("FS-demo#needs  "));
    let output = run(
        &root,
        &["list", ".", "--selector", "FS-demo#needs", "--size=words"],
    );
    assert!(text(&output.stdout).starts_with("FS-demo#needs  "));
    let output = run(
        &root,
        &[
            "check",
            ".",
            "--rule",
            "FS-demo#needs must cite at least one REQ.",
            "--only",
            "missing-citation",
        ],
    );
    assert!(text(&output.stdout).contains("FS-demo#needs must cite REQ (--rule)"));
    let output = run(
        &root,
        &[
            "check",
            ".",
            "--rule",
            "FS-demo#missing must cite at least one REQ.",
            "--only",
            "invalid-rule",
        ],
    );
    assert!(text(&output.stdout).contains("literal subject FS-demo#missing does not resolve"));
}

#[test]
fn object_targets_validate_qualifiers_and_workspace_kinds() {
    let root = scratch("malformed-target-qualifier");
    let output = run(
        &root,
        &[
            "check",
            ".",
            "--rule",
            "Each FS must cite at least one bad//GOAL.",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("invalid namespace qualifier segment (empty)"));

    let workspace = scratch("workspace-target-kind");
    fs::remove_file(workspace.join("docs/fs/FS-demo.md")).expect("remove root fixture FS");
    write(
        &workspace,
        "grund.toml",
        "grund_config_version = 1\nproject_name = \"root\"\n\n[workspace]\nmembers = [\"app\", \"api\"]\n",
    );
    write(
        &workspace,
        "app/grund.toml",
        "grund_config_version = 1\nproject_name = \"app\"\n[id]\nformat = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n[scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &workspace,
        "app/docs/FS-demo.md",
        "# FS-demo: Workspace target\n\nCites §api/POLICY-allowed.\n",
    );
    write(
        &workspace,
        "api/grund.toml",
        "grund_config_version = 1\nproject_name = \"api\"\n[id]\nformat = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"POLICY\"\nfolder = \"docs\"\nindex = false\n[scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &workspace,
        "api/docs/POLICY-allowed.md",
        "# POLICY-allowed: Allowed\n\nPolicy.\n",
    );
    let output = run(
        &workspace,
        &[
            "check",
            ".",
            "--rule",
            "Each FS must cite at least one api/POLICY.",
            "--only",
            "missing-citation",
        ],
    );
    assert_ne!(output.status.code(), Some(2), "{}", text(&output.stderr));
    assert!(!text(&output.stdout).contains("must cite api/POLICY"));
}
