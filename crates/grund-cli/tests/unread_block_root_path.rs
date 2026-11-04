//! The unread-workspace-block warning over a grouping block whose own directory
//! is the unread tree (agent-grounds/grund#1145): a version-2 root listing
//! `group`, `group/grund.toml` with `include_root = false` and one member `a`,
//! and a `group/README.md` nobody scans. The tree is named under the block root,
//! so the block root itself is `.`, from every run base (§FS-check.4.10.5).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const TAIL: &str = ", so its citations are never checked. Set include_root = true, \
or configure another project's source scope to read it.\n";

/// The issue's tree under the system temp directory, where no config above it
/// is discovered.
fn tree(name: &str) -> PathBuf {
    let root = std::env::temp_dir()
        .join(format!("grund-unread-block-root-{}", std::process::id()))
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("group/a")).expect("create fixture");
    let files = [
        (
            "grund.toml",
            "grund_config_version = 2\n\n[workspace]\nmembers = [\"group\"]\n\n[schema.kinds.FS]\n",
        ),
        (
            "README.md",
            "# Root\n\nThe root cites \u{a7}group/a/FS-a.\n",
        ),
        (
            "group/grund.toml",
            "grund_config_version = 2\n\n[workspace]\ninclude_root = false\nmembers = [\"a\"]\n",
        ),
        ("group/README.md", "# Group\n\nCites \u{a7}FS-x.\n"),
        (
            "group/a/grund.toml",
            "grund_config_version = 2\n\n[schema.kinds.FS]\n",
        ),
        (
            "group/a/README.md",
            "# FS-a: A\n\nThe member cites \u{a7}FS-a.\n",
        ),
    ];
    for (path, body) in files {
        fs::write(root.join(path), body).expect("write fixture file");
    }
    root
}

fn check(cwd: &Path) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("check")
        .current_dir(cwd)
        .output()
        .expect("spawn grund");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// §FS-check.4.10.5: run from the block, the location is the block's own
/// `grund.toml` and the unread tree, the block root, is `.`.
#[test]
#[ignore = "red until #1145 names a block root that is the unread tree `.`; implement removes this"]
fn block_root_is_named_dot_from_the_block() {
    let root = tree("from-block");
    let stderr = format!("warning: grund.toml:4: no project scans `.`{TAIL}");
    assert_eq!(check(&root.join("group")), (Some(0), String::new(), stderr));
}

/// §FS-check.4.10.5: run from the workspace root, the location moves to the
/// run's root and the unread tree stays `.`, because it is named under the block root.
#[test]
#[ignore = "red until #1145 names a block root that is the unread tree `.`; implement removes this"]
fn block_root_is_named_dot_from_the_workspace_root() {
    let root = tree("from-root");
    let stderr = format!("warning: group/grund.toml:4: no project scans `.`{TAIL}");
    assert_eq!(check(&root), (Some(0), String::new(), stderr));
}
