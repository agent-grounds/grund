//! An ignore file the walk cannot parse is one failure of the run, whichever walk root met it
//! (§FS-check.2.4, §FS-check.2.4.2): a project walked only from the directory that holds the file
//! reports it and exits `2` exactly as one whose kind home sits below the file does. The trees
//! are built in the temp directory, because a malformed `.ignore` committed to this repository
//! would be read by every other tool that honours ignore files.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REMEDY: &str = "make the path readable, or keep it out of the scan";
const ALPHA: &str = "error parsing glob '{alpha': unclosed alternate group; \
                     missing '}' (maybe escape '{' with '[{]'?)";

/// One project tree under the temp directory, removed on drop.
struct Tree(PathBuf);

impl Tree {
    /// A version-2 project `p` holding `.ignore` with `ignore` at its root. With `kind_home`, an
    /// `FS` kind keeps its home at `docs/`, so the walk starts below the file; without, the
    /// project declares no kinds and the walk starts at the config root, where the file is.
    fn new(name: &str, kind_home: bool, ignore: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1305-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the tree");
        let mut config = String::from("grund_config_version = 2\nproject_name = \"p\"\n");
        if kind_home {
            config.push_str(
                "\n[schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.FS]\n\
                 folders = [\"docs\"]\n",
            );
            fs::create_dir(root.join("docs")).expect("create the kind home");
            fs::write(root.join("docs/x.md"), "# Notes\n").expect("write the note");
        } else {
            fs::write(root.join("x.md"), "# Notes\n").expect("write the note");
        }
        fs::write(root.join("grund.toml"), config).expect("write the config");
        fs::write(root.join(".ignore"), ignore).expect("write the ignore file");
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
            root: self.0.clone(),
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
    root: PathBuf,
}

impl Run {
    fn shown(&self) -> String {
        format!(
            "exit {}\nstdout:\n{}\nstderr:\n{}",
            self.code, self.stdout, self.stderr
        )
    }

    /// The run's own failure lines (§FS-errors.2.2): a located finding reads `path:line:`.
    fn failure_lines(&self) -> Vec<&str> {
        self.stderr
            .lines()
            .filter(|line| line.starts_with("error: "))
            .collect()
    }

    fn mentions_root(&self, root: &Path) -> bool {
        let root = root.to_string_lossy();
        self.stdout.contains(root.as_ref()) || self.stderr.contains(root.as_ref())
    }
}

/// The one line a single malformed line in `.ignore` earns, wherever the walk met it.
fn one_line_failure() -> String {
    format!("error: .ignore: line 1: {ALPHA}; {REMEDY}")
}

#[test]
#[ignore = "red until #1305 reads the walk root's own entry error; implement removes this"]
fn a_malformed_ignore_file_at_the_only_walk_root_is_reported_once() {
    let tree = Tree::new("root-one", false, "{alpha\n");
    let run = tree.check();
    assert_eq!(run.code, 2, "{}", run.shown());
    assert_eq!(run.stdout, "", "{}", run.shown());
    assert_eq!(
        run.stderr,
        format!("{}\n", one_line_failure()),
        "{}",
        run.shown()
    );
}

#[test]
#[ignore = "red until #1305 reads the walk root's own entry error; implement removes this"]
fn two_malformed_lines_at_the_only_walk_root_are_reported_at_the_ignore_file() {
    // Two bad lines reach the walk as one `ignore::Error::Partial`, which names no path itself.
    let tree = Tree::new("root-two", false, "{alpha\n{beta\n");
    let run = tree.check();
    assert_eq!(run.code, 2, "{}", run.shown());
    let lines = run.failure_lines();
    assert!(!lines.is_empty(), "no failure reported:\n{}", run.shown());
    assert!(
        lines.iter().all(|line| line.starts_with("error: .ignore:")),
        "every failure line must sit at `.ignore`:\n{}",
        run.shown()
    );
    for glob in ["'{alpha'", "'{beta'"] {
        assert!(
            run.stderr.contains(glob),
            "{glob} unnamed:\n{}",
            run.shown()
        );
    }
    let mut unique = lines.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        lines.len(),
        "a line repeats:\n{}",
        run.shown()
    );
    // §FS-errors.4: a report carries no absolute path.
    assert!(
        !run.mentions_root(&run.root),
        "absolute path:\n{}",
        run.shown()
    );
}

#[test]
fn a_malformed_ignore_file_above_a_kind_home_is_reported_once() {
    let tree = Tree::new("kind-home", true, "{alpha\n");
    let run = tree.check();
    assert_eq!(run.code, 2, "{}", run.shown());
    assert_eq!(run.stdout, "", "{}", run.shown());
    assert_eq!(
        run.stderr,
        format!("{}\n", one_line_failure()),
        "{}",
        run.shown()
    );
}
