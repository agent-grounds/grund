//! Black-box: two in-tree names that read a file differently, where its own name claims nothing,
//! are refused and both named (§FS-scan.checks.reading-conflict, §FS-fmt.2.3.2.3,
//! §FS-check.2.1.2). Sort order must not come back in here.
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::Tree;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"r\"\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\
    [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n";
const REFUSAL: &str = "docs/license.md:1: error: docs/license.md and docs/license.rs are one file \
    that reads two ways, as markdown and as rust; keep one name, or give the file a name that \
    claims a reading\n";
const NOT_REWRITTEN: &str = "warning: docs/license.md: not rewritten: docs/license.md and \
    docs/license.rs read this file differently\n";
const LICENSE: &str = "# FS-l: L\n\nSee $$FS-a.\n";

fn tree(case: &str) -> Tree {
    let tree = Tree::new(case, CONFIG);
    tree.put("LICENSE", LICENSE);
    tree.put("docs/fs/a.md", "# FS-a: A\n\nCites \u{a7}FS-a.\n");
    tree.link("docs/license.md", "../LICENSE");
    tree.link("docs/license.rs", "../LICENSE");
    tree
}

/// A whole run and a run naming either spelling print the same one line.
#[test]
#[ignore = "red until #1274 refuses a reading conflict; implement removes this"]
fn the_file_is_refused_naming_both_spellings_on_every_scope() {
    let tree = tree("scopes");
    for scope in [None, Some("docs/license.md"), Some("docs/license.rs")] {
        let mut args = vec!["check"];
        args.extend(scope);
        let check = tree.grund(&args);
        assert_eq!(check.stdout, REFUSAL, "{scope:?}");
        assert_eq!(check.code, 1, "{scope:?}");
    }
}

#[test]
#[ignore = "red until #1274 refuses a reading conflict; implement removes this"]
fn its_json_sites_name_both_spellings() {
    let json = tree("json").grund(&["check", "--format", "json"]);
    assert_eq!(
        json.sites("reading-conflict"),
        ["docs/license.md:1", "docs/license.rs:1"]
    );
    assert_eq!(json.code, 1);
}

/// Hideable by neither axis, like the import codes (§FS-check.2.1.2).
#[test]
#[ignore = "red until #1274 refuses a reading conflict; implement removes this"]
fn neither_selection_axis_hides_it() {
    let tree = tree("selection");
    for args in [
        ["check", "--ignore", "reading-conflict"],
        ["check", "--only", "dangling"],
        ["check", "--only", "reading-conflict"],
    ] {
        let check = tree.grund(&args);
        assert_eq!(check.stdout, REFUSAL, "{args:?}");
        assert_eq!(check.code, 1, "{args:?}");
    }
}

/// It is read under neither name, so what it declares is gone and a citation to it is reported.
#[test]
#[ignore = "red until #1274 refuses a reading conflict; implement removes this"]
fn the_file_is_read_under_neither_name() {
    let tree = tree("neither");
    assert!(
        !tree.grund(&["list"]).stdout.contains("FS-l"),
        "read under one of them"
    );
    tree.put(
        "docs/fs/b.md",
        "# FS-b: B\n\nCites \u{a7}FS-b and \u{a7}FS-l.\n",
    );
    let check = tree.grund(&["check"]);
    assert!(
        check
            .stdout
            .contains("docs/fs/b.md:3: error: unknown reference FS-l"),
        "{}",
        check.stdout
    );
    assert!(check.stdout.contains(REFUSAL), "{}", check.stdout);
}

/// `fmt` leaves it alone and says so once, narrowed or not, with its exit code unchanged. A
/// narrow run does the project walk that finds the other name
/// (§FS-scan.identity.reading.every-caller).
#[test]
#[ignore = "red until #1274 refuses a reading conflict; implement removes this"]
fn fmt_leaves_the_file_alone_and_says_so_once() {
    let tree = tree("fmt");
    for args in [
        vec!["fmt", "--write"],
        vec!["fmt", "--check"],
        vec!["fmt", "docs/license.md", "--write"],
        vec!["fmt", "docs/license.rs", "--write"],
        vec!["fmt", "docs/license.rs", "--check"],
    ] {
        let run = tree.grund(&args);
        assert_eq!(tree.read("LICENSE"), LICENSE, "{args:?} rewrote the file");
        assert_eq!(run.stderr, NOT_REWRITTEN, "{args:?}");
        assert_eq!(run.code, 0, "{args:?}");
    }
}

/// `refs` and `cover` scan narrowly, and agree with the whole run that it was not read.
#[test]
#[ignore = "red until #1274 refuses a reading conflict; implement removes this"]
fn a_narrow_refs_or_cover_run_does_not_read_it_either() {
    let tree = tree("narrow");
    tree.put("LICENSE", "# FS-l: L\n\nSee \u{a7}FS-a.\n");
    for spelling in ["docs/license.md", "docs/license.rs"] {
        let refs = tree.grund(&["refs", "FS-a", spelling]);
        assert!(
            !refs.stdout.contains("\u{a7}FS-a"),
            "{spelling}: {}",
            refs.stdout
        );
        let cover = tree.grund(&["cover", spelling]);
        assert!(
            !cover.stdout.contains("\u{a7}FS-a"),
            "{spelling}: {}",
            cover.stdout
        );
    }
}

/// Where the links also leave the root, the conflict line stands and one warning is printed.
#[test]
#[ignore = "red until #1274 refuses a reading conflict; implement removes this"]
fn a_conflict_behind_an_outward_link_prints_one_warning() {
    let outside = Tree::new("outward-target", "");
    outside.put("LICENSE", LICENSE);
    let tree = Tree::new("outward", CONFIG);
    tree.put("docs/fs/a.md", "# FS-a: A\n\nCites \u{a7}FS-a.\n");
    tree.link("docs/license.md", outside.root.join("LICENSE"));
    tree.link("docs/license.rs", outside.root.join("LICENSE"));
    assert_eq!(tree.grund(&["check"]).stdout, REFUSAL);
    assert_eq!(tree.grund(&["fmt", "--check"]).stderr, NOT_REWRITTEN);
}

/// The catalog row is live, and prints the remedy the message carries (§FS-errors.5.5).
#[test]
#[ignore = "red until #1274 takes the planned mark off its rows; implement removes this"]
fn config_explain_answers_for_both_new_codes() {
    let tree = tree("explain");
    for (code, fix) in [
        (
            "reading-conflict",
            "keep one name, or give the file a name that claims a reading",
        ),
        ("reading-moved", "rename the target or copy the file"),
    ] {
        let explain = tree.grund(&["config", "explain", code]);
        assert_eq!(explain.code, 0, "{code}: {}", explain.stderr);
        assert!(explain.stdout.contains(fix), "{code}: {}", explain.stdout);
    }
}

/// Guard: two names of one language do not read differently, so there is no conflict and the
/// file is read once.
#[test]
fn two_names_of_one_language_are_no_conflict() {
    let tree = Tree::new("one-language", CONFIG);
    tree.put("LICENSE", "/// FS-l: L\nfn l() {}\n");
    tree.link("src/a.rs", "../LICENSE");
    tree.link("src/b.rs", "../LICENSE");
    let list = tree.grund(&["list"]);
    assert!(list.stdout.contains("FS-l  src/a.rs:1"), "{}", list.stdout);
    assert_eq!(tree.grund(&["check"]).code, 0);
}
