//! What the e2e cases cannot hold for #1152: `config-later-place` under `--format=json`, whose
//! message is longer than the cap a `tests/e2e` stderr line is held to, as the other version-2
//! window warnings' were (§FS-config-v2.checks.config-later-place), and a later `files` entry
//! that links out of the project root, whose target lies outside any fixture
//! (§FS-fields.forms.places.window).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const HEAD: &str = "grund_config_version = 2\nproject_name = \"later-place\"\n\n";
const CONST_A: &str = "## CONST-a: T\n\n### 1. 1\n";
const LATER: &str = ", loaded for now because it is not the row's first `files` entry; \
    correct it or take it off the row; this warning becomes an error in grund 0.23.0";

/// A scratch directory holding `project/`, the config root, beside what a link may reach.
struct Tree(PathBuf);

impl Tree {
    fn new(row: &str, files: &[(&str, &str)]) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1152-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let tree = Self(dir);
        let config = format!("{HEAD}{row}");
        tree.write("project/grund.toml", &config);
        for (path, contents) in files {
            tree.write(&format!("project/{path}"), contents);
        }
        tree
    }

    fn write(&self, path: &str, contents: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn project(&self) -> PathBuf {
        self.0.join("project")
    }

    fn run(&self, args: &[&str]) -> Output {
        run_in(&self.project(), args)
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run grund")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// §FS-config-v2.checks.config-later-place: one warning finding carrying the code, its `path`
/// the config file as a report path and its `line` the `form` key's, on stderr.
#[test]
#[ignore = "red until #1152 warns for a later entry with config-later-place; implement removes this"]
fn a_later_entry_is_one_located_warning_finding_under_json() {
    let tree = Tree::new(
        "[schema.kinds.CONST]\nfiles = [\"values/a.md\", \"values/absent.md\"]\nform = \"value\"\n",
        &[("values/a.md", CONST_A), ("cite.md", "See §CONST-a.1.\n")],
    );
    let out = tree.run(&["check", "--format=json"]);
    let message = format!(
        "grund.toml:6: value home for kind `CONST` does not exist: values/absent.md{LATER}"
    );
    let expected = format!(
        "{{\"severity\":\"warning\",\"path\":\"grund.toml\",\"line\":6,\
         \"code\":\"config-later-place\",\"message\":{},\"sites\":null,\"authority\":null}}\n",
        serde_json::to_string(&message).unwrap()
    );
    assert_eq!(stderr(&out), expected);
    assert_eq!(out.status.code(), Some(0));
}

/// §FS-fields.forms.places.window: a later `files` entry that links out of the project root
/// is followed as a link (§FS-config-v2.schema.places.boundaries.3) and loads through the
/// window, where the first entry of that shape is refused in the same words.
#[cfg(unix)]
#[test]
#[ignore = "red until #1152 warns for a later entry with config-later-place; implement removes this"]
fn a_later_file_that_links_out_of_the_root_warns() {
    let tree = Tree::new(
        "[schema.kinds.CONST]\nfiles = [\"values/a.md\", \"values/link.md\"]\nform = \"value\"\n",
        &[
            ("values/a.md", CONST_A),
            ("cite.md", "See §CONST-a.1 and §CONST-y.1.\n"),
        ],
    );
    tree.write("outside.md", "## CONST-y: T\n\n### 1. 1\n");
    std::os::unix::fs::symlink("../../outside.md", tree.project().join("values/link.md")).unwrap();
    let out = tree.run(&["check"]);
    assert_eq!(
        stderr(&out),
        format!(
            "warning: grund.toml:6: value home for kind `CONST` must normalize inside the project root: values/link.md{LATER}\n"
        )
    );
    assert_eq!(out.status.code(), Some(0));
}

/// §FS-fields.forms.places.first: the same link written first is refused, as it was.
#[cfg(unix)]
#[test]
fn a_first_file_that_links_out_of_the_root_is_refused() {
    let tree = Tree::new(
        "[schema.kinds.CONST]\nfiles = [\"values/link.md\", \"values/a.md\"]\nform = \"value\"\n",
        &[("values/a.md", CONST_A)],
    );
    tree.write("outside.md", "## CONST-y: T\n\n### 1. 1\n");
    std::os::unix::fs::symlink("../../outside.md", tree.project().join("values/link.md")).unwrap();
    let out = tree.run(&["check"]);
    assert_eq!(
        stderr(&out),
        "error: grund.toml:6: value home for kind `CONST` must normalize inside the project root: values/link.md\n"
    );
    assert_eq!(out.status.code(), Some(2));
}
