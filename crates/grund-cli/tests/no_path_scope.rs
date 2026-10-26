//! §FS-scan.path.none: in a project with a config, a run given no `<path>` covers
//! the project discovery finds, however far below its root the run starts, and only
//! an explicit path narrows it. So from a subdirectory `grund <command>` is the run
//! given the config root's directory, and `grund <command> .` is a narrowed run. The
//! fixture is grund#854's: a section-locked citation in `src/`, and in `docs/` a
//! citation of an ID nothing declares.
//!
//! The help pages say the same in one line (§FS-cli.2.3.2); that test is the red
//! half of the contract until the pages change.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// §FS-cli.2.3.2: the line, worded once for every page whose path narrows.
const NO_PATH_LINE: &str = "With no PATH, the run covers the whole project or workspace \
                            discovered from the current directory; a PATH narrows it.";
const LOCK: &str = ".grund/sections/FS-login.lock";
const CONFIG: &str = "grund_config_version = 2\nproject_name = \"no-path-scope\"\n\n\
                      [schema]\nid_format = \"{kind}-{slug}\"\n\n\
                      [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\nindex = false\n";
const LOGIN: &str = "# FS-login: Login\n\n## format: Formatting\n\n### .1: Username\n\nTrim it.\n";
const AUDIT: &str = "# FS-audit: Audit\n\nSee \u{a7}FS-missing and \u{a7}FS-login.\n";
const SOURCE: &str = "/// Trims the user name (\u{a7}FS-login.format.1).\npub fn trim() {}\n";
const UNKNOWN: &str = "docs/functional-spec/FS-audit.md:3: error: unknown reference FS-missing";

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// One run's exit code, stdout and stderr.
type Run = (Option<i32>, String, String);

struct Repo {
    root: PathBuf,
}

impl Repo {
    /// grund#854's project, with no `.grund/` yet: `format.1` is a relative
    /// heading, so its citation is a locked address.
    fn new(what: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-854-{what}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        let repo = Self { root };
        repo.write("grund.toml", CONFIG);
        repo.write("docs/functional-spec/FS-login.md", LOGIN);
        repo.write("docs/functional-spec/FS-audit.md", AUDIT);
        repo.write("src/lib.rs", SOURCE);
        repo
    }

    /// The same project once a root-scoped `fmt --write` has written its lock, so
    /// that nothing under `src/` is left to report.
    fn locked(what: &str) -> Self {
        let repo = Self::new(what);
        let (code, _, stderr) = repo.run("", &["fmt", "--write"]);
        assert_eq!(code, Some(0), "the fixture's fmt --write failed: {stderr}");
        assert!(repo.has(LOCK), "the fixture's fmt --write wrote no lock");
        repo
    }

    fn write(&self, path: &str, body: &str) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn has(&self, path: &str) -> bool {
        self.root.join(path).exists()
    }

    /// Runs grund in `cwd`, relative to the fixture root.
    fn run(&self, cwd: &str, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.root.join(cwd))
            .output()
            .expect("spawn grund");
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// `args` with `path` appended as the positional path.
fn with_path<'a>(args: &[&'a str], path: &'a str) -> Vec<&'a str> {
    let mut args = args.to_vec();
    args.push(path);
    args
}

/// §FS-scan.path.none, §FS-section-lock.write.rules: from `src/`, a bare
/// `fmt --write` is the root-scoped run and writes the lock, and links the citation
/// in `docs/`, as the run given the config root's directory does, while the run
/// given `.` is narrowed to `src/` and writes nothing.
#[test]
fn a_bare_fmt_write_from_a_subdirectory_writes_the_lock_and_dot_does_not() {
    let written = (
        Some(0),
        format!(
            "rewrote 2 lines: 1 link written, 1 section-lock entry\n  {LOCK} (1) created\n  docs/functional-spec/FS-audit.md (1)\n"
        ),
        String::new(),
    );

    let bare = Repo::new("fmt-bare");
    assert_eq!(bare.run("src", &["fmt", "--write"]), written);
    assert!(bare.has(LOCK), "a bare fmt --write from src/ wrote no lock");

    let root = Repo::new("fmt-root");
    assert_eq!(root.run("src", &["fmt", "--write", ".."]), written);
    assert!(root.has(LOCK), "fmt --write .. from src/ wrote no lock");

    let dot = Repo::new("fmt-dot");
    assert_eq!(
        dot.run("src", &["fmt", "--write", "."]),
        (Some(0), "rewrote 0 lines\n".to_owned(), String::new())
    );
    assert!(!dot.has(LOCK), "fmt --write . from src/ wrote the lock");
}

