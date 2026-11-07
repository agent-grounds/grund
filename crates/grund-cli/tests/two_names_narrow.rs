//! Black-box: a run naming a path lists a file's names as the whole run does, whatever else sits
//! beside the name it was handed (§FS-scan.checks.reading-conflict, §FS-scan.checks.reading-moved,
//! §FS-scan.identity.findings.content). Each fixture's first root holds a sibling, because a walk
//! that counts what it was handed from the wrong root only shows when the two counts differ.
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::{TXT_IS_RUST, Tree};

const V1: &str = "project_name = \"n\"\n[scan]\nextensions = [\"md\", \"rs\", \"txt\"]\n\
    include = [\"aa\", \"zz\"]\n";

/// Asserts the whole run and a run naming each of `scopes` print `want` on stdout.
fn agree(tree: &Tree, scopes: &[&str], want: &str) {
    assert_eq!(tree.grund(&["check"]).stdout, want, "whole run");
    for scope in scopes {
        assert_eq!(tree.grund(&["check", scope]).stdout, want, "{scope}");
    }
}

#[test]
fn a_refusal_stands_on_one_line_under_version_1() {
    let tree = Tree::new("refusal-v1", V1);
    tree.put("LICENSE", "x\n");
    tree.put("aa/other.md", "other\n");
    tree.put("zz/other.md", "other\n");
    tree.link("aa/license.md", "../LICENSE");
    tree.link("zz/license.rs", "../LICENSE");
    let line = "aa/license.md:1: error: aa/license.md and zz/license.rs are one file that reads \
        two ways, as markdown and as rs; keep one name, or give the file a name that claims a \
        reading\n";
    agree(&tree, &["aa/license.md", "zz/license.rs"], line);
}

#[test]
fn a_refusal_stands_on_one_line_under_version_2() {
    let config = "grund_config_version = 2\nproject_name = \"n\"\n[schema.sources]\n\
        languages = [\"markdown\", \"rust\"]\n[schema.kinds.FS]\nfolders = [\"aa\", \"zz\"]\n\
        index = false\n";
    let tree = Tree::new("refusal-v2", config);
    tree.put("LICENSE", "x\n");
    tree.put("aa/z-sibling.md", "# FS-s: S\n\nCites \u{a7}FS-s.\n");
    tree.link("aa/license.md", "../LICENSE");
    tree.link("zz/license.rs", "../LICENSE");
    let line = "aa/license.md:1: error: aa/license.md and zz/license.rs are one file that reads \
        two ways, as markdown and as rust; keep one name, or give the file a name that claims a \
        reading\n";
    agree(&tree, &["aa/license.md", "zz/license.rs"], line);
}

/// A link listed after its target read the file as the target's own name does before, so nothing
/// moved, and naming the link changes none of that (§FS-scan.checks.reading-moved).
#[test]
fn a_link_listed_after_its_target_earns_no_warning_when_named() {
    let tree = Tree::new("after", V1);
    tree.put("aa/m-real.md", "real\n");
    tree.put("aa/other.md", "other\n");
    tree.link("zz/link.txt", "../aa/m-real.md");
    assert_eq!(tree.grund(&["check"]).stdout, "");
    assert_eq!(tree.grund(&["check", "zz/link.txt"]).stdout, "");
}

/// A link listed before its target is where the file used to be read another way, on either
/// scope. The target sorts last in its root, which is the name a wrong count would list first.
#[test]
fn a_link_listed_before_its_target_earns_the_warning_when_named() {
    let tree = Tree::new("before", TXT_IS_RUST);
    tree.put("fs/m-real.md", "# FS-m: M\n\nCites \u{a7}FS-m.\n");
    tree.put("fs/b-other.md", "# FS-o: O\n\nCites \u{a7}FS-o.\n");
    tree.link("fs/a-link.txt", "m-real.md");
    let line = "fs/a-link.txt:1: warning: read as markdown, as its target's own name reads, not \
        as rust (since 0.22.0); rename the target or copy the file\n";
    assert_eq!(tree.grund(&["check"]).stdout, line);
    assert_eq!(tree.grund(&["check", "fs/a-link.txt"]).stdout, line);
}

