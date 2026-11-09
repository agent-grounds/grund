//! Black-box: one directory entry that failed for one reason is one line, however many names
//! reach it (§FS-check.2.4.2), and a narrowed run names it under the path it was handed, in a
//! plain project and at a workspace root alike, with the same exit code it has today
//! (§FS-check.1.3.6.3). A link and its target stay two entries
//! (§FS-scan.scope.links.unresolvable.unopenable). Git carries no mode and no fixture here can
//! hold one, so each test builds its tree, and every test that bars something skips where the
//! mode does not bar this user, as it does not bar root.
#![cfg(unix)]

#[path = "support/two_spellings_tree.rs"]
mod two_spellings_tree;

use grund_core::{EmbeddingRequest, embedding_call};
use serde_json::json;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use two_spellings_tree::{
    SHAPES, SKIP, Shape, Tree, assert_none, broken, caution, config, denied, denied_link,
    denied_link_passed, denied_passed, denied_root,
};

/// The report's tree, on the two commands its reproducer judged.
#[test]
fn the_reports_tree_is_one_line_from_check_and_list() {
    let mut tree = Tree::new("report", Shape::Plain, &config("e2e/cases"));
    if !tree.reports_tree() {
        return;
    }
    tree.assert_prints(&[
        (&["check"], 2, vec![denied_root("e2e/cases")]),
        (
            &["list", "--kind", "E2E"],
            2,
            vec![denied_root("e2e/cases")],
        ),
    ]);
}

#[test]
fn the_reports_tree_is_one_io_entry_in_check_json() {
    let mut tree = Tree::new("json", Shape::Plain, &config("e2e/cases"));
    if !tree.reports_tree() {
        return;
    }
    let run = tree.grund(&["check", "--format", "json"]);
    assert_eq!(run.code, 2, "{}", run.shown());
    assert_eq!(
        run.io(),
        [("error".to_string(), "e2e/cases".to_string())],
        "{}",
        run.shown()
    );
}

/// The list the embedding API hands over carries the one failure, once.
#[test]
fn the_reports_tree_is_one_entry_in_the_embedding_list() {
    let mut tree = Tree::new("embedding", Shape::Plain, &config("e2e/cases"));
    if !tree.reports_tree() {
        return;
    }
    let envelope = embedding_call(EmbeddingRequest {
        operation: "list_ids".into(),
        root: tree.root.clone(),
        explicit: true,
        args: Vec::new(),
        options: json!({}),
    });
    let paths: Vec<&str> = envelope["result"]["scan_errors"]
        .as_array()
        .unwrap_or_else(|| panic!("no scan_errors list in {envelope}"))
        .iter()
        .map(|error| error["path"].as_str().expect("a path"))
        .collect();
    assert_eq!(paths, ["e2e/cases"], "{envelope}");
}

/// The E2E folder elsewhere, so the manifest pass is out of play and both lines are the walk's.
#[test]
fn with_the_manifest_pass_out_of_play_the_tree_is_still_one_line() {
    let mut tree = Tree::new("elsewhere", Shape::Plain, &config("zz/cases"));
    tree.dirs(&["zz/cases"]);
    if !tree.reports_tree() {
        return;
    }
    tree.assert_prints(&[
        (&["check"], 2, vec![denied("e2e/cases")]),
        (&["list", "--kind", "E2E"], 2, vec![denied("e2e/cases")]),
    ]);
    let run = tree.grund(&["check", "--format", "json"]);
    assert_eq!(
        run.io(),
        [("error".to_string(), "e2e/cases".to_string())],
        "{}",
        run.shown()
    );
}

/// A fix that merges failures by reason alone fails here: two directories that fail alike.
#[test]
fn a_second_directory_that_fails_alike_keeps_its_own_line() {
    let mut tree = Tree::new("guard", Shape::Plain, &config("e2e/cases"));
    tree.dirs(&["other/cases"]);
    if !tree.reports_tree() || !tree.bar("other/cases") {
        return;
    }
    tree.assert_prints(&[(
        &["check"],
        2,
        vec![denied_root("e2e/cases"), denied("other/cases")],
    )]);
}

/// Control, green today and after: a fix that compares by resolved target merges these.
#[test]
fn a_link_and_its_target_stay_two_lines() {
    let mut tree = Tree::new("apart", Shape::Plain, &config("zz/cases"));
    tree.dirs(&["zz/cases", "real"]);
    tree.link("docs/ext", "../real");
    if !tree.bar("real") {
        return;
    }
    tree.assert_prints(&[(&["check"], 2, vec![denied_link("docs/ext"), denied("real")])]);
}

