//! Test module: what the scanner *records* about an inline citation site
//! (§FS-inline-citation-style.7.1) — the enclosing block's span, its measured
//! width, whether it carries a note, and the lines that fail the configured
//! layout.
//!
//! Asserted on the `Citation` the scan returns rather than through a checker
//! finding, because §7.1 is a claim about the record: every other suite reads
//! these fields through a verdict, which cannot distinguish a field that is
//! wrong from a rule that never fired.

use crate::model::InlineCitationSite;
use crate::testing::{
    checked_layout_config, legacy_fs_folder_config, scan_findings, test_root, write,
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

/// §FS-inline-citation-style.7.1: a citation in an ordinary comment block
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
            "// §FS-001-login: reject an expired credential\n",
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
            has_note: true,
            layout_violations: Vec::new(),
        })]
    );
}

/// §FS-inline-citation-style.7.1: multiple citations in the same block carry
/// the same span — the record is the block's, not each citation's, so the
/// widest line of the block is every one of its citations' `max_columns` and
/// the opening prose line is inside the span although it cites nothing.
#[test]
fn two_citations_in_one_block_carry_one_span() {
    let found = sites(
        "two_citations_in_one_block_carry_one_span",
        concat!(
            "// Walks the credential store.\n",
            "// §FS-001-login: one error per expired credential.\n",
            "// §FS-002-session — and one more, laid out wrong.\n",
            "pub fn sweep() {}\n",
        ),
        false,
    );
    let expected = Some(InlineCitationSite {
        first_line: 1,
        last_line: 3,
        max_columns: 51,
        has_note: true,
        layout_violations: Vec::new(),
    });
    assert_eq!(found, vec![expected.clone(), expected]);
}

/// §FS-inline-citation-style.7.1: a doc-comment block records no inline
/// citation site at all — the same `None` a Markdown citation carries, and for
/// the same reason a declaring block does: what the language calls
/// documentation is not a note.
#[test]
fn a_doc_comment_block_records_no_site() {
    let found = sites(
        "a_doc_comment_block_records_no_site",
        concat!(
            "//! §FS-002-session: the module's own documentation.\n",
            "\n",
            "/// §FS-001-login: reject an expired credential.\n",
            "pub fn login() {}\n",
        ),
        false,
    );
    assert_eq!(found, vec![None, None]);
}

/// §FS-inline-citation-style.7.1: `has_note` is the block's, not the
/// citation's — a comment that is nothing but citations carries none.
#[test]
fn a_block_of_bare_citations_records_no_note() {
    let found = sites(
        "a_block_of_bare_citations_records_no_note",
        concat!("// §FS-001-login\n", "pub fn login() {}\n"),
        false,
    );
    assert_eq!(
        found,
        vec![Some(InlineCitationSite {
            first_line: 1,
            last_line: 1,
            max_columns: 16,
            has_note: false,
            layout_violations: Vec::new(),
        })]
    );
}

/// §FS-inline-citation-style.7.1: beside the four fields, the site records the
/// block's judged lines that fail the configured layout, 1-based and ascending
/// — and every citation of the block carries that same list.
#[test]
fn the_recorded_layout_violations_are_ascending_and_shared_by_the_block() {
    let found = sites(
        "the_recorded_layout_violations_are_ascending_and_shared_by_the_block",
        concat!(
            "// §FS-001-login: this line opens the note correctly.\n",
            "// §FS-002-session — and this one does not.\n",
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
