//! Test module: strict formatter scan errors across the public API (§FS-fmt.3).

use super::*;
use crate::scanner::ApiScanError;
use crate::testing::{test_root, write};
use crate::writers::FmtScanAbort;

/// A whole-declaration-set rewrite keeps its fatal scan result structured
/// for embedding callers, with every path in scan order.
#[test]
fn public_fmt_api_preserves_every_strict_scan_error() {
    let root = test_root("public_fmt_api_preserves_every_strict_scan_error");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("docs/FS-001-alpha.md"),
        "# FS-001-alpha: Alpha\n\nCites §FS-001-alpha.\n",
    );
    std::os::unix::fs::symlink("nowhere.md", root.join("docs/FS-002-gone.md"))
        .expect("create first broken symlink");
    std::os::unix::fs::symlink("nowhere-either.md", root.join("docs/FS-003-also-gone.md"))
        .expect("create second broken symlink");

    let error = format_references(FmtOpts {
        path: root,
        path_provided: true,
        cross_refs: true,
        ..FmtOpts::default()
    })
    .expect_err("strict formatter must reject an incomplete declaration set");
    let abort = error
        .downcast_ref::<FmtScanAbort>()
        .expect("strict errors remain structured across the public API");

    assert_eq!(
        abort.scan_errors,
        vec![
            ApiScanError {
                path: "docs/FS-002-gone.md".to_string(),
                message: "broken symlink: the target does not exist; make the path readable, or keep it out of the scan".to_string(),
            },
            ApiScanError {
                path: "docs/FS-003-also-gone.md".to_string(),
                message: "broken symlink: the target does not exist; make the path readable, or keep it out of the scan".to_string(),
            },
        ]
    );
    assert_eq!(
        abort.to_string(),
        concat!(
            "nothing was rewritten: docs/FS-002-gone.md: broken symlink: the target does not exist; make the path readable, or keep it out of the scan\n",
            "nothing was rewritten: docs/FS-003-also-gone.md: broken symlink: the target does not exist; make the path readable, or keep it out of the scan",
        )
    );
}

/// §FS-fmt.7.4.3: with cross-references off no preflight reads the tree, so a file
/// the walk cannot read is met after it has planned the first file's `$$`; the run
/// still writes nothing and its partial output lists no change.
#[test]
fn a_read_failure_partway_through_the_walk_rewrites_nothing() {
    let root = test_root("a_read_failure_partway_through_the_walk_rewrites_nothing");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[scan]\ninclude = [\"docs\"]\n\n[fmt.cross_refs]\nenabled = false\n",
    );
    let first = "# FS-001-alpha: Alpha\n\nCites $$FS-001-alpha.\n";
    write(&root.join("docs/a.md"), first);
    std::fs::write(
        root.join("docs/z.md"),
        b"# FS-002-beta: Beta\n\nbad \xff bytes\n",
    )
    .expect("write a file that is not UTF-8");

    let error = format_references(FmtOpts {
        path: root.clone(),
        path_provided: true,
        write: true,
        ..FmtOpts::default()
    })
    .expect_err("a file the walk cannot read refuses the run");
    assert!(
        error.downcast_ref::<FmtScanAbort>().is_none(),
        "the walk, not a preflight, refuses"
    );
    let failure = super::embedding_failure::error_data(error, &[]);

    assert_eq!(failure["partial_output"]["changes"], serde_json::json!([]));
    assert_eq!(
        std::fs::read_to_string(root.join("docs/a.md")).expect("read docs/a.md"),
        first
    );
}

/// §FS-distribution.3.3.2: an fmt refused for a conflict block raises `conflict-marker`
/// at the block's first line, carries every block's finding and writes nothing
/// (§FS-declarations.checks.conflict-marker, §FS-fmt.7.4).
#[test]
fn a_conflict_block_refuses_fmt_across_the_public_api() {
    let root = test_root("a_conflict_block_refuses_fmt_across_the_public_api");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema]\nmarker = \"§\"\n\n[schema.kinds.FS]\n",
    );
    let login = concat!(
        "# FS-login: Login\n\n## format: Format\n\n### .1: Username\n\n",
        "<<<<<<< HEAD\nA username is 2 to 32 characters.\n=======\n",
        "A username is 3 to 64 characters.\n>>>>>>> other\n",
    );
    let usage = "# Use\n\nUsernames follow \u{a7}FS-login.format.1.\nSee also $$FS-login.\n";
    write(&root.join("docs/login.md"), login);
    write(&root.join("docs/use.md"), usage);

    let error = format_references(FmtOpts {
        path: root.clone(),
        path_provided: true,
        write: true,
        ..FmtOpts::default()
    })
    .expect_err("a conflict block refuses the run");
    let failure = super::embedding_failure::error_data(error, &[]);

    assert_eq!(failure["kind"], "operation");
    assert_eq!(failure["code"], "conflict-marker");
    assert_eq!(failure["path"], "docs/login.md");
    assert_eq!(failure["line"], 7);
    assert_eq!(failure["sites"].as_array().map(Vec::len), Some(1));
    assert_eq!(failure["sites"][0]["line"], 7);
    assert_eq!(failure["partial_output"]["changes"], serde_json::json!([]));
    let conflicts = &failure["details"]["conflicts"];
    assert_eq!(conflicts.as_array().map(Vec::len), Some(1));
    assert_eq!(conflicts[0]["code"], "conflict-marker");
    for (name, text) in [("docs/login.md", login), ("docs/use.md", usage)] {
        let now = std::fs::read_to_string(root.join(name)).expect("read a fixture back");
        assert_eq!(now, text, "{name} is left as the run found it");
    }
    assert!(!root.join(".grund/sections").exists(), "no lock is written");
}
