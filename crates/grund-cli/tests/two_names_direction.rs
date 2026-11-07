//! Black-box: a citation-direction finding about a declaration holds at any spelling, because the
//! obligation follows the declaration's kind, so it is one line from every scope. One about a file
//! is asked of the row that owns the spelling, and stays at it (§FS-scan.identity.findings.content,
//! §FS-scan.identity.findings.place, §FS-config.3.9.2).
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::Tree;

const MUST: &str = "grund_config_version = 2\nproject_name = \"n\"\n\
    [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\
    [schema.kinds.GOAL]\nfolders = [\"docs/goal\"]\nindex = false\n\
    [rules.citations.FS]\nmust = [\"GOAL\"]\n";
const MUST_NOT: &str = "grund_config_version = 2\nproject_name = \"n\"\n\
    [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\
    [schema.kinds.AR]\nfolders = [\"docs/ar\"]\nindex = false\n\
    [rules.citations.FS]\nmust-not = [\"AR\"]\n";

/// Asserts the whole run, `--full` and a run naming each of `scopes` print `want`.
fn agree(tree: &Tree, scopes: &[&str], want: &str) {
    assert_eq!(tree.grund(&["check"]).stdout, want, "whole run");
    assert_eq!(tree.grund(&["check", "--full"]).stdout, want, "--full");
    for scope in scopes {
        assert_eq!(tree.grund(&["check", scope]).stdout, want, "{scope}");
    }
}

#[test]
fn a_declaration_that_owes_a_citation_is_one_line_from_every_scope() {
    let tree = Tree::new("direction-must", MUST);
    tree.put("docs/fs/a.md", "# FS-a: A\n\n\u{a7}FS-a\n");
    tree.put("docs/goal/g.md", "# GOAL-g: G\n\n\u{a7}GOAL-g\n");
    tree.link("notes/link.md", "../docs/fs/a.md");
    agree(
        &tree,
        &["notes/link.md", "notes", "docs/fs/a.md"],
        "docs/fs/a.md:1: error: FS-a must cite GOAL (citation direction); add such a citation \
         within it (also at notes/link.md:1)\n",
    );
    assert_eq!(
        tree.grund(&["check", "--format=json"])
            .sites("missing-citation"),
        ["docs/fs/a.md:1", "notes/link.md:1"]
    );
}

#[test]
fn a_declaration_that_cites_what_it_must_not_is_one_line_from_every_scope() {
    let tree = Tree::new("direction-must-not", MUST_NOT);
    tree.put("docs/ar/b.md", "# AR-b: B\n\n\u{a7}AR-b\n");
    tree.put("docs/fs/q.md", "# FS-q: Q\n\n\u{a7}AR-b \u{a7}FS-q\n");
    tree.link("notes/link.md", "../docs/fs/q.md");
    agree(
        &tree,
        &["notes/link.md", "notes", "docs/fs/q.md"],
        "docs/fs/q.md:3: error: FS must not cite AR (citation direction) \u{2014} re-point the \
         citation or downgrade it to a plain Markdown link (also at notes/link.md:3)\n",
    );
}

/// The name a place rule sits at decides nothing here: a home's name that sorts first is the
/// anchor, wherever the declaration's kind is at home.
#[test]
fn a_second_name_in_the_same_home_is_named_in_the_clause() {
    let tree = Tree::new("direction-home", MUST);
    tree.put("docs/fs/z.md", "# FS-a: A\n\n\u{a7}FS-a\n");
    tree.put("docs/goal/g.md", "# GOAL-g: G\n\n\u{a7}GOAL-g\n");
    tree.link("docs/fs/a-link.md", "z.md");
    agree(
        &tree,
        &["docs/fs/a-link.md", "docs/fs/z.md", "docs/fs"],
        "docs/fs/a-link.md:1: error: FS-a must cite GOAL (citation direction); add such a \
         citation within it (also at docs/fs/z.md:1)\n",
    );
}

/// A file unit's obligation is its row's, so only the spelling in that row owes it: no clause, no
/// sites, and no line for the other spelling. Passes either way: the split it holds was already
/// there, and the fix has to keep it.
#[test]
fn a_file_unit_keeps_its_own_spelling() {
    let must = "grund_config_version = 2\nproject_name = \"n\"\n\
        [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\
        [schema.kinds.GOAL]\nfolders = [\"docs/goal\"]\nindex = false\n\
        [schema.kinds.e2e]\nfolders = [\"tests/e2e\"]\ncitable = false\n\
        [rules.citations.e2e]\nmust = [\"FS\"]\n";
    let tree = Tree::new("direction-file-must", must);
    tree.put("docs/fs/a.md", "# FS-a: A\n\n\u{a7}FS-a\n");
    tree.put("docs/goal/g.md", "# GOAL-g: G\n\n\u{a7}GOAL-g\n");
    tree.put("lib/n.rs", "// \u{a7}GOAL-g\nfn n() {}\n");
    tree.link("tests/e2e/n.rs", "../../lib/n.rs");
    let whole = tree.grund(&["check", "--format=json"]);
    let found = whole.findings();
    assert_eq!(found.len(), 1, "{}", whole.stdout);
    assert_eq!(found[0]["code"], "missing-citation");
    assert_eq!(found[0]["path"], "tests/e2e/n.rs");
    assert!(found[0]["sites"].is_null(), "{}", whole.stdout);
    let line = tree.grund(&["check"]).stdout;
    assert!(!line.contains("(also at"), "{line}");
    assert_eq!(tree.grund(&["check", "tests/e2e/n.rs"]).stdout, line);
}
