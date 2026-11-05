//! Black-box: a failure reading one E2E case belongs to that case alone, so every other case is
//! read wherever the failing one sorts (§FS-declarations.e2e-case.failure, §FS-check.2.4). The e2e
//! corpus pins the bytes where a committed manifest can make the failure. These pin what it cannot
//! hold: git carries no mode, and a parse failure's reason prints an absolute temporary path, so
//! that line is not judged here. The tests that need a mode skip where it does not bar the user.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.E2E]\nfolders = [\"e2e/cases\"]\n";

const UNDECODABLE: &[u8] = b"\xff\xfe\n";

/// A v2 project whose named cases all hold a healthy manifest, and a `notes.md` citing each.
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
    fn new(test: &str, cases: &[&str]) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1256-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for case in cases {
            let dir = root.join("e2e/cases").join(case);
            fs::create_dir_all(&dir).expect("create the case");
            fs::write(dir.join("expected.exit"), "0\n").expect("write the manifest");
        }
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        let citations = cases
            .iter()
            .map(|case| format!("§E2E-{case}"))
            .collect::<Vec<_>>()
            .join(" and ");
        fs::write(
            root.join("notes.md"),
            format!("# Notes\n\nSee {citations}.\n"),
        )
        .expect("write the notes");
        Self {
            root,
            locked: Vec::new(),
        }
    }

    fn case_file(&self, case: &str, file: &str) -> PathBuf {
        self.root.join("e2e/cases").join(case).join(file)
    }

    fn write(&self, case: &str, file: &str, bytes: &[u8]) {
        fs::write(self.case_file(case, file), bytes).expect("write the case's file");
    }

    fn lock(&mut self, path: PathBuf) -> PathBuf {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("set the mode");
        self.locked.push(path.clone());
        path
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
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// The scan-failure lines of a text run: a located finding opens with its path, not `error:`.
fn failures(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|line| line.starts_with("error: "))
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

/// What a case nothing is wrong with is owed beside another's failure, in each report: its citation
/// resolves, its own finding prints, and `grund list` names it.
fn assert_read(tree: &Tree, case: &str) {
    let id = format!("E2E-{case}");
    let check = tree.run(&["check"]);
    assert_eq!(check.code, 2, "{id}, check: {check:#?}");
    let unknown = format!("unknown reference {id}");
    assert!(
        !check.stdout.contains(&unknown),
        "a correct citation of {id} is reported as unknown: {check:#?}"
    );
    let own_finding = format!("e2e/cases/{case}:1: error: {id} is not listed");
    assert!(
        check
            .stdout
            .lines()
            .any(|line| line.starts_with(&own_finding)),
        "{id}'s own finding is gone: {check:#?}"
    );

    let json = tree.run(&["check", "--format", "json"]);
    assert_eq!(json.code, 2, "{id}, check json: {json:#?}");
    let all = format!("{}\n{}", json.stdout, json.stderr);
    assert!(
        !all.contains(&unknown),
        "a correct citation of {id} is reported as unknown in JSON: {json:#?}"
    );
    let path = format!("\"path\":\"e2e/cases/{case}\"");
    assert!(
        all.lines()
            .any(|line| line.contains(&path) && line.contains("\"missing-index-entry\"")),
        "{id}'s own finding is gone from JSON: {json:#?}"
    );

    let list = tree.run(&["list", "--kind", "E2E"]);
    assert_eq!(list.code, 2, "{id}, list: {list:#?}");
    let row = format!("{id}  e2e/cases/{case}:1");
    assert!(
        list.stdout.lines().any(|line| line == row),
        "list does not name {id}: {list:#?}"
    );
}

/// Whether the mode bars this process: it does not bar root, which proves nothing here.
fn barred(path: &Path) -> bool {
    if path.is_dir() {
        fs::read_dir(path).is_err()
    } else {
        fs::read(path).is_err()
    }
}

/// The control: with every manifest healthy each case is read and nothing failed, so what the
/// tests below judge is the one file they break.
#[test]
fn healthy_manifests_read_every_case() {
    let tree = Tree::new("control", &["a", "b", "c"]);
    let check = tree.run(&["check"]);
    assert_eq!(check.code, 1, "{check:#?}");
    assert_eq!(failures(&check.stderr), Vec::<&str>::new(), "{check:#?}");
    let list = tree.run(&["list", "--kind", "E2E"]);
    assert_eq!(
        (list.code, list.stdout.as_str()),
        (
            0,
            "E2E-a  e2e/cases/a:1\nE2E-b  e2e/cases/b:1\nE2E-c  e2e/cases/c:1\n"
        ),
        "{list:#?}"
    );
}

/// A guard (§AR-scanner.6.6): a cases root that cannot be listed leaves nothing
/// to continue over. Its line count is not judged, because the ordinary walk reports it as well.
#[test]
fn an_unlistable_cases_root_reads_no_case_and_exits_2() {
    let mut tree = Tree::new("root", &["a", "b"]);
    let root = tree.lock(tree.root.join("e2e/cases"));
    if !barred(&root) {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    let list = tree.run(&["list", "--kind", "E2E"]);
    assert_eq!((list.code, list.stdout.as_str()), (2, ""), "{list:#?}");
    assert!(
        failures(&list.stderr)
            .iter()
            .all(|line| line.starts_with("error: e2e/cases: ")),
        "{list:#?}"
    );
}

#[test]
fn an_expected_exit_that_does_not_parse_leaves_the_other_cases_read() {
    let tree = Tree::new("parse", &["a", "b", "c"]);
    tree.write("b", "expected.exit", b"not a number\n");
    assert_read(&tree, "a");
    assert_read(&tree, "c");
}

#[test]
fn an_expected_exit_at_mode_000_leaves_the_other_cases_read() {
    let mut tree = Tree::new("exit-mode", &["a", "b", "c"]);
    let manifest = tree.lock(tree.case_file("b", "expected.exit"));
    if !barred(&manifest) {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    assert_read(&tree, "a");
    assert_read(&tree, "c");
}

#[test]
fn a_fixture_directory_at_mode_000_leaves_the_other_cases_read() {
    let mut tree = Tree::new("fixture-mode", &["a", "b", "c"]);
    let fixtures = tree.case_file("b", "repo");
    fs::create_dir(&fixtures).expect("create the fixture directory");
    let fixtures = tree.lock(fixtures);
    if !barred(&fixtures) {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    assert_read(&tree, "a");
    assert_read(&tree, "c");
}

/// Two failures that read alike are still two entries: a run that keeps only the first, or
/// collapses equal lines, reports one failure where there are two.
#[test]
fn two_failing_cases_are_two_entries_in_json() {
    let tree = Tree::new("json-two", &["a", "b", "c", "d", "e"]);
    tree.write("b", "expected.exit", UNDECODABLE);
    tree.write("d", "expected.exit", UNDECODABLE);
    let run = tree.run(&["check", "--format", "json"]);
    assert_eq!(run.code, 2, "{run:#?}");
    assert_eq!(json_failures(&run).len(), 2, "{run:#?}");
    assert_read(&tree, "e");
}

/// Two failures with different reasons, so a run that finishes the walk and hands back only one
/// of them stays red here, whichever it keeps.
#[test]
fn two_different_failures_are_both_reported() {
    let tree = Tree::new("text-two", &["a", "b", "c", "d", "e"]);
    tree.write("b", "expected.exit", b"not a number\n");
    tree.write("d", "expected.exit", UNDECODABLE);
    for args in [&["check"][..], &["list", "--kind", "E2E"]] {
        let run = tree.run(args);
        assert_eq!(run.code, 2, "grund {args:?}: {run:#?}");
        let lines = failures(&run.stderr);
        assert_eq!(lines.len(), 2, "grund {args:?}: {run:#?}");
        assert!(
            lines.iter().any(|line| line.contains("invalid digit")),
            "grund {args:?}: {run:#?}"
        );
        assert!(
            lines.iter().any(|line| line.contains("valid UTF-8")),
            "grund {args:?}: {run:#?}"
        );
    }
    assert_read(&tree, "c");
    assert_read(&tree, "e");
}
