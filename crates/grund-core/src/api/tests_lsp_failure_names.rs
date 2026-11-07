//! Test module: the editor's report asks a scan failure's names as `grund check <path>` does, so
//! a failure under the anchor stays an error under any of them, in the tree where the name the
//! whole run keeps is under the anchor neither as written nor resolved (§FS-check.1.3.6.3,
//! §FS-lsp.4.1, §AR-scanner.1.entry). Unix-only: the tree is links and a mode.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::lsp_snapshot::lsp_snapshot;
use crate::queries::LspSnapshotOpts;
use crate::testing::{physical_test_root, symlink, write};

/// A workspace root with `aa -> zz`, `realE/lnk -> ../zz` and `zz/x` barred, so its whole walk
/// keeps `aa/x`; `None` where the mode does not bar this user.
fn third_name_tree(name: &str) -> Option<PathBuf> {
    let root = physical_test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"q\"\n\n[schema]\n\
         slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.doc]\ncitable = false\n\
         folders = [\"docs\"]\n\n[workspace]\nmembers = [\"m\"]\n",
    );
    write(
        &root.join("m/grund.toml"),
        "grund_config_version = 2\nproject_name = \"m\"\n\n[schema]\n\
         slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.FS]\nfolders = [\"spec\"]\n",
    );
    fs::create_dir_all(root.join("m/spec")).expect("create the member's home");
    write(&root.join("docs/notes.md"), "# Notes\n");
    fs::create_dir_all(root.join("realE")).expect("create the directory");
    fs::create_dir_all(root.join("zz/x")).expect("create the barred directory");
    symlink("zz", &root.join("aa"));
    symlink("../zz", &root.join("realE/lnk"));
    fs::set_permissions(root.join("zz/x"), fs::Permissions::from_mode(0o000)).expect("bar it");
    if fs::read_dir(root.join("zz/x")).is_ok() {
        fs::set_permissions(root.join("zz/x"), fs::Permissions::from_mode(0o755)).expect("free it");
        return None;
    }
    Some(root)
}

/// The paths of the `io` errors a snapshot anchored at `rel` reports.
fn io_errors(root: &Path, rel: &str) -> Vec<String> {
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: root.join(rel),
        path_provided: true,
        ..LspSnapshotOpts::default()
    })
    .expect("build the snapshot");
    snapshot
        .report
        .errors
        .iter()
        .filter(|finding| finding.code == "io")
        .map(|finding| finding.path.clone().expect("a located failure"))
        .collect()
}

#[test]
fn a_failure_under_the_anchor_stays_an_error_under_any_of_its_names() {
    let Some(root) = third_name_tree("a_failure_under_the_anchor_stays_an_error") else {
        return;
    };

    let under_the_anchor = io_errors(&root, "realE");
    let under_another_name = io_errors(&root, "zz");
    let outside = io_errors(&root, "docs");
    fs::set_permissions(root.join("zz/x"), fs::Permissions::from_mode(0o755)).expect("free it");

    // An editor is told a finding's resolved path (§FS-lsp.4.1), so every name reads `zz/x`. What
    // the names decide here is that the anchor `realE` holds the failure at all.
    let one_failure = |paths: &[String]| paths.len() == 1 && paths[0].ends_with("zz/x");
    assert!(one_failure(&under_the_anchor), "{under_the_anchor:?}");
    assert!(one_failure(&under_another_name), "{under_another_name:?}");
    assert!(outside.is_empty(), "{outside:?}");
}
