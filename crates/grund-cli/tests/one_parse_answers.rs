//! What every command answers from one parse of each file from 0.19.0
//! (§FS-declarations.body, §FS-declarations.section, §FS-show.2.3.3,
//! §FS-languages.comment-wrapper.headings, §FS-declarations.section.relative.where):
//! one tree per row of the table §DF-one-parse-answers-from-0-19 moves, the doc-comment
//! headings that ride it, and the `parse-moves` code that goes with it
//! (§FS-declarations.checks.parse-moves). Each test asserts the record's answer, so each
//! failed while 0.18.0's compatibility reading answered instead.
#[path = "support/agreement_tree.rs"]
#[allow(dead_code)]
mod agreement_tree;

use agreement_tree::{Tree, stderr, stdout};
use serde_json::Value;

/// `{kind}-{slug}` with named sections, `FS` declared anywhere, no grounding floor.
const CONFIG: &str = "grund_config_version = 1\nproject_name = \"t\"\n\
    [reference]\nmarker = \"\u{a7}\"\nstrict = true\nrequire_grounding = false\n\
    [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n";

fn tree(name: &str, path: &str, body: &str) -> Tree {
    Tree::new(
        &format!("one-parse-{name}"),
        &[("grund.toml", CONFIG), (path, body)],
    )
}

/// The one JSON object `show` prints for `query`.
fn show(tree: &Tree, args: &[&str]) -> Value {
    let output = tree.grund(&[args, &["--format", "json"]].concat());
    serde_json::from_str(&stdout(&output))
        .unwrap_or_else(|error| panic!("show {args:?}: {error}: {}", stderr(&output)))
}

