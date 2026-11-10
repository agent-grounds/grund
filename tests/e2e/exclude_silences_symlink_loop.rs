//! A version-2 `[schema.sources] exclude` pattern that matches a looping directory link below a
//! root keeps the link and its loop out of the scan, as it does any other entry
//! (§FS-config-v2.schema.sources.exclude.roots, §FS-check.2.4.3): the loop's error offers "keep
//! it out of the scan", and in version 2 `exclude` is that fix. Both ways a loop is found are
//! pinned — the walker's own detection of a link to its ancestor, and the pruning of a link whose
//! target is at or above the walk root (§FS-config.3.5.5) — beside the control that an
//! un-excluded loop is still reported. The trees are built in the temp directory, because a
//! committed directory link is a different tree on a platform that cannot check one out.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;

const LOOP: &str = "symlink loop";

/// One version-2 project under the temp directory, removed on drop: an `FS` kind homed at
/// `docs/`, one cited declaration, and the directory link `link -> target`.
struct Tree(PathBuf);

impl Tree {
    /// `exclude` is the body of the `[schema.sources] exclude` array, or `None` for no table.
    fn new(name: &str, link: &str, target: &str, exclude: Option<&str>) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1333-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/fs")).expect("create the kind home");
        let mut config = String::from(
            "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
             slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.FS]\nfolders = [\"docs\"]\n\
             index = false\n",
        );
        if let Some(exclude) = exclude {
            config.push_str(&format!("\n[schema.sources]\nexclude = [{exclude}]\n"));
        }
        fs::write(root.join("grund.toml"), config).expect("write the config");
        fs::write(
            root.join("docs/a.md"),
            "# FS-a: A\n\nText, see \u{a7}FS-a.\n",
        )
        .expect("write");
        symlink(target, root.join(link)).expect("create the link");
        Self(root)
    }

    fn check(&self) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        Run {
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn shown(&self) -> String {
        format!(
            "exit {}\nstdout:\n{}\nstderr:\n{}",
            self.code, self.stdout, self.stderr
        )
    }

    fn reports_loop(&self) -> bool {
        self.stdout.contains(LOOP) || self.stderr.contains(LOOP)
    }
}

/// Asserts the run is clean: exit `0`, no loop line, and `success`.
fn assert_silenced(run: &Run, spelling: &str) {
    assert!(
        !run.reports_loop(),
        "exclude {spelling} left the loop:\n{}",
        run.shown()
    );
    assert_eq!(run.code, 0, "exclude {spelling}:\n{}", run.shown());
}

#[test]
fn an_unexcluded_loop_below_a_root_is_still_reported() {
    let tree = Tree::new("bare", "docs/fs/self", ".", None);
    let run = tree.check();
    assert_eq!(run.code, 2, "{}", run.shown());
    assert!(
        run.stderr.contains(
            "error: docs/fs/self: symlink loop: the target is the ancestor directory docs/fs; \
             make the path readable, or keep it out of the scan"
        ),
        "{}",
        run.shown()
    );
}

#[test]
#[ignore = "red until #1333 makes symlink_loop_report ask the version 2 exclude globs; implement removes this"]
fn an_exclude_matching_a_link_to_its_ancestor_keeps_the_loop_out() {
    for (name, spelling) in [
        ("name", "\"self\""),
        ("dir", "\"self/\""),
        ("anchored", "\"/docs/fs/self\""),
        ("path", "\"docs/fs/self\""),
        ("globstar", "\"**/self\""),
        ("wildcard", "\"self*\""),
    ] {
        let tree = Tree::new(
            &format!("ancestor-{name}"),
            "docs/fs/self",
            ".",
            Some(spelling),
        );
        assert_silenced(&tree.check(), spelling);
    }
}

#[test]
fn an_unexcluded_link_above_the_walk_root_is_still_reported() {
    let tree = Tree::new("above-bare", "docs/up", "..", None);
    let run = tree.check();
    assert_eq!(run.code, 2, "{}", run.shown());
    assert!(
        run.stderr.contains("error: docs/up: symlink loop:"),
        "{}",
        run.shown()
    );
}

#[test]
#[ignore = "red until #1333 makes symlink_loop_report ask the version 2 exclude globs; implement removes this"]
fn an_exclude_matching_a_link_above_the_walk_root_keeps_the_loop_out() {
    for (name, spelling) in [("name", "\"up\""), ("anchored", "\"/docs/up\"")] {
        let tree = Tree::new(&format!("above-{name}"), "docs/up", "..", Some(spelling));
        assert_silenced(&tree.check(), spelling);
    }
}
