//! Black-box: a narrowed `grund cover` reads the E2E case its path holds (§FS-cover.1.2), so a
//! failure at a file the path holds as the tree spells it exits `2` and names it, as `check` does
//! (§FS-declarations.e2e-case.failure): the file itself, the case, and a case that is a link out of
//! the cases folder. A path holding none of the case's failing files prints what it printed
//! (§DF-cover-reads-the-case-its-path-holds).

use super::*;

const MANIFEST: &str = "error: e2e/cases/b/expected.exit: ";

impl Tree {
    /// A tree whose case `b` cannot be decoded, its `command.args` healthy, and when `linked`, `b`
    /// a link to `../../elsewhere/b`, inside the project and outside the cases folder.
    fn broken_case(test: &str, linked: bool) -> Self {
        let tree = Self::new(&format!("cover-{test}"));
        let case = if linked { "elsewhere/b" } else { "e2e/cases/b" };
        tree.write(&format!("{case}/expected.exit"), UNDECODABLE);
        tree.write(&format!("{case}/command.args"), b"check\n");
        if linked {
            let path = tree.root.join("e2e/cases/b");
            fs::remove_dir_all(&path).expect("clear the case");
            symlink(Path::new("../../elsewhere/b"), path).expect("link the case");
        }
        tree
    }
}

/// What a run whose path holds no failing file owes: exit `0`, and no word about the manifest.
fn assert_quiet(run: &Run) {
    assert_eq!(run.code, 0, "{run:#?}");
    assert!(!run.stderr.contains("expected.exit"), "{run:#?}");
    assert!(!run.stdout.contains("expected.exit"), "{run:#?}");
}

/// The control: a plain case directory already holds its manifest's failure.
#[test]
fn cover_over_a_plain_case_holds_its_manifest() {
    let tree = Tree::broken_case("plain-case", false);
    assert_holds(&tree.run(&["cover", "e2e/cases/b"]), MANIFEST);
}

#[test]
fn cover_naming_a_plain_cases_manifest_holds_it() {
    let tree = Tree::broken_case("plain-file", false);
    assert_holds(&tree.run(&["cover", "e2e/cases/b/expected.exit"]), MANIFEST);
}

#[test]
fn cover_naming_a_plain_cases_manifest_holds_it_in_json() {
    let tree = Tree::broken_case("plain-json", false);
    // §FS-cover.4: `cover` reports a scan failure on stderr in either format, as `cover e2e/cases`.
    let run = tree.run(&["cover", "e2e/cases/b/expected.exit", "--format", "json"]);
    assert_holds(&run, MANIFEST);
}

#[test]
fn cover_naming_a_manifest_in_a_linked_case_holds_it() {
    let tree = Tree::broken_case("linked-file", true);
    assert_holds(&tree.run(&["cover", "e2e/cases/b/expected.exit"]), MANIFEST);
}

#[test]
fn cover_over_a_case_linked_out_of_the_folder_holds_its_manifest() {
    let tree = Tree::broken_case("linked-case", true);
    assert_holds(&tree.run(&["cover", "e2e/cases/b"]), MANIFEST);
}

/// §FS-check.1.3.6.3: a readable file of the case holds only itself, linked or not.
#[test]
fn cover_naming_another_file_of_the_case_holds_no_failure() {
    for linked in [false, true] {
        let tree = Tree::broken_case(&format!("other-file-{linked}"), linked);
        assert_quiet(&tree.run(&["cover", "e2e/cases/b/command.args"]));
    }
}

/// §DF-e2e-case-failure-at-the-file.2.4: the link's target does not resolve to the configured
/// spelling, so it holds nothing there.
#[test]
fn cover_over_the_links_target_holds_no_failure() {
    let tree = Tree::broken_case("target", true);
    assert_quiet(&tree.run(&["cover", "elsewhere/b"]));
    assert_quiet(&tree.run(&["cover", "elsewhere/b/expected.exit"]));
}

/// A healthy sibling's case and file print exactly what they print on a tree with no bad case.
#[test]
fn cover_over_a_healthy_sibling_prints_what_a_healthy_tree_does() {
    let broken = Tree::broken_case("sibling-broken", true);
    let healthy = Tree::new("cover-sibling-healthy");
    for path in ["e2e/cases/a", "e2e/cases/a/expected.exit"] {
        let (run, control) = (broken.run(&["cover", path]), healthy.run(&["cover", path]));
        assert_eq!(
            (run.code, &run.stdout, &run.stderr),
            (control.code, &control.stdout, &control.stderr),
            "{path}"
        );
    }
}
