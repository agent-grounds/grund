//! A missing `<path>` given to `check` in a workspace whose root block leaves a
//! tree unread: the run still says the block's warning, then refuses the path
//! (§FS-scan.scope.homes.lookup, §FS-check.4.10.7). The library's refusal
//! of an unmarked path (§FS-scan.scope.homes.lookup) does not move
//! the CLI's marked path ahead of the warning.

#[path = "support/temporary_root.rs"]
mod temporary_root;
use temporary_root::TemporaryRoot;

use std::fs;
use std::process::Command;

#[test]
fn missing_path_is_refused_after_the_unread_block_warning() {
    // §FS-repository-maintenance.4.2: claim a direct leaf without an unowned pid parent.
    let root = TemporaryRoot::new(&std::env::temp_dir(), "grund-missing-path-run-warning");
    fs::create_dir_all(root.join("a/docs")).expect("create fixture");
    fs::create_dir_all(root.join("docs")).expect("create fixture");
    let files = [
        (
            "grund.toml",
            "grund_config_version = 1\n\n[workspace]\nmembers = [\"a\"]\ninclude_root = false\n",
        ),
        ("a/grund.toml", "grund_config_version = 1\n"),
        ("a/docs/FS-x.md", "# FS-x: X\n"),
        ("docs/FS-root.md", "# FS-root: Root\n"),
    ];
    for (path, body) in files {
        fs::write(root.join(path), body).expect("write fixture file");
    }
    let out = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["check", "nope"])
        .current_dir(&*root)
        .output()
        .expect("spawn grund");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "stderr: {stderr}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
    assert_eq!(
        stderr,
        "warning: grund.toml:5: no project scans `docs`, so its citations are never checked. \
         Set include_root = true, or configure another project's source scope to read it.\n\
         error: path does not exist: nope\n"
    );
}
