//! Black-box: which spelling a failure reading an E2E case is reported at
//! (§FS-declarations.e2e-case.failure). A file the walk or another case already reported, under any
//! spelling, is that one failure and one line (§FS-check.2.4.2), named by the spelling the run's
//! own path reached it under, else the one listed first: the runs of a plain tree that meet the
//! walk's report, and the link trees where the run's spelling is the one the line takes. A fixture
//! directory's line goes only where a report says it at that spelling or kept that spelling as a
//! name, so where the walk kept none it is still two lines (§DF-e2e-case-on-the-one-walk).

use super::*;

impl Tree {
    /// Replaces the case with a link, its target as the cases folder would spell it.
    fn link_case(&self, case: &str, target: &str) {
        let path = self.root.join("e2e/cases").join(case);
        fs::remove_dir_all(&path).expect("clear the case");
        symlink(Path::new(target), path).expect("link the case");
    }

    /// Moves the cases folder to `real` and leaves a link in its place.
    fn link_folder(&self) {
        let folder = self.root.join("e2e/cases");
        fs::rename(&folder, self.root.join("real")).expect("move the folder");
        symlink(Path::new("../real"), folder).expect("link the folder");
    }

    /// A link to the project root, beside it, as a path spelled from outside the project.
    fn link_to_root(&mut self) -> String {
        let name = self.root.file_name().expect("a name").to_string_lossy();
        let link = self.root.with_file_name(format!("{name}-link"));
        let _ = fs::remove_file(&link);
        symlink(&self.root, &link).expect("link the root");
        self.links.push(link.clone());
        link.to_str().expect("a UTF-8 temporary path").to_string()
    }
}

/// The run's failure lines, each by how it opens, in the order they print.
fn assert_lines(run: &Run, openings: &[&str]) {
    let lines = failure_lines(run);
    assert_eq!(lines.len(), openings.len(), "{run:#?}");
    for (line, opening) in lines.iter().zip(openings) {
        assert!(
            line.starts_with(opening),
            "`{line}` does not open `{opening}`: {run:#?}"
        );
    }
}

/// What a run holding the file owes on any tree: exit `2`, and exactly these lines.
fn assert_fails(run: &Run, openings: &[&str]) {
    assert_eq!(run.code, 2, "{run:#?}");
    assert_lines(run, openings);
}

/// §AR-scanner.1.relisted: the walk spells below the canonical root as the pass does, however the
/// run reached the file, so a plain tree meets in the comparison and prints one line.
#[test]
fn a_manifest_the_walk_also_reads_is_one_line_however_the_run_reaches_it() {
    let tree = tree_scanning_manifests("scanned-spellings");
    tree.write("e2e/cases/b/spec.refs", UNDECODABLE);
    let line = ["error: e2e/cases/b/spec.refs: "];
    let absolute = tree.root.join("e2e/cases/b");
    let absolute = absolute.to_str().expect("a UTF-8 temporary path");
    for args in [
        &["check"][..],
        &["check", "."],
        &["check", "e2e/cases"],
        &["check", "e2e/cases/b"],
        &["check", "./e2e/cases/b/"],
        &["check", "e2e/cases/b/spec.refs"],
        &["check", absolute],
    ] {
        assert_fails(&tree.run(args), &line);
    }
    for args in [
        &["check", "."][..],
        &["check", "spec.refs"],
        &["check", "../b"],
    ] {
        assert_fails(&tree.run_in("e2e/cases/b", args), &line);
    }
}

