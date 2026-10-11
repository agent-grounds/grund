//! The trees of `command_agreement.rs` for which sections a declaration has and
//! which anchor a link takes: the reproduced row r3 of #576, and the rows of
//! §REQ-one-answer.differences.sections the command line can show.

use super::agreement_tree::{CONFIG_SLUG, Tree, Verdict, agree, json_lines, stdout};
use serde_json::Value;
use std::fs;

/// `FS` declared in source as well as Markdown: a folder home with no index.
const CONFIG_FOLDER: &str = "grund_config_version = 1\nproject_name = \"t\"\n\
    [reference]\nmarker = \"\u{a7}\"\nstrict = true\n[id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n";

/// The first JSON row one command prints, or `null`.
fn first(tree: &Tree, args: &[&str]) -> Value {
    tree.json(args).into_iter().next().unwrap_or(Value::Null)
}

/// Whether the body `grund <coordinate> --full` prints holds `text`.
fn body_holds(tree: &Tree, coordinate: &str, text: &str) -> bool {
    first(tree, &[coordinate, "--full", "--format", "json"])["body"]
        .as_str()
        .is_some_and(|body| body.contains(text))
}

/// The `refs` row for the citation on `line`, or `null`.
fn citation_on(tree: &Tree, id: &str, line: u64) -> Value {
    tree.json(&["refs", id, "--format", "json"])
        .into_iter()
        .find(|row| row["line"] == line)
        .unwrap_or(Value::Null)
}

/// Whether `--toc` lists section `1`, and whether `grund <ID>.1` answers.
fn toc_and_query_agree(tree: &Tree, id: &str) -> Verdict {
    let listed = first(tree, &[id, "--toc", "--format", "json"])["sections"]
        .as_array()
        .is_some_and(|sections| sections.iter().any(|s| s["path"] == "1"));
    agree(
        "--toc lists section 1 / a direct query answers it",
        listed,
        tree.code(&[&format!("{id}.1")]) == 0,
    )
}

/// r3, row P05: the anchor `show` reports is the one `fmt --cross-refs` writes
/// (§REQ-one-answer.writes.anchors), and both are the rendered heading's, a
/// closing `#` run taken off (§FS-show.3.1.3.1).
pub fn the_anchor_is_one_and_the_headings() -> Verdict {
    let tree = Tree::new(
        "r3-closing-hashes",
        &[
            ("grund.toml", CONFIG_SLUG),
            (
                "docs/fs.md",
                concat!(
                    "# FS-a: Alpha ##\n\nAlpha body.\n\n## 1. Steps ##\n\nStep text.\n\n",
                    "# FS-b: Beta\n\nSee \u{a7}FS-a and \u{a7}FS-a.1.\n",
                ),
            ),
        ],
    );
    let shown = json_lines(&stdout(&tree.grund(&["FS-a", "--format", "json"])));
    let shown = shown
        .first()
        .and_then(|v| v["anchor"].as_str())
        .unwrap_or("")
        .to_string();
    tree.grund(&["fmt", "--cross-refs", "--write"]);
    let written = fs::read_to_string(tree.0.join("docs/fs.md")).expect("read fmt output");
    let wrote = written
        .split("fs.md#")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .unwrap_or("")
        .to_string();
    if shown != wrote || wrote != "fs-a-alpha" || !written.contains("fs.md#1-steps)") {
        return Err(format!(
            "show's anchor {shown:?}, fmt's {wrote:?}; the heading's is \"fs-a-alpha\" \
             (and \"1-steps\")"
        ));
    }
    Ok(())
}

/// Row P01: in a file with no sections, the body `show --full` prints is the one
/// `refs` places a citation in (§REQ-one-answer.sections.map).
pub fn a_body_ends_where_refs_ends_it() -> Verdict {
    let tree = Tree::new(
        "p01-plain-appendix",
        &[
            ("grund.toml", CONFIG_SLUG),
            (
                "docs/fs.md",
                "# FS-a: Alpha\n\nAlpha body.\n\n# Appendix\n\nAppendix text cites \u{a7}FS-a.\n",
            ),
        ],
    );
    agree(
        "show --full holds the Appendix / refs places its citation in FS-a",
        body_holds(&tree, "FS-a", "Appendix text"),
        citation_on(&tree, "FS-a", 7)["enclosing_declaration"] == "FS-a",
    )
}

