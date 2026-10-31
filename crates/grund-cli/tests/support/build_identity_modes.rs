//! A forced rebuild derives the right label after an executable-bit change
//! that leaves watched mtimes unchanged (§FS-distribution.4.1.1). Both directions
//! leave the ordinary next build's label and rerun unasserted, then require a
//! fresh derivation and two no-op builds through the real-Cargo harness.

use super::*;
use std::os::unix::fs::PermissionsExt;

fn expect_forced_mode_transition(initial: u32, changed: u32) {
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
    // No watched mtime moved: leave the next build's label and rerun unasserted.
    let _ = ws.build(root, &[]);
    ws.clean(root);
    ws.expect("after chmod and clean", root, &[], label, Rerun::Must);
    ws.settled("after chmod and clean", root, &[], label);
}

#[test]
fn forcing_a_rebuild_after_an_unstaged_executable_bit_change_updates_the_label() {
    expect_forced_mode_transition(0o644, 0o755);
}

#[test]
fn forcing_a_rebuild_after_reverting_an_unstaged_executable_bit_change_updates_the_label() {
    expect_forced_mode_transition(0o755, 0o644);
}