/// §AR-scanner.1.relisted: a member's walk and its pass spell from the member's root, from the
/// workspace root and from inside the member alike.
#[test]
fn a_manifest_the_walk_also_reads_in_a_member_is_one_line() {
    let tree = tree_scanning_manifests("scanned-member");
    let member = fs::read(tree.root.join("grund.toml")).expect("read the config");
    tree.write("m/grund.toml", &member);
    tree.write(
        "grund.toml",
        b"grund_config_version = 2\nproject_name = \"ws\"\n\n[workspace]\nmembers = [\"m\"]\n",
    );
    tree.write("m/e2e/cases/a/expected.exit", b"0\n");
    tree.write("m/e2e/cases/b/expected.exit", b"0\n");
    tree.write("m/e2e/cases/b/spec.refs", UNDECODABLE);
    let line = ["error: e2e/cases/b/spec.refs: "];
    assert_fails(&tree.run(&["check"]), &["error: m/e2e/cases/b/spec.refs: "]);
    assert_fails(&tree.run(&["check", "m/e2e/cases/b"]), &line);
    assert_fails(&tree.run_in("m", &["check"]), &line);
    assert_fails(&tree.run_in("m", &["check", "e2e/cases/b"]), &line);
    assert_fails(&tree.run_in("m/e2e/cases/b", &["check", "."]), &line);
    assert_fails(
        &tree.run_in("m/e2e/cases/b", &["check", "spec.refs"]),
        &line,
    );
}

/// §FS-check.2.4.2: a version 1 folder keeps a `..` in what it read, and a failure is published at
/// the folder's one lexical path, which a run can name and which meets the walk's report.
#[test]
fn a_version_1_folder_through_a_parent_step_is_one_line_every_run_holds() {
    let line = ["error: e2e/cases/b/spec.refs: "];
    // The pass alone reads the manifest under `md`, and the walk reads it too under `refs`.
    for extension in ["md", "refs"] {
        let tree = Tree::new(&format!("version-1-step-{extension}"));
        let config = format!(
            "grund_config_version = 1\n[id]\nformat = \"{{kind}}-{{slug}}\"\n\
             [[kinds]]\nkind = \"E2E\"\nfolder = \"e2e/../e2e/cases\"\nindex = false\n\
             [scan]\ninclude = [\"e2e\"]\nextensions = [\"{extension}\"]\n"
        );
        tree.write("grund.toml", config.as_bytes());
        tree.write("e2e/cases/b/spec.refs", UNDECODABLE);
        for args in [
            &["check"][..],
            &["check", "e2e"],
            &["check", "e2e/cases"],
            &["check", "e2e/cases/b"],
            &["check", "e2e/cases/b/spec.refs"],
        ] {
            assert_fails(&tree.run(args), &line);
        }
        assert_fails(&tree.run_in("e2e/cases/b", &["check", "."]), &line);
    }
}

/// §FS-config-v2.schema.places.boundaries.1: a version 2 place folds its `..` as it parses, so this
/// tree has the one spelling of any other.
#[test]
fn a_folder_configured_through_a_parent_step_is_one_line() {
    let tree = Tree::new("folder-spelling");
    let config = CONFIG.replace("e2e/cases", "e2e/../e2e/cases");
    let config = format!(
        "{config}\n[schema.sources]\nlanguages = [\"markdown\"]\n\n\
         [schema.sources.extensions]\nrefs = \"markdown\"\n"
    );
    tree.write("grund.toml", config.as_bytes());
    tree.write("e2e/cases/b/spec.refs", UNDECODABLE);
    for args in [&["check"][..], &["check", "e2e/cases/b"]] {
        let run = tree.run(args);
        assert_eq!(run.code, 2, "{run:#?}");
        assert_eq!(failure_lines(&run).len(), 1, "{run:#?}");
    }
}

/// Each run of a project reached through a link, every one of which holds the file: the root is
/// canonical once discovered, so the link leaves the walk and the pass spelling alike.
fn assert_one_line_through_a_link(tree: &mut Tree, file: &str) {
    let link = tree.link_to_root();
    let line = format!("error: e2e/cases/b/{file}: ");
    let line = [line.as_str()];
    let case = format!("{link}/e2e/cases/b");
    let named = format!("{case}/{file}");
    let parent = tree.root.parent().expect("a parent").to_path_buf();
    let parent = parent.to_str().expect("a UTF-8 temporary path");
    assert_fails(&tree.run_in(parent, &["check", &link]), &line);
    assert_fails(&tree.run_in(parent, &["check", &case]), &line);
    assert_fails(&tree.run_in(parent, &["check", &named]), &line);
    assert_fails(&tree.run_in(&link, &["check"]), &line);
    assert_fails(&tree.run_in(&link, &["check", "."]), &line);
    assert_fails(&tree.run_in(&link, &["check", "e2e/cases/b"]), &line);
    assert_fails(&tree.run_in(&link, &["check", &case]), &line);
    assert_fails(&tree.run_in(&case, &["check", "."]), &line);
    assert_fails(&tree.run_in(&case, &["check", file]), &line);
}

