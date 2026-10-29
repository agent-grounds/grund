//! Test module: what the scanner *records* about an inline citation site
//! (§AR-scanner.3.1) — the enclosing block's span, its measured
//! width, whether it carries a note, and the lines that fail the configured
//! layout.
//!
//! Asserted on the `Citation` the scan returns rather than through a checker
//! finding, because §AR-scanner.3.1 is a claim about the record: every other suite reads
//! these fields through a verdict, which cannot distinguish a field that is
//! wrong from a rule that never fired.

use crate::model::InlineCitationSite;
use crate::testing::{
    canonical_test_path, checked_layout_config, legacy_fs_folder_config, scan_findings, test_root,
    write,
};

/// Two declarations plus one source file, and the citations the scan found in
/// that source file, in encounter order.
fn sites(name: &str, source: &str, gated_layout: bool) -> Vec<Option<InlineCitationSite>> {
    let root = test_root(name);
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(
        &root.join("docs/functional-spec/FS-002-session.md"),
        "# FS-002-session: Session\n",
    );
    write(&root.join("src/auth.rs"), source);
    let config = if gated_layout {
        checked_layout_config(root.clone(), "citation-first-colon")
    } else {
        legacy_fs_folder_config(root.clone())
    };
    let findings = scan_findings(&config, &root);
    findings
        .citations
        .iter()
        .filter(|citation| citation.file.ends_with("auth.rs"))
        .map(|citation| citation.inline_site.clone())
        .collect()
}

/// §AR-scanner.3.1: a citation in an ordinary comment block
/// carries that block's `(first_line, last_line, max_columns, has_note)`.
///
/// `max_columns` is 46 for a line of 47 bytes: the `§` marker costs one column,
/// not two, so the recorded width is a character count and a fixture that
/// happened to be ASCII would not have shown the difference.
#[test]
fn an_inline_note_records_its_own_block_span_and_width() {
    let found = sites(
        "an_inline_note_records_its_own_block_span_and_width",
        concat!(
            "// \u{a7}FS-001-login: reject an expired credential\n",
            "pub fn login() {}\n",
        ),
        false,
    );
    assert_eq!(
        found,
        vec![Some(InlineCitationSite {
            first_line: 1,
            last_line: 1,
            max_columns: 46,
            line_columns: vec![46].into(),
            has_note: true,
            layout_violations: Vec::new(),
        })]
    );
}

/// §AR-scanner.3.1: multiple citations in the same block carry
/// the same span — the record is the block's, not each citation's, so the
/// widest line of the block is every one of its citations' `max_columns` and
/// the opening prose line is inside the span although it cites nothing.
#[test]
fn two_citations_in_one_block_carry_one_span() {
    let found = sites(
        "two_citations_in_one_block_carry_one_span",
        concat!(
            "// Walks the credential store.\n",
            "// \u{a7}FS-001-login: one error per expired credential.\n",
            "// \u{a7}FS-002-session — and one more, laid out wrong.\n",
            "pub fn sweep() {}\n",
        ),
        false,
    );
    let expected = Some(InlineCitationSite {
        first_line: 1,
        last_line: 3,
        max_columns: 51,
        line_columns: vec![30, 51, 50].into(),
        has_note: true,
        layout_violations: Vec::new(),
    });
    assert_eq!(found, vec![expected.clone(), expected]);
}

/// §AR-scanner.3.1: a doc-comment block records no inline
/// citation site at all — the same `None` a Markdown citation carries, and for
/// the same reason a declaring block does: what the language calls
/// documentation is not a note.
#[test]
fn a_doc_comment_block_records_no_site() {
    let found = sites(
        "a_doc_comment_block_records_no_site",
        concat!(
            "//! \u{a7}FS-002-session: the module's own documentation.\n",
            "\n",
            "/// \u{a7}FS-001-login: reject an expired credential.\n",
            "pub fn login() {}\n",
        ),
        false,
    );
    assert_eq!(found, vec![None, None]);
}

/// §AR-scanner.3.1: `has_note` is the block's, not the
/// citation's — a comment that is nothing but citations carries none.
#[test]
fn a_block_of_bare_citations_records_no_note() {
    let found = sites(
        "a_block_of_bare_citations_records_no_note",
        concat!("// \u{a7}FS-001-login\n", "pub fn login() {}\n"),
        false,
    );
    assert_eq!(
        found,
        vec![Some(InlineCitationSite {
            first_line: 1,
            last_line: 1,
            max_columns: 16,
            line_columns: vec![16].into(),
            has_note: false,
            layout_violations: Vec::new(),
        })]
    );
}

