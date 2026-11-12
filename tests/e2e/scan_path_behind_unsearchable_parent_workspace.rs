//! Black-box: §FS-scan.scope.homes.lookup.path in a workspace run. A workspace run walks each
//! project whole and narrows its report to the `<path>` afterwards (§FS-check.1.3.6.1), so no walk
//! starts at the path and the walk never asks it. A path behind a directory the user cannot
//! search must still fail at the path as given, with exit `2`, on `check` in text and JSON and on
//! `list`, `cover`, `fmt`, `show` and `refs`, whether or not the root is a project of the run and
//! whether the barred directory is outside every member or is a member itself. It is never
//! located only at the directory that refused the lookup, and the run never passes over a tree it
//! did not read. The controls keep what must not move: a readable path is read, and a path whose
//! lookup says not found is `path does not exist`. Every barred case skips where mode 000 does
//! not bar the user (root).
#![cfg(unix)]

#[path = "../../crates/grund-cli/tests/support/fixture_cleanup.rs"]
mod fixture_cleanup;

use fixture_cleanup::remove_fixture_tree;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

/// The fix §FS-check.2.4.3 gives a path the command was given.
const PASSED_FIX: &str = "make the path readable, or keep it out of the scan by not passing it, \
    because exclude does not prune a path the command is given";

/// A config with one FS home, `docs`, and whatever `extra` adds.
fn config(name: &str, extra: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"{name}\"\n\n\
         [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
         [schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n{extra}"
    )
}

/// A workspace root at `<base>/w` with the member `m`: each declares `FS-a` in `docs/a.md`, and
/// `rr/sub/x.md`, outside every member, and `m/docs/b.md` each cite an unknown `FS-nope`.
struct Workspace {
    base: PathBuf,
    include_root: bool,
    locked: Vec<PathBuf>,
}

struct Run {
    shown: String,
    code: i32,
    stdout: String,
    stderr: String,
}