#[test]
fn a_manifest_the_walk_also_reads_through_a_linked_root_is_one_line() {
    let mut tree = tree_scanning_manifests("linked-root-manifest");
    tree.write("e2e/cases/b/spec.refs", UNDECODABLE);
    assert_one_line_through_a_link(&mut tree, "spec.refs");
}

#[test]
fn a_fixture_directory_through_a_linked_root_is_one_line() {
    let mut tree = Tree::new("linked-root-fixture");
    tree.write("e2e/cases/b/fx/file.txt", b"held\n");
    if !tree.lock("e2e/cases/b/fx") {
        eprintln!("{SKIP}");
        return;
    }
    assert_one_line_through_a_link(&mut tree, "fx");
}

/// §FS-declarations.e2e-case.failure: a case linked to a sibling reaches the sibling's file, so the
/// two cases meet one failure, one line: at the case listed first, and at the target where the run
/// names it, so the run naming the target holds the file.
#[test]
fn a_case_linked_to_a_sibling_it_sorts_before_leaves_the_siblings_line() {
    let tree = Tree::new("linked-case-first");
    tree.write("e2e/cases/b/spec.refs", UNDECODABLE);
    tree.link_case("a", "b");
    let held = "error: e2e/cases/b/spec.refs: ";
    assert_fails(&tree.run(&["check"]), &["error: e2e/cases/a/spec.refs: "]);
    assert_fails(&tree.run(&["check", "e2e/cases/b"]), &[held]);
    assert_fails(&tree.run(&["check", "e2e/cases/b/spec.refs"]), &[held]);
    assert_fails(&tree.run_in("e2e/cases/b", &["check", "."]), &[held]);
    assert_fails(
        &tree.run_in("e2e/cases/b", &["check", "spec.refs"]),
        &[held],
    );
    let json = tree.run(&["check", "e2e/cases/b", "--format", "json"]);
    assert_eq!(json.code, 2, "{json:#?}");
    let entries = json_failures(&json);
    assert_eq!(entries.len(), 1, "{json:#?}");
    assert!(
        entries.iter().any(
            |entry| entry.contains("\"severity\":\"error\",\"path\":\"e2e/cases/b/spec.refs\"")
        ),
        "{json:#?}"
    );
}

/// §FS-declarations.e2e-case.failure: two cases linked to one directory read one file, so it is
/// one line, and each is held by the run naming it, which the line is then named by.
#[test]
fn two_cases_linked_to_one_directory_each_keep_their_line() {
    let tree = Tree::new("linked-case-pair");
    tree.write("elsewhere/x/expected.exit", b"0\n");
    tree.write("elsewhere/x/spec.refs", UNDECODABLE);
    tree.link_case("b", "../../elsewhere/x");
    tree.link_case("c", "../../elsewhere/x");
    assert_fails(
        &tree.run(&["check", "e2e/cases/c"]),
        &["error: e2e/cases/c/spec.refs: "],
    );
    assert_fails(
        &tree.run(&["check", "e2e/cases/b"]),
        &["error: e2e/cases/b/spec.refs: "],
    );
}

/// §DF-e2e-case-failure-at-the-file.2.2: the walk's report at a linked case's target does not take
/// the line at the configured spelling, so a run holding only that spelling exits `2`. Which of the
/// two the walk and the pass meet is why the fixture is here: a directory under two spellings is
/// still two lines, while a manifest is one file and one line (§DF-e2e-case-on-the-one-walk).
fn assert_linked_case_holds(tree: &Tree, file: &str) {
    let held = format!("error: e2e/cases/b/{file}: ");
    let beside = format!("warning: elsewhere/b/{file}: ");
    for path in ["e2e/cases", "e2e"] {
        assert_fails(&tree.run(&["check", path]), &[&held, &beside]);
    }
}

