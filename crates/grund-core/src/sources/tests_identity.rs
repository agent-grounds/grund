//! One answer to "is this one file" (§AR-system.2.13, §FS-scan.identity.buffers): a path not yet
//! written is keyed through its existing prefix, so a directory link and its target agree on it.
//! So is a file that exists and cannot be stated, in a directory listed but not searched.

use std::fs;

use super::identity::physical_path_key;
use crate::testing::{DirectoryMode, test_root};

#[test]
fn an_unsaved_file_under_a_directory_link_is_one_key_with_its_target() {
    let root = std::env::temp_dir().join(format!("grund-1274-key-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("notes")).unwrap();
    std::os::unix::fs::symlink("notes", root.join("alias")).unwrap();
    let through_link = physical_path_key(&root.join("alias/new.md"));
    let direct = physical_path_key(&root.join("notes/new.md"));
    fs::remove_dir_all(&root).unwrap();
    assert_eq!(through_link, direct);
}

/// A directory link and its target are one location even where the whole-path resolve is denied:
/// `values` at 0444 is listed but not searched, so stating the file fails and only the prefix walk
/// finds that the two spellings meet (§FS-scan.identity.buffers).
#[test]
fn a_file_that_cannot_be_stated_under_a_directory_link_is_one_key_with_its_target() {
    let root = test_root("unstatable_file_is_one_key");
    fs::create_dir_all(root.join("values")).unwrap();
    fs::write(root.join("values/a.json"), "{}\n").unwrap();
    std::os::unix::fs::symlink("values", root.join("alias")).unwrap();
    let _barred = DirectoryMode::set(&root.join("values"), 0o444);
    let denied = fs::metadata(root.join("values/a.json"))
        .is_err_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied);
    if !denied {
        eprintln!("skipped: stating the file was not denied, as when running as root");
        return;
    }
    assert_eq!(
        physical_path_key(&root.join("alias/a.json")),
        physical_path_key(&root.join("values/a.json"))
    );
}
