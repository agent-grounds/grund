//! Black-box: a directory link whose target lookup is refused on the way, because a directory
//! above the target cannot be searched, is not a broken link: it is reported at the link, whatever
//! its name, and a run given the link as its path does not say the path does not exist
//! (§FS-scan.scope.links.owed.refused, §FS-scan.scope.links.unresolvable.unopenable). A broken
//! link with no extension stays the declared blind spot (§FS-scan.scope.links.blind-spot), and a
//! link the fence prunes stays silent (§FS-scan.scope.links.owed.fence). Git carries no mode, so
//! the e2e corpus cannot hold these trees, and every test skips where mode 000 does not bar the
//! user (root).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n";

/// The line §FS-scan.scope.links.unresolvable.unopenable gives the link `docs/x`.
const LINE: &str = "error: docs/x: unreadable symlink: permission denied; \
    make the path readable, or keep it out of the scan";

/// The issue's tree under `<base>/p`: `docs/a.md` declares and cites `FS-a`, and the hidden
/// `.r/sub/x.md` cites an unknown `FS-nope`, so the run is clean unless it reads through a link
/// into `.r`. `<base>/outside` lies outside the project.
struct Tree {
    base: PathBuf,
    locked: Vec<PathBuf>,
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Tree {
    fn new(test: &str) -> Self {
        let base = std::env::temp_dir().join(format!("grund-1325-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let project = base.join("p");
        fs::create_dir_all(project.join("docs")).expect("create the home");
        fs::create_dir_all(project.join(".r/sub")).expect("create the target");
        fs::create_dir_all(base.join("outside/sub")).expect("create the outside target");
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
        .expect("write the citation behind the link");
        Self {
            base,
            locked: Vec::new(),
        }
    }

    /// Creates the link `p/docs/<name>` pointing at `target`, written relative to the link.
    fn link(&self, name: &str, target: &str) {
        symlink(target, self.base.join("p/docs").join(name)).expect("create the link");
    }

    /// Bars the directory `rel`, relative to the base, and says whether the mode bars this user:
    /// it does not bar root, and then nothing here is refused and the test proves nothing.
    fn bar(&mut self, rel: &str) -> bool {
        let path = self.base.join(rel);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("set the mode");
        self.locked.push(path.clone());
        let barred = fs::read_dir(&path).is_err();
        if !barred {
            eprintln!("skipped: the mode does not bar this user, so no lookup is refused");
        }
        barred
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
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// The issue's case: `docs/x -> ../.r/sub` with `.r` barred, or `None` where the mode does
    /// not bar this user.
    fn barred(test: &str) -> Option<Self> {
        let mut tree = Self::new(test);
        tree.link("x", "../.r/sub");
        tree.bar("p/.r").then_some(tree)
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        for path in &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.base);
    }
}

impl Run {
    fn says_line(&self) -> bool {
        self.stderr.lines().any(|line| line == LINE)
    }
}

/// `(severity, path, message)` of every `io` record a `--format json` run printed on stderr.
fn io_records(stderr: &str) -> Vec<(String, String, String)> {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|record| record["code"] == "io")
        .map(|record| {
            let field = |name: &str| record[name].as_str().unwrap_or("(none)").to_string();
            (field("severity"), field("path"), field("message"))
        })
        .collect()
}

/// Control: with nothing barred the walk reads through the link and finds `FS-nope`.
#[test]
fn a_readable_link_is_read_through() {
    let tree = Tree::new("readable");
    tree.link("x", "../.r/sub");
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 1, "{}{}", run.stdout, run.stderr);
    assert!(
        format!("{}{}", run.stdout, run.stderr)
            .contains("docs/x/x.md:3: error: unknown reference FS-nope"),
        "{}{}",
        run.stdout,
        run.stderr
    );
}

/// Control: the target itself barred is reported at the link, as it already is.
#[test]
fn a_barred_target_is_reported_at_the_link() {
    let mut tree = Tree::new("subdenied");
    tree.link("x", "../.r/sub");
    if !tree.bar("p/.r/sub") {
        return;
    }
    let run = tree.run(&["check"]);
    assert_eq!((run.code, run.says_line()), (2, true), "{}", run.stderr);
}

/// Control: the same link named with a scanned extension is reported at the link, as it already
/// is.
#[test]
fn a_link_with_an_extension_behind_a_barred_parent_is_reported() {
    let mut tree = Tree::new("mdname");
    tree.link("x.md", "../.r/sub");
    if !tree.bar("p/.r") {
        return;
    }
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(
        run.stderr
            .contains("error: docs/x.md: unreadable symlink: permission denied"),
        "{}",
        run.stderr
    );
}

/// Control: a target that the lookup says is not there is a broken link, and with no extension
/// it is the declared blind spot (§FS-scan.scope.links.blind-spot).
#[test]
fn a_missing_target_with_no_extension_stays_the_blind_spot() {
    let tree = Tree::new("missing");
    tree.link("x", "../.r/nothing");
    let run = tree.run(&["check"]);
    assert_eq!(
        (run.code, run.stdout.trim()),
        (0, "success"),
        "{}",
        run.stderr
    );
}

/// Control: a link leaving the project, its target behind a barred directory out there, is
/// pruned by the fence in silence (§FS-scan.scope.links.owed.fence).
#[test]
fn an_outward_link_behind_a_barred_parent_stays_pruned() {
    let mut tree = Tree::new("outward");
    tree.link("x", "../../outside/sub");
    if !tree.bar("outside") {
        return;
    }
    let run = tree.run(&["check"]);
    assert_eq!(
        (run.code, run.stdout.trim()),
        (0, "success"),
        "{}",
        run.stderr
    );
}

/// The issue: `check`, `check docs` and `check --full` each exit 2 with the link's line, where
/// they printed `success` and exited 0.
#[test]
fn a_link_behind_a_barred_parent_fails_check_at_the_link() {
    let Some(tree) = Tree::barred("check") else {
        return;
    };
    for args in [&["check"][..], &["check", "docs"], &["check", "--full"]] {
        let run = tree.run(args);
        assert_eq!(
            (run.code, run.says_line()),
            (2, true),
            "grund {}:\n{}{}",
            args.join(" "),
            run.stdout,
            run.stderr
        );
    }
}

/// The issue in JSON: one `io` error at `docs/x`, where the run printed nothing and exited 0.
#[test]
fn a_link_behind_a_barred_parent_is_one_json_error_at_the_link() {
    let Some(tree) = Tree::barred("json") else {
        return;
    };
    let run = tree.run(&["check", "--format", "json"]);
    let io = io_records(&run.stderr);
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert_eq!(io.len(), 1, "{}", run.stderr);
    assert_eq!(
        (io[0].0.as_str(), io[0].1.as_str()),
        ("error", "docs/x"),
        "{}",
        run.stderr
    );
    assert!(
        io[0].2.contains("unreadable symlink: permission denied"),
        "{}",
        run.stderr
    );
}

/// The link given as the path: it exists, so the run does not say that the path does not exist,
/// and it fails at the link.
#[test]
fn a_link_behind_a_barred_parent_given_as_the_path_exists() {
    let Some(tree) = Tree::barred("scoped") else {
        return;
    };
    let run = tree.run(&["check", "docs/x"]);
    assert!(
        !run.stderr.contains("path does not exist"),
        "{}",
        run.stderr
    );
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(
        run.stderr
            .contains("error: docs/x: unreadable symlink: permission denied"),
        "{}",
        run.stderr
    );
}

/// `cover`, `list` and `fmt --check` read the same scan, so they fail at the same link.
#[test]
fn a_link_behind_a_barred_parent_fails_cover_list_and_fmt() {
    let Some(tree) = Tree::barred("others") else {
        return;
    };
    for args in [&["cover"][..], &["list"], &["fmt", "--check"]] {
        let run = tree.run(args);
        assert_eq!(
            run.code,
            2,
            "grund {}:\n{}{}",
            args.join(" "),
            run.stdout,
            run.stderr
        );
        assert!(
            run.stderr
                .contains("docs/x: unreadable symlink: permission denied"),
            "grund {}:\n{}",
            args.join(" "),
            run.stderr
        );
    }
}

/// Behind a refusal the run cannot tell whether anything stands there, so a link whose target
/// would not exist is reported the same way (§FS-scan.scope.links.owed.refused).
#[test]
fn a_link_behind_a_barred_parent_is_reported_whatever_stands_behind_it() {
    let mut tree = Tree::new("unknown");
    tree.link("x", "../.r/nothing");
    if !tree.bar("p/.r") {
        return;
    }
    let run = tree.run(&["check"]);
    assert_eq!((run.code, run.says_line()), (2, true), "{}", run.stderr);
}

/// Control: only a refused lookup is rerouted. An extensionless cycle fails its lookup with a loop,
/// not a refusal, so it keeps the route it had until #1331 rules on it: a bare run passes, and the
/// link given as the path does not exist (§FS-scan.scope.links.owed.refused).
#[test]
fn an_extensionless_cycle_keeps_its_route() {
    let tree = Tree::new("cycle");
    tree.link("x", "y");
    tree.link("y", "x");
    let run = tree.run(&["check"]);
    assert_eq!(
        (run.code, run.stdout.trim()),
        (0, "success"),
        "{}",
        run.stderr
    );
    let run = tree.run(&["check", "docs/x"]);
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert_eq!(
        run.stderr.trim(),
        "error: path does not exist: docs/x",
        "{}",
        run.stderr
    );
}
