//! Black-box: a symlink loop's reason names the ancestor physically, so the routes to one link have
//! one reason and print one line (§FS-check.2.4.2), exit `2` as a loop always is
//! (§FS-scan.scope.links.unresolvable). The tests marked `ignore` are red until #1293; the rest are
//! its guards and hold today: a fix must not merge two links or lose the config root's wording.
//!
//! Where a run has a second entry name for the link (`grund check b`, `include = ["a", "b"]`), one
//! line is §FS-check.2.4.2's other half, which #1288 brings, so these pin only the word.
#![cfg(unix)]

use grund_core::{EmbeddingRequest, embedding_call};
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;

const REMEDY: &str = "make the path readable, or keep it out of the scan";
const V2: &str = "grund_config_version = 2\nproject_name = \"repro\"\n\n[schema.kinds.FS]\n\
                  folders = [\"docs\"]\nindex = false\n";

/// The line a loop at `link` naming `ancestor` prints.
fn named(link: &str, ancestor: &str) -> String {
    format!(
        "error: {link}: symlink loop: the target is the ancestor directory {ancestor}; {REMEDY}"
    )
}

/// The line a loop at `link` whose ancestor has no name in the report prints.
fn contains(link: &str) -> String {
    format!("error: {link}: symlink loop: the target contains the link; {REMEDY}")
}

/// A project at `<base>/proj`, with `<base>` free for what lies outside the config root.
struct Tree {
    base: PathBuf,
}