fn body(tree: &Tree, query: &str) -> String {
    show(tree, &[query, "--full"])["body"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

/// `(path, title)` of each section `--toc` lists.
fn toc(tree: &Tree, id: &str) -> Vec<(String, String)> {
    show(tree, &[id, "--toc"])["sections"]
        .as_array()
        .map(|sections| {
            sections
                .iter()
                .map(|s| {
                    let field = |k: &str| s[k].as_str().unwrap_or_default().to_string();
                    (field("path"), field("title"))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn toc_paths(tree: &Tree, id: &str) -> Vec<String> {
    toc(tree, id).into_iter().map(|(path, _)| path).collect()
}

/// The section `refs` places the citation of `id` on `line` in, `None` for none.
fn owner(tree: &Tree, id: &str, line: u64) -> Option<String> {
    let rows = tree.json(&["refs", id, "--format", "json"]);
    let row = rows
        .iter()
        .find(|row| row["line"] == line)
        .unwrap_or_else(|| panic!("refs {id} lists no citation on line {line}: {rows:?}"));
    row["enclosing_section"].as_str().map(str::to_string)
}

// §FS-declarations.body (P01): a Markdown body ends at a plain same-or-higher heading.
#[test]
fn p01_a_markdown_body_ends_at_a_plain_heading() {
    let t = tree(
        "p01",
        "docs/fs/a.md",
        "# FS-a: Alpha\n\nAlpha body.\n\n# Appendix\n\nAppendix text cites \u{a7}FS-a.\n",
    );
    let full = body(&t, "FS-a");
    assert!(
        !full.contains("Appendix text"),
        "show FS-a --full: {full:?}"
    );
}

// §FS-languages.comment-wrapper.headings (P02): a bare `/** … */` heading line is section .1.
#[test]
fn p02_a_bare_block_comment_heading_is_a_section() {
    let t = tree(
        "p02",
        "src/lib.rs",
        "/**\n * FS-b: Beta\n *\n * Beta body.\n *\n## 1. Sub\n *\n * Sub body.\n */\npub struct B;\n",
    );
    let output = t.grund(&["FS-b.1"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).starts_with("## 1. Sub"),
        "printed as written: {:?}",
        stdout(&output)
    );
}

// §FS-show.2.3.3 (P03): `/// # 1. Sub` is no section, so `--toc` does not list it.
#[test]
fn p03_a_top_level_doc_heading_is_not_listed() {
    let t = tree(
        "p03",
        "src/lib.rs",
        "/// FS-b: Beta\n///\n/// Beta body.\n///\n/// # 1. Sub\n///\n/// Sub body.\npub struct B;\n",
    );
    assert_eq!(toc_paths(&t, "FS-b"), Vec::<String>::new());
}

// §FS-declarations.section (P07): a section ends at a plain heading of its level.
#[test]
fn p07_a_section_ends_at_a_plain_heading() {
    let t = tree(
        "p07",
        "docs/fs/a.md",
        "# FS-a: Alpha\n\nAlpha body.\n\n## 1. One\n\nOne body.\n\n## Notes\n\nNotes text cites \u{a7}FS-a.\n",
    );
    let section = body(&t, "FS-a.1");
    assert!(!section.contains("Notes text"), "show FS-a.1: {section:?}");
}

// §FS-languages.comment-block (P10): a `//` note next to a `///` block is outside its body.
#[test]
fn p10_an_adjacent_note_is_outside_the_body() {
    let t = tree(
        "p10",
        "src/lib.rs",
        "/// FS-b: Beta\n///\n/// Beta body.\n// Note cites \u{a7}FS-b.\npub struct B;\n",
    );
    let full = body(&t, "FS-b");
    assert!(!full.contains("Note cites"), "show FS-b --full: {full:?}");
}

// §AR-scanner.2.4.4 (P12): ownership ends a doc-comment section at a duplicate heading.
#[test]
fn p12_a_duplicate_heading_ends_the_section_for_refs() {
    let t = tree(
        "p12-duplicate",
        "src/lib.rs",
        "/// FS-b: Beta\n///\n/// Beta body.\n///\n/// ## 1. One\n///\n/// One body.\n///\n\
         /// ## 2. Two\n///\n/// Two body.\n///\n/// ## 1. One again\n///\n\
         /// Again cites \u{a7}FS-b.\npub struct B;\n",
    );
    assert_eq!(owner(&t, "FS-b", 15), None);
}

// §FS-declarations.section (P12, D17b): and at a plain heading of the section's depth.
#[test]
fn p12_a_plain_heading_ends_the_section_for_refs() {
    let t = tree(
        "p12-plain",
        "src/lib.rs",
        "/// FS-b: Beta\n///\n/// Beta body.\n///\n/// ## 1. One\n///\n/// One body.\n///\n\
         /// ## Plain\n///\n/// Plain cites \u{a7}FS-b.\npub struct B;\n",
    );
    assert_eq!(owner(&t, "FS-b", 11), None);
}

// §FS-declarations.body (D17a): a deeper declaration does not end its parent's body.
#[test]
fn d17a_a_deeper_declaration_stays_in_the_parents_body() {
    let t = tree(
        "d17a",
        "docs/fs/a.md",
        "# FS-a: Alpha\n\nLead.\n\n## FS-c: Child\n\nChild body.\n\n# FS-d: Delta\n",
    );
    let full = body(&t, "FS-a");
    assert!(full.contains("Child body"), "show FS-a --full: {full:?}");
}

// §FS-languages.comment-wrapper.headings (D17d): a closing `*/` is envelope in `show`.
#[test]
fn d17d_show_drops_the_closer_of_a_block_comment_heading() {
    let t = tree(
        "d17d",
        "src/J.java",
        "/**\n * FS-j: J\n *\n * Lead.\n *\n * ## 1. Step */\npublic class J {}\n",
    );
    let section = body(&t, "FS-j.1");
    assert_eq!(section.trim_end(), "## 1. Step", "show FS-j.1");
}

// §FS-declarations.heading: the `--toc` title drops a heading's closing hashes.
#[test]
fn the_toc_title_drops_closing_hashes() {
    let t = tree(
        "title",
        "docs/fs/a.md",
        "# FS-a: Alpha\n\nAlpha body.\n\n## 1. Steps ##\n\nStep text.\n",
    );
    assert_eq!(toc(&t, "FS-a"), vec![("1".into(), "Steps".into())]);
}

// §FS-declarations.section.relative.where: a relative doc-comment heading is its section,
// and a citation below it moves into that section.
#[test]
fn a_relative_doc_comment_heading_is_a_section() {
    let t = tree(
        "relative",
        "src/lib.rs",
        "/// FS-thing: A thing\n///\n/// ## format: Formatting\n///\n/// ### .1: Username\n\
         ///\n/// Username cites \u{a7}FS-thing.\npub struct Thing;\n",
    );
    assert_eq!(toc_paths(&t, "FS-thing"), vec!["format", "format.1"]);
    assert_eq!(owner(&t, "FS-thing", 7).as_deref(), Some("format.1"));
}

// §FS-declarations.section.relative.where: a complete mixed doc-comment heading.
#[test]
fn a_complete_mixed_doc_comment_heading_is_a_section() {
    let t = tree(
        "mixed-complete",
        "src/lib.rs",
        "/// FS-x: Example\n///\n/// ## 1. Overview\n///\n/// Overview text.\n///\n\
         /// ### 1.goals: Goals\n///\n/// Goals text.\npub fn x() {}\n",
    );
    assert_eq!(toc_paths(&t, "FS-x"), vec!["1", "1.goals"]);
}

// §FS-declarations.section.relative.where: a relative mixed doc-comment heading.
#[test]
fn a_relative_mixed_doc_comment_heading_is_a_section() {
    let t = tree(
        "mixed-relative",
        "src/lib.rs",
        "/// FS-x: Example\n///\n/// ## 1. Overview\n///\n/// Overview text.\n///\n\
         /// ### .goals: Goals\n///\n/// Goals text.\npub fn x() {}\n",
    );
    assert_eq!(toc_paths(&t, "FS-x"), vec!["1", "1.goals"]);
}

// §FS-declarations.section.relative.where: the parent is read from the 0.19.0 map, so a
// relative child of a bare `/** … */` `## 1.` line resolves under it (#682's R1-01).
#[test]
fn a_relative_child_of_a_bare_block_comment_heading_resolves() {
    let t = Tree::new(
        "one-parse-r1-01",
        &[
            ("grund.toml", CONFIG),
            (
                "src/Thing.java",
                concat!(
                    "/**\n * FS-thing: A thing\n *\n## 1. Sub\n *\n * ### .1. Child\n *\n",
                    " * Child text.\n */\npublic class Thing {}\n",
                ),
            ),
            ("docs/use.md", "Uses \u{a7}FS-thing.\n"),
        ],
    );
    assert_eq!(toc_paths(&t, "FS-thing"), vec!["1", "1.1"]);
    let check = t.grund(&["check"]);
    assert_eq!(check.status.code(), Some(0), "{}", stdout(&check));
}

// §FS-declarations.checks.parse-moves: no tree gets a `parse-moves` finding.
#[test]
fn no_tree_gets_a_parse_moves_finding() {
    let t = Tree::new(
        "one-parse-no-finding",
        &[
            ("grund.toml", CONFIG),
            (
                "src/lib.rs",
                "/// FS-thing: A thing\n///\n/// ## format: Formatting\n///\n\
                 /// ### .1: Username\n///\n/// Username text.\npub struct Thing;\n",
            ),
            ("docs/use.md", "Uses \u{a7}FS-thing.\n"),
        ],
    );
    let check = t.grund(&["check"]);
    assert_eq!(stdout(&check), "success\n");
    assert_eq!(check.status.code(), Some(0));
}

// §FS-declarations.checks.parse-moves: the code is gone, so selecting it is refused.
#[test]
fn selecting_parse_moves_is_refused_as_an_unknown_code() {
    let t = tree("unknown-code", "docs/fs/a.md", "# FS-a: Alpha\n\nBody.\n");
    for flag in ["--only", "--ignore"] {
        let output = t.grund(&["check", flag, "parse-moves"]);
        assert_eq!(output.status.code(), Some(2), "check {flag} parse-moves");
        assert!(
            stderr(&output).contains("unknown check finding code \"parse-moves\""),
            "check {flag} parse-moves: {}",
            stderr(&output)
        );
    }
}