impl Workspace {
    fn new(test: &str, include_root: bool) -> Self {
        let base = std::env::temp_dir().join(format!(
            "grund-1354-ws-{test}-{include_root}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        let root = base.join("w");
        for dir in ["docs", "m/docs", "rr/sub"] {
            fs::create_dir_all(root.join(dir)).expect("create the tree");
        }
        let workspace =
            format!("\n[workspace]\nmembers = [\"m\"]\ninclude_root = {include_root}\n");
        fs::write(root.join("grund.toml"), config("root", &workspace)).expect("write the root");
        fs::write(root.join("m/grund.toml"), config("m", "")).expect("write the member");
        for home in ["docs", "m/docs"] {
            fs::write(
                root.join(home).join("a.md"),
                "# FS-a: A\n\nText, see \u{a7}FS-a.\n",
            )
            .expect("write the declaration");
        }
        for file in ["rr/sub/x.md", "m/docs/b.md"] {
            fs::write(root.join(file), "# Notes\n\nSee \u{a7}FS-nope.\n").expect("write the file");
        }
        Self {
            base,
            include_root,
            locked: Vec::new(),
        }
    }

    /// The workspace with `dir` at mode 000, or `None` where the mode does not bar this user.
    fn barred(test: &str, include_root: bool, dir: &str) -> Option<Self> {
        let mut workspace = Self::new(test, include_root);
        let path = workspace.base.join("w").join(dir);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("set the mode");
        workspace.locked.push(path.clone());
        if fs::read_dir(&path).is_ok() {
            eprintln!("skipped: the mode does not bar this user, so no lookup is refused");
            return None;
        }
        Some(workspace)
    }

    /// `grund <args>` from the workspace root.
    fn run(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(self.base.join("w"))
            .output()
            .expect("run grund");
        Run {
            shown: format!(
                "grund {} (include_root = {})",
                args.join(" "),
                self.include_root
            ),
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Workspace {
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

/// The ID `show` and `refs` take: unqualified where the root is a project, else the member's,
/// because an unqualified ID needs a project alias when `include_root = false`.
fn shown_id(include_root: bool) -> &'static str {
    if include_root { "FS-a" } else { "m/FS-a" }
}

/// R1-01: with `rr` barred, a file, a name that may not exist and a directory under it each fail
/// at the path, and so does `list`, where the run passed with `nothing to scan` or failed only at
/// `rr`, whichever project the root is.
#[test]
fn a_barred_path_in_a_workspace_is_a_permission_failure_at_the_path() {
    let mut failures = Vec::new();
    for include_root in [true, false] {
        let Some(workspace) = Workspace::barred("check", include_root, "rr") else {
            return;
        };
        for path in ["rr/sub/x.md", "rr/sub/nothere.md", "rr/sub"] {
            failures.extend(workspace.run(&["check", path]).permission_failure_at(path));
        }
        let path = "rr/sub/x.md";
        failures.extend(workspace.run(&["list", path]).permission_failure_at(path));
    }
    assert_none(failures);
}

/// R1-01 in JSON: one `io` error at the path, where the run printed only cautions.
#[test]
fn a_barred_path_in_a_workspace_is_one_json_error_at_the_path() {
    let mut failures = Vec::new();
    for include_root in [true, false] {
        let Some(workspace) = Workspace::barred("json", include_root, "rr") else {
            return;
        };
        let run = workspace.run(&["check", "rr/sub/x.md", "--format", "json"]);
        let at_path: Vec<serde_json::Value> = run
            .stderr
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|record| record["path"] == "rr/sub/x.md" && record["severity"] == "error")
            .collect();
        let want = format!("permission denied; {PASSED_FIX}");
        if run.code != 2
            || at_path.len() != 1
            || at_path[0]["code"] != "io"
            || at_path[0]["message"] != want.as_str()
        {
            failures.push(format!(
                "{}: want exit 2 and one `io` error at `rr/sub/x.md` reading `{want}`; got exit \
                 {}\n  stdout: {:?}\n  stderr: {:?}",
                run.shown, run.code, run.stdout, run.stderr
            ));
        }
    }
    assert_none(failures);
}

/// R1-01 for the other commands that take a `<path>`, as the single-project pins loop them.
#[test]
fn a_barred_path_in_a_workspace_fails_every_command_at_the_path() {
    let mut failures = Vec::new();
    for include_root in [true, false] {
        let Some(workspace) = Workspace::barred("others", include_root, "rr") else {
            return;
        };
        let (path, id) = ("rr/sub/x.md", shown_id(include_root));
        let commands: [&[&str]; 4] = [
            &["cover", path],
            &["fmt", "--check", path],
            &["show", id, path],
            &["refs", id, path],
        ];
        for args in commands {
            failures.extend(workspace.run(args).permission_failure_at(path));
        }
    }
    assert_none(failures);
}

/// R1-01 for a member the user cannot search: the path in it fails at the path, where the root
/// run passed over it, and `show` of the root's ID fails there too.
#[test]
fn a_path_in_a_barred_member_is_a_permission_failure_at_the_path() {
    let mut failures = Vec::new();
    for include_root in [true, false] {
        let Some(workspace) = Workspace::barred("member", include_root, "m") else {
            return;
        };
        let path = "m/docs/b.md";
        failures.extend(workspace.run(&["check", path]).permission_failure_at(path));
        failures.extend(workspace.run(&["list", path]).permission_failure_at(path));
        if include_root {
            let run = workspace.run(&["show", "FS-a", path]);
            failures.extend(run.permission_failure_at(path));
        }
    }
    assert_none(failures);
}

/// Control: with nothing barred, the path under the root project is read and its unknown
/// citation is the finding.
#[test]
fn a_readable_path_in_a_workspace_is_read() {
    let workspace = Workspace::new("readable", true);
    let run = workspace.run(&["check", "rr/sub/x.md"]);
    assert_eq!(run.code, 1, "{}{}", run.stdout, run.stderr);
    assert!(
        run.stdout
            .lines()
            .any(|line| line == "rr/sub/x.md:3: error: unknown reference FS-nope"),
        "{}{}",
        run.stdout,
        run.stderr
    );
}

/// Control: a path whose lookup says not found keeps `path does not exist`, beside a barred `rr`.
#[test]
fn an_absent_path_in_a_workspace_does_not_exist() {
    let mut failures = Vec::new();
    for include_root in [true, false] {
        let workspace = Workspace::barred("absent", include_root, "rr")
            .unwrap_or_else(|| Workspace::new("absent-unbarred", include_root));
        let run = workspace.run(&["check", "nope/x.md"]);
        let line = "error: path does not exist: nope/x.md";
        if run.code != 2 || !run.stdout.is_empty() || !run.stderr.lines().any(|l| l == line) {
            failures.push(format!(
                "{}: want exit 2, empty stdout and `{line}`; got exit {}\n  stdout: {:?}\n  \
                 stderr: {:?}",
                run.shown, run.code, run.stdout, run.stderr
            ));
        }
    }
    assert_none(failures);
}
