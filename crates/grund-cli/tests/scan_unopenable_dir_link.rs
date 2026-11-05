//! Black-box: a directory link whose target cannot be opened fails at the link, once, and never
//! at a walk root that met it (§FS-scan.scope.links.unresolvable.unopenable). A link out of the
//! project is pruned in silence whether or not its target can be opened
//! (§FS-scan.scope.links.owed.fence). What is judged is the `path` of every `io` diagnostic in
//! `--format json`, and the exit code, not the wording. Git carries no mode, so the e2e corpus
//! cannot hold these trees, and every test skips where mode 000 does not bar the user (root).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs\"]\n";

/// A version-1 config whose E2E folder is `e2e`, the shape the ticket's last run used.
const V1_CONFIG: &str = "grund_config_version = 1\nproject_name = \"p\"\n\n\
    [[kinds]]\nkind = \"E2E\"\nfolder = \"e2e\"\ntitle = \"Executable user scenarios\"\n";

/// Where no walk root may ever be the location: the config root and the home.
const WALK_ROOTS: [&str; 2] = [".", "docs"];

/// A clean v2 project at `<base>/p`, its one declaration cited, so a run over it reports nothing
/// and exits 0 until a test adds the directory nobody can open. `<base>` is outside the project,
/// so a link to `../../outside` leaves it.
struct Tree {
    base: PathBuf,
    locked: Vec<PathBuf>,
}

struct Run {
    code: i32,
    stderr: String,
    /// `(severity, path)` of every `io` diagnostic, in the order printed.
    io: Vec<(String, String)>,
}

impl Run {
    fn paths(&self) -> Vec<&str> {
        let mut paths: Vec<&str> = self.io.iter().map(|(_, path)| path.as_str()).collect();
        paths.sort_unstable();
        paths
    }

    fn names_a_walk_root(&self) -> bool {
        self.io
            .iter()
            .any(|(_, path)| WALK_ROOTS.contains(&path.as_str()))
    }
}