impl Tree {
    fn new(test: &str, config: &str) -> Self {
        let base = std::env::temp_dir().join(format!("grund-1293-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let tree = Self { base };
        tree.dirs(&["docs"]);
        fs::write(tree.root().join("grund.toml"), config).expect("write the config");
        fs::write(
            tree.root().join("docs/FS-one.md"),
            "# FS-one: One\n\nBody.\n",
        )
        .expect("write");
        tree
    }

    /// `a/sub/up -> ..` and `b -> a`: one link, one directory with two names.
    fn reports_tree(test: &str) -> Self {
        let tree = Self::new(test, V2);
        tree.dirs(&["a/sub"]);
        tree.link("a/sub/up", "..");
        tree.link("b", "a");
        tree
    }

    fn root(&self) -> PathBuf {
        self.base.join("proj")
    }

    fn dirs(&self, rels: &[&str]) {
        for rel in rels {
            fs::create_dir_all(self.root().join(rel)).expect("create the directory");
        }
    }

    /// Creates `link`, relative to the project, pointing at `target` as written.
    fn link(&self, link: &str, target: &str) {
        symlink(target, self.root().join(link)).expect("create the link");
    }

    fn grund(&self, args: &[&str]) -> Run {
        self.grund_in(self.root(), args)
    }

    fn grund_in(&self, dir: PathBuf, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(dir)
            .output()
            .expect("run grund");
        Run {
            shown: format!("grund {}", args.join(" ")),
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Asserts each command exits `2` and prints exactly these failure lines.
    fn assert_prints(&self, wants: &[(&[&str], Vec<String>)]) {
        for (args, lines) in wants {
            self.grund(args).assert_lines(lines);
        }
    }

    /// Asserts each command exits `2` and every failure line it prints names ancestor `a`.
    fn assert_word_is_a(&self, commands: &[&[&str]]) {
        for args in commands {
            let run = self.grund(args);
            let lines = run.failure_lines();
            assert_eq!(run.code, 2, "{}", run.report());
            assert!(!lines.is_empty(), "{}", run.report());
            let word = format!("the ancestor directory a; {REMEDY}");
            let others: Vec<_> = lines.iter().filter(|line| !line.ends_with(&word)).collect();
            assert!(
                others.is_empty(),
                "named otherwise: {others:#?}\n{}",
                run.report()
            );
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

struct Run {
    shown: String,
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    /// The scan failures a run reports, from either stream.
    fn failure_lines(&self) -> Vec<&str> {
        let lines = self.stdout.lines().chain(self.stderr.lines());
        lines.filter(|line| line.starts_with("error: ")).collect()
    }

    /// `(path, message)` of every `io` record of a `--format json` run.
    fn io(&self) -> Vec<(String, String)> {
        let lines = self.stdout.lines().chain(self.stderr.lines());
        lines
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|record| record["code"] == "io")
            .map(|record| {
                let field = |name: &str| record[name].as_str().unwrap_or("(none)").to_string();
                (field("path"), field("message"))
            })
            .collect()
    }

    fn assert_lines(&self, lines: &[String]) {
        assert_eq!(self.failure_lines(), lines, "{}", self.report());
        assert_eq!(self.code, 2, "{}", self.report());
    }

    fn report(&self) -> String {
        let Self {
            shown,
            code,
            stdout,
            stderr,
        } = self;
        format!("{shown}: exit {code}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    }
}

#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn the_reports_tree_is_one_line_from_check_and_list() {
    let tree = Tree::reports_tree("surfaces");
    tree.assert_prints(&[
        (&["check"], vec![named("a/sub/up", "a")]),
        (&["list"], vec![named("a/sub/up", "a")]),
    ]);
}

#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn the_reports_tree_is_one_line_from_fmt_check() {
    let tree = Tree::reports_tree("fmt");
    let want = named("a/sub/up", "a").replacen("error: ", "error: nothing was rewritten: ", 1);
    tree.assert_prints(&[(&["fmt", "--check"], vec![want])]);
}

#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn the_reports_tree_is_one_io_entry_in_check_json() {
    let tree = Tree::reports_tree("json");
    let run = tree.grund(&["check", "--format", "json"]);
    let message = format!("symlink loop: the target is the ancestor directory a; {REMEDY}");
    assert_eq!(
        run.io(),
        [("a/sub/up".to_string(), message)],
        "{}",
        run.report()
    );
    assert_eq!(run.code, 2, "{}", run.report());
}

#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn the_reports_tree_is_one_entry_in_the_embedding_list() {
    let tree = Tree::reports_tree("embedding");
    let envelope = embedding_call(EmbeddingRequest {
        operation: "list_ids".into(),
        root: tree.root(),
        explicit: true,
        args: Vec::new(),
        options: json!({}),
    });
    let errors = envelope["result"]["scan_errors"]
        .as_array()
        .unwrap_or_else(|| panic!("no scan_errors list in {envelope}"));
    let paths: Vec<_> = errors.iter().map(|error| error["path"].as_str()).collect();
    assert_eq!(paths, [Some("a/sub/up")], "{envelope}");
}

/// The word is the whole of this; the two names `check b` prints are #1288's (see the file's head).
#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn a_run_narrowed_through_either_name_names_the_physical_ancestor() {
    let tree = Tree::reports_tree("narrowed");
    tree.assert_word_is_a(&[&["check", "b"], &["check", "b/sub"], &["check", "b/sub/up"]]);
}

/// A narrowed walk root below the ancestor reads `contains the link` today, beside the two names.
#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn a_run_narrowed_to_the_links_own_name_is_one_line() {
    let tree = Tree::reports_tree("own-name");
    let one = || vec![named("a/sub/up", "a")];
    tree.assert_prints(&[(&["check", "a"], one()), (&["check", "a/sub"], one())]);
}

#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn two_included_names_for_one_directory_name_the_physical_ancestor() {
    let config = "grund_config_version = 1\n\n[scan]\ninclude = [\"a\", \"b\", \"docs\"]\n";
    let tree = Tree::new("include", config);
    tree.dirs(&["a/sub"]);
    tree.link("a/sub/up", "..");
    tree.link("b", "a");
    tree.assert_word_is_a(&[&["check"]]);
}

/// One name, and still two reasons today: the walk root below the ancestor has no word for it.
#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn overlapping_included_roots_are_one_line() {
    let config = "grund_config_version = 1\n\n[scan]\ninclude = [\"a/sub\", \"a\", \"docs\"]\n";
    let tree = Tree::new("overlap", config);
    tree.dirs(&["a/sub"]);
    tree.link("a/sub/up", "..");
    tree.assert_prints(&[(&["check"], vec![named("a/sub/up", "a")])]);
}

/// The one word that moves with no second name in the tree: the overlap above needs it, because a
/// reason cannot depend on which walk root met the link.
#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn a_target_above_the_walk_root_and_below_the_config_root_is_named() {
    let config = "grund_config_version = 1\n\n[scan]\ninclude = [\"a/x\", \"docs\"]\n";
    let tree = Tree::new("above-walk-root", config);
    tree.dirs(&["a/x"]);
    tree.link("a/x/up", "..");
    tree.assert_prints(&[(&["check"], vec![named("a/x/up", "a")])]);
}

/// The second name can be a link on the way, with no link at the root: `a/up` is `b`.
#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn a_link_on_the_way_does_not_name_the_ancestor() {
    let tree = Tree::new("on-the-way", V2);
    tree.dirs(&["a", "b"]);
    tree.link("a/up", "../b");
    tree.link("b/back", "../a/up");
    tree.assert_prints(&[(&["check"], vec![named("b/back", "b")])]);
}

#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn a_two_hop_loop_under_an_alias_is_one_line_per_link() {
    let tree = Tree::new("two-hop", V2);
    tree.dirs(&["real/a", "real/b"]);
    tree.link("real/a/link", "../b");
    tree.link("real/b/link", "../a");
    tree.link("alias", "real");
    tree.assert_prints(&[(
        &["check"],
        vec![
            named("real/a/link", "real/b"),
            named("real/b/link", "real/a"),
        ],
    )]);
}

