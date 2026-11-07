//! Test module: a path that does not resolve keeps its components and nothing else
//! (§FS-scan.identity.unresolved), read at the public seam as text, because `Path` compares by
//! component and `a/b/` equals `a/b`: an assertion between `PathBuf`s passes on the defect.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::*;
use crate::LspSnapshotOpts;
use crate::sources::canonical_snapshot_path;
use crate::testing::{symlink, test_root, write};

/// A project at its canonical path, so what the tests expect is spelled as the snapshot spells it.
fn project(name: &str) -> PathBuf {
    let root = fs::canonicalize(test_root(name)).expect("canonical test root");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"Repro\"\n",
    );
    write(&root.join("real/ok.md"), "# A note\n");
    root
}

fn text(path: &Path) -> &str {
    path.to_str().expect("a UTF-8 test path")
}

/// Every `path` of `lsp_snapshot`'s `report.errors`, as the `String` a caller is handed.
fn finding_paths(root: &Path) -> Vec<String> {
    lsp_snapshot(LspSnapshotOpts {
        path: root.to_path_buf(),
        path_provided: true,
        open_documents: Default::default(),
    })
    .expect("snapshot")
    .report
    .errors
    .into_iter()
    .filter_map(|finding| finding.path)
    .collect()
}

/// Puts a directory's mode back before the temp dir is cleaned up.
struct ModeGuard<'a>(&'a Path);

impl Drop for ModeGuard<'_> {
    fn drop(&mut self) {
        let _ = fs::set_permissions(self.0, fs::Permissions::from_mode(0o755));
    }
}

/// The control: this passes before the fix too, where `fs::canonicalize` answers for the whole path.
#[test]
fn an_existing_path_reads_as_its_canonical_text() {
    let root = project("an_existing_path_reads_as_its_canonical_text");
    let path = root.join("real/ok.md");
    assert_eq!(text(&canonical_snapshot_path(&path)), text(&path));
}

#[test]
#[ignore = "red until #1300 stops joining an empty path; implement removes this"]
fn a_path_with_one_missing_component_ends_in_that_component() {
    let root = project("a_path_with_one_missing_component_ends_in_that_component");
    let path = root.join("real/unsaved.md");
    assert_eq!(text(&canonical_snapshot_path(&path)), text(&path));
}

#[test]
#[ignore = "red until #1300 stops joining an empty path; implement removes this"]
fn a_path_with_several_missing_components_ends_in_the_last() {
    let root = project("a_path_with_several_missing_components_ends_in_the_last");
    let path = root.join("real/a/b/unsaved.md");
    assert_eq!(text(&canonical_snapshot_path(&path)), text(&path));
}

#[test]
#[ignore = "red until #1300 stops joining an empty path; implement removes this"]
fn a_broken_link_finding_reads_as_the_link_path() {
    let root = project("a_broken_link_finding_reads_as_the_link_path");
    symlink("nowhere.md", &root.join("real/broken.md"));
    let expected = root.join("real/broken.md");
    let paths = finding_paths(&root);
    assert!(
        paths.iter().any(|path| path == text(&expected)),
        "no finding reads {}: {paths:?}",
        expected.display()
    );
}

#[test]
#[ignore = "red until #1300 stops joining an empty path; implement removes this"]
fn an_unsearchable_entry_finding_reads_as_the_entry_path() {
    let root = project("an_unsearchable_entry_finding_reads_as_the_entry_path");
    let dir = root.join("d");
    let expected = dir.join("sub");
    fs::create_dir_all(&expected).expect("create d/sub");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o444)).expect("set the mode");
    let _guard = ModeGuard(&dir);
    // Mode 444 does not bar root, which would prove nothing, so the test passes without asserting.
    if fs::symlink_metadata(&expected).is_ok() {
        return;
    }
    let paths = finding_paths(&root);
    assert!(
        paths.iter().any(|path| path == text(&expected)),
        "no finding reads {}: {paths:?}",
        expected.display()
    );
}

/// The path that exists but has no canonical form: the kernel opens a deleted file through its
/// descriptor's `/proc` link, and `realpath` follows that link's text to a name that is gone. No
/// component is missing, so this reaches the final join with nothing to append, and needs no race.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "red until #1300 stops joining an empty path; implement removes this"]
fn an_existing_path_with_no_canonical_form_keeps_its_lexical_text() {
    use std::os::fd::AsRawFd;

    let root = project("an_existing_path_with_no_canonical_form_keeps_its_lexical_text");
    let deleted = root.join("deleted");
    let file = fs::File::create(&deleted).expect("create the file");
    fs::remove_file(&deleted).expect("delete it, keeping the descriptor");
    let path = PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()));
    // Without a mounted `/proc` the case does not arise, so the test passes without asserting.
    if !path.exists() || fs::canonicalize(&path).is_ok() {
        return;
    }
    assert_eq!(text(&canonical_snapshot_path(&path)), text(&path));
}