#[test]
fn a_linked_case_keeps_its_fixture_line_beside_the_walks_report_of_the_target() {
    let mut tree = Tree::new("linked-case-fixture");
    tree.write("elsewhere/b/expected.exit", b"0\n");
    tree.write("elsewhere/b/fx/file.txt", b"held\n");
    tree.link_case("b", "../../elsewhere/b");
    if !tree.lock("elsewhere/b/fx") {
        eprintln!("{SKIP}");
        return;
    }
    assert_linked_case_holds(&tree, "fx");
}

/// §FS-check.2.4.2: the walk read the manifest at the link's target, and the case reads that one
/// file, so it is one line, named by the spelling a run holding only the case reached it under.
#[test]
fn a_linked_case_keeps_its_manifest_line_beside_the_walks_report_of_the_target() {
    let tree = tree_scanning_manifests("linked-case-manifest");
    tree.write("elsewhere/b/expected.exit", b"0\n");
    tree.write("elsewhere/b/spec.refs", UNDECODABLE);
    tree.link_case("b", "../../elsewhere/b");
    for path in ["e2e/cases", "e2e"] {
        assert_fails(
            &tree.run(&["check", path]),
            &["error: e2e/cases/b/spec.refs: "],
        );
    }
}

/// The same for a linked folder's fixture directory: `e2e` holds the configured spelling alone, and
/// `e2e/cases`, which the partition resolves through the link, holds both.
fn assert_linked_folder_holds(tree: &Tree, file: &str) {
    let held = format!("error: e2e/cases/b/{file}: ");
    assert_fails(
        &tree.run(&["check", "e2e"]),
        &[&held, &format!("warning: real/b/{file}: ")],
    );
    assert_fails(
        &tree.run(&["check", "e2e/cases"]),
        &[&held, &format!("error: real/b/{file}: ")],
    );
}

#[test]
fn a_linked_folder_keeps_its_fixture_line_beside_the_walks_report_of_the_target() {
    let mut tree = Tree::new("linked-folder-fixture");
    tree.write("e2e/cases/b/fx/file.txt", b"held\n");
    tree.link_folder();
    if !tree.lock("real/b/fx") {
        eprintln!("{SKIP}");
        return;
    }
    assert_linked_folder_holds(&tree, "fx");
}

/// §FS-check.2.4.2: the same one file through a linked folder, one line at the configured spelling.
#[test]
fn a_linked_folder_keeps_its_manifest_line_beside_the_walks_report_of_the_target() {
    let tree = tree_scanning_manifests("linked-folder-manifest");
    tree.write("e2e/cases/b/spec.refs", UNDECODABLE);
    tree.link_folder();
    for path in ["e2e", "e2e/cases"] {
        assert_fails(
            &tree.run(&["check", path]),
            &["error: e2e/cases/b/spec.refs: "],
        );
    }
}

/// §FS-check.1.3.6.3: a healthy sibling holds no spelling of the one failing file, so it cautions
/// about it once, at the spelling listed first, and the failure leaves its exit alone.
#[test]
fn a_healthy_sibling_on_a_link_tree_keeps_its_cautions() {
    let linked_case = Tree::new("healthy-linked-case");
    linked_case.write("e2e/cases/b/spec.refs", UNDECODABLE);
    linked_case.link_case("a", "b");
    let run = linked_case.run(&["check", "e2e/cases/c"]);
    assert_ne!(run.code, 2, "{run:#?}");
    assert_lines(&run, &["warning: e2e/cases/a/spec.refs: "]);

    let linked_folder = tree_scanning_manifests("healthy-linked-folder");
    linked_folder.write("e2e/cases/b/spec.refs", UNDECODABLE);
    linked_folder.link_folder();
    let run = linked_folder.run(&["check", "e2e/cases/a"]);
    assert_ne!(run.code, 2, "{run:#?}");
    assert_lines(&run, &["warning: real/b/spec.refs: "]);
}

/// §FS-declarations.e2e-case.failure: a path through a link to the failing case holds the file,
/// one line named by the run's own spelling, so this run is not the caution a path past it gets.
#[test]
fn a_path_through_a_link_to_the_failing_case_holds_the_file() {
    let tree = Tree::new("through-link");
    tree.write("e2e/cases/b/spec.refs", UNDECODABLE);
    tree.link_case("c", "b");
    assert_fails(
        &tree.run(&["check", "e2e/cases/c"]),
        &["error: e2e/cases/c/spec.refs: "],
    );
}

