//! A numbered, named or relative heading that follows a nested declaration in Markdown, back
//! inside the enclosing declaration's body, is a coordinate of the innermost declaration whose
//! body holds it (§FS-declarations.section.map.nested, §FS-declarations.section.map,
//! §FS-declarations.body). `show` prints it, `--toc` lists it, a citation of it checks clean,
//! and it is a claimant like any other in that body: a path repeated across the nested
//! declaration is a duplicate (§FS-declarations.checks.duplicate-section.2), and an unmarked
//! heading after it is suggested the next unused sibling
//! (§FS-declarations.checks.unmarked-heading.3). The verdicts this moves are
//! §DF-section-after-nested-declaration. Issue #1359 is the report's shape: `check` told the
//! author to number the heading as `AR-ci.1`, and once numbered nothing resolved it.
//!
//! These are not goldens because a golden has no skip, and every test here but the control is
//! red until the scanner records the heading on its owner. The trees are built in the temp
//! directory, so run them with a `TMPDIR` that no git tree or grund config encloses.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const CONFIG: &str = "grund_config_version = 2\n\n[schema]\nheading_depth = \"must\"\n\n\
                      [schema.kinds.AR]\nfolders = [\"docs\"]\nindex = false\n";

/// The report's file: a plain heading after the nested `AR-matrix`, back in `AR-ci`'s body.
const CI_PLAIN: &str = "# AR-ci: CI\n\nThe CI lead.\n\n## AR-matrix: Matrix\n\n\
                        The matrix body.\n\n## Pull-request workflows\n\n\
                        The pull-request workflows body.\n";

/// The same file with the heading numbered as `check` advised.
const CI_NUMBERED: &str = "# AR-ci: CI\n\nThe CI lead.\n\n## AR-matrix: Matrix\n\n\
                           The matrix body.\n\n## 1. Pull-request workflows\n\n\
                           The pull-request workflows body.\n";

/// Every declaration of [`CI_NUMBERED`] and the advised coordinate, cited from outside the home.
const CI_NOTES: &str = "The notes cite \u{a7}AR-ci, \u{a7}AR-ci.1 and \u{a7}AR-matrix.\n";

/// A scratch project removed on drop; `name` keeps concurrent tests apart.
struct Tree(PathBuf);

impl Tree {
    /// The config, `notes.md` holding `notes` at the root, and each `(path, text)` under `docs/`.
    fn new(name: &str, docs: &[(&str, &str)], notes: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1359-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).expect("create the tree");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        fs::write(root.join("notes.md"), notes).expect("write the notes");
        let tree = Self(root);
        for (path, text) in docs {
            tree.write(path, text);
        }
        tree
    }

    fn write(&self, path: &str, text: &str) {
        fs::write(self.0.join("docs").join(path), text).expect("write a document");
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund")
    }

    /// `grund <args>` prints exactly `stdout`, writes nothing to stderr, and exits `code`.
    fn assert_prints(&self, args: &[&str], stdout: &str, code: i32) {
        let output = self.run(args);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            stdout,
            "stdout of grund {args:?}; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            "",
            "grund {args:?}"
        );
        assert_eq!(output.status.code(), Some(code), "grund {args:?}");
    }

    /// `grund <args>` prints nothing to stdout, exactly `stderr` to stderr, and exits 1.
    fn assert_refuses(&self, args: &[&str], stderr: &str) {
        let output = self.run(args);
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            stderr,
            "grund {args:?}"
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "",
            "grund {args:?}"
        );
        assert_eq!(output.status.code(), Some(1), "grund {args:?}");
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The control, green today: a numbered section *before* the nested declaration resolves, so the
/// fixture and the build are sound and a red test below is about the position alone.
#[test]
fn a_section_before_the_nested_declaration_resolves() {
    let tree = Tree::new(
        "control",
        &[(
            "ctl.md",
            "# AR-ctl: Control\n\nThe control lead.\n\n## 1. Before the child\n\n\
             The control section body.\n\n## AR-sub: Sub\n\nThe sub body.\n",
        )],
        "The notes cite \u{a7}AR-ctl, \u{a7}AR-ctl.1 and \u{a7}AR-sub.\n",
    );
    tree.assert_prints(&["check"], "success\n", 0);
    tree.assert_prints(
        &["AR-ctl.1"],
        "## 1. Before the child\n\nThe control section body.\n",
        0,
    );
    tree.assert_prints(
        &["AR-ctl", "--toc"],
        "The control lead.\n\n## 1. Before the child\n",
        0,
    );
}

