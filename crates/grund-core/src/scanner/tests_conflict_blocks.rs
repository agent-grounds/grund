//! Test module: the conflict blocks the file pass records (§AR-scanner.2.9,
//! §FS-declarations.body.conflict). Every marker line here is inside an indented
//! string literal, so no line of this file is itself a marker line.

use std::path::Path;

use super::conflict_blocks::{blocks, marker_of};
use crate::grammar::render_id;
use crate::model::format_path;
use crate::resolver::{
    WorkspaceContext, load_classifying_workspace_context, load_workspace_context,
};
use crate::testing::{test_root, write};

const CONFIG: &str = concat!(
    "grund_config_version = 2\nproject_name = \"demo\"\n\n",
    "[schema]\nmarker = \"\u{a7}\"\n\n[schema.kinds.FS]\n",
);

/// Each recorded block as `path:first-last owners`, read by the scan `check`
/// takes and by the one `fmt` takes, which classifies nothing (§AR-scanner.2.4.2);
/// the two must agree, because the finding and the refusal name one block.
fn recorded(root: &Path) -> Vec<String> {
    let check = load_classifying_workspace_context(root, true).expect("load check context");
    let fmt = load_workspace_context(root, true).expect("load fmt context");
    let base = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let read = |context: &WorkspaceContext| {
        let project = &context.projects[0];
        let grammar = project.config.frame().grammar();
        let mut blocks: Vec<String> = project
            .findings
            .conflict_blocks
            .iter()
            .map(|block| {
                let file = block
                    .file
                    .strip_prefix(&base)
                    .or_else(|_| block.file.strip_prefix(root))
                    .unwrap_or(&block.file);
                let owners: Vec<String> = block
                    .owners
                    .iter()
                    .map(|id| render_id(grammar, id))
                    .collect();
                let owners = owners.join(" ");
                format!(
                    "{}:{}-{} [{owners}]",
                    format_path(file),
                    block.first,
                    block.last
                )
            })
            .collect();
        blocks.sort();
        blocks
    };
    let (by_check, by_fmt) = (read(&check), read(&fmt));
    assert_eq!(
        by_check, by_fmt,
        "check's scan and fmt's scan record the same blocks"
    );
    by_check
}

fn project(name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let root = test_root(name);
    write(&root.join("grund.toml"), CONFIG);
    for (path, text) in files {
        write(&root.join(path), text);
    }
    root
}

/// §FS-declarations.body.conflict.markers: column 0, exactly seven of one
/// character, then a space, the end of the line, or a CR before it.
#[test]
fn a_marker_line_is_seven_of_one_character_at_column_zero() {
    for (line, marker) in [
        ("<<<<<<< HEAD", Some(b'<')),
        ("|||||||", Some(b'|')),
        ("=======", Some(b'=')),
        ("=======\r", Some(b'=')),
        (">>>>>>> other", Some(b'>')),
        ("<<<<<<<< HEAD", None),
        ("<<<<<<<HEAD", None),
        (" <<<<<<< HEAD", None),
        ("======", None),
        ("=======\tx", None),
        ("<<<<<<=", None),
        ("", None),
        ("# FS-a: A", None),
    ] {
        assert_eq!(marker_of(line), marker, "{line:?}");
    }
}

/// §FS-declarations.body.conflict.block: `<` opens and the next `>` closes; a
/// middle marker outside a block is text; a nested opener opens nothing; a lone
/// closer is a block of one line; an unclosed opener ends at its last marker.
#[test]
fn marker_lines_cut_blocks_as_the_definition_says() {
    assert_eq!(blocks(&[(1, b'<'), (3, b'='), (5, b'>')]), vec![(1, 5)]);
    assert_eq!(
        blocks(&[(1, b'='), (2, b'|')]),
        Vec::<(usize, usize)>::new()
    );
    assert_eq!(
        blocks(&[(1, b'<'), (2, b'<'), (3, b'='), (4, b'>'), (6, b'>')]),
        vec![(1, 4), (6, 6)]
    );
    assert_eq!(blocks(&[(2, b'<'), (4, b'|'), (6, b'=')]), vec![(2, 6)]);
    assert_eq!(blocks(&[(2, b'<'), (3, b'<')]), vec![(2, 3)]);
}

