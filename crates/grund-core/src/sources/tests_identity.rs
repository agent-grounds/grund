//! One answer to "is this one file" (§AR-system.2.13, §FS-scan.identity.buffers): a path not yet
//! written is keyed through its existing prefix, so a directory link and its target agree on it.

use std::fs;

use super::identity::physical_path_key;

#[test]
#[ignore = "red until #1274 keys an unsaved file through its existing prefix; implement removes this"]
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