/// The report: `check` advises numbering the plain heading as `AR-ci.1` (true today), and once it
/// is numbered a citation of `AR-ci.1` checks clean (today: `missing section AR-ci.1`).
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn the_coordinate_check_advises_resolves_once_the_heading_is_numbered() {
    let tree = Tree::new("advised", &[("ci.md", CI_PLAIN)], CI_NOTES);
    let advice = tree.run(&["check"]);
    assert!(
        String::from_utf8_lossy(&advice.stdout).contains(
            "docs/ci.md:9: error: unmarked heading inside AR-ci; number it \
             (## 1. Pull-request workflows) as AR-ci.1, declare an ID, or use a bold label"
        ),
        "check advises AR-ci.1: {}",
        String::from_utf8_lossy(&advice.stdout)
    );

    tree.write("ci.md", CI_NUMBERED);
    tree.assert_prints(&["check"], "success\n", 0);
}

/// `grund AR-ci.1` prints the numbered heading after `AR-matrix` (today: `section not found`), and
/// the nested declaration does not gain it.
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn show_prints_the_section_after_the_nested_declaration() {
    let tree = Tree::new("show", &[("ci.md", CI_NUMBERED)], CI_NOTES);
    tree.assert_prints(
        &["AR-ci.1"],
        "## 1. Pull-request workflows\n\nThe pull-request workflows body.\n",
        0,
    );
    tree.assert_refuses(
        &["AR-matrix.1"],
        "section not found: AR-matrix.1\n\
         hint: run `grund AR-matrix --toc` to print the lead with the section map\n",
    );
}

/// `grund AR-ci --toc` lists the section after `AR-matrix` (today: the lead alone).
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn toc_lists_the_section_after_the_nested_declaration() {
    let tree = Tree::new("toc", &[("ci.md", CI_NUMBERED)], CI_NOTES);
    tree.assert_prints(
        &["AR-ci", "--toc"],
        "The CI lead.\n\n## 1. Pull-request workflows\n",
        0,
    );
}

/// An enabled named heading in the same place is `AR-named.pr` (today: `section not found`).
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn a_named_section_after_the_nested_declaration_resolves() {
    let tree = Tree::new(
        "named",
        &[(
            "named.md",
            "# AR-named: Named\n\nThe named lead.\n\n## AR-named-child: Named child\n\n\
             The named child body.\n\n## pr: Pull-request workflows\n\nThe named section body.\n",
        )],
        "The notes cite \u{a7}AR-named, \u{a7}AR-named.pr and \u{a7}AR-named-child.\n",
    );
    tree.assert_prints(&["check"], "success\n", 0);
    tree.assert_prints(
        &["AR-named.pr"],
        "## pr: Pull-request workflows\n\nThe named section body.\n",
        0,
    );
}

/// A relative heading under a section after the nested declaration takes its path from it:
/// `### .1 Push` under `## 1. Workflows` is `AR-rel.1.1` (today: a relative heading without a
/// parent, and `missing section AR-rel.1`).
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn a_relative_heading_under_a_section_after_the_nested_declaration_resolves() {
    let tree = Tree::new(
        "relative",
        &[(
            "rel.md",
            "# AR-rel: Relative\n\nThe relative lead.\n\n## AR-rel-child: Relative child\n\n\
             The relative child body.\n\n## 1. Workflows\n\nThe workflows body.\n\n\
             ### .1 Push\n\nThe push body.\n",
        )],
        "The notes cite \u{a7}AR-rel, \u{a7}AR-rel.1 and \u{a7}AR-rel-child.\n",
    );
    tree.assert_prints(&["check"], "success\n", 0);
    tree.assert_prints(&["AR-rel.1.1"], "### 1.1 Push\n\nThe push body.\n", 0);
}