/// A config root spelled through a link has no physical prefix to strip, and the word stays `a`.
#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn a_config_root_reached_through_a_link_still_names_the_ancestor_relatively() {
    let tree = Tree::reports_tree("root-link");
    symlink("proj", tree.base.join("plink")).expect("create the link");
    let run = tree.grund_in(tree.base.clone(), &["check", "plink"]);
    run.assert_lines(&[named("a/sub/up", "a")]);
}

/// The report base decides how the one location is spelled, and when it has no name at all.
#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn the_report_base_spells_the_one_location() {
    let tree = Tree::reports_tree("path-base");
    let args: &[&str] = &["check", "--path-base=invocation"];
    let inside = tree.grund_in(tree.root().join("a"), args);
    inside.assert_lines(&[contains("sub/up")]);
    let beside = tree.grund_in(tree.root().join("docs"), args);
    beside.assert_lines(&[named("../a/sub/up", "../a")]);
}

/// Only the count: the word a member's frame gives its ancestor is not this ticket's to settle.
#[test]
#[ignore = "red until #1293 names a loop's ancestor physically; implement removes this"]
fn a_workspace_root_prints_one_line_for_a_members_loop() {
    let member = "grund_config_version = 2\nproject_name = \"m\"\n\n[schema.kinds.FS]\n\
                  folders = [\"spec\"]\nindex = false\n";
    let tree = Tree::new(
        "workspace",
        &format!("{V2}\n[workspace]\nmembers = [\"m\"]\n"),
    );
    tree.dirs(&["m/spec", "m/a/sub"]);
    fs::write(tree.root().join("m/grund.toml"), member).expect("write the member's config");
    tree.link("m/a/sub/up", "..");
    tree.link("m/b", "a");
    let run = tree.grund(&["check"]);
    let lines = run.failure_lines();
    assert_eq!(lines.len(), 1, "{}", run.report());
    assert!(
        lines[0].starts_with("error: m/a/sub/up: symlink loop:"),
        "{}",
        run.report()
    );
    assert_eq!(run.code, 2, "{}", run.report());
}

/// Control: without `b` the loop already prints one line, so a fix that silences it fails here.
#[test]
fn one_name_for_the_directory_stays_one_line() {
    let tree = Tree::new("control-one", V2);
    tree.dirs(&["a/sub"]);
    tree.link("a/sub/up", "..");
    let one = || vec![named("a/sub/up", "a")];
    tree.assert_prints(&[(&["check"], one()), (&["list"], one())]);
}

/// Control: a fix that merges every loop into one line fails here.
#[test]
fn two_links_that_loop_stay_two_lines() {
    let tree = Tree::new("control-two", V2);
    tree.dirs(&["a/sub", "z/sub"]);
    tree.link("a/sub/up", "..");
    tree.link("z/sub/up", "..");
    let two = || vec![named("a/sub/up", "a"), named("z/sub/up", "z")];
    tree.assert_prints(&[(&["check"], two()), (&["list"], two())]);
}

/// The config root has no name in the report, on either route and in a narrowed run alike.
#[test]
fn a_target_at_the_config_root_keeps_its_wording() {
    let tree = Tree::new("config-root", V2);
    tree.dirs(&["a/sub"]);
    tree.link("a/sub/up", "../..");
    tree.link("b", "a");
    tree.link("top", ".");
    tree.assert_prints(&[(&["check"], vec![contains("a/sub/up"), contains("top")])]);
    let run = tree.grund(&["check", "b"]);
    assert!(
        run.failure_lines().contains(&contains("a/sub/up").as_str()),
        "{}",
        run.report()
    );
    let word = format!("the target contains the link; {REMEDY}");
    assert!(
        run.failure_lines().iter().all(|line| line.ends_with(&word)),
        "{}",
        run.report()
    );
    assert_eq!(run.code, 2, "{}", run.report());
}

/// Outside the config root the walk reaches the ancestor under one name only, because a link out
/// is fenced, and that name stays relative: a report does not grow an absolute path (§FS-errors.4).
#[test]
fn an_ancestor_outside_the_config_root_keeps_its_relative_word() {
    let config =
        "grund_config_version = 1\n\n[scan]\ninclude = [\"../outer\", \"../alias\", \"docs\"]\n";
    let tree = Tree::new("outside", config);
    fs::create_dir_all(tree.base.join("outer/sub")).expect("create the outside tree");
    symlink("..", tree.base.join("outer/sub/up")).expect("create the link");
    symlink("outer", tree.base.join("alias")).expect("create the link");
    tree.link("l1", "../outer");
    tree.link("l2", "../outer");
    tree.assert_prints(&[(&["check"], vec![named("../outer/sub/up", "../outer")])]);
}
