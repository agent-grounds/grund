//! Black-box: a `<path>` given to a command is missing only when its lookup says nothing can stand
//! there (§FS-scan.scope.homes.lookup.path). Behind a directory the user cannot search, the lookup
//! is refused, so a file, a directory and a name that may not exist are each reported as a
//! permission failure at the path as given, on `check`, `list`, `cover`, `fmt`, `show` and `refs`
//! alike, and never as `path does not exist`. The controls hold what must not move: a path whose
//! lookup says not found, or that runs through a regular file, keeps `path does not exist`, a link
//! to the same place keeps its line (§FS-scan.scope.links.owed.refused), and a readable path is
//! read. Git carries no mode, so the e2e corpus cannot hold these trees, and every barred case
//! skips where mode 000 does not bar the user (root).
#![cfg(unix)]

#[path = "../../crates/grund-cli/tests/support/fixture_cleanup.rs"]
mod fixture_cleanup;

use fixture_cleanup::remove_fixture_tree;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n";

/// The fix §FS-check.2.4.3 gives a path the command was given, which `grund check .r` already
/// prints after the `permission denied` reason.
const PASSED_FIX: &str = "make the path readable, or keep it out of the scan by not passing it, \
    because exclude does not prune a path the command is given";

/// The issue's tree under `<base>/p`: `docs/a.md` declares and cites `FS-a`, and the hidden
/// `.r/sub/x.md`, a regular file and no link, cites an unknown `FS-nope`.
struct Tree {
    base: PathBuf,
    locked: Vec<PathBuf>,
}

struct Run {
    shown: String,
    code: i32,
    stdout: String,
    stderr: String,
}

