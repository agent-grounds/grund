//! A Git-visible executable-bit change must update the label even when chmod
//! leaves the file's modification time unchanged (§FS-distribution.4.1.1).
//! The two directions are separate tests so a stale clean label cannot hide a
//! stale dirty label on reversal. These use the real-Cargo freshness harness.

use super::*;
use std::os::unix::fs::PermissionsExt;

fn expect_mode_transition(initial: u32, changed: u32) {
    let ws = Workspace::new("file-mode");
    let root = &ws.root;
    let path = root.join("docs/notes.md");
    ws.git(root, &["config", "core.filemode", "true"]);
    fs::set_permissions(&path, fs::Permissions::from_mode(initial)).unwrap();
    let clean = format!("{DEV}+{}", ws.head(root));
    let dirty = format!("{clean}.dirty");
    let initial_label = if initial == 0o644 { &clean } else { &dirty };
    ws.expect("before chmod", root, &[], initial_label, Rerun::Must);
    ws.settled("before chmod", root, &[], initial_label);

    let mtime = fs::metadata(&path).unwrap().modified().unwrap();
    let index = root.join(".git/index");
    let index_mtime = fs::metadata(&index).unwrap().modified().unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(changed)).unwrap();
    assert_eq!(mtime, fs::metadata(&path).unwrap().modified().unwrap());
    let status = ws.git(
        root,
        &[
            "--no-optional-locks",
            "status",
            "--porcelain",
            "--untracked-files=no",
        ],
    );
    let (label, expected_status) = if changed == 0o644 {
        (&clean, "")
    } else {
        (&dirty, " M docs/notes.md\n")
    };
    assert_eq!(status, expected_status);
    assert_eq!(
        index_mtime,
        fs::metadata(&index).unwrap().modified().unwrap()
    );
    ws.expect("after chmod, same mtime", root, &[], label, Rerun::Must);
    ws.settled("after chmod", root, &[], label);
}

#[test]
fn an_unstaged_executable_bit_change_updates_the_label() {
    expect_mode_transition(0o644, 0o755);
}

#[test]
fn reverting_an_unstaged_executable_bit_change_updates_the_label() {
    expect_mode_transition(0o755, 0o644);
}
