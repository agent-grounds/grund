//! Black-box: what an E2E case holds is read whole, as the case's data
//! (§FS-declarations.e2e-case.excluded). `exclude` and the ignore files decide which case
//! directories are read. Neither trims the fixture list `grund E2E-<name>` prints (§FS-show.2.4),
//! and neither hides a failure inside a case that is read (§FS-declarations.e2e-case.failure). A
//! fixture's own `.gitignore` is fixture content, and so is a pattern that keeps fixture trees out
//! of the walk. Git carries no mode a fixture could keep, so the e2e corpus cannot hold the barred
//! trees, and the tests that need a mode skip where it does not bar this user.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const SKIP: &str = "skipped: the mode does not bar this user, so nothing here can be unreadable";
const REMEDY: &str = "make the path readable, or keep it out of the scan";

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
    /// A version-2 project, a git repository so that a `.gitignore` is one the walk honours, whose
    /// `E2E` kind keeps case `a` in `e2e/cases`; `sources` is written under `[schema.sources]`.
    fn new(test: &str, sources: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("grund-1271-whole-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the root");
        let git = Command::new("git")
            .args(["init", "-q", "."])
            .current_dir(&root)
            .output()
            .expect("spawn git init");
        assert!(git.status.success(), "git init {}", root.display());
        let tree = Self {
            root,
            locked: Vec::new(),
        };
        tree.write(
            "grund.toml",
            &format!(
                "grund_config_version = 2\nproject_name = \"p\"\n\n\
                 [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n[schema.sources]\n{sources}\n\n\
                 [schema.kinds.E2E]\nfolders = [\"e2e/cases\"]\nindex = false\n"
            ),
        );
        tree.write("e2e/cases/a/expected.exit", "0\n");
        tree
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(path, contents).expect("write the file");
    }

    /// Takes the directory's mode, and answers whether that bars this user.
    fn lock(&mut self, relative: &str) -> bool {
        let path = self.root.join(relative);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("take the mode");
        self.locked.push(path.clone());
        fs::read_dir(&path).is_err()
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

    /// The fixture list `grund E2E-a --full` prints, in its order.
    fn fixtures(&self) -> Vec<String> {
        let run = self.run(&["E2E-a", "--full"]);
        assert_eq!(run.code, 0, "{run:#?}");
        run.stdout
            .lines()
            .skip_while(|line| *line != "fixtures:")
            .filter_map(|line| line.strip_prefix("- ").map(str::to_string))
            .collect()
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

fn error_lines(run: &Run) -> Vec<&str> {
    run.stderr
        .lines()
        .filter(|line| line.starts_with("error: "))
        .collect()
}

/// A fixture's own `.gitignore` and `.ignore` are content of the fixture repository, so the
/// paths they name stay in the case's fixture list.
#[test]
fn a_fixtures_own_ignore_files_do_not_trim_its_list() {
    let tree = Tree::new("ignore-files", "");
    tree.write("e2e/cases/a/repo/.gitignore", "gen/\n");
    tree.write("e2e/cases/a/repo/.ignore", "keep/\n");
    tree.write("e2e/cases/a/repo/gen/g.md", "# G\n");
    tree.write("e2e/cases/a/repo/keep/k.md", "# K\n");
    assert_eq!(
        tree.fixtures(),
        [
            "expected.exit",
            "repo/.gitignore",
            "repo/.ignore",
            "repo/gen/g.md",
            "repo/keep/k.md"
        ]
    );
}

/// The pattern grund's own configuration keeps fixture trees out of the walk with leaves the
/// fixture list whole, and a fixture directory that cannot be listed still fails the run.
#[test]
fn an_exclude_pattern_for_fixture_trees_keeps_the_list_and_the_failure() {
    let mut tree = Tree::new("exclude", "exclude = [\"repo/\", \"expected.repo/\"]");
    tree.write("e2e/cases/a/repo/docs/d.md", "# D\n");
    tree.write("e2e/cases/a/expected.repo/out.md", "# O\n");
    assert_eq!(
        tree.fixtures(),
        ["expected.exit", "expected.repo/out.md", "repo/docs/d.md"]
    );
    if !tree.lock("e2e/cases/a/repo/docs") {
        eprintln!("{SKIP}");
        return;
    }
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 2, "{run:#?}");
    assert_eq!(
        error_lines(&run),
        [format!(
            "error: e2e/cases/a/repo/docs: permission denied; {REMEDY}"
        )],
        "{run:#?}"
    );
}

/// A version 1 `[scan] exclude` name keeps out a case directory of that name, never a
/// directory of that name inside a case, so a fixture directory there that cannot be listed
/// still fails the run.
#[test]
fn a_version_1_exclude_name_inside_a_case_hides_no_failure() {
    let mut tree = Tree::new("v1-name", "");
    tree.write(
        "grund.toml",
        "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\n\n\
         [[kinds]]\nkind = \"E2E\"\nfolder = \"e2e/cases\"\nindex = false\n\n\
         [scan]\nexclude = [\"fx\"]\n",
    );
    tree.write("e2e/cases/a/fx/f.md", "# F\n");
    if !tree.lock("e2e/cases/a/fx") {
        eprintln!("{SKIP}");
        return;
    }
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 2, "{run:#?}");
    assert_eq!(
        error_lines(&run),
        [format!(
            "error: e2e/cases/a/fx: permission denied; {REMEDY}"
        )],
        "{run:#?}"
    );
}
