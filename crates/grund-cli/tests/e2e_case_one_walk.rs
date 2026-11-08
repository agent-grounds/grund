//! Black-box: the one walk reads the E2E cases (§FS-declarations.e2e-case.excluded), so `exclude`
//! and an ignore file keep a case out, and one unreadable manifest is one failure however many
//! spellings reach it (§FS-check.2.4.2, §FS-declarations.e2e-case.failure). Git carries neither a
//! mode nor a link a fixture could keep, so the e2e corpus cannot hold the second tree. The tests
//! that need a mode skip where it does not bar the user.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

const UNDECODABLE: &[u8] = b"\xff\xfe\n";
const SKIP: &str = "skipped: the mode does not bar this user, so nothing here can be unreadable";

/// A version-2 project whose `E2E` kind keeps cases `a`, `b` and `c` in `e2e/cases`, with
/// `notes.md` citing `a` and `c`; `sources` is written under `[schema.sources]`.
fn config(sources: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"p\"\n\n\
         [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
         [schema.sources]\n{sources}\n\
         [schema.kinds.E2E]\nfolders = [\"e2e/cases\"]\nindex = false\n"
    )
}

/// Tool report grund.273: `zz` links to `realE`, the kind's folder is spelled through the link, and
/// the include reaches the manifest under both spellings.
const LINKED: &str = "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\n\n\
    [[kinds]]\nkind = \"E2E\"\nfolder = \"zz/cases\"\nindex = false\n\n\
    [scan]\ninclude = [\"realE\", \"zz/cases/b\"]\nextensions = [\"refs\", \"md\"]\n";

struct Tree {
    root: PathBuf,
    locked: Vec<PathBuf>,
}

#[derive(Debug)]
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Tree {
    fn new(test: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("grund-1271-walk-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the root");
        Self {
            root,
            locked: Vec::new(),
        }
    }

    /// The exclusion tree: three healthy cases, `notes.md` citing `a` and `c`.
    fn cases(test: &str, sources: &str) -> Self {
        let tree = Self::new(test);
        tree.write("grund.toml", config(sources).as_bytes());
        for case in ["a", "b", "c"] {
            tree.write(&format!("e2e/cases/{case}/expected.exit"), b"0\n");
        }
        tree.write("notes.md", "# Notes\n\nSee §E2E-a and §E2E-c.\n".as_bytes());
        tree
    }

    /// The grund.273 tree: cases `a` and `b` under `realE/cases`, `zz` a link to `realE`.
    fn linked(test: &str) -> Self {
        let tree = Self::new(test);
        tree.write("grund.toml", LINKED.as_bytes());
        tree.write("realE/cases/a/expected.exit", b"0\n");
        tree.write("realE/cases/b/expected.exit", b"0\n");
        tree.write("realE/cases/b/spec.refs", b"\n");
        symlink(Path::new("realE"), tree.root.join("zz")).expect("link the folder");
        tree
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(path, bytes).expect("write the file");
    }

    /// Takes the file's mode, and answers whether that bars this user.
    fn lock(&mut self, relative: &str) -> bool {
        let path = self.root.join(relative);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("take the mode");
        self.locked.push(path.clone());
        fs::read(&path).is_err()
    }

    fn run(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(&self.root)
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
        for path in &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o644));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn error_lines(run: &Run) -> Vec<&str> {
    run.stderr
        .lines()
        .filter(|line| line.starts_with("error: "))
        .collect()
}

fn io_entries(run: &Run) -> usize {
    format!("{}\n{}", run.stdout, run.stderr)
        .matches("\"code\":\"io\"")
        .count()
}

/// The `E2E` IDs `grund list` names, in its order.
fn listed(tree: &Tree) -> Vec<String> {
    let run = tree.run(&["list", "--kind", "E2E"]);
    run.stdout
        .lines()
        .filter_map(|line| line.split_whitespace().next().map(str::to_string))
        .collect()
}

