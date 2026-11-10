//! Black-box: a directory-only ignore pattern such as `docs/ext/` covers a link that names a
//! directory whether or not its target opens, a loop and a link to a target at mode 000 alike,
//! and does not cover a dangling link, which names a file (§FS-scan.scope.links.owed). The link
//! is judged by the directory rules under its link name, as a followed one is
//! (§FS-scan.scope.links.names). What is judged is the `path` of every `io` diagnostic in
//! `--format json`, and the exit code. Git carries no mode, so the e2e corpus cannot hold these
//! trees, and the mode-000 tests skip where the mode does not bar the user (root).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs\"]\n";

/// A clean v2 git project at `<temp>/p`, its one declaration cited, with `.gitignore` holding
/// `ignore`: a run over it reports nothing until a test adds a link the scan cannot resolve.
struct Tree {
    root: PathBuf,
    locked: Vec<PathBuf>,
}

struct Run {
    code: i32,
    stderr: String,
    /// The `path` of every `io` diagnostic, sorted.
    io: Vec<String>,
}

impl Tree {
    fn new(test: &str, ignore: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1334-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).expect("create the home");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        fs::write(root.join("docs/FS-b.md"), "# FS-b: B\n\nPlain text.\n")
            .expect("write the declaration");
        fs::write(
            root.join("docs/README.md"),
            "# Index\n\n- [\u{a7}FS-b](FS-b.md#fs-b-b)\n",
        )
        .expect("write the index");
        fs::write(root.join(".gitignore"), ignore).expect("write the ignore file");
        let init = Command::new("git")
            .args(["init", "-q", "."])
            .current_dir(&root)
            .status()
            .expect("run git");
        assert!(init.success());
        Self {
            root,
            locked: Vec::new(),
        }
    }

    /// Creates the link `<link>` pointing at `target`, written relative to the link.
    fn link(&self, link: &str, target: &str) {
        symlink(target, self.root.join(link)).expect("create the link");
    }

    /// Creates the directory `rel` with a dangling citation in it, then bars it, and says whether
    /// the mode bars this user: it does not bar root, and then the test proves nothing.
    fn barred_dir(&mut self, rel: &str) -> bool {
        let path = self.root.join(rel);
        fs::create_dir_all(&path).expect("create the directory");
        fs::write(path.join("x.md"), "# Notes\n\nSee \u{a7}FS-nope.\n").expect("write the note");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("set the mode");
        self.locked.push(path.clone());
        let barred = fs::read_dir(&path).is_err();
        if !barred {
            eprintln!("skipped: the mode does not bar this user, so nothing here fails to open");
        }
        barred
    }

    /// `grund check <args> --format json` from the project root.
    fn check(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .args(args)
            .args(["--format", "json"])
            .env("NO_COLOR", "1")
            .current_dir(&self.root)
            .output()
            .expect("run grund");
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let mut io: Vec<String> = stderr
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|record| record["code"] == "io")
            .map(|record| record["path"].as_str().unwrap_or("(none)").to_string())
            .collect();
        io.sort_unstable();
        Run {
            code: output.status.code().expect("exited with a status"),
            stderr,
            io,
        }
    }

    /// Asserts that a run over the project, and one over the home, exits `code` and reports an
    /// `io` failure at exactly `paths`.
    fn assert_reports(&self, code: i32, paths: &[&str]) {
        for args in [&[][..], &["docs"]] {
            let run = self.check(args);
            assert_eq!(
                (
                    run.code,
                    run.io.iter().map(String::as_str).collect::<Vec<_>>()
                ),
                (code, paths.to_vec()),
                "grund check {args:?}:\n{}",
                run.stderr
            );
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        for path in &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// The ticket's `slash` case: `docs/ext -> ../real` with `real` at mode 000, and `.gitignore`
/// holding `docs/ext/` and `real/`. The same lines keep the link out when `real` opens, so they
/// keep it out when it does not.
#[test]
fn a_directory_only_pattern_keeps_out_a_link_whose_target_cannot_be_opened() {
    let mut tree = Tree::new("slash", "docs/ext/\nreal/\n");
    tree.link("docs/ext", "../real");
    if !tree.barred_dir("real") {
        return;
    }
    tree.assert_reports(0, &[]);
}

/// The ticket's `loopslash` case: `docs/self -> .` names a directory, so `docs/self/` covers it.
#[test]
fn a_directory_only_pattern_keeps_out_a_loop() {
    let tree = Tree::new("loopslash", "docs/self/\n");
    tree.link("docs/self", ".");
    tree.assert_reports(0, &[]);
}

/// The walker's precedence holds on the error path as it does when the target opens: a later
/// `!docs/ext/` re-includes the link that `docs/ext` ignored, so its report is owed.
#[test]
fn a_directory_only_negation_re_includes_a_link_whose_target_cannot_be_opened() {
    let mut tree = Tree::new("renegate", "docs/ext\n!docs/ext/\nreal/\n");
    tree.link("docs/ext", "../real");
    if !tree.barred_dir("real") {
        return;
    }
    tree.assert_reports(2, &["docs/ext"]);
}

/// Control, green today: the ticket's `noslash` case, the workaround.
#[test]
fn a_pattern_without_the_slash_keeps_out_a_link_whose_target_cannot_be_opened() {
    let mut tree = Tree::new("noslash", "docs/ext\nreal/\n");
    tree.link("docs/ext", "../real");
    if !tree.barred_dir("real") {
        return;
    }
    tree.assert_reports(0, &[]);
}

/// Control, green today: a `!docs/ext/` after `docs/ext/` re-includes the link, and its report
/// stays.
#[test]
fn a_negated_directory_only_pattern_leaves_the_report() {
    let mut tree = Tree::new("negated", "docs/ext/\n!docs/ext/\nreal/\n");
    tree.link("docs/ext", "../real");
    if !tree.barred_dir("real") {
        return;
    }
    tree.assert_reports(2, &["docs/ext"]);
}

/// Control, green today: a dangling link names a file, so `docs/x.md/` does not cover it and its
/// report stays.
#[test]
fn a_directory_only_pattern_does_not_keep_out_a_dangling_link() {
    let tree = Tree::new("dangling", "docs/x.md/\n");
    tree.link("docs/x.md", "nowhere.md");
    tree.assert_reports(2, &["docs/x.md"]);
}
