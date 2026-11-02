//! Test module: a declaration line written on a one-line block comment is
//! titled without the comment's closing `*/` (§FS-declarations.line.block-comment,
//! §FS-languages.comment-wrapper.headings). The closer is the envelope of the
//! comment that carries the declaration, as it is for a section heading on the
//! line that closes a block (`tests_source_section_titles`); a Markdown heading
//! and a docstring keep a `*/` their author wrote.
//!
//! The e2e case `list-one-line-block-comment-declaration-title` pins what a user
//! sees of the same titles in `grund list`.

use crate::model::Catalog;
use crate::testing::{embedded_value_config, scan_tree, test_root, write};

/// The recorded title of the one declaration whose slug is `slug`.
fn title(findings: &Catalog, slug: &str) -> Option<String> {
    let decls = findings
        .declarations
        .iter()
        .filter(|(id, _)| id.slug.as_deref() == Some(slug))
        .flat_map(|(_, decls)| decls)
        .collect::<Vec<_>>();
    assert_eq!(decls.len(), 1, "the fixture declares *-{slug} exactly once");
    decls[0].title.clone()
}

/// §FS-declarations.line.block-comment: `/** FS-b: Beta */` is titled `Beta`, as
/// `//! FS-b: Beta` is, whatever the opener and however much whitespace sits
/// before the closer; the first `*/` ends the block, so text after it is not
/// the title either. Markdown and a docstring are not C-family block comments,
/// so nothing on their declaration line is envelope.
#[test]
fn a_declaration_on_a_one_line_block_comment_drops_the_closer() {
    let root = test_root("declaration_line_titles_block_closer");
    write(&root.join("src/control.rs"), "//! AR-001-control: Beta\n");
    write(
        &root.join("src/javadoc.rs"),
        "/** AR-002-javadoc: Beta */\npub struct B;\n",
    );
    write(&root.join("src/inner.rs"), "/*! AR-003-inner: Beta */\n");
    write(
        &root.join("src/cline.c"),
        "/* AR-004-cline: Beta */\nint c(void) { return 0; }\n",
    );
    write(
        &root.join("src/Wide.java"),
        "/** AR-005-wide: Beta   */\nclass Wide {}\n",
    );
    write(
        &root.join("src/tail.c"),
        "/* AR-006-tail: Beta */ tail */\nint t(void) { return 0; }\n",
    );
    write(
        &root.join("docs/AR-007-literal.md"),
        "# AR-007-literal: Beta */\n\nA Markdown title that ends in */ on purpose.\n",
    );
    write(
        &root.join("src/docstring.py"),
        "\"\"\"AR-008-docstring: Beta */\n\nA docstring title that ends in */.\n\"\"\"\n",
    );
    let config = embedded_value_config(root.clone());
    let (findings, errors) = scan_tree(&config, Some(&root), true).expect("scan closer fixture");
    assert!(errors.is_empty(), "fixture should be readable: {errors:?}");

    let beta = Some("Beta".to_string());
    assert_eq!(
        title(&findings, "control"),
        beta,
        "the line-comment control"
    );
    let kept_closer = ["javadoc", "inner", "cline", "wide", "tail"]
        .into_iter()
        .filter_map(|slug| {
            let recorded = title(&findings, slug);
            (recorded != beta).then(|| format!("AR-*-{slug}: {recorded:?}"))
        })
        .collect::<Vec<_>>();
    assert!(
        kept_closer.is_empty(),
        "a one-line block comment's closing `*/` is envelope, not title; expected {beta:?}:\n{}",
        kept_closer.join("\n")
    );
    for slug in ["literal", "docstring"] {
        assert_eq!(
            title(&findings, slug),
            Some("Beta */".to_string()),
            "AR-*-{slug}: a `*/` outside a C-family block comment is the author's"
        );
    }
}