/// Control, green today and after: a fix that compares by resolved target merges these too.
#[test]
fn two_links_to_one_target_stay_three_lines_with_it() {
    let mut tree = Tree::new("three", Shape::Plain, &config("zz/cases"));
    tree.dirs(&["zz/cases", "secret"]);
    tree.link("docs/a", "../secret");
    tree.link("docs/b", "../secret");
    if !tree.bar("secret") {
        return;
    }
    tree.assert_prints(&[(
        &["check"],
        2,
        vec![
            denied_link("docs/a"),
            denied_link("docs/b"),
            denied("secret"),
        ],
    )]);
}

/// A link whose target no walk reads, reached under two names, is one entry and one line.
#[test]
fn a_link_under_two_names_to_a_hidden_target_is_one_line() {
    let mut misreports = Vec::new();
    for shape in SHAPES {
        let mut tree = Tree::new("hidden", shape, &config("zz/cases"));
        if !tree.link_under_two_names(".secret") {
            return;
        }
        misreports.extend(tree.misreports(&[(&["check"], 2, vec![denied_link("e2e/link")])]));
    }
    assert_none(&misreports);
}

/// With a walked target, the link's two names are one line and the target keeps its own.
#[test]
fn a_link_under_two_names_to_a_walked_target_is_two_lines() {
    let mut tree = Tree::new("walked", Shape::Plain, &config("zz/cases"));
    if !tree.link_under_two_names("secret") {
        return;
    }
    tree.assert_prints(&[(
        &["check"],
        2,
        vec![denied_link("e2e/link"), denied("secret")],
    )]);
}

/// A broken link below a directory link: one entry, without waiting for the file pass.
#[test]
fn a_broken_link_below_a_directory_link_is_one_line() {
    let mut misreports = Vec::new();
    for shape in SHAPES {
        let tree = Tree::new("broken", shape, &config("zz/cases"));
        tree.dirs(&["b", "zz/cases"]);
        tree.link("b/broken.md", "missing.md");
        tree.link("a", "b");
        misreports.extend(tree.misreports(&[(&["check"], 2, vec![broken("a/broken.md")])]));
    }
    assert_none(&misreports);
}

/// `d` lists but cannot be searched, so `d/sub` fails at the walk under each name. The entry's
/// parent still resolves. A file `d/x.md` here is the file pass's, which is #1274's, not this.
#[test]
fn a_parent_that_lists_but_cannot_be_searched_is_one_line() {
    let mut tree = Tree::new("search", Shape::Plain, &config("zz/cases"));
    tree.dirs(&["zz/cases", "d/sub"]);
    tree.link("l", "d");
    let path = tree.root.join("d");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).expect("set the mode");
    tree.locked.push(path.clone());
    if fs::metadata(path.join("sub")).is_ok() {
        eprintln!("{SKIP}");
        return;
    }
    tree.assert_prints(&[(&["check"], 2, vec![denied("d/sub")])]);
}

/// The whole run's lines are the plain project's at a workspace root.
#[test]
fn the_reports_tree_at_a_workspace_root_is_one_line() {
    let mut tree = Tree::new("ws", Shape::WorkspaceRoot, &config("e2e/cases"));
    if !tree.reports_tree() {
        return;
    }
    tree.assert_prints(&[(&["check"], 2, vec![denied_root("e2e/cases")])]);
}

/// Each run names the failure under the path it was handed, as one error and no caution, and
/// keeps its exit; a run over a path that holds no name of it gives one caution and exits 0.
#[test]
fn scoped_runs_over_the_reports_tree_name_the_path_handed() {
    let mut misreports = Vec::new();
    for shape in SHAPES {
        let mut tree = Tree::new("scoped", shape, &config("e2e/cases"));
        if !tree.reports_tree() {
            return;
        }
        // §FS-check.2.4.3: a workspace root narrows the report, so the path did not make it read.
        let handed = |path: &str| match shape {
            Shape::Plain => denied_passed(path),
            Shape::WorkspaceRoot => denied_root(path),
        };
        misreports.extend(tree.misreports(&[
            (&["check", "realE"], 2, vec![denied_root("realE/cases")]),
            (&["check", "realE/cases"], 2, vec![handed("realE/cases")]),
            (&["check", "e2e"], 2, vec![denied_root("e2e/cases")]),
            (&["check", "e2e/cases"], 2, vec![handed("e2e/cases")]),
            (&["check", "docs"], 0, vec![caution("e2e/cases")]),
        ]));
    }
    assert_none(&misreports);
}

