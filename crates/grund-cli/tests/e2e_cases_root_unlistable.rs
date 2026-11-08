//! Black-box: an E2E cases root that cannot be listed is one failure, so every surface reports it
//! once (§FS-check.2.4.2). That holds whether the ordinary walk and the manifest pass both met it,
//! or the pass alone did (§AR-scanner.6.unlistable-root). Git carries no mode a fixture could keep
//! unreadable, so the e2e corpus cannot hold these trees. Every test that bars a directory skips
//! where the mode does not bar this user, as it does not bar root.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const REMEDY: &str = "make the path readable, or keep it out of the scan";
const SKIP: &str = "skipped: the mode does not bar this user, so nothing here can be unreadable";

/// A scratch directory for one test's trees, its barred directories given back on drop.
struct Scratch {
    root: PathBuf,
    locked: Vec<PathBuf>,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1262-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the scratch root");
        Self {
            root,
            locked: Vec::new(),
        }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(&path, contents).expect("write the file");
    }

    /// The report's two cases, `a` and `b`, each with an `expected.exit` holding `0`.
    fn cases(&self, relative: &str) {
        for case in ["a", "b"] {
            self.write(&format!("{relative}/{case}/expected.exit"), "0\n");
        }
    }

    /// Takes the directory's mode, and answers whether that bars this user.
    fn lock(&mut self, relative: &str) -> bool {
        let path = self.path(relative);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("take the mode");
        self.locked.push(path.clone());
        fs::read_dir(&path).is_err()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        for path in &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// The report's config: a version-2 project whose citable `E2E` kind keeps its cases in `folder`.
fn config(folder: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"q\"\n\n[schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
         [schema.kinds.E2E]\nfolders = [\"{folder}\"]\n"
    )
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn grund(dir: &Path, args: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("run grund");
    Run {
        code: output.status.code().expect("exited with a status"),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

impl Run {
    /// The scan failures a run reports, from either stream. A located finding reads
    /// `path:line: error:`, so a line that starts `error: ` is a failure of the run
    /// (§FS-errors.2.2), and outside the report scope it is the caution of §FS-check.1.3.6.3.
    /// Other warnings, the version-1 notice among them, say nothing about a failure.
    fn failure_lines(&self) -> Vec<&str> {
        self.stdout
            .lines()
            .chain(self.stderr.lines())
            .filter(|line| {
                line.starts_with("error: ")
                    || line.starts_with("warning: ") && line.contains(" — outside the report scope")
            })
            .collect()
    }

    /// The `io` records of a `--format json` run, from either stream.
    fn io_records(&self) -> Vec<serde_json::Value> {
        self.stdout
            .lines()
            .chain(self.stderr.lines())
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|record| record["code"] == "io")
            .collect()
    }

    fn shown(&self) -> String {
        format!("stdout:\n{}\nstderr:\n{}", self.stdout, self.stderr)
    }
}

fn unlistable(path: &str) -> String {
    format!("error: {path}: permission denied; {REMEDY}")
}

/// The commands that do not exit `code` with `expected` as their one failure line, each with
/// what it printed, so that one assertion shows every surface.
fn misreports(dir: &Path, commands: &[&[&str]], code: i32, expected: &str) -> Vec<String> {
    commands
        .iter()
        .filter_map(|args| {
            let run = grund(dir, args);
            (run.code != code || run.failure_lines() != [expected]).then(|| {
                let args = args.join(" ");
                format!(
                    "grund {args}: exit {}, wanted {code} and `{expected}`\n{}",
                    run.code,
                    run.shown()
                )
            })
        })
        .collect()
}

fn assert_none(misreports: &[String], runs: usize) {
    assert!(
        misreports.is_empty(),
        "one directory that failed to list once is one line; {} of {runs} runs printed otherwise:\n\n{}",
        misreports.len(),
        misreports.join("\n")
    );
}

/// Every command exits `2` and reports one failure, on the one line `expected`.
fn assert_one_line(dir: &Path, commands: &[&[&str]], expected: &str) {
    assert_none(&misreports(dir, commands, 2, expected), commands.len());
}

/// `check --format json` exits `2` with one `io` entry, an error at `path`.
fn assert_one_io_entry(dir: &Path, path: &str) {
    let run = grund(dir, &["check", "--format", "json"]);
    assert_eq!(run.code, 2, "{}", run.shown());
    let records = run.io_records();
    assert_eq!(
        records.len(),
        1,
        "one failure is one io entry\n{}",
        run.shown()
    );
    assert_eq!(records[0]["path"], path, "{}", run.shown());
    assert_eq!(records[0]["severity"], "error", "{}", run.shown());
    assert_eq!(
        records[0]["message"],
        format!("permission denied; {REMEDY}"),
        "{}",
        run.shown()
    );
}

/// The report's tree, as its steps build it, under every command its reproducer judged.
#[test]
fn an_unlistable_cases_root_is_one_line_from_every_command() {
    let mut scratch = Scratch::new("report");
    scratch.write("grund.toml", &config("e2e/cases"));
    scratch.cases("e2e/cases");
    if !scratch.lock("e2e/cases") {
        eprintln!("{SKIP}");
        return;
    }
    assert_one_line(
        &scratch.root,
        &[
            &["check"],
            &["list", "--kind", "E2E"],
            &["check", "--full"],
            &["list"],
            &["refs", "E2E-a"],
            &["show", "E2E-a"],
        ],
        &unlistable("e2e/cases"),
    );
}

#[test]
fn an_unlistable_cases_root_is_one_io_entry_in_check_json() {
    let mut scratch = Scratch::new("json");
    scratch.write("grund.toml", &config("e2e/cases"));
    scratch.cases("e2e/cases");
    if !scratch.lock("e2e/cases") {
        eprintln!("{SKIP}");
        return;
    }
    assert_one_io_entry(&scratch.root, "e2e/cases");
}

/// With the folder elsewhere, the one line names the folder, never a directory the tree lacks.
#[test]
fn a_cases_root_elsewhere_is_one_line_naming_its_folder() {
    let mut scratch = Scratch::new("elsewhere");
    scratch.write("grund.toml", &config("tests/e2e"));
    scratch.cases("tests/e2e");
    if !scratch.lock("tests/e2e") {
        eprintln!("{SKIP}");
        return;
    }
    assert_one_line(
        &scratch.root,
        &[&["check"], &["list", "--kind", "E2E"]],
        &unlistable("tests/e2e"),
    );
    assert!(!scratch.path("e2e").exists(), "the run created e2e/cases");
}

/// A workspace root reports its member's failure once, spelled from the report base.
#[test]
fn a_members_unlistable_cases_root_is_one_line_from_the_workspace_root() {
    let mut scratch = Scratch::new("workspace");
    scratch.write(
        "grund.toml",
        "grund_config_version = 2\nproject_name = \"ws\"\n\n[schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
         [workspace]\nmembers = [\"m\"]\n",
    );
    scratch.write("m/grund.toml", &config("e2e/cases"));
    scratch.cases("m/e2e/cases");
    if !scratch.lock("m/e2e/cases") {
        eprintln!("{SKIP}");
        return;
    }
    assert_one_line(&scratch.root, &[&["check"]], &unlistable("m/e2e/cases"));
}

/// Under a path scope the one line is the caution outside the report scope (§FS-check.1.3.6.3)
/// and the error inside it, never one of each.
#[test]
fn under_a_path_scope_the_one_line_keeps_the_severity_its_location_earns() {
    let mut scratch = Scratch::new("scoped");
    scratch.write("grund.toml", &config("tests/e2e"));
    scratch.cases("tests/e2e");
    scratch.write("docs/n.md", "# Note\n");
    if !scratch.lock("tests/e2e") {
        eprintln!("{SKIP}");
        return;
    }
    // Outside the report scope the caution leaves the exit where the report put it: `0` here.
    let caution = "warning: tests/e2e: permission denied — outside the report scope, so a citation \
                   declared there may be reported as unresolved";
    let mut wrong = misreports(&scratch.root, &[&["check", "docs"]], 0, caution);
    let inside = unlistable("tests/e2e");
    wrong.extend(misreports(
        &scratch.root,
        &[&["check", "tests/e2e"]],
        2,
        &inside,
    ));
    assert_none(&wrong, 2);
}

/// The walk skips a scan root that is a directory link out of the project, and a version-1 config
/// may name one as the `E2E` folder. Then the manifest pass alone meets the failure. Dropping its
/// line would leave the failure unreported (§REQ-no-missed-citation.1), and it names the folder,
/// not a hard-coded `e2e/cases` (§AR-scanner.6.unlistable-root).
#[test]
fn a_cases_root_only_the_manifest_pass_meets_is_one_line_naming_its_folder() {
    let mut scratch = Scratch::new("linked");
    scratch.write(
        "proj/grund.toml",
        "grund_config_version = 1\n\n[[kinds]]\nkind = \"E2E\"\nfolder = \"tests/e2e\"\n",
    );
    fs::create_dir_all(scratch.path("proj/tests")).expect("create tests");
    scratch.cases("outside");
    std::os::unix::fs::symlink(scratch.path("outside"), scratch.path("proj/tests/e2e"))
        .expect("link the cases out of the project");
    if !scratch.lock("outside") {
        eprintln!("{SKIP}");
        return;
    }
    let project = scratch.path("proj");
    assert_one_line(
        &project,
        &[&["check"], &["list", "--kind", "E2E"]],
        &unlistable("tests/e2e"),
    );
    assert_one_io_entry(&project, "tests/e2e");
}

/// A cases root that links to a barred directory inside the project: the walk reports the link at
/// its own path, in its own words (§FS-scan.scope.links.unresolvable.unopenable), and the manifest
/// pass's listing of it is the same failure, so the one line is the walk's (§FS-check.2.4.2).
#[test]
fn a_cases_root_that_links_to_a_barred_target_is_the_walks_one_line() {
    let mut scratch = Scratch::new("linked-inward");
    scratch.write("grund.toml", &config("tests/e2e"));
    fs::create_dir_all(scratch.path("tests")).expect("create tests");
    scratch.cases(".store/cases");
    std::os::unix::fs::symlink("../.store/cases", scratch.path("tests/e2e"))
        .expect("link the cases");
    if !scratch.lock(".store/cases") {
        eprintln!("{SKIP}");
        return;
    }
    assert_one_line(
        &scratch.root,
        &[&["check"], &["list", "--kind", "E2E"]],
        &format!("error: tests/e2e: unreadable symlink: permission denied; {REMEDY}"),
    );
}

/// A walk failure beside the cases root is no listing failure of it. An ignore file with two
/// malformed lines is reported at the ignore file, each line once (§FS-check.2.4.2), and the root
/// is still listed by the walk, so its cases are still read and a citation of one resolves. This
/// guards against a walk failure that leaves the root listable taking the cases with it.
#[test]
fn a_walk_report_at_the_cases_root_that_is_no_listing_failure_leaves_the_cases_read() {
    let scratch = Scratch::new("ignore-file");
    scratch.write(
        "grund.toml",
        &format!(
            "{}\n[schema.kinds.doc]\ncitable = false\nfolders = [\"docs\"]\n",
            config("e2e/cases")
        ),
    );
    scratch.cases("e2e/cases");
    // `\u{a7}` is the marker, escaped so this repository does not read a citation.
    scratch.write("docs/n.md", "# Note\n\nProven by \u{a7}E2E-a.\n");
    scratch.write("e2e/.ignore", "{a\n{b\n");

    let list = grund(&scratch.root, &["list", "--kind", "E2E"]);
    let lines = list.failure_lines();
    assert!(
        list.code == 2
            && lines.len() == 1
            && lines[0].starts_with("error: e2e/.ignore: ")
            && lines[0].contains("error parsing glob"),
        "the walk reports the ignore file at the file, once\n{}",
        list.shown()
    );
    assert_eq!(
        list.stdout,
        "E2E-a  e2e/cases/a:1\nE2E-b  e2e/cases/b:1\n",
        "the cases are still read\n{}",
        list.shown()
    );

    let refs = grund(&scratch.root, &["refs", "E2E-a"]);
    assert_eq!(
        refs.stdout,
        "docs/n.md:3: \u{a7}E2E-a\n",
        "a citation of a case still resolves\n{}",
        refs.shown()
    );

    let check = grund(&scratch.root, &["check"]);
    assert_eq!(check.code, 2, "{}", check.shown());
    assert!(
        !check
            .stdout
            .lines()
            .any(|line| line.starts_with("docs/n.md:")),
        "the citation of E2E-a is not reported\n{}",
        check.shown()
    );
}