/// Row P02: a `/** … */` heading written without ` * ` is a section for `--toc`
/// and for a direct query alike (§REQ-one-answer.sections.map).
pub fn a_block_comment_section_is_one_section() -> Verdict {
    let tree = Tree::new(
        "p02-bare-block-heading",
        &[
            ("grund.toml", CONFIG_FOLDER),
            (
                "src/lib.rs",
                "/**\n * FS-b: Beta\n *\n * Beta body.\n *\n## 1. Sub\n *\n * Sub body.\n */\npub struct B;\n",
            ),
        ],
    );
    toc_and_query_agree(&tree, "FS-b")
}

/// Row P03: `/// # 1. Sub` is a section for `--toc` and for a direct query alike
/// (§REQ-one-answer.sections.map).
pub fn a_top_level_doc_heading_is_one_section() -> Verdict {
    let tree = Tree::new(
        "p03-top-level-doc-heading",
        &[
            ("grund.toml", CONFIG_FOLDER),
            (
                "src/lib.rs",
                "/// FS-b: Beta\n///\n/// Beta body.\n///\n/// # 1. Sub\n///\n/// Sub body.\npub struct B;\n",
            ),
        ],
    );
    toc_and_query_agree(&tree, "FS-b")
}

/// Row P07: a plain `## Notes` after `## 1. One` is in section 1 for `show` and
/// for `refs` alike (§REQ-one-answer.sections.map).
pub fn a_plain_heading_is_in_one_section() -> Verdict {
    let tree = Tree::new(
        "p07-plain-notes",
        &[
            ("grund.toml", CONFIG_SLUG),
            (
                "docs/fs.md",
                "# FS-a: Alpha\n\nAlpha body.\n\n## 1. One\n\nOne body.\n\n## Notes\n\nNotes text cites \u{a7}FS-a.\n",
            ),
        ],
    );
    agree(
        "show FS-a.1 holds Notes / refs places its citation in section 1",
        body_holds(&tree, "FS-a.1", "Notes text"),
        citation_on(&tree, "FS-a", 11)["enclosing_section"] == "1",
    )
}

/// Row P10: a `//` note next to a `///` body is in the declaration for `show`
/// and for `refs` alike (§REQ-one-answer.sections.map).
pub fn an_adjacent_note_is_in_one_body() -> Verdict {
    let tree = Tree::new(
        "p10-adjacent-note",
        &[
            ("grund.toml", CONFIG_FOLDER),
            (
                "src/lib.rs",
                "/// FS-b: Beta\n///\n/// Beta body.\n// Note cites \u{a7}FS-b.\npub struct B;\n",
            ),
        ],
    );
    agree(
        "show --full holds the note / refs places its citation in FS-b",
        body_holds(&tree, "FS-b", "Note cites"),
        citation_on(&tree, "FS-b", 4)["enclosing_declaration"] == "FS-b",
    )
}

/// Row P12: in a doc-comment, the heading that ends a section for `show` ends it
/// for line ownership too (§REQ-one-answer.sections.map). A plain heading agrees
/// on this tree today; the numbered top-level heading shows the row.
pub fn a_doc_comment_section_ends_once() -> Verdict {
    let tree = Tree::new(
        "p12-doc-comment-ownership",
        &[
            ("grund.toml", CONFIG_FOLDER),
            (
                "src/lib.rs",
                concat!(
                    "/// FS-b: Beta\n///\n/// Beta body.\n///\n/// ## 1. One\n///\n",
                    "/// One body.\n///\n/// # 2. Two\n///\n/// Two text cites \u{a7}FS-b.\n",
                    "pub struct B;\n",
                ),
            ),
        ],
    );
    agree(
        "show FS-b.1 holds line 11 / refs places its citation in section 1",
        body_holds(&tree, "FS-b.1", "Two text"),
        citation_on(&tree, "FS-b", 11)["enclosing_section"] == "1",
    )
}