/// §AR-scanner.3.1: beside the four fields, the site records the
/// block's judged lines that fail the configured layout, 1-based and ascending
/// — and every citation of the block carries that same list.
#[test]
fn the_recorded_layout_violations_are_ascending_and_shared_by_the_block() {
    let found = sites(
        "the_recorded_layout_violations_are_ascending_and_shared_by_the_block",
        concat!(
            "// \u{a7}FS-001-login: this line opens the note correctly.\n",
            "// \u{a7}FS-002-session — and this one does not.\n",
            "pub fn sweep() {}\n",
        ),
        true,
    );
    let violations = found
        .iter()
        .map(|site| site.as_ref().expect("a site").layout_violations.clone())
        .collect::<Vec<_>>();
    assert_eq!(violations, vec![vec![2], vec![2]]);
    assert!(
        found
            .iter()
            .all(|site| site.as_ref().expect("a site").first_line == 1),
        "one span for the whole block: {found:?}"
    );
}

/// §AR-scanner.3.1, §FS-inline-citation-style.4.1.1.1: every block form retains
/// all physical lines, including uncited lines, blank interiors and delimiters.
/// Widths count Unicode scalars, tabs and indentation (§FS-inline-citation-style.2.3.2).
/// A custom source with no doc profile lets a docstring be an inline site;
/// Python's own docstrings remain excluded by their existing classification.
#[test]
fn every_block_form_retains_widths_with_physical_line_association() {
    use crate::config::load_config;

    let root = test_root("every_block_form_retains_widths_with_physical_line_association");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n\
         [schema.kinds.FS]\nfolders = [\"docs\"]\n\
         [schema.sources]\nlanguages = [\"markdown\", \"python\", \"notes\"]\n\
         [schema.sources.extensions]\nnote = \"notes\"\n\
         [schema.sources.definitions.notes]\nclass = \"source\"\n\
         comment_prefixes = [\"//\", \"#\", \"/*\", \"*\"]\n\
         docstrings = \"python\"\ndoc_comments = \"none\"\n",
    );
    write(
        &root.join("docs/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    let cases: &[(&str, &str, &[usize])] = &[
        (
            "hash",
            "\t# é界e\u{301}\n\t#\n\t# \u{a7}FS-001-login\n",
            &[7, 2, 16],
        ),
        (
            "slash",
            "\t// é界e\u{301}\n\t//\n\t// \u{a7}FS-001-login\n",
            &[8, 3, 17],
        ),
        (
            "closed",
            "  /* é界e\u{301}\n\n * \u{a7}FS-001-login\n */\n",
            &[9, 0, 16, 3],
        ),
        (
            "bare",
            "  /* é界e\u{301}\n\n\u{a7}FS-001-login\n */\n",
            &[9, 0, 13, 3],
        ),
        (
            "unclosed",
            "  /* é界e\u{301}\n\n * \u{a7}FS-001-login\n",
            &[9, 0, 16],
        ),
        (
            "docstring",
            "  \"\"\"é界e\u{301}\n\n\u{a7}FS-001-login\n  \"\"\"\n",
            &[9, 0, 13, 5],
        ),
    ];
    for (name, block, _) in cases {
        write(
            &root.join(format!("src/{name}.note")),
            &format!("code\n\n{block}"),
        );
    }
    write(
        &root.join("src/doc.py"),
        "\"\"\"\u{a7}FS-001-login\nDocumentation.\n\"\"\"\n",
    );
    let config = load_config(&root).expect("custom source config");
    let findings = scan_findings(&config, &root);
    for (name, _, widths) in cases {
        let path = canonical_test_path(&root.join(format!("src/{name}.note")));
        let citations = findings
            .citations
            .iter()
            .filter(|citation| citation.file == path)
            .collect::<Vec<_>>();
        assert_eq!(citations.len(), 1, "one citation in {name}");
        let site = citations[0].inline_site.as_ref().expect("an inline site");
        assert_eq!(site.first_line, 3, "{name}");
        assert_eq!(site.last_line, 2 + widths.len(), "{name}");
        assert_eq!(site.line_columns.as_ref(), *widths, "{name}");
        assert_eq!(site.max_columns, *widths.iter().max().unwrap(), "{name}");
    }
    let docs = findings
        .citations
        .iter()
        .filter(|citation| citation.file.ends_with("doc.py"))
        .collect::<Vec<_>>();
    assert_eq!(docs.len(), 1);
    assert!(
        docs[0].inline_site.is_none(),
        "Python documentation is not a note"
    );
}