impl Tree {
    fn new(test: &str) -> Self {
        let base = std::env::temp_dir().join(format!("grund-1266-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let project = base.join("p");
        fs::create_dir_all(project.join("docs")).expect("create the home");
        fs::write(project.join("grund.toml"), CONFIG).expect("write the config");
        fs::write(project.join("docs/FS-b.md"), "# FS-b: B\n\nPlain text.\n")
            .expect("write the declaration");
        fs::write(project.join("notes.md"), "# Notes\n\nSee \u{a7}FS-b.\n")
            .expect("write the notes");
        Self {
            base,
            locked: Vec::new(),
        }
    }

    /// Creates the directory `rel`, relative to the base.
    fn dir(&self, rel: &str) {
        fs::create_dir_all(self.base.join(rel)).expect("create the directory");
    }

    /// Creates the link `p/<link>` pointing at `target`, written relative to the link.
    fn link(&self, link: &str, target: &str) {
        symlink(target, self.base.join("p").join(link)).expect("create the link");
    }

    /// Bars the directory `rel` and says whether the mode bars this user: it does not bar root,
    /// and then nothing here fails to open and the test proves nothing.
    fn bar(&mut self, rel: &str) -> bool {
        let path = self.base.join(rel);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("set the mode");
        self.locked.push(path.clone());
        let barred = fs::read_dir(&path).is_err();
        if !barred {
            eprintln!("skipped: the mode does not bar this user, so nothing here fails to open");
        }
        barred
    }

    /// `grund check <args> --format json` from the project root.
    fn check(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .args(args)
            .args(["--format", "json"])
            .env("NO_COLOR", "1")
            .current_dir(self.base.join("p"))
            .output()
            .expect("run grund");
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let io = stderr
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|record| record["code"] == "io")
            .map(|record| {
                let field = |name: &str| record[name].as_str().unwrap_or("(none)").to_string();
                (field("severity"), field("path"))
            })
            .collect();
        Run {
            code: output.status.code().expect("exited with a status"),
            stderr,
            io,
        }
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

/// The control, green today: the same unreadable directory standing at `docs/ext` with no link
/// is reported once, at its own path. Every link below is to be located as this is.
#[test]
fn an_unreadable_directory_is_reported_once_at_its_own_path() {
    let mut tree = Tree::new("control");
    tree.dir("p/docs/ext");
    if !tree.bar("p/docs/ext") {
        return;
    }
    let run = tree.check(&[]);
    assert_eq!(
        (run.code, run.paths()),
        (2, vec!["docs/ext"]),
        "{}",
        run.stderr
    );
}

/// `docs/ext -> ../real` with `real` unreadable: the link is one failure, met by the walk from
/// the config root and the walk from the home `docs`, and `real` is a second one of its own.
#[test]
fn an_inward_link_is_reported_at_the_link_and_its_target_once_each() {
    let mut tree = Tree::new("inward");
    tree.dir("p/real");
    tree.link("docs/ext", "../real");
    if !tree.bar("p/real") {
        return;
    }
    let run = tree.check(&[]);
    assert_eq!(
        (run.code, run.paths()),
        (2, vec!["docs/ext", "real"]),
        "{}",
        run.stderr
    );
}

/// The link's own path as the scope: the run still fails, and it names no walk root, in scope or
/// out of it.
#[test]
fn an_inward_link_scoped_to_its_path_names_no_walk_root() {
    let mut tree = Tree::new("inward-scoped");
    tree.dir("p/real");
    tree.link("docs/ext", "../real");
    if !tree.bar("p/real") {
        return;
    }
    let run = tree.check(&["docs/ext"]);
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(
        !run.names_a_walk_root(),
        "a walk root named in:\n{}",
        run.stderr
    );
    assert!(
        run.io
            .iter()
            .any(|(severity, path)| severity == "error"
                && ["docs/ext", "real"].contains(&path.as_str())),
        "no in-scope failure in:\n{}",
        run.stderr
    );
}

/// Two such links in one directory are two places, each named, not one line for the directory.
#[test]
fn several_links_in_one_directory_are_each_reported_at_their_own_path() {
    let mut tree = Tree::new("several");
    tree.dir("p/ra");
    tree.dir("p/rb");
    tree.link("docs/a", "../ra");
    tree.link("docs/b", "../rb");
    if !tree.bar("p/ra") || !tree.bar("p/rb") {
        return;
    }
    let run = tree.check(&[]);
    assert_eq!(
        (run.code, run.paths()),
        (2, vec!["docs/a", "docs/b", "ra", "rb"]),
        "{}",
        run.stderr
    );
}

/// `docs/ext -> ../../outside` with `outside` unreadable: the fence prunes the link on its
/// canonical target, as it would a readable one, so neither a whole-project run nor one over the
/// home reports anything, and both exit 0.
#[test]
#[ignore = "red until #1266 asks the project-root fence of a link whose target cannot be opened; implement removes this"]
fn an_outward_link_whose_target_cannot_be_opened_is_pruned_in_silence() {
    let mut tree = Tree::new("outward");
    tree.dir("outside");
    tree.link("docs/ext", "../../outside");
    if !tree.bar("outside") {
        return;
    }
    for args in [&[][..], &["docs"]] {
        let run = tree.check(args);
        assert_eq!(
            (run.code, run.paths()),
            (0, vec![]),
            "grund check {args:?}:\n{}",
            run.stderr
        );
    }
}

/// The outward link named as the scope is the root gate's to judge; whatever that says of the
/// link, no walk root is where anything failed.
#[test]
fn an_outward_link_scoped_to_its_path_names_no_walk_root() {
    let mut tree = Tree::new("outward-scoped");
    tree.dir("outside");
    tree.link("docs/ext", "../../outside");
    if !tree.bar("outside") {
        return;
    }
    let run = tree.check(&["docs/ext"]);
    assert!(
        !run.names_a_walk_root(),
        "a walk root named in:\n{}",
        run.stderr
    );
}

/// A version-1 E2E folder that is itself a link out of the project: the walk from the config
/// root names no `.`. The E2E pass's own `e2e/cases` line is #1253's, and is not judged here.
#[test]
fn a_version_1_e2e_folder_linking_out_names_no_config_root() {
    let mut tree = Tree::new("v1-e2e");
    fs::write(tree.base.join("p/grund.toml"), V1_CONFIG).expect("write the config");
    tree.dir("outside/cases");
    tree.link("e2e", "../outside");
    if !tree.bar("outside") {
        return;
    }
    let run = tree.check(&["--full"]);
    assert!(
        !run.io.iter().any(|(_, path)| path == "."),
        "the config root named in:\n{}",
        run.stderr
    );
}