/// §FS-declarations.body.conflict.where and .owners: in a Markdown file that
/// declares, every block counts, a fenced one and one inside an HTML comment too
/// (§FS-declarations.body.conflict.markers), and each names the declarations it
/// overlaps, each ID once, in the order they open; one after a heading that
/// closed every body names none.
#[test]
fn a_declaring_markdown_file_records_every_block_with_its_owners() {
    let abc = concat!(
        "# FS-a: A\n",
        "<<<<<<< HEAD\n",
        "# FS-b: B\n",
        "=======\n",
        "# FS-c: C\n",
        ">>>>>>> other\n",
        "\n",
        "# Appendix\n",
        "\n",
        "```text\n",
        "<<<<<<< HEAD\n",
        "Keep.\n",
        "=======\n",
        "Drop.\n",
        ">>>>>>> other\n",
        "```\n",
    );
    let plain = concat!(
        "# Notes\n\n",
        "<<<<<<< HEAD\n",
        "a\n",
        "=======\n",
        "b\n",
        ">>>>>>> other\n"
    );
    let login = concat!(
        "# FS-login: Login\n",
        "\n",
        "Lead.\n",
        "\n",
        "<!--\n",
        "<<<<<<< HEAD\n",
        "a\n",
        "=======\n",
        "b\n",
        ">>>>>>> other\n",
        "-->\n",
    );
    let root = project(
        "a_declaring_markdown_file_records_every_block_with_its_owners",
        &[
            ("docs/abc.md", abc),
            ("docs/plain.md", plain),
            ("docs/login.md", login),
        ],
    );
    assert_eq!(
        recorded(&root),
        vec![
            "docs/abc.md:11-15 []",
            "docs/abc.md:2-6 [FS-a FS-b FS-c]",
            "docs/login.md:6-10 [FS-login]",
        ]
    );
}

/// §FS-declarations.body.conflict.where: whether a Markdown file declares is read
/// off its raw lines, so a fence the conflict left open does not hide the one
/// declaration below it and the block is still recorded, naming none.
#[test]
fn a_fence_the_conflict_left_open_does_not_hide_the_file_declaring() {
    let login = concat!(
        "<<<<<<< HEAD\n",
        "Short.\n",
        "=======\n",
        "```text\n",
        "alice\n",
        ">>>>>>> other\n",
        "\n",
        "# FS-login: Login\n",
    );
    let root = project(
        "a_fence_the_conflict_left_open_does_not_hide_the_file_declaring",
        &[("docs/login.md", login)],
    );
    assert_eq!(recorded(&root), vec!["docs/login.md:1-6 []"]);
}

/// §FS-declarations.body.conflict.where: in a source file a block counts inside a
/// body, a docstring's or a `/** */` comment's as much as a line-style run's, and
/// right after the line-style run it cut, and nowhere else in the code.
#[test]
fn a_source_file_records_blocks_in_and_right_after_a_body_only() {
    let lib = concat!(
        "/// FS-parse: Parse\n",
        "///\n",
        "/// Parses input.\n",
        "<<<<<<< HEAD\n",
        "/// Trims first.\n",
        "=======\n",
        "/// Splits first.\n",
        ">>>>>>> other\n",
        "pub fn parse() {}\n",
        "\n",
        "fn other() {\n",
        "<<<<<<< HEAD\n",
        "    1\n",
        "=======\n",
        "    2\n",
        ">>>>>>> other\n",
        "}\n",
    );
    let render = concat!(
        "\"\"\"FS-render: Render\n",
        "\n",
        "<<<<<<< HEAD\n",
        "Total first.\n",
        "=======\n",
        "Items first.\n",
        ">>>>>>> other\n",
        "\"\"\"\n",
    );
    let java = concat!(
        "/**\n",
        " * FS-parse-java: Parse\n",
        " *\n",
        "<<<<<<< HEAD\n",
        " * Trims first.\n",
        "=======\n",
        " * Splits first.\n",
        ">>>>>>> other\n",
        " */\n",
        "public class Parse {}\n",
    );
    let root = project(
        "a_source_file_records_blocks_in_and_right_after_a_body_only",
        &[
            ("src/lib.rs", lib),
            ("src/render.py", render),
            ("src/Parse.java", java),
        ],
    );
    assert_eq!(
        recorded(&root),
        vec![
            "src/Parse.java:4-8 [FS-parse-java]",
            "src/lib.rs:4-8 [FS-parse]",
            "src/render.py:3-7 [FS-render]",
        ]
    );
}