/// The same for a link standing at the failed entry itself.
#[test]
fn scoped_runs_over_a_link_under_two_names_name_the_path_handed() {
    let mut misreports = Vec::new();
    for shape in SHAPES {
        let mut tree = Tree::new("scoped-link", shape, &config("zz/cases"));
        if !tree.link_under_two_names(".secret") {
            return;
        }
        // §FS-check.2.4.3: a workspace root narrows the report, so the path did not make it read.
        let handed = |path: &str| match shape {
            Shape::Plain => denied_link_passed(path),
            Shape::WorkspaceRoot => denied_link(path),
        };
        misreports.extend(tree.misreports(&[
            (&["check", "realE/link"], 2, vec![handed("realE/link")]),
            (&["check", "e2e/link"], 2, vec![handed("e2e/link")]),
        ]));
    }
    assert_none(&misreports);
}

/// The exit code's guard: the whole run keeps `aa/x`, which is under `realE` neither as written
/// nor resolved, so a scope that asks physically drops `check realE` from exit 2 to exit 0.
#[test]
fn a_third_name_keeps_the_exit_code_of_a_scoped_run() {
    let mut misreports = Vec::new();
    for shape in SHAPES {
        let mut tree = Tree::new("third", shape, &config("zz/cases"));
        if !tree.third_name() {
            return;
        }
        misreports.extend(tree.misreports(&[
            (&["check"], 2, vec![denied("aa/x")]),
            (&["check", "realE"], 2, vec![denied("realE/lnk/x")]),
        ]));
    }
    assert_none(&misreports);
}

/// `include` keeps its order: the earlier root's name, not the lexicographically first across
/// the roots, and `--full` only adds, so its lines are the plain run's.
#[test]
fn the_earlier_roots_name_is_kept_and_full_only_adds() {
    let v1 = "grund_config_version = 1\nproject_name = \"q\"\n\n\
              [[kinds]]\nkind = \"E2E\"\nfolder = \"zz/cases\"\n\
              title = \"Executable user scenarios\"\n\n\
              [scan]\ninclude = [\"realE\", \"e2e\"]\n";
    let mut tree = Tree::new("order", Shape::Plain, v1);
    tree.dirs(&["zz/cases"]);
    if !tree.reports_tree() {
        return;
    }
    tree.assert_prints(&[
        (&["check"], 2, vec![denied("realE/cases")]),
        (&["check", "--full"], 2, vec![denied("realE/cases")]),
    ]);
}

/// Control, green today and after: a parse failure is not the listing failure. Since #1304 they
/// are at two paths, the ignore file and the directory (§FS-check.2.4.2), and stay two lines.
#[test]
fn a_parse_failure_and_a_listing_failure_stay_two_lines() {
    let mut tree = Tree::new("reasons", Shape::Plain, &config("e2e/cases"));
    tree.dirs(&["e2e/cases"]);
    tree.write("e2e/.ignore", "{a\n{b\n");
    if !tree.bar("e2e/cases") {
        return;
    }
    let run = tree.grund(&["check"]);
    let lines = run.failure_lines();
    assert_eq!(run.code, 2, "{}", run.shown());
    assert!(
        lines.len() == 2
            && lines[0].starts_with("error: e2e/.ignore: ")
            && lines[0].contains("error parsing glob")
            && lines[1] == denied_root("e2e/cases"),
        "{}",
        run.shown()
    );
}

/// Control, green today and after: a file is read once however many entries reach it, so a fix
/// that stops reading files once or splits on the entry prints a second line here.
#[test]
fn a_file_two_entries_reach_is_read_once_so_it_fails_on_one_line() {
    let mut tree = Tree::new("file", Shape::Plain, &config("zz/cases"));
    tree.dirs(&["zz/cases"]);
    tree.write("docs/t.md", "# T\n");
    tree.link("docs/l.md", "t.md");
    if !tree.bar("docs/t.md") {
        return;
    }
    tree.assert_prints(&[(&["check"], 2, vec![denied("docs/l.md")])]);
}
