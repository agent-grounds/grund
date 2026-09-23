//! Test module: configuration validation (§AR-core-module-layout.1.5)

use super::load_config;
use crate::testing::{test_root, write};

/// §FS-config.3.1.8: the load-time invariant the three budget keys carry —
/// `inline_note_suggested_lines ≤ inline_note_max_lines` — so a config that
/// suggests more lines than it permits is refused rather than loaded.
#[test]
fn inline_note_config_rejects_soft_cap_above_hard_cap() {
    let root = test_root("inline_note_config_rejects_soft_cap_above_hard_cap");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\ninline_note_suggested_lines = 4\ninline_note_max_lines = 3\n",
    );

    let err = match load_config(&root) {
        Ok(_) => panic!("invalid inline-note caps should fail"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("reference.inline_note_suggested_lines must be <= inline_note_max_lines"),
        "unexpected error: {err:#}"
    );
}

/// §FS-config.3: `project_description` parses as optional one-line
/// top-level metadata next to `project_name`.
#[test]
fn config_parses_project_description() {
    let root = test_root("config_parses_project_description");
    write(
        &root.join("grund.toml"),
        "project_name = \"api\"\nproject_description = \"Payment API service\"\n",
    );

    let config = load_config(&root).expect("load config");
    assert_eq!(
        config.project_description.as_deref(),
        Some("Payment API service")
    );
}

