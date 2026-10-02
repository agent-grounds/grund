//! Test module: a section heading written in a source comment is recorded with
//! the title the same heading gets in a Markdown file (§AR-scanner.2.2,
//! §FS-rules.5.1.1). The comment that carries the heading — `///`, `//!`, a
//! block comment's ` * `, `#`, `--`, `;` — says where the body lives, not what
//! the author titled the section, and the chapter fact and the `list --selector`
//! row both take their label from this one record.
//!
//! The e2e cases `check-rules-chapter-source-*` and `list-selector-source-*`
//! pin what a user sees of a named chapter. A numbered section reaches neither
//! surface, so its half of the agreement is pinned here, on the record itself.

use std::path::PathBuf;

use super::*;
use crate::config::Config;
use crate::model::Findings;
use crate::testing::{embedded_value_config, test_root, write};

/// One declaration per way a heading can be carried in source, each beside the
/// envelope its headings sit behind. The Python docstring is the control: its
/// lines reach the scanner already stripped of the quotes around them.
const ENVELOPES: &[(&str, &str)] = &[
    ("outer", "///"),
    ("inner", "//!"),
    ("javadoc", " * "),
    ("hash", "#"),
    ("sql", "--"),
    ("lisp", ";"),
    ("docstring", "\"\"\""),
];

/// `AR` declarations with named sections on, each carrying the same two
/// headings, `## terms: Terms` and `## 1. Overview`: once in Markdown, and once
/// in every envelope of `ENVELOPES`.
fn scan_envelopes(name: &str) -> Findings {
    let root = test_root(name);
    write(
        &root.join("docs/AR-001-markdown.md"),
        "# AR-001-markdown: the Markdown control\n\n\
         ## terms: Terms\n\nThe vocabulary.\n\n## 1. Overview\n\nNumbered.\n",
    );
    write(
        &root.join("src/outer.rs"),
        "/// AR-002-outer: an outer Rust doc-comment\n///\n\
         /// ## terms: Terms\n///\n/// The vocabulary.\n///\n\
         /// ## 1. Overview\n///\n/// Numbered.\npub fn outer() {}\n",
    );
    write(
        &root.join("src/inner.rs"),
        "//! AR-003-inner: an inner Rust doc-comment\n//!\n\
         //! ## terms: Terms\n//!\n//! The vocabulary.\n//!\n\
         //! ## 1. Overview\n//!\n//! Numbered.\n\npub fn inner() {}\n",
    );
    write(
        &root.join("src/Javadoc.java"),
        "/**\n * AR-004-javadoc: a block comment's continuation lines\n *\n\
         \x20* ## terms: Terms\n *\n * The vocabulary.\n *\n\
         \x20* ## 1. Overview\n *\n * Numbered.\n */\nclass Javadoc {}\n",
    );
    write(
        &root.join("src/hash.rb"),
        "# AR-005-hash: a hash comment\n#\n\
         # ## terms: Terms\n#\n# The vocabulary.\n#\n\
         # ## 1. Overview\n#\n# Numbered.\ndef hash_comment; end\n",
    );
    write(
        &root.join("src/sql.sql"),
        "-- AR-006-sql: a double-dash comment\n--\n\
         -- ## terms: Terms\n--\n-- The vocabulary.\n--\n\
         -- ## 1. Overview\n--\n-- Numbered.\nSELECT 1;\n",
    );
    write(
        &root.join("src/lisp.lisp"),
        "; AR-007-lisp: a semicolon comment\n;\n\
         ; ## terms: Terms\n;\n; The vocabulary.\n;\n\
         ; ## 1. Overview\n;\n; Numbered.\n(defun lisp-comment ())\n",
    );
    write(
        &root.join("src/docstring.py"),
        "def docstring():\n    \"\"\"AR-008-docstring: a Python docstring\n\n\
         \x20   ## terms: Terms\n\n    The vocabulary.\n\n\
         \x20   ## 1. Overview\n\n    Numbered.\n    \"\"\"\n",
    );
    let config = named_config(root.clone());
    let (findings, errors) = scan_tree(&config, Some(&root), true).expect("scan envelope fixture");
    assert!(errors.is_empty(), "fixture should be readable: {errors:?}");
    findings
}

fn named_config(root: PathBuf) -> Config {
    let mut config = embedded_value_config(root);
    config.named_sections = true;
    for kind in &mut config.kinds {
        if kind.kind == "AR" {
            kind.folder = Some("docs".to_string());
            kind.file = None;
        }
    }
    config.rebuild_grammar().expect("rebuild named grammar");
    config
}

/// Every recorded `(section path, title)` of the one `AR` declaration whose
/// slug is `slug`, in path order.
fn titles(findings: &Findings, slug: &str) -> Vec<(String, String)> {
    let decls = findings
        .declarations
        .iter()
        .filter(|(id, _)| id.kind == "AR" && id.slug.as_deref() == Some(slug))
        .flat_map(|(_, decls)| decls)
        .collect::<Vec<_>>();
    assert_eq!(
        decls.len(),
        1,
        "the fixture declares AR-*-{slug} exactly once"
    );
    decls[0]
        .sections
        .iter()
        .map(|(path, info)| (path.clone(), info.title.clone()))
        .collect()
}

/// §AR-scanner.2.2, §FS-rules.5.1.1: the same two headings record the same two
/// titles whatever comment carries them, for a named and a numbered path alike.
/// The Markdown record is the reference, and it is pinned to the bytes it holds
/// today, since a Markdown section's anchor is slugged from it
/// (§DF-md-link-anchor-strategy).
#[test]
fn a_heading_in_a_source_comment_records_its_markdown_title() {
    let findings = scan_envelopes("source_section_titles");
    let markdown = titles(&findings, "markdown");
    assert_eq!(
        markdown,
        vec![
            ("1".to_string(), "1 Overview".to_string()),
            ("terms".to_string(), "terms: Terms".to_string()),
        ],
        "a Markdown section's recorded title is unchanged"
    );

    let mismatched = ENVELOPES
        .iter()
        .filter_map(|(slug, envelope)| {
            let recorded = titles(&findings, slug);
            (recorded != markdown).then(|| format!("`{envelope}` (AR-*-{slug}): {recorded:?}"))
        })
        .collect::<Vec<_>>();
    assert!(
        mismatched.is_empty(),
        "a source heading must record the title its Markdown twin records, {markdown:?}, \
         and not the comment around it:\n{}",
        mismatched.join("\n")
    );
}