impl Tree {
    fn new(test: &str) -> Self {
        let base = std::env::temp_dir().join(format!("grund-1354-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let project = base.join("p");
        fs::create_dir_all(project.join("docs")).expect("create the home");
        fs::create_dir_all(project.join(".r/sub")).expect("create the barred directory");
        fs::write(project.join("grund.toml"), CONFIG).expect("write the config");
        fs::write(
            project.join("docs/a.md"),
            "# FS-a: A\n\nText, see \u{a7}FS-a.\n",
        )
        .expect("write the declaration");
        fs::write(
            project.join(".r/sub/x.md"),
            "# Notes\n\nSee \u{a7}FS-nope.\n",
        )
        .expect("write the file behind the barred directory");
        Self {
            base,
            locked: Vec::new(),
        }
    }

    /// The issue's case: `.r` at mode 000, with the link `docs/x -> ../.r/sub` beside it for
    /// comparison, or `None` where the mode does not bar this user and nothing is refused.
    fn barred(test: &str) -> Option<Self> {
        let mut tree = Self::new(test);
        symlink("../.r/sub", tree.base.join("p/docs/x")).expect("create the link");
        let path = tree.base.join("p/.r");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("set the mode");
        tree.locked.push(path.clone());
        if fs::metadata(path.join("sub/x.md")).is_ok() {
            eprintln!("skipped: the mode does not bar this user, so no lookup is refused");
            return None;
        }
        Some(tree)
    }

    /// `grund <args>` from the project root.
    fn run(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(self.base.join("p"))
            .output()
            .expect("run grund");
        Run {
            shown: format!("grund {}", args.join(" ")),
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        // The mode first: a barred directory cannot be emptied.
        for path in &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        remove_fixture_tree(&self.base, "remove fixture");
    }
}

impl Run {
    /// What went wrong for §FS-scan.scope.homes.lookup.path, or `None`: exit `2`, the permission
    /// line at `path` exactly as given, and never `path does not exist`.
    fn permission_failure_at(&self, path: &str) -> Option<String> {
        let line = format!("error: {path}: permission denied; {PASSED_FIX}");
        let missing = self.stderr.contains("path does not exist");
        if self.code == 2 && !missing && self.stderr.lines().any(|l| l == line) {
            return None;
        }
        Some(format!(
            "{}: want exit 2 and the line `{line}`, never `path does not exist`; \
             got exit {}\n  stdout: {:?}\n  stderr: {:?}",
            self.shown, self.code, self.stdout, self.stderr
        ))
    }
}

/// Every failure in `failures`, all at once, so one case never hides another.
fn assert_none(failures: Vec<String>) {
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Control: with `.r` readable the path is read, and its unknown citation is the finding.
#[test]
fn a_readable_path_is_read() {
    let tree = Tree::new("readable");
    let run = tree.run(&["check", ".r/sub/x.md"]);
    assert_eq!(run.code, 1, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .contains(".r/sub/x.md:3: error: unknown reference FS-nope"),
        "{}{}",
        run.stdout,
        run.stderr
    );
}

/// Control: a lookup that says nothing can stand there is a missing path, barred tree or not:
/// not found, or a component on the way that is a regular file.
#[test]
fn a_path_whose_lookup_says_not_there_does_not_exist() {
    let tree = Tree::barred("absent").unwrap_or_else(|| Tree::new("absent-unbarred"));
    let mut failures = Vec::new();
    for path in ["nope/x.md", "docs/a.md/x.md"] {
        let run = tree.run(&["check", path]);
        let line = format!("error: path does not exist: {path}\n");
        if (run.code, run.stdout.as_str(), run.stderr.as_str()) != (2, "", line.as_str()) {
            failures.push(format!(
                "{}: want exit 2, empty stdout and `{}`; got exit {}\n  stdout: {:?}\n  stderr: {:?}",
                run.shown,
                line.trim_end(),
                run.code,
                run.stdout,
                run.stderr
            ));
        }
    }
    assert_none(failures);
}

/// Control: the link to the same barred place keeps the line #1325 gave it.
#[test]
fn a_link_behind_the_barred_directory_keeps_its_line() {
    let Some(tree) = Tree::barred("link") else {
        return;
    };
    let run = tree.run(&["check", "docs/x"]);
    let line = format!("error: docs/x: unreadable symlink: permission denied; {PASSED_FIX}");
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(run.stderr.lines().any(|l| l == line), "{}", run.stderr);
}

/// The issue: a file, a directory, and a name that may not exist behind the barred `.r` each fail
/// at the path given, where the run said `path does not exist`.
#[test]
#[ignore = "red until #1354 stops reading a refused lookup as a missing path; implement removes this"]
fn a_barred_path_is_a_permission_failure_at_the_path() {
    let Some(tree) = Tree::barred("check") else {
        return;
    };
    assert_none(
        [".r/sub/x.md", ".r/sub", ".r/sub/nothere.md"]
            .into_iter()
            .filter_map(|path| tree.run(&["check", path]).permission_failure_at(path))
            .collect(),
    );
}

/// The issue in JSON: one `io` error at the path given, where the run printed the text line.
#[test]
#[ignore = "red until #1354 stops reading a refused lookup as a missing path; implement removes this"]
fn a_barred_path_is_one_json_error_at_the_path() {
    let Some(tree) = Tree::barred("json") else {
        return;
    };
    let run = tree.run(&["check", ".r/sub/x.md", "--format", "json"]);
    let at_path: Vec<serde_json::Value> = run
        .stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|record| record["path"] == ".r/sub/x.md" && record["severity"] == "error")
        .collect();
    let want = format!("permission denied; {PASSED_FIX}");
    assert!(
        run.code == 2
            && !run.stderr.contains("path does not exist")
            && at_path.len() == 1
            && at_path[0]["code"] == "io"
            && at_path[0]["message"] == want.as_str(),
        "{}: want exit 2 and one `io` error at `.r/sub/x.md` reading `{want}`; got exit {}\n  \
         stdout: {:?}\n  stderr: {:?}",
        run.shown,
        run.code,
        run.stdout,
        run.stderr
    );
}

/// Every command that scans from a `<path>` reaches the same site, so each fails at the path.
#[test]
#[ignore = "red until #1354 stops reading a refused lookup as a missing path; implement removes this"]
fn a_barred_path_fails_every_command_at_the_path() {
    let Some(tree) = Tree::barred("others") else {
        return;
    };
    let path = ".r/sub/x.md";
    let commands: [&[&str]; 5] = [
        &["list", path],
        &["cover", path],
        &["fmt", "--check", path],
        &["show", "FS-a", path],
        &["refs", "FS-a", path],
    ];
    assert_none(
        commands
            .into_iter()
            .filter_map(|args| tree.run(args).permission_failure_at(path))
            .collect(),
    );
}
