//! Black-box: a failure reading an E2E case is located at the file or directory that failed
//! (§FS-declarations.e2e-case.failure), so a run narrowed to a path holds the failure exactly when
//! the path holds that file (§FS-check.1.3.6.3, §FS-check.2.4) and one a walk also met is one line
//! (§FS-check.2.4.2). The e2e corpus pins the bytes a committed manifest can make. These pin what
//! it cannot hold: a working directory inside the tree, a mode, a link, and a parse failure, whose
//! reason prints an absolute temporary path. The tests that need a mode skip where it does not bar
//! the user.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.E2E]\nfolders = [\"e2e/cases\"]\n";

const UNDECODABLE: &[u8] = b"\xff\xfe\n";
const SKIP: &str = "skipped: the mode does not bar this user, so nothing here can be unreadable";
const OUTSIDE: &str = "outside the report scope";

/// A project whose cases `a`, `b` and `c` hold a healthy manifest, each test breaking `b`.
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
        let root = std::env::temp_dir().join(format!("grund-1264-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the root");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        let tree = Self {
            root,
            locked: Vec::new(),
        };
        for case in ["a", "b", "c"] {
            tree.write(&format!("e2e/cases/{case}/expected.exit"), b"0\n");
        }
        tree
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(path, bytes).expect("write the file");
    }

    /// Takes the path's mode, and answers whether that bars this user.
    fn lock(&mut self, relative: &str) -> bool {
        let path = self.root.join(relative);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("take the mode");
        self.locked.push(path.clone());
        if path.is_dir() {
            fs::read_dir(&path).is_err()
        } else {
            fs::read(&path).is_err()
        }
    }

    fn run_in(&self, directory: &str, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(self.root.join(directory))
            .output()
            .expect("run grund");
        Run {
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    fn run(&self, args: &[&str]) -> Run {
        self.run_in(".", args)
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

/// The run's lines about a read failure, in either channel: every `error:` line that names no
/// finding's location, and every caution.
fn failure_lines(run: &Run) -> Vec<&str> {
    run.stderr
        .lines()
        .filter(|line| line.starts_with("error: ") || line.contains(OUTSIDE))
        .collect()
}

/// The `io` entries of a JSON run, whichever stream carries them.
fn json_failures(run: &Run) -> Vec<String> {
    format!("{}\n{}", run.stdout, run.stderr)
        .lines()
        .filter(|line| line.contains("\"code\":\"io\""))
        .map(str::to_string)
        .collect()
}

/// What a run holding the failing file owes: exit `2`, and no claim that the failure is elsewhere.
fn assert_holds(run: &Run, line_start: &str) {
    assert_eq!(run.code, 2, "{run:#?}");
    assert!(!run.stderr.contains(OUTSIDE), "{run:#?}");
    assert!(
        run.stderr.lines().any(|line| line.starts_with(line_start)),
        "no line opening `{line_start}`: {run:#?}"
    );
}

/// The control: nothing failed, so a case named on the command line answers with its own finding.
#[test]
fn a_healthy_case_named_exits_on_its_own_finding() {
    let tree = Tree::new("control");
    let run = tree.run(&["check", "e2e/cases/b"]);
    assert_eq!(run.code, 1, "{run:#?}");
    assert_eq!(failure_lines(&run), Vec::<&str>::new(), "{run:#?}");
}

/// A guard: the same bytes in an ordinary file are what a case's manifest is being brought into
/// line with, so a change to the partition itself shows here (§FS-check.1.3.6.3).
#[test]
fn the_same_bytes_in_an_ordinary_file_are_held_only_by_a_path_at_the_file() {
    let tree = Tree::new("ordinary");
    tree.write("notes.md", UNDECODABLE);
    assert_holds(&tree.run(&["check", "notes.md"]), "error: notes.md: ");
    let beside = tree.run(&["check", "e2e/cases/b"]);
    assert_eq!(beside.code, 1, "{beside:#?}");
    assert!(
        beside
            .stderr
            .lines()
            .any(|line| line.starts_with("warning: notes.md: ") && line.contains(OUTSIDE)),
        "{beside:#?}"
    );
}

/// How a path is spelled from inside the tree is not judged here (#1207): what it holds is.
#[test]
fn standing_in_the_failing_case_holds_the_failure() {
    let tree = Tree::new("in-case");
    tree.write("e2e/cases/b/expected.exit", UNDECODABLE);
    for args in [&["check", "."][..], &["check", "expected.exit"]] {
        let run = tree.run_in("e2e/cases/b", args);
        assert_eq!(run.code, 2, "grund {args:?}: {run:#?}");
        assert!(!run.stderr.contains(OUTSIDE), "grund {args:?}: {run:#?}");
        assert!(
            run.stderr.lines().any(|line| line.starts_with("error: ")
                && line.contains("expected.exit: ")
                && line.contains("valid UTF-8")),
            "grund {args:?}: {run:#?}"
        );
    }
}

/// A path inside a member resolves against that member (§FS-check.1.3.6.3), and is reported from it.
#[test]
fn standing_in_a_workspace_member_holds_the_failure() {
    let tree = Tree::new("in-member");
    tree.write(
        "grund.toml",
        b"grund_config_version = 2\nproject_name = \"ws\"\n\n[workspace]\nmembers = [\"m\"]\n",
    );
    tree.write("m/grund.toml", CONFIG.as_bytes());
    tree.write("m/e2e/cases/a/expected.exit", b"0\n");
    tree.write("m/e2e/cases/b/expected.exit", UNDECODABLE);
    assert_holds(
        &tree.run_in("m", &["check", "e2e/cases/b"]),
        "error: e2e/cases/b/expected.exit: ",
    );
    assert_holds(
        &tree.run_in("m", &["check", "e2e/cases/b/expected.exit"]),
        "error: e2e/cases/b/expected.exit: ",
    );
    assert_holds(
        &tree.run(&["check", "m/e2e/cases/b/expected.exit"]),
        "error: e2e/cases/b/expected.exit: ",
    );
}

#[test]
fn an_expected_exit_that_does_not_parse_is_held_by_a_path_at_it() {
    let tree = Tree::new("parse");
    tree.write("e2e/cases/b/expected.exit", b"not-an-exit\n");
    assert_holds(
        &tree.run(&["check", "e2e/cases/b"]),
        "error: e2e/cases/b/expected.exit: parse ",
    );
    let sibling = tree.run(&["check", "e2e/cases/c"]);
    assert_eq!(sibling.code, 1, "{sibling:#?}");
    assert!(
        sibling.stderr.lines().any(|line| line
            .starts_with("warning: e2e/cases/b/expected.exit: parse ")
            && line.contains(OUTSIDE)),
        "{sibling:#?}"
    );
}

#[test]
fn an_expected_exit_at_mode_000_is_held_by_a_path_at_it() {
    let mut tree = Tree::new("exit-mode");
    if !tree.lock("e2e/cases/b/expected.exit") {
        eprintln!("{SKIP}");
        return;
    }
    let line = "error: e2e/cases/b/expected.exit: permission denied; ";
    assert_holds(&tree.run(&["check", "e2e/cases/b"]), line);
    assert_holds(&tree.run(&["check", "e2e/cases/b/expected.exit"]), line);
}

/// §FS-check.2.4.2: a run whose path enters the case walks the fixture directory as well, so the
/// walk and the pass meet one failure, and it is one line, one `io` entry.
#[test]
fn a_fixture_directory_at_mode_000_is_one_failure_at_that_directory() {
    let mut tree = Tree::new("fixture-mode");
    tree.write("e2e/cases/b/fixture/deep/file.txt", b"held\n");
    if !tree.lock("e2e/cases/b/fixture/deep") {
        eprintln!("{SKIP}");
        return;
    }
    let line = "error: e2e/cases/b/fixture/deep: permission denied; make the path readable, \
                or keep it out of the scan";
    let named = tree.run(&["check", "e2e/cases/b"]);
    assert_eq!(named.code, 2, "{named:#?}");
    assert_eq!(failure_lines(&named), vec![line], "{named:#?}");

    let whole = tree.run(&["check"]);
    assert_eq!(whole.code, 2, "{whole:#?}");
    assert_eq!(failure_lines(&whole), vec![line], "{whole:#?}");

    let json = tree.run(&["check", "--format", "json", "e2e/cases/b"]);
    let entries = json_failures(&json);
    assert_eq!(json.code, 2, "{json:#?}");
    assert_eq!(entries.len(), 1, "{json:#?}");
    assert!(
        entries[0].contains("\"path\":\"e2e/cases/b/fixture/deep\""),
        "{json:#?}"
    );

    let sibling = tree.run(&["check", "e2e/cases/c"]);
    assert_eq!(sibling.code, 1, "{sibling:#?}");
    assert_eq!(
        failure_lines(&sibling),
        vec![
            "warning: e2e/cases/b/fixture/deep: permission denied — outside the report scope, \
             so a citation declared there may be reported as unresolved"
        ],
        "{sibling:#?}"
    );
}

/// A case's first failure does not hide its second (§FS-declarations.e2e-case.failure): each
/// fixture directory that cannot be listed is its own report.
#[test]
fn two_fixture_directories_at_mode_000_are_two_failures() {
    let mut tree = Tree::new("two-fixtures");
    tree.write("e2e/cases/b/one/file.txt", b"held\n");
    tree.write("e2e/cases/b/two/file.txt", b"held\n");
    if !tree.lock("e2e/cases/b/one") || !tree.lock("e2e/cases/b/two") {
        eprintln!("{SKIP}");
        return;
    }
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 2, "{run:#?}");
    let lines = failure_lines(&run);
    assert_eq!(lines.len(), 2, "{run:#?}");
    for directory in ["one", "two"] {
        let opening = format!("error: e2e/cases/b/{directory}: permission denied; ");
        assert!(
            lines.iter().any(|line| line.starts_with(&opening)),
            "{run:#?}"
        );
    }
}

/// The location is the tree's spelling under the configured folder, never the link's target
/// (§AR-scanner.6.6), so the target's own spelling stays the outside it has always been.
#[test]
fn a_linked_cases_folder_locates_the_failure_under_its_configured_spelling() {
    let tree = Tree::new("linked-folder");
    tree.write("real/a/expected.exit", b"0\n");
    tree.write("real/b/expected.exit", UNDECODABLE);
    fs::remove_dir_all(tree.root.join("e2e/cases")).expect("clear the folder");
    symlink(Path::new("../real"), tree.root.join("e2e/cases")).expect("link the folder");
    let line = "error: e2e/cases/b/expected.exit: ";
    assert_holds(&tree.run(&["check", "e2e/cases/b"]), line);
    assert_holds(&tree.run(&["check"]), line);
    let target = tree.run(&["check", "real/b"]);
    assert_eq!(target.code, 0, "{target:#?}");
    assert!(
        target.stderr.lines().any(
            |line| line.starts_with("warning: e2e/cases/b/expected.exit: ")
                && line.contains(OUTSIDE)
        ),
        "{target:#?}"
    );
    // §FS-declarations.e2e-case.failure: standing in the case through the link, the working
    // directory is the target, so this path is the other spelling too.
    let standing = tree.run_in("e2e/cases/b", &["check", "."]);
    assert_eq!(standing.code, 0, "{standing:#?}");
    assert!(
        standing.stderr.lines().any(|line| line
            .starts_with("warning: e2e/cases/b/expected.exit: ")
            && line.contains(OUTSIDE)),
        "{standing:#?}"
    );
}

/// A tree whose extensions send `spec.refs` and `expected.exit` through the walk as well.
fn tree_scanning_manifests(test: &str) -> Tree {
    let tree = Tree::new(test);
    let config = format!(
        "{CONFIG}\n[schema.sources]\nlanguages = [\"markdown\"]\n\n\
         [schema.sources.extensions]\nrefs = \"markdown\"\nexit = \"markdown\"\n"
    );
    tree.write("grund.toml", config.as_bytes());
    tree
}

/// §FS-check.2.4.2: the walk and the pass meet one unreadable manifest, so it is one line.
#[test]
fn a_manifest_at_mode_000_the_walk_also_reads_is_one_line() {
    let mut tree = tree_scanning_manifests("scanned-mode");
    tree.write("e2e/cases/b/spec.refs", b"E2E-a\n");
    if !tree.lock("e2e/cases/b/spec.refs") {
        eprintln!("{SKIP}");
        return;
    }
    let line = "error: e2e/cases/b/spec.refs: permission denied; ";
    for args in [
        &["check", "e2e/cases/b"][..],
        &["check", "e2e/cases/b/spec.refs"],
    ] {
        let run = tree.run(args);
        assert_holds(&run, line);
        assert_eq!(failure_lines(&run).len(), 1, "{run:#?}");
    }
    let json = tree.run(&["check", "e2e/cases/b", "--format", "json"]);
    assert_eq!(json_failures(&json).len(), 1, "{json:#?}");
}

/// §AR-scanner.6.6: the walk reads the same file and finds nothing wrong with its bytes, so the
/// value only the pass can judge keeps its one line.
#[test]
fn an_expected_exit_that_does_not_parse_the_walk_also_reads_is_one_line() {
    let tree = tree_scanning_manifests("scanned-parse");
    tree.write("e2e/cases/b/expected.exit", b"not-an-exit\n");
    let line = "error: e2e/cases/b/expected.exit: parse ";
    for args in [
        &["check", "e2e/cases/b"][..],
        &["check", "e2e/cases/b/expected.exit"],
    ] {
        let run = tree.run(args);
        assert_holds(&run, line);
        assert_eq!(failure_lines(&run).len(), 1, "{run:#?}");
    }
}

/// §AR-scanner.1.relisted: a manifest that is a link is an entry of its own, so the walk's report
/// of the file behind it does not stand in for it, and the run naming the case still holds it.
#[test]
fn a_manifest_linked_to_a_file_the_walk_reported_keeps_its_own_line() {
    let tree = Tree::new("linked-manifest");
    tree.write("notes.md", UNDECODABLE);
    symlink(
        Path::new("../../../notes.md"),
        tree.root.join("e2e/cases/b/spec.refs"),
    )
    .expect("link the manifest");
    let run = tree.run(&["check", "e2e/cases/b"]);
    assert_eq!(run.code, 2, "{run:#?}");
    let lines = failure_lines(&run);
    assert_eq!(lines.len(), 2, "{run:#?}");
    assert!(
        lines[0].starts_with("error: e2e/cases/b/spec.refs: "),
        "{run:#?}"
    );
    assert!(lines[1].starts_with("warning: notes.md: "), "{run:#?}");
}

/// §AR-scanner.1.relisted: two cases' entries are two locations even where one links to the other,
/// or the second case named on the command line would go green.
#[test]
fn a_manifest_linked_to_another_cases_manifest_is_held_by_its_own_case() {
    let tree = Tree::new("linked-sibling");
    tree.write("e2e/cases/b/spec.refs", UNDECODABLE);
    symlink(
        Path::new("../b/spec.refs"),
        tree.root.join("e2e/cases/c/spec.refs"),
    )
    .expect("link the manifest");
    let run = tree.run(&["check", "e2e/cases/c"]);
    assert_eq!(run.code, 2, "{run:#?}");
    assert!(
        run.stderr
            .lines()
            .any(|line| line.starts_with("error: e2e/cases/c/spec.refs: ")),
        "{run:#?}"
    );
}