/// Naming the file itself, with a sibling beside it, keeps the name the whole run lists first.
#[test]
fn a_content_finding_keeps_the_whole_runs_first_name() {
    let tree = Tree::new("content", V1);
    tree.put("aa/b.md", "\u{a7}FS-001-gone\n");
    tree.put("aa/c.md", "other\n");
    tree.link("zz/b-link.md", "../aa/b.md");
    let line = "aa/b.md:1: error: unknown reference FS-001-gone (also at zz/b-link.md:1)\n";
    agree(&tree, &["aa/b.md", "zz/b-link.md"], line);
}

/// An empty first home holds nothing before the link's own root, so the run was handed one file
/// and not what the next root held.
#[test]
fn an_empty_first_home_changes_nothing() {
    let config = "grund_config_version = 2\nproject_name = \"n\"\n[schema.sources]\n\
        languages = [\"markdown\"]\n[schema.kinds.FS]\nfolders = [\"aa\", \"bb\"]\nindex = false\n";
    let tree = Tree::new("empty", config);
    tree.put("aa/nothing.txt", "not admitted\n");
    tree.put("bb/a.md", "# FS-a: A\n\n\u{a7}FS-gone and \u{a7}FS-a.\n");
    tree.link("notes/link.md", "../bb/a.md");
    let line = "bb/a.md:3: error: unknown reference FS-gone (also at notes/link.md:3)\n";
    agree(&tree, &["notes/link.md"], line);
}

/// A file with one line of its own under each home, which the whole run reports at both names.
fn grounded_at_two_places(case: &str) -> Tree {
    let config = "grund_config_version = 2\nproject_name = \"n\"\n[schema.sources]\n\
        languages = [\"markdown\", \"rust\"]\n[schema.kinds.FS]\nfolders = [\"docs/fs\"]\n\
        index = false\n[rules.citations.grounding]\nmust = \"file\"\n";
    let tree = Tree::new(case, config);
    tree.put("docs/fs/a.md", "# FS-a: A\n\nCites \u{a7}FS-a.\n");
    tree.put("lib/n.rs", "fn n() {}\n");
    tree.link("tests/e2e/n.rs", "../../lib/n.rs");
    tree
}

const UNGROUNDED: &str = ":1: error: ungrounded source file: no \u{a7} citation to a declared ID; cite a declared ID \
    within it\n";

/// A finding about where a file sits is about one spelling, so the path narrows it as written: the
/// link's name does not bring its target's line along, nor the target's name the link's.
#[test]
fn a_place_finding_narrows_by_the_name_it_stands_at() {
    let tree = grounded_at_two_places("place-narrow");
    let (lib, link) = (
        format!("lib/n.rs{UNGROUNDED}"),
        format!("tests/e2e/n.rs{UNGROUNDED}"),
    );
    assert_eq!(tree.grund(&["check"]).stdout, format!("{lib}{link}"));
    assert_eq!(tree.grund(&["check", "tests/e2e/n.rs"]).stdout, link);
    assert_eq!(tree.grund(&["check", "lib/n.rs"]).stdout, lib);
    assert_eq!(tree.grund(&["check", "tests/e2e"]).stdout, link);
}

/// A run handed a link outside the tree reaches the file by no in-tree name, so the file's physical
/// path is what stands in for the name it was handed, and the path still narrows the report.
#[test]
fn a_run_from_outside_the_tree_keeps_the_place_finding() {
    let tree = grounded_at_two_places("place-outside");
    let outside = tree.root.with_extension("outside");
    let _ = std::fs::remove_file(&outside);
    std::os::unix::fs::symlink(tree.root.join("lib"), &outside).expect("create the outside link");
    let handed = outside.to_str().expect("a UTF-8 path");
    let run = tree.grund(&["check", handed]);
    let _ = std::fs::remove_file(&outside);
    assert_eq!(run.stdout, format!("lib/n.rs{UNGROUNDED}"));
}