/// A nested declaration one level further down: `### 1.2 Second` after `### AR-deep-child` inside
/// `## 1. Overview` is `AR-deep.1.2` (today: `section not found`).
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn a_section_after_a_deeper_nested_declaration_resolves() {
    let tree = Tree::new(
        "deep",
        &[(
            "deep.md",
            "# AR-deep: Deep\n\nThe deep lead.\n\n## 1. Overview\n\nThe overview body.\n\n\
             ### AR-deep-child: Deep child\n\nThe deep child body.\n\n\
             ### 1.2 Second\n\nThe second body.\n",
        )],
        "The notes cite \u{a7}AR-deep, \u{a7}AR-deep.1, \u{a7}AR-deep.1.2 and \u{a7}AR-deep-child.\n",
    );
    tree.assert_prints(&["check"], "success\n", 0);
    tree.assert_prints(&["AR-deep.1.2"], "### 1.2 Second\n\nThe second body.\n", 0);
    tree.assert_prints(
        &["AR-deep", "--toc"],
        "The deep lead.\n\n## 1. Overview\n### 1.2 Second\n",
        0,
    );
}

/// Bodies nested two deep: each heading belongs to the innermost declaration whose body holds it,
/// `### 1.` after `### AR-two-inner` to `AR-two-outer` and `## 1.` after both to `AR-two`
/// (today: both resolve nowhere).
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn each_heading_belongs_to_the_innermost_body_that_holds_it() {
    let tree = Tree::new(
        "innermost",
        &[(
            "two.md",
            "# AR-two: Two\n\nThe two lead.\n\n## AR-two-outer: Outer\n\nThe outer body.\n\n\
             ### AR-two-inner: Inner\n\nThe inner body.\n\n\
             ### 1. Back in outer\n\nThe back-in-outer body.\n\n\
             ## 1. Back in two\n\nThe back-in-two body.\n",
        )],
        "The notes cite \u{a7}AR-two, \u{a7}AR-two.1, \u{a7}AR-two-outer, \u{a7}AR-two-outer.1 and \u{a7}AR-two-inner.\n",
    );
    tree.assert_prints(&["check"], "success\n", 0);
    tree.assert_prints(
        &["AR-two-outer.1"],
        "### 1. Back in outer\n\nThe back-in-outer body.\n",
        0,
    );
    tree.assert_prints(
        &["AR-two.1"],
        "## 1. Back in two\n\nThe back-in-two body.\n",
        0,
    );
}

/// `## 1.` before the nested declaration and `## 1.` after it both claim `AR-dup.1`: `check`
/// reports the duplicate and `show` refuses the coordinate (today: silence, and `show` picks the
/// first).
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn a_path_repeated_across_the_nested_declaration_is_a_duplicate_section() {
    let tree = Tree::new(
        "duplicate",
        &[(
            "dup.md",
            "# AR-dup: Duplicate\n\nThe duplicate lead.\n\n## 1. Before\n\nThe before body.\n\n\
             ## AR-dup-child: Duplicate child\n\nThe duplicate child body.\n\n\
             ## 1. After\n\nThe after body.\n",
        )],
        "The notes cite \u{a7}AR-dup and \u{a7}AR-dup-child.\n",
    );
    tree.assert_prints(
        &["check"],
        "docs/dup.md:5: error: duplicate section AR-dup.1 (also declared at docs/dup.md:13); \
         renumber or rename all but one of the headings\n",
        1,
    );
    tree.assert_refuses(
        &["AR-dup.1"],
        "ambiguous section: AR-dup.1 (declared at docs/dup.md:5, docs/dup.md:13)\n",
    );
}

/// An unmarked heading after `## 1. Numbered`, itself after the nested declaration, is advised
/// `AR-sugg.2`, the next unused sibling (today: `AR-sugg.1`, the coordinate already taken, and
/// a citation of `AR-sugg.1` fails as missing).
#[test]
#[ignore = "red until #1359 records a heading after a nested declaration on the enclosing \
            declaration; implement removes this"]
fn an_unmarked_heading_after_such_a_section_is_advised_the_next_unused_coordinate() {
    let tree = Tree::new(
        "suggestion",
        &[(
            "sugg.md",
            "# AR-sugg: Suggestion\n\nThe suggestion lead.\n\n\
             ## AR-sugg-child: Suggestion child\n\nThe suggestion child body.\n\n\
             ## 1. Numbered\n\nThe numbered body.\n\n## Plain heading\n\nThe plain body.\n",
        )],
        "The notes cite \u{a7}AR-sugg, \u{a7}AR-sugg.1 and \u{a7}AR-sugg-child.\n",
    );
    tree.assert_prints(
        &["check"],
        "docs/sugg.md:13: error: unmarked heading inside AR-sugg; number it \
         (## 2. Plain heading) as AR-sugg.2, declare an ID, or use a bold label; \
         this became an error in grund 0.16.0\n",
        1,
    );
}
