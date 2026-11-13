//! Black-box: an explicit path that is itself a directory link out of the project root is not
//! followed (§FS-scan.scope.links), and the empty-scan warning says so rather than blaming the
//! extensions it never consulted (§FS-check.2.2.2.6). A real directory whose walk refused such a
//! link below it names that link the same way (§FS-check.2.2.2.6.1). The controls hold what must
//! not move: a file below the link is read, a link inside the project is walked, an exclude
//! pattern is named ahead of an outward link, and a real directory with no file grund reads and
//! no outward link below it keeps the extensions wording (§FS-check.2.2.2.2). An outward link
//! whose target refuses the lookup takes the same warning; git carries no mode, so that case
//! builds its own tree and skips where mode 000 does not bar the user (root).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The words §FS-check.2.2.2.6 gives an outward link, after the path.
const LINK_WORDING: &str = "is a directory link out of the project root, and grund does not \
                            follow one (name a file below it, or move the target inside the project).";
/// The words §FS-check.2.2.2.2 gives a tree with no file of a read extension.
const EXTENSIONS_WORDING: &str = "matched grund's extensions";

/// `t/p` is a version-2 project, `t/outside` a tree beside it holding `a.md`. `p/x -> ../outside`
/// leaves the project, `p/y -> in` stays in it, and `p/bin` holds only `tool.txt`.
struct Tree {
    root: PathBuf,
    /// The directories set to mode 000, opened again before the tree is removed.
    locked: Vec<PathBuf>,
}

impl Tree {
    fn new(test: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1376-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let tree = Self {
            root,
            locked: Vec::new(),
        };
        tree.write(
            "p/grund.toml",
            "grund_config_version = 2\nproject_name = \"p\"\n\n[schema.kinds.FS]\n\
             folders = [\"docs/fs\"]\nindex = false\n",
        );
        tree.write(
            "p/docs/fs/FS-real.md",
            "# FS-real: Real\n\nSee \u{a7}FS-real.\n",
        );
        tree.write("p/in/b.md", "# B\n\nSee \u{a7}FS-nope.\n");
        tree.write("p/bin/tool.txt", "a tool\n");
        tree.write("outside/a.md", "# A\n\nSee \u{a7}FS-nope.\n");
        tree.link("p/x", "../outside");
        tree.link("p/y", "in");
        git_init(&tree.root.join("p"));
        tree
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(path, content).expect("write the fixture");
    }

    /// Creates the link `relative` pointing at `target`, written relative to the link.
    fn link(&self, relative: &str, target: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        symlink(target, path).expect("create the link");
    }

    /// §FS-check.2.2.2.6.1's own fixture: `p/real` holds nothing but `subx`, a link out of the
    /// project to a directory whose one file is a `.md` the walk would read.
    fn with_link_below(test: &str) -> Self {
        let tree = Self::new(test);
        tree.write("outside/sub/s.md", "# Notes\n\nSee \u{a7}FS-nope.\n");
        tree.link("p/real/subx", "../../outside/sub");
        tree
    }

    /// Sets the directory `relative` to mode 000 and says whether that bars this user: it does
    /// not bar root, and then every lookup below it answers and the test proves nothing.
    fn bar(&mut self, relative: &str) -> bool {
        let path = self.root.join(relative);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("set the mode");
        self.locked.push(path.clone());
        let barred = fs::read_dir(&path).is_err();
        if !barred {
            eprintln!("skipped: the mode does not bar this user, so every lookup answers");
        }
        barred
    }

    /// Runs `grund` from inside the project `p`.
    fn grund(&self, args: &[&str]) -> Run {
        self.grund_in("p", args)
    }

