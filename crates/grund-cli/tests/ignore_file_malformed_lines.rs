//! Black-box: an ignore file the walk cannot parse is one failure at the ignore file, spelled
//! relative to the project, each malformed line named once however many walk roots read it, and
//! no absolute path in the reason (§FS-check.2.4.2, §FS-errors.4). One malformed line already
//! reports that way; two are folded by the `ignore` crate into one error with no path of its own,
//! which was placed at every walk root with the file's absolute path (tool report grund.263). The
//! e2e corpus cannot hold these trees: the reason a wrong run prints holds the temporary path.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// The report's project: a non-citable `docs` kind and an `E2E` kind, so the walk has two roots.
const CONFIG: &str = "grund_config_version = 2\nproject_name = \"q\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.doc]\ncitable = false\nfolders = [\"docs\"]\n\n\
    [schema.kinds.E2E]\nfolders = [\"e2e/cases\"]\nindex = false\n";

const TWO_LINES: &str = "{alpha\n{beta\n";

/// One test's copy of the report's clean tree, removed on drop.
struct Tree {
    root: PathBuf,
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
            std::env::temp_dir().join(format!("grund-1271-ignore-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let tree = Self { root };
        tree.write("grund.toml", CONFIG);
        tree.write("docs/notes.md", "# Notes\n\nNothing to cite here.\n");
        for case in ["a", "b"] {
            tree.write(&format!("e2e/cases/{case}/command.args"), "check {repo}\n");
            tree.write(&format!("e2e/cases/{case}/expected.exit"), "0\n");
        }
        tree
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(path, contents).expect("write the file");
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
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Where each diagnostic of the run is located: `error: <path>: ` and `<path>:<line>: error: `.
fn locations(run: &Run) -> Vec<String> {
    format!("{}\n{}", run.stdout, run.stderr)
        .lines()
        .filter_map(|line| {
            for channel in ["error: ", "warning: "] {
                if let Some(rest) = line.strip_prefix(channel) {
                    return rest.split(": ").next().map(str::to_string);
                }
            }
            let (path, rest) = line.split_once(':')?;
            let (number, rest) = rest.split_once(": ")?;
            (number.parse::<u32>().is_ok()
                && (rest.starts_with("error:") || rest.starts_with("warning:")))
            .then(|| path.to_string())
        })
        .collect()
}

/// What every run over a malformed ignore file owes: exit `2`, every diagnostic at `file`, no
/// absolute path, and each malformed glob named exactly once.
fn assert_at_the_file(tree: &Tree, args: &[&str], file: &str, globs: &[&str]) {
    let run = tree.run(args);
    assert_eq!(run.code, 2, "{args:?}: {run:#?}");
    let found = locations(&run);
    assert!(!found.is_empty(), "{args:?}: nothing reported: {run:#?}");
    for location in &found {
        assert_eq!(
            location, file,
            "{args:?}: located away from the ignore file: {run:#?}"
        );
    }
    let output = format!("{}\n{}", run.stdout, run.stderr);
    let scratch = tree
        .root
        .file_name()
        .expect("a name")
        .to_string_lossy()
        .into_owned();
    assert!(
        !output.contains(&scratch),
        "{args:?}: an absolute path is printed: {run:#?}"
    );
    for glob in globs {
        let named = output.matches(&format!("'{glob}'")).count();
        assert_eq!(
            named, 1,
            "{args:?}: `{glob}` is named {named} times: {run:#?}"
        );
    }
}

/// The control: the clean tree passes in silence, so a diagnostic below is the ignore file's.
#[test]
fn the_clean_tree_passes_in_silence() {
    let tree = Tree::new("clean");
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 0, "{run:#?}");
    assert_eq!(locations(&run), Vec::<String>::new(), "{run:#?}");
}

/// The control: one malformed line is already reported at the file, relative, once.
#[test]
fn one_malformed_line_is_one_failure_at_the_ignore_file() {
    let tree = Tree::new("one-line");
    tree.write("e2e/.ignore", "{alpha\n");
    assert_at_the_file(&tree, &["check"], "e2e/.ignore", &["{alpha"]);
}

#[test]
fn two_malformed_lines_under_the_e2e_folder_are_reported_at_the_ignore_file() {
    let tree = Tree::new("e2e-two");
    tree.write("e2e/.ignore", TWO_LINES);
    for args in [&["check"][..], &["list", "--kind", "E2E"]] {
        assert_at_the_file(&tree, args, "e2e/.ignore", &["{alpha", "{beta"]);
    }
}

/// Every walk root reads a root ignore file, and the one failure is still one line per bad line.
#[test]
fn two_malformed_lines_in_a_root_ignore_file_are_reported_once_at_it() {
    for file in [".ignore", ".gitignore"] {
        let tree = Tree::new(&format!("root{file}"));
        tree.write(file, TWO_LINES);
        assert_at_the_file(&tree, &["check"], file, &["{alpha", "{beta"]);
    }
}

/// A run naming the ignore file holds its failure, as a run naming any failing file does.
#[test]
fn a_run_naming_the_ignore_file_holds_its_failure() {
    let tree = Tree::new("named");
    tree.write("e2e/.ignore", TWO_LINES);
    let run = tree.run(&["check", "e2e/.ignore"]);
    assert_eq!(run.code, 2, "{run:#?}");
    assert!(!run.stderr.contains("outside the report scope"), "{run:#?}");
}