/// §FS-config.3: a `project_description` with an embedded line break is a
/// config error at the offending line — the key feeds single-line
/// workspace member bullets.
#[test]
fn config_rejects_multiline_project_description() {
    let root = test_root("config_rejects_multiline_project_description");
    write(
        &root.join("grund.toml"),
        "project_description = \"first\\nsecond\"\n",
    );

    let err = match load_config(&root) {
        Ok(_) => panic!("multi-line project_description should fail"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("project_description must be a single line"),
        "unexpected error: {err:#}"
    );
}

/// §FS-config.3.4.12: `rules = true` loads on a citable, scanned Markdown kind
/// with a home, and every other shape is refused at load, before any title is
/// read as a sentence.
#[test]
fn rules_key_requires_a_citable_scanned_markdown_home() {
    let root = test_root("rules_key_requires_a_citable_scanned_markdown_home");
    let load = |kind_row: &str| {
        write(
            &root.join("grund.toml"),
            &format!(
                "grund_config_version = 1\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\n\n[[kinds]]\n{kind_row}"
            ),
        );
        load_config(&root)
    };

    let config =
        load("kind = \"RULE\"\nfolder = \"docs/rules\"\nrules = true\n").expect("rule kind");
    assert!(
        config
            .kinds
            .iter()
            .any(|kind| kind.kind == "RULE" && kind.rules)
    );
    assert!(
        config
            .kinds
            .iter()
            .any(|kind| kind.kind == "FS" && !kind.rules)
    );

    for (shape, kind_row) in [
        (
            "non-citable",
            "kind = \"skill\"\nfolder = \"skills\"\ncitable = false\nrules = true\n",
        ),
        ("homeless", "kind = \"RULE\"\nrules = true\n"),
        (
            "not Markdown",
            "kind = \"RULE\"\nfile = \"docs/rules.txt\"\nrules = true\n",
        ),
        (
            "unwalked",
            "kind = \"RULE\"\nfolder = \"docs/rules\"\nscan = false\nrules = true\n",
        ),
        (
            "JSON-value",
            "kind = \"RULE\"\nfile = \"docs/rules.json\"\nvalues = true\nrules = true\n",
        ),
        (
            "external-snapshot",
            "kind = \"RULE\"\nfile = \"docs/rules.md\"\nfetch = \"scripts/fetch-rules\"\nrules = true\n",
        ),
    ] {
        let err = match load(kind_row) {
            Ok(_) => panic!("a {shape} rule kind should be refused"),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains(
                "sets `rules = true` but rule kinds must be citable, scanned Markdown kinds"
            ),
            "{shape}: unexpected error: {err}"
        );
    }

    let err = match load("kind = \"RULE\"\nfolder = \"docs/rules\"\nrules = true\nrules = true\n") {
        Ok(_) => panic!("a repeated key should be refused"),
        Err(err) => err.to_string(),
    };
    assert!(
        err.contains("[[kinds]] sets `rules` twice"),
        "unexpected error: {err}"
    );
}

/// §FS-config.3.4.13: `value_chapter` loads on a citable, scanned kind with one
/// existing home in a project that enables named sections, and every other shape
/// is refused at load — before any declaration's chapter is read.
/// §FS-values.2.5: authority comes from this key alone, so a project that never
/// sets it can never grow a chapter root.
#[test]
fn value_chapter_key_requires_named_sections_and_a_scanned_home() {
    let root = test_root("value_chapter_key_requires_named_sections_and_a_scanned_home");
    std::fs::create_dir_all(root.join("docs/architecture")).expect("create folder home");
    write(&root.join("values.json"), "{}\n");
    let load = |id_block: &str, kind_row: &str| {
        write(
            &root.join("grund.toml"),
            &format!(
                "grund_config_version = 1\n{id_block}\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\n\n[[kinds]]\n{kind_row}"
            ),
        );
        load_config(&root)
    };
    let named = "\n[id]\nnamed_sections = true\n";

    load(
        named,
        "kind = \"AR\"\nfolder = \"docs/architecture\"\nvalue_chapter = \"values\"\n",
    )
    .expect("a citable, scanned, homed kind takes the key under named sections");

    for (shape, id_block, kind_row, expected) in [
        (
            "without named sections",
            "",
            "kind = \"AR\"\nfolder = \"docs/architecture\"\nvalue_chapter = \"values\"\n",
            "[[kinds]] sets `value_chapter` but [id] named_sections is not true",
        ),
        (
            "non-citable",
            named,
            "kind = \"skill\"\nfolder = \"docs/architecture\"\ncitable = false\nvalue_chapter = \"values\"\n",
            "kind `skill` sets `value_chapter` with `citable = false`",
        ),
        (
            "homeless",
            named,
            "kind = \"AR\"\nvalue_chapter = \"values\"\n",
            "kind `AR` sets `value_chapter` without exactly one `file` or `folder` home",
        ),
        (
            "unwalked",
            named,
            "kind = \"AR\"\nfolder = \"docs/architecture\"\nscan = false\nvalue_chapter = \"values\"\n",
            "kind `AR` sets `value_chapter` with `scan = false`",
        ),
        (
            "whole-value",
            named,
            "kind = \"AR\"\nfile = \"values.json\"\nvalues = true\nvalue_chapter = \"values\"\n",
            "kind `AR` sets `value_chapter` and `values = true`",
        ),
        (
            "off-grammar handle",
            named,
            "kind = \"AR\"\nfolder = \"docs/architecture\"\nvalue_chapter = \"Values\"\n",
            "[[kinds]] `value_chapter` must be a section handle matching `[a-z][a-z0-9-]*`",
        ),
        (
            "set twice",
            named,
            "kind = \"AR\"\nfolder = \"docs/architecture\"\nvalue_chapter = \"values\"\nvalue_chapter = \"specs\"\n",
            "[[kinds]] sets `value_chapter` twice",
        ),
        (
            "missing home",
            named,
            "kind = \"AR\"\nfolder = \"docs/absent\"\nvalue_chapter = \"values\"\n",
            "value home for kind `AR` does not exist",
        ),
    ] {
        let err = match load(id_block, kind_row) {
            Ok(_) => panic!("a {shape} value chapter should be refused"),
            Err(err) => err.to_string(),
        };
        assert!(err.contains(expected), "{shape}: unexpected error: {err}");
        assert!(
            !err.contains("unknown config key"),
            "{shape}: the key is refused as unknown rather than validated: {err}"
        );
    }

    // §FS-config.3.4.13: absent and set to nothing are the same, so the key's
    // own absence is not a nameless chapter waiting for a declaration.
    let config = load(
        named,
        "kind = \"AR\"\nfolder = \"docs/architecture\"\nvalue_chapter = \"\"\n",
    )
    .expect("an empty handle is the key's absence");
    assert!(
        config.kinds.iter().all(|kind| kind.value_chapter.is_none()),
        "an empty handle left a chapter behind"
    );
}
