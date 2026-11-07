//! Black-box: a finding that names other sites prints one line from every scope, a file with a
//! second name beside a third file or a name that is the tail of another site's path. A rule settles
//! its anchor and sites on the names a whole run gives them and renders from those, and the pass that
//! finishes the report reads no message (§FS-scan.identity.findings.content, §FS-check.1.3.6.2).
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::Tree;

const DOCS: &str = "grund_config_version = 2\nproject_name = \"n\"\n\
    [schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n";
const DOCS_FS: &str = "grund_config_version = 2\nproject_name = \"n\"\n\
    [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n";
const DECL: &str = "# FS-d: A\n\n\u{a7}FS-d\n";

/// Asserts the whole run and a run naming each of `scopes` print `want` as text, and one report as
/// JSON.
fn agree(tree: &Tree, scopes: &[&str], want: &str) {
    let whole = tree.grund(&["check"]);
    assert_eq!(whole.stdout, want, "whole run: {}", whole.stderr);
    let json = tree.grund(&["check", "--format=json"]).stdout;
    for scope in scopes {
        assert_eq!(tree.grund(&["check", scope]).stdout, want, "{scope}");
        let narrow = tree.grund(&["check", scope, "--format=json"]).stdout;
        assert_eq!(narrow, json, "{scope} as JSON");
    }
}

/// The handed name is the first declaring name of no run but this one, and it sorts the third file
/// ahead of the file it names.
#[test]
fn a_duplicate_with_a_third_file_anchors_as_the_whole_run_does() {
    let tree = Tree::new("dup-third", DOCS_FS);
    tree.put("docs/fs/a.md", DECL);
    tree.put("docs/fs/b.md", DECL);
    tree.link("notes/link.md", "../docs/fs/a.md");
    agree(
        &tree,
        &["notes/link.md", "notes", "docs/fs/b.md", "docs/fs/a.md"],
        "docs/fs/a.md:1: error: duplicate declaration of FS-d (also declared at docs/fs/b.md:1) \
         (also at notes/link.md:1)\n",
    );
}

/// `notes/link.md` is the tail of `docs/notes/link.md` and `docs/m.md` of no other site: no path of
/// the message may be edited through a path another site's text ends in.
#[test]
fn a_name_that_is_the_tail_of_another_sites_path_leaves_that_path_alone() {
    let tree = Tree::new("dup-tail", DOCS);
    tree.put("docs/b.md", DECL);
    tree.put("docs/m.md", DECL);
    tree.put("docs/notes/link.md", DECL);
    tree.link("notes/link.md", "../docs/m.md");
    agree(
        &tree,
        &["notes/link.md", "notes", "docs/notes/link.md", "docs/m.md"],
        "docs/b.md:1: error: duplicate declaration of FS-d (also declared at docs/m.md:1, \
         docs/notes/link.md:1)\n",
    );
}

/// `notes/link.md:1` is a substring of the third file's `docs/notes/link.md:1`, and it is still a
/// name of this line, so the closing clause keeps it.
#[test]
fn a_name_that_is_the_tail_of_a_third_file_keeps_its_clause() {
    let tree = Tree::new("dup-clause", DOCS);
    tree.put("docs/a.md", DECL);
    tree.put("docs/notes/link.md", DECL);
    tree.link("notes/link.md", "../docs/a.md");
    let want = "docs/a.md:1: error: duplicate declaration of FS-d (also declared at \
                docs/notes/link.md:1) (also at notes/link.md:1)\n";
    agree(
        &tree,
        &["notes/link.md", "notes", "docs/notes/link.md", "docs/a.md"],
        want,
    );
    let sites = tree
        .grund(&["check", "--format=json"])
        .sites("duplicate")
        .join(" ");
    assert_eq!(sites, "docs/a.md:1 notes/link.md:1 docs/notes/link.md:1");
}

/// Not a regression pin: this tree printed the same line before the rule rendered from first names,
/// because the message names one file. It holds that rendering to the same bytes.
#[test]
fn a_duplicate_section_with_a_tail_name_prints_one_line() {
    let config = "grund_config_version = 2\nproject_name = \"f\"\n\
        [schema.sources]\nlanguages = [\"markdown\"]\n\
        [schema.kinds.AR]\nfolders = [\"docs\"]\nindex = false\n";
    let tree = Tree::new("dup-section", config);
    tree.put(
        "docs/notes/v.md",
        "# AR-v: V\n\nCites \u{a7}AR-v.\n\n## 1. One\n\nText.\n\n## 1. Again\n\nText.\n",
    );
    tree.link("notes/v.md", "../docs/notes/v.md");
    agree(
        &tree,
        &["notes/v.md", "notes", "docs/notes/v.md"],
        "docs/notes/v.md:5: error: duplicate section AR-v.1 (also declared at \
         docs/notes/v.md:9); renumber or rename all but one of the headings (also at \
         notes/v.md:5)\n",
    );
}

const VALUES: &str = "grund_config_version = 1\nproject_name = \"s\"\n\
    [reference]\nstrict = true\n[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\
    [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\
    [scan]\ninclude = [\"docs\", \"notes\"]\nextensions = [\"md\"]\n";

fn priced(case: &str, bound: &str) -> Tree {
    let tree = Tree::new(case, VALUES);
    tree.put("values/price.md", "# CONST-price: Price\n\n## 1. 1200\n");
    tree.link("notes/price.md", "../values/price.md");
    tree.link("docs/values/price.md", "../../values/price.md");
    tree.put(
        "docs/fs/a.md",
        &format!("# FS-a: A\n\nCosts `{bound}` (\u{a7}CONST-price.1).\n\n\u{a7}FS-a\n"),
    );
    tree
}

/// The author's literal is the author's: it spells a name of the declaring file, and no scope may
/// rewrite it as though it were the site the message names.
#[test]
fn a_bound_literal_spelling_a_name_stays_as_written() {
    let tree = priced("values-literal", "notes/price.md:3");
    let line = tree.grund(&["check"]).stdout;
    assert!(
        line.contains("bound `notes/price.md:3`, declared `1200` at docs/values/price.md:3"),
        "{line}"
    );
    agree(&tree, &["notes/price.md", "notes", "docs/fs/a.md"], &line);
}

/// Not a regression pin: this tree printed the same line before the rule asked for the first name,
/// which the old pass reached by editing the message.
#[test]
fn a_value_mismatch_names_the_declaration_as_the_whole_run_does() {
    let tree = priced("values-name", "1250");
    let line = tree.grund(&["check"]).stdout;
    assert!(
        line.contains("bound `1250`, declared `1200` at docs/values/price.md:3"),
        "{line}"
    );
    agree(
        &tree,
        &["notes/price.md", "notes", "docs/values/price.md"],
        &line,
    );
}