/// §FS-scan.path.none, §FS-scan.path.check: from `src/`, a bare `check` reports
/// the unknown reference in `docs/` and exits 1, as the run given the config
/// root's directory does, while `check .` reports `src/` alone and exits 0.
#[test]
fn a_bare_check_from_a_subdirectory_reports_the_project_and_dot_does_not() {
    let repo = Repo::locked("check");
    let bare = repo.run("src", &["check"]);
    assert_eq!(bare.0, Some(1), "bare check from src/: {bare:?}");
    assert!(
        bare.1.lines().any(|line| line == UNKNOWN),
        "bare check from src/ did not report docs/: {bare:?}"
    );
    assert_eq!(repo.run("src", &["check", ".."]), bare);

    let dot = repo.run("src", &["check", "."]);
    assert_eq!(dot.0, Some(0), "check . from src/: {dot:?}");
    assert!(
        !dot.1.contains("FS-missing"),
        "check . from src/ reported docs/: {dot:?}"
    );
}

/// §FS-scan.path.none: every command whose path bounds what it scans or reports
/// gives, from `src/`, with no path the answer it gives for the config root's
/// directory, and another for `.`; one whose path only chooses the project that
/// answers gives one answer either way.
#[test]
fn from_a_subdirectory_no_path_is_the_project_and_dot_narrows() {
    let repo = Repo::locked("commands");
    let narrowing: &[&[&str]] = &[
        &["check"],
        &["list"],
        &["refs", "FS-login"],
        &["refs", "--glob", "FS-login"],
        &["refs", "--from", "FS-audit"],
        &["cover"],
        &["show", "--batch", "--all", "--format=json"],
        &["complete", "ids"],
    ];
    let answering: &[&[&str]] = &[
        &["show", "FS-login", "--toc"],
        &["id", "FS", "New thing"],
        &["config", "show"],
        &["schema"],
    ];
    let mut mismatches = Vec::new();
    for args in narrowing {
        let bare = repo.run("src", args);
        let root = repo.run("src", &with_path(args, ".."));
        let dot = repo.run("src", &with_path(args, "."));
        if bare != root {
            mismatches.push(format!(
                "grund {}: no path {bare:?} differs from `..` {root:?}",
                args.join(" ")
            ));
        }
        if bare == dot {
            mismatches.push(format!(
                "grund {}: no path and `.` agree, so `.` did not narrow: {bare:?}",
                args.join(" ")
            ));
        }
    }
    for args in answering {
        let bare = repo.run("src", args);
        let dot = repo.run("src", &with_path(args, "."));
        if bare != dot {
            mismatches.push(format!(
                "grund {}: no path {bare:?} differs from `.` {dot:?}",
                args.join(" ")
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// The `--help` page of `command`.
fn help_page(command: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args([command, "--help"])
        .output()
        .expect("spawn grund");
    assert_eq!(output.status.code(), Some(0), "grund {command} --help");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// §FS-cli.2.3.2: each page whose path narrows says, in the one line, what a run
/// with no path covers, and none of them says PATH defaults to `.`. `show`'s path
/// narrows only its generated batches, so its `--path` entry says what those cover
/// with no path, and the page names no `.` default.
#[test]
fn each_narrowing_help_page_says_what_a_run_with_no_path_covers() {
    let mut failures = Vec::new();
    for command in ["check", "fmt", "cover", "list", "refs"] {
        let page = help_page(command);
        if !page.lines().any(|line| line == NO_PATH_LINE) {
            failures.push(format!("grund {command} --help: no line {NO_PATH_LINE:?}"));
        }
        for line in page
            .lines()
            .filter(|line| line.contains("PATH defaults to `.`"))
        {
            failures.push(format!("grund {command} --help: still says {line:?}"));
        }
    }
    let page = help_page("show");
    for line in page.lines().filter(|line| line.contains("default `.`")) {
        failures.push(format!("grund show --help: still says {line:?}"));
    }
    let entry = page
        .lines()
        .skip_while(|line| !line.starts_with("  --path PATH"))
        .take_while(|line| !line.contains("e.g."))
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ");
    for says in [
        "--batch --all",
        "--glob",
        "with no PATH cover the whole project or workspace discovered from the current directory",
    ] {
        if !entry.contains(says) {
            failures.push(format!(
                "grund show --help: --path entry {entry:?} lacks {says:?}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