    /// Runs `grund` from the directory `relative` of the tree.
    fn grund_in(&self, relative: &str, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(self.root.join(relative))
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
        for path in &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct Run {
    shown: String,
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn report(&self) -> String {
        let Self {
            shown,
            code,
            stdout,
            stderr,
        } = self;
        format!("{shown}: exit {code}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    }

    /// Exit `0` and the one link warning naming `path` as written, never the extensions.
    fn assert_link_warning(&self, path: &str) {
        assert_eq!(self.code, 0, "{}", self.report());
        let expected = format!("warning: nothing to scan — `{path}` {LINK_WORDING}");
        assert!(
            self.stderr.lines().any(|line| line == expected),
            "expected the line\n{expected}\n{}",
            self.report()
        );
        assert!(
            !self.stderr.contains(EXTENSIONS_WORDING),
            "the extensions were blamed for a link the walk refused\n{}",
            self.report()
        );
    }

    /// Exit `0` and the one warning naming `link`, an outward link the walk refused below `path`,
    /// each as written; never the extensions (§FS-check.2.2.2.6.1).
    fn assert_link_below_warning(&self, path: &str, link: &str) {
        assert_eq!(self.code, 0, "{}", self.report());
        let expected = format!(
            "warning: nothing to scan — nothing under `{path}` was read: `{link}` {LINK_WORDING}"
        );
        assert!(
            self.stderr.lines().any(|line| line == expected),
            "expected the line\n{expected}\n{}",
            self.report()
        );
        assert!(
            !self.stderr.contains(EXTENSIONS_WORDING),
            "the extensions were blamed for a link the walk refused below the path\n{}",
            self.report()
        );
    }

    /// Exit `0` and §FS-check.2.2.2.2's extensions wording for `path`, naming no link.
    fn assert_extensions_warning(&self, path: &str) {
        assert_eq!(self.code, 0, "{}", self.report());
        let expected =
            format!("warning: nothing to scan — no files under `{path}` {EXTENSIONS_WORDING}");
        assert!(self.stderr.starts_with(&expected), "{}", self.report());
        assert!(!self.stderr.contains(LINK_WORDING), "{}", self.report());
    }

    /// Exit `1`, and the citation below the walked path is reported.
    fn assert_read(&self) {
        assert_eq!(self.code, 1, "{}", self.report());
        assert!(self.stdout.contains("FS-nope"), "{}", self.report());
    }

    /// Exit `2` with the walk's own lookup failure at `path`, and no empty-scan warning at all.
    fn assert_lookup_failure(&self, path: &str) {
        assert_eq!(self.code, 2, "{}", self.report());
        let expected = format!("error: {path}: unreadable symlink: permission denied");
        assert!(
            self.stderr.lines().any(|line| line.starts_with(&expected)),
            "expected a line starting\n{expected}\n{}",
            self.report()
        );
        assert!(
            !self.stderr.contains("nothing to scan"),
            "{}",
            self.report()
        );
    }
}

#[test]
fn an_outward_directory_link_is_named_as_the_reason() {
    let tree = Tree::new("bare");
    tree.grund(&["check", "x"]).assert_link_warning("x");
}

#[test]
fn every_spelling_of_the_link_is_named_as_written() {
    let tree = Tree::new("spellings");
    for path in ["x/", "./x"] {
        tree.grund(&["check", path]).assert_link_warning(path);
    }
}

#[test]
fn the_json_warning_carries_the_same_reason() {
    let tree = Tree::new("json");
    let run = tree.grund(&["check", "x", "--format=json"]);
    assert_eq!(run.code, 0, "{}", run.report());
    assert!(
        run.stderr.contains("\"empty-scan\"") && run.stderr.contains(LINK_WORDING),
        "{}",
        run.report()
    );
    assert!(!run.stderr.contains(EXTENSIONS_WORDING), "{}", run.report());
}

/// The control: the file below the link is read when it is named, so its extension is not why.
#[test]
fn a_file_below_the_link_is_read() {
    let tree = Tree::new("file");
    tree.grund(&["check", "x/a.md"]).assert_read();
}

/// The control: a directory link that stays inside the project is walked.
#[test]
fn an_inward_directory_link_is_walked() {
    let tree = Tree::new("inward");
    tree.grund(&["check", "y"]).assert_read();
}

/// The control: a real directory with no file grund reads, and no outward link below it, keeps
/// the extensions wording (§FS-check.2.2.2.6.1).
#[test]
fn a_directory_with_no_read_extension_keeps_the_extensions_wording() {
    let tree = Tree::new("noext");
    tree.grund(&["check", "bin"])
        .assert_extensions_warning("bin");
}

/// An outward link whose target refuses the lookup is pruned at the same gate, in silence, as one
/// whose target opens (§FS-scan.scope.homes.lookup.outward), so it takes the same warning. The
/// controls share the barred target and differ in one term of that gate each: `z.md` has a name
/// the walk reads as a file, and `zin` stays inside the project. Both reach the walk's lookup
/// failure at the root.
#[test]
fn an_outward_link_whose_target_refuses_the_lookup_is_named_as_the_reason() {
    let mut tree = Tree::new("unlooked");
    tree.write("locked/inner/a.md", "# A\n\nSee \u{a7}FS-nope.\n");
    tree.write("p/lockedin/inner/a.md", "# A\n\nSee \u{a7}FS-nope.\n");
    tree.link("p/z", "../locked/inner");
    tree.link("p/z.md", "../locked/inner");
    tree.link("p/zin", "lockedin/inner");
    if !(tree.bar("locked") && tree.bar("p/lockedin")) {
        return;
    }
    tree.grund(&["check", "z"]).assert_link_warning("z");
    tree.grund(&["check", "z.md"]).assert_lookup_failure("z.md");
    tree.grund(&["check", "zin"]).assert_lookup_failure("zin");
}

/// §FS-check.2.2.2.6.1: the walk refused `real/subx` below `real` at the outward-link gate, so
/// the warning names that link, not the extensions `s.md` behind it would have passed.
#[test]
fn an_outward_link_below_the_path_is_named_as_the_reason() {
    let tree = Tree::with_link_below("below");
    tree.grund(&["check", "real"])
        .assert_link_below_warning("real", "real/subx");
}

/// §FS-check.2.2.2.6.1: the link is spelled from the path as the user wrote it.
#[test]
fn every_spelling_of_the_path_spells_the_link_below_it() {
    let tree = Tree::with_link_below("below-spellings");
    tree.grund(&["check", "real/"])
        .assert_link_below_warning("real/", "real/subx");
    tree.grund(&["check", "./real"])
        .assert_link_below_warning("./real", "./real/subx");
}

/// §FS-check.2.2.2.6.1: a link met deeper down is named by its path from the one handed.
#[test]
fn an_outward_link_deeper_below_the_path_is_named_by_its_path() {
    let tree = Tree::new("below-deeper");
    tree.write("outside/sub/s.md", "# Notes\n\nSee \u{a7}FS-nope.\n");
    tree.link("p/real/a/subx", "../../../outside/sub");
    tree.grund(&["check", "real"])
        .assert_link_below_warning("real", "real/a/subx");
}

/// §FS-check.2.2.2.6.1: a file of an extension grund does not read, beside the link, does not
/// turn the message back to the extensions; the link is the one place the run never looked.
#[test]
fn a_file_of_an_unread_extension_beside_the_link_does_not_hide_it() {
    let tree = Tree::with_link_below("below-beside-txt");
    tree.write("p/real/tool.txt", "a tool\n");
    tree.grund(&["check", "real"])
        .assert_link_below_warning("real", "real/subx");
}

/// §FS-check.2.2.2.6.1: grund does not look behind a link it does not follow, so a target with
/// no file it would read takes the same message.
#[test]
fn an_outward_link_whose_target_holds_no_read_file_is_still_named() {
    let tree = Tree::new("below-unread-target");
    tree.write("outside/none/tool.txt", "a tool\n");
    tree.link("p/real/subx", "../../outside/none");
    tree.grund(&["check", "real"])
        .assert_link_below_warning("real", "real/subx");
}

/// §FS-check.2.2.2.6.1, §FS-errors.4.1: of several outward links, the first in path order is
/// named, whatever order the directory lists them in, so two runs name the same link. They are
/// made in path order, which a directory that lists the newest first reverses.
#[test]
fn of_several_outward_links_below_the_path_the_first_in_path_order_is_named() {
    let tree = Tree::new("below-several");
    tree.write("outside/sub/s.md", "# Notes\n\nSee \u{a7}FS-nope.\n");
    for name in ["a", "b", "c", "d", "e"] {
        tree.link(&format!("p/real/{name}"), "../../outside/sub");
    }
    tree.grund(&["check", "real"])
        .assert_link_below_warning("real", "real/a");
}

/// §FS-check.2.2.2.6.1: the gate is the member's root in a member run, so a link from one member
/// into another leaves it and is named, as §FS-check.2.2.2.6 names `m/x -> ../n/y`.
#[test]
fn a_link_below_the_path_from_one_member_into_another_is_named() {
    let tree = Tree::new("below-member");
    tree.write(
        "w/grund.toml",
        "grund_config_version = 2\nproject_name = \"w\"\n\n[workspace]\nmembers = [\"m\", \"n\"]\n",
    );
    tree.write(
        "w/m/grund.toml",
        "grund_config_version = 2\nproject_name = \"m\"\n",
    );
    tree.write(
        "w/n/grund.toml",
        "grund_config_version = 2\nproject_name = \"n\"\n",
    );
    tree.write("w/n/y/a.md", "# A\n\nSee \u{a7}FS-nope.\n");
    tree.link("w/m/real/x", "../../n/y");
    git_init(&tree.root.join("w"));
    tree.grund_in("w/m", &["check", "real"])
        .assert_link_below_warning("real", "real/x");
}

/// §FS-check.2.2.2.6.1, §FS-workspace.5.3: a member whose own scan read nothing because its only
/// entry is an outward link is named by its root from the report base, beside that link.
#[test]
fn a_workspace_run_names_the_link_below_a_member_that_read_nothing() {
    let tree = Tree::new("below-workspace-root");
    tree.write(
        "w/grund.toml",
        "grund_config_version = 2\nproject_name = \"w\"\n\n[workspace]\nmembers = [\"m\"]\n",
    );
    tree.write("w/a.md", "# A\n");
    tree.write(
        "w/m/grund.toml",
        "grund_config_version = 2\nproject_name = \"m\"\n",
    );
    tree.write("outside/sub/s.md", "# Notes\n\nSee \u{a7}FS-nope.\n");
    tree.link("w/m/subx", "../../outside/sub");
    git_init(&tree.root.join("w"));
    tree.grund_in("w", &["check"])
        .assert_link_below_warning("m", "m/subx");
}

/// §FS-check.2.2.2.6.1: the JSON warning's `message` carries the same text.
#[test]
fn the_json_warning_names_the_link_below_the_path() {
    let tree = Tree::with_link_below("below-json");
    let run = tree.grund(&["check", "real", "--format=json"]);
    assert_eq!(run.code, 0, "{}", run.report());
    let message =
        format!("nothing to scan — nothing under `real` was read: `real/subx` {LINK_WORDING}");
    assert!(
        run.stderr.contains("\"empty-scan\"") && run.stderr.contains(&message),
        "expected an empty-scan object whose message is\n{message}\n{}",
        run.report()
    );
    assert!(!run.stderr.contains(EXTENSIONS_WORDING), "{}", run.report());
}

/// The control: an exclude pattern that pruned a file the walk would have read is the cause the
/// run proved, so it is named ahead of an outward link beside it (§FS-check.2.2.2.6.1).
#[test]
fn an_exclude_pattern_is_named_ahead_of_an_outward_link_below_the_path() {
    let tree = Tree::with_link_below("below-pattern");
    tree.write(
        "p/grund.toml",
        "grund_config_version = 2\nproject_name = \"p\"\n\n[schema.sources]\nexclude = [\"gen\"]\n\n\
         [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n",
    );
    tree.write("p/real/gen/g.md", "# G\n\nSee \u{a7}FS-nope.\n");
    let run = tree.grund(&["check", "real"]);
    assert_eq!(run.code, 0, "{}", run.report());
    let expected = "warning: nothing to scan — every file under `real` that grund would read is \
                    excluded by the [schema.sources] exclude pattern `gen`.";
    assert!(
        run.stderr.lines().any(|line| line == expected),
        "expected the line\n{expected}\n{}",
        run.report()
    );
    assert!(!run.stderr.contains(LINK_WORDING), "{}", run.report());
}

/// The control: a directory link below the path that stays inside the project is walked, so a
/// directory whose only link reaches `bin` keeps the extensions wording (§FS-check.2.2.2.6.1).
#[test]
fn an_inward_link_below_the_path_keeps_the_extensions_wording() {
    let tree = Tree::new("below-inward");
    tree.link("p/real/inbin", "../bin");
    tree.grund(&["check", "real"])
        .assert_extensions_warning("real");
}

/// Makes `dir` a git repository, as `Tree::new` makes `p` one.
fn git_init(dir: &Path) {
    let status = Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(dir)
        .status()
        .expect("run git init");
    assert!(status.success(), "git init failed");
}