/// The control: a broken case nothing keeps out fails the run, and every healthy case is listed.
#[test]
fn a_broken_case_nothing_keeps_out_fails_the_run() {
    let tree = Tree::cases("control", "");
    assert_eq!(listed(&tree), ["E2E-a", "E2E-b", "E2E-c"]);
    tree.write("e2e/cases/b/expected.exit", UNDECODABLE);
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 2, "{run:#?}");
    assert_eq!(
        error_lines(&run),
        [
            "error: e2e/cases/b/expected.exit: stream did not contain valid UTF-8; make the path readable, or keep it out of the scan"
        ],
        "{run:#?}"
    );
}

/// The two ways the configuration keeps a case directory out, each on a tree of its own.
fn excluded_trees(test: &str) -> Vec<Tree> {
    let by_exclude = Tree::cases(&format!("{test}-exclude"), "exclude = [\"e2e/cases/b\"]\n");
    let by_ignore = Tree::cases(&format!("{test}-ignore"), "");
    by_ignore.write(".ignore", b"e2e/cases/b/\n");
    vec![by_exclude, by_ignore]
}

/// §FS-declarations.e2e-case.excluded: nothing in an excluded case can fail the run.
#[test]
fn an_excluded_case_with_a_broken_manifest_does_not_fail_the_run() {
    for tree in excluded_trees("broken") {
        tree.write("e2e/cases/b/expected.exit", UNDECODABLE);
        let run = tree.run(&["check"]);
        assert_eq!(run.code, 0, "{run:#?}");
        assert_eq!(error_lines(&run), Vec::<&str>::new(), "{run:#?}");
    }
}

/// §FS-declarations.e2e-case.excluded: an excluded case declares no ID, so a citation dangles.
#[test]
fn an_excluded_case_declares_no_id() {
    for tree in excluded_trees("healthy") {
        assert_eq!(listed(&tree), ["E2E-a", "E2E-c"]);
        tree.write("notes.md", "# Notes\n\nSee §E2E-b.\n".as_bytes());
        let run = tree.run(&["check"]);
        assert_eq!(run.code, 1, "{run:#?}");
        assert!(run.stdout.contains("E2E-b"), "{run:#?}");
    }
}

/// The control of grund.273: the readable linked tree passes.
#[test]
fn the_readable_linked_tree_passes() {
    let run = Tree::linked("readable").run(&["check"]);
    assert_eq!(run.code, 0, "{run:#?}");
}

/// §FS-check.2.4.2: one unreadable manifest the walk and the case read under two spellings is one
/// failure, so one line and one `io` entry.
#[test]
fn one_unreadable_manifest_under_two_spellings_is_one_line() {
    let mut tree = Tree::linked("one-file");
    if !tree.lock("realE/cases/b/spec.refs") {
        eprintln!("{SKIP}");
        return;
    }
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 2, "{run:#?}");
    assert_eq!(error_lines(&run).len(), 1, "{run:#?}");
    let json = tree.run(&["check", "--format", "json"]);
    assert_eq!(io_entries(&json), 1, "{json:#?}");
}

/// The guard: two unreadable manifests are two failures, one line each, not one and not four.
#[test]
fn two_unreadable_manifests_under_two_spellings_are_two_lines() {
    let mut tree = Tree::linked("two-files");
    tree.write("realE/cases/a/spec.refs", b"\n");
    if !tree.lock("realE/cases/a/spec.refs") || !tree.lock("realE/cases/b/spec.refs") {
        eprintln!("{SKIP}");
        return;
    }
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 2, "{run:#?}");
    let lines = error_lines(&run);
    assert_eq!(lines.len(), 2, "{run:#?}");
    assert!(
        lines
            .iter()
            .any(|line| line.contains("cases/a/spec.refs: ")),
        "{run:#?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("cases/b/spec.refs: ")),
        "{run:#?}"
    );
    let json = tree.run(&["check", "--format", "json"]);
    assert_eq!(io_entries(&json), 2, "{json:#?}");
}