/// §AR-scanner.1.relisted: the walk reports a fixture link it cannot open as `unreadable symlink:`,
/// the pass meets the bare reason at the same spelling, and those are one failure, one line.
#[test]
fn a_fixture_link_nobody_can_open_is_the_walks_one_line() {
    let mut tree = Tree::new("fixture-link-unopenable");
    tree.write("locked/file.txt", b"held\n");
    symlink(
        Path::new("../../../locked"),
        tree.root.join("e2e/cases/b/fixture"),
    )
    .expect("link the fixture");
    if !tree.lock("locked") {
        eprintln!("{SKIP}");
        return;
    }
    assert_fails(
        &tree.run(&["check", "e2e/cases/b"]),
        &[
            "error: e2e/cases/b/fixture: unreadable symlink: permission denied",
            "warning: locked: permission denied",
        ],
    );
}

/// A root enters the case under the pass's spelling and `realE` reaches it first, so the walk
/// reports the failure at `realE/...` and keeps `zz/...` as a name.
const ENTERED_CASE: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.doc]\ncitable = false\nfolders = [\"docs\", \"realE\", \"zz/cases/b\"]\n\n\
    [schema.kinds.E2E]\nfolders = [\"zz/cases\"]\nindex = false\n";

/// The same two roots, as a version 1 config spells them.
const ENTERED_CASE_V1: &str = "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"E2E\"\nfolder = \"zz/cases\"\nindex = false\n\
    [scan]\ninclude = [\"realE\", \"zz/cases/b\"]\n";

/// No root enters the case or `zz`, so the walk never meets the pass's spelling.
const FOLDER_ONLY: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.doc]\ncitable = false\nfolders = [\"docs\", \"realE\"]\n\n\
    [schema.kinds.E2E]\nfolders = [\"zz/cases\"]\nindex = false\n";

impl Tree {
    /// `zz -> realE` below `base`, the cases in `realE/cases`, under `config`, with case `b`
    /// holding `fix`, a fixture directory the caller breaks.
    fn linked_cases(test: &str, base: &str, config: &str) -> Self {
        let tree = Self::new(test);
        fs::remove_dir_all(tree.root.join("e2e")).expect("clear the default cases");
        tree.write(&format!("{base}/grund.toml"), config.as_bytes());
        tree.write(&format!("{base}/docs/notes.md"), b"# Notes\n");
        for case in ["a", "b"] {
            tree.write(&format!("{base}/realE/cases/{case}/expected.exit"), b"0\n");
        }
        symlink(Path::new("realE"), tree.root.join(base).join("zz")).expect("link the folder");
        tree
    }

    /// The same, with `fix` a directory this user cannot list, or `None` where the mode does not
    /// bar this user.
    fn barred_fixture(test: &str, base: &str, config: &str) -> Option<Self> {
        let mut tree = Self::linked_cases(test, base, config);
        let fix = format!("{base}/realE/cases/b/fix");
        fs::create_dir_all(tree.root.join(&fix)).expect("create the fixture");
        if tree.lock(&fix) {
            Some(tree)
        } else {
            eprintln!("{SKIP}");
            None
        }
    }
}

const AT_TARGET: &str = "error: realE/cases/b/fix: permission denied";
const AT_LINK: &str = "error: zz/cases/b/fix: permission denied";

/// §FS-check.2.4.2: the walk kept the pass's spelling as a name of its report, so the two are one
/// failure: one line whether the run names it under the survivor or under that spelling, and no
/// exit moves (§FS-check.1.3.6.3).
#[test]
fn a_case_failure_the_walk_kept_the_spelling_of_is_one_line() {
    let Some(tree) = Tree::barred_fixture("kept-spelling", ".", ENTERED_CASE) else {
        return;
    };
    assert_fails(&tree.run(&["check"]), &[AT_TARGET]);
    for path in ["zz", "zz/cases", "zz/cases/b", "zz/cases/b/fix"] {
        assert_fails(&tree.run(&["check", path]), &[AT_LINK]);
    }
    assert_fails(&tree.run(&["check", "realE"]), &[AT_TARGET]);

    let beside = tree.run(&["check", "docs"]);
    assert_eq!(beside.code, 0, "{beside:#?}");
    assert_lines(&beside, &["warning: realE/cases/b/fix: permission denied"]);
}

/// §FS-check.2.4.2: a version 1 config's two roots meet the same way, and `list` reports one line.
#[test]
fn a_case_failure_the_walk_kept_the_spelling_of_is_one_line_under_version_1() {
    let Some(tree) = Tree::barred_fixture("kept-spelling-v1", ".", ENTERED_CASE_V1) else {
        return;
    };
    assert_fails(&tree.run(&["check"]), &[AT_TARGET]);
    assert_fails(&tree.run(&["check", "zz"]), &[AT_LINK]);
    assert_fails(&tree.run(&["check", "realE/cases/b"]), &[AT_TARGET]);
    assert_lines(&tree.run(&["list", "--kind", "E2E"]), &[AT_TARGET]);
}

/// §FS-check.2.4.2: one `io` entry, where a run naming the link held two identical ones.
#[test]
fn a_case_failure_the_walk_kept_the_spelling_of_is_one_json_entry() {
    let Some(tree) = Tree::barred_fixture("kept-spelling-json", ".", ENTERED_CASE) else {
        return;
    };
    for (args, path) in [
        (&["check", "--format", "json"][..], "realE/cases/b/fix"),
        (&["check", "zz", "--format", "json"], "zz/cases/b/fix"),
    ] {
        let run = tree.run(args);
        assert_eq!(run.code, 2, "{run:#?}");
        let entries = json_failures(&run);
        assert_eq!(entries.len(), 1, "{run:#?}");
        assert!(entries[0].contains(path), "{run:#?}");
    }
}

/// §FS-check.2.4.2: a member's walk and pass meet below the member's root, however it is reached.
#[test]
fn a_case_failure_the_walk_kept_the_spelling_of_is_one_line_in_a_member() {
    let Some(tree) = Tree::barred_fixture("kept-spelling-member", "m", ENTERED_CASE) else {
        return;
    };
    tree.write(
        "grund.toml",
        b"grund_config_version = 2\nproject_name = \"ws\"\n\n[workspace]\nmembers = [\"m\"]\n",
    );
    assert_fails(&tree.run(&["check"]), &["error: m/realE/cases/b/fix: "]);
    assert_fails(&tree.run(&["check", "m/zz"]), &[AT_LINK]);
    assert_fails(&tree.run_in("m", &["check"]), &[AT_TARGET]);
    assert_fails(&tree.run_in("m", &["check", "zz"]), &[AT_LINK]);
}

/// §AR-scanner.1.relisted: the name is held under the walk's own words, so the kept spelling of a
/// fixture link is found by the report's text, and the one line keeps those words.
#[test]
fn a_fixture_link_the_walk_kept_the_spelling_of_is_the_walks_one_line() {
    let mut tree = Tree::linked_cases("kept-spelling-link", ".", ENTERED_CASE_V1);
    tree.write("barred/file.txt", b"held\n");
    let fix = tree.root.join("realE/cases/b/fix");
    symlink(Path::new("../../../barred"), fix).expect("link the fixture");
    if !tree.lock("barred") {
        eprintln!("{SKIP}");
        return;
    }
    assert_fails(
        &tree.run(&["check"]),
        &["error: realE/cases/b/fix: unreadable symlink: permission denied"],
    );
    assert_fails(
        &tree.run(&["check", "zz"]),
        &["error: zz/cases/b/fix: unreadable symlink: permission denied"],
    );
}

/// §FS-check.2.4.2: a case's failure compares by spelling. The walk never met `zz`, so no report
/// kept it and the pass's line stands beside the walk's, though both resolve to one directory.
#[test]
fn a_case_failure_at_a_spelling_the_walk_never_met_keeps_its_line() {
    let Some(tree) = Tree::barred_fixture("unmet-spelling", ".", FOLDER_ONLY) else {
        return;
    };
    assert_fails(&tree.run(&["check"]), &[AT_TARGET, AT_LINK]);
}
