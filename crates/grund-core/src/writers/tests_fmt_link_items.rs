//! Test module: what the link pass records for each link it writes, which the
//! `markdown link` row's items are read from (§FS-fmt.3.6). The pass sees the
//! real spans, so the span rule is pinned here rather than by searching a row's
//! text (§FS-fmt.3.6), and so is the bracket rule an item is read by
//! (§FS-fmt.3.6).
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4).

use super::fmt_links::{WrappedLine, wrap_markdown_links_with_targets};
use super::fmt_row::LineRewrite;
use super::{FmtLineOpts, fmt_line};
use crate::config::{Config, load_config};
use crate::grammar::{DocstringCursor, LineLinks};
use crate::model::Catalog;
use crate::resolver::{SettledHeadings, ShorthandTargets};
use crate::testing::{scan_findings, test_root, write};

/// A project whose `FS` kind numbers its IDs without a slug, so `<§>FS-1` is the
/// start of `<§>FS-10`, and declares `FS-1` and `FS-2` but not `FS-10`.
fn project(name: &str) -> (crate::testing::Rooted<Config>, Catalog) {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"links\"\n\n\
         [schema]\nid_format = \"{kind}-{number}\"\n\n\
         [schema.kinds.FS]\nfolders = [\"docs/specs\"]\nindex = false\n",
    );
    write(&root.join("docs/specs/FS-1.md"), "# FS-1: Login\n\nBody.\n");
    write(
        &root.join("docs/specs/FS-2.md"),
        "# FS-2: Logout\n\nBody.\n",
    );
    let config = load_config(&root).expect("load config");
    let findings = scan_findings(&config, &root);
    (root.hold(config), findings)
}

/// The pass over `line` in `docs/notes.md`, with the record it keeps.
fn pass(config: &Config, findings: &Catalog, line: &str) -> WrappedLine {
    let targets = ShorthandTargets::new(config.frame(), Some(findings), None);
    wrap_markdown_links_with_targets(
        line,
        LineLinks::default(),
        &config.root.join("docs/notes.md"),
        config,
        findings,
        None,
        None,
        &targets,
        &SettledHeadings::default(),
    )
}

/// The link `--write` puts around a lone `§<id>` in `docs/notes.md`.
fn link(config: &Config, findings: &Catalog, id: &str) -> String {
    pass(config, findings, &format!("\u{a7}{id}")).line
}

/// §FS-fmt.3.6: the spans stand in source order, each after the last; a fresh
/// one is one whole citation token and a re-pointed one runs from `[` to its `)`;
/// and replacing each with the text written gives the line the write leaves.
fn assert_spans(read: &str, wrapped: &WrappedLine) {
    let mut rebuilt = String::new();
    let mut cursor = 0;
    for written in &wrapped.links {
        let span = written.span.clone();
        assert!(
            span.start >= cursor,
            "{span:?} starts before {cursor} in {read:?}"
        );
        let replaced = written.replaced(read);
        assert!(
            replaced.starts_with('\u{a7}')
                || (replaced.starts_with("[\u{a7}") && replaced.ends_with(')')),
            "{replaced:?} is neither a citation token nor a whole wrap"
        );
        rebuilt.push_str(&read[cursor..span.start]);
        rebuilt.push_str(&written.written);
        cursor = span.end;
    }
    rebuilt.push_str(&read[cursor..]);
    assert_eq!(rebuilt, wrapped.line, "the spans of {read:?}");
}

/// The row a line whose only kind is `markdown link` prints.
fn row(read: &str, wrapped: &WrappedLine) -> String {
    let mut rewrite = LineRewrite::default();
    wrapped.account(read, &mut rewrite);
    rewrite.label()
}

/// §FS-fmt.3.6: `<§>FS-1` is the start of the unresolved `<§>FS-10`, which the
/// pass leaves alone; the span is the `<§>FS-1` token after it, not the first
/// place its text stands.
#[test]
fn a_span_is_the_citation_token_and_not_the_first_match_of_its_text() {
    let (config, findings) = project("a_span_is_the_citation_token_and_not_the_first_match");
    let read = "See \u{a7}FS-10, then \u{a7}FS-1.";
    let wrapped = pass(&config, &findings, read);
    assert_spans(read, &wrapped);
    assert_eq!(wrapped.links.len(), 1);
    let start = read.rfind('\u{a7}').expect("the second marker");
    assert_eq!(wrapped.links[0].span, start..read.len() - 1);
    let fs1 = link(&config, &findings, "FS-1");
    assert_eq!(wrapped.line, format!("See \u{a7}FS-10, then {fs1}."));
    assert_eq!(
        row(read, &wrapped),
        format!("markdown link: \u{a7}FS-1 \u{2192} {fs1}")
    );
}

/// §FS-fmt.3.6: a link already right writes nothing and is not
/// listed; the fresh citation after it, with the same text, is the one span.
#[test]
fn a_correct_link_beside_a_fresh_one_lists_only_the_fresh_one() {
    let (config, findings) = project("a_correct_link_beside_a_fresh_one_lists_only_the_fresh_one");
    let fs1 = link(&config, &findings, "FS-1");
    let read = format!("See {fs1}, then \u{a7}FS-1 again.");
    let wrapped = pass(&config, &findings, &read);
    assert_spans(&read, &wrapped);
    assert_eq!(wrapped.links.len(), 1);
    assert_eq!(wrapped.links[0].replaced(&read), "\u{a7}FS-1");
    assert!(wrapped.links[0].span.start > fs1.len());
    assert_eq!(
        row(&read, &wrapped),
        format!("markdown link: \u{a7}FS-1 \u{2192} {fs1}")
    );
}

/// §FS-fmt.3.6: a re-pointed link's left side runs to the `)` that closes it,
/// title and all, so a title holding `, ` stays inside its one item; the write
/// drops it, and the item shows that.
#[test]
fn a_dropped_title_holding_a_comma_stays_inside_its_item() {
    let (config, findings) = project("a_dropped_title_holding_a_comma_stays_inside_its_item");
    let (fs1, fs2) = (
        link(&config, &findings, "FS-1"),
        link(&config, &findings, "FS-2"),
    );
    let old = "[\u{a7}FS-1](specs/FS-1.md \"Login, again\")";
    let read = format!("See {old} and \u{a7}FS-2.");
    let wrapped = pass(&config, &findings, &read);
    assert_spans(&read, &wrapped);
    assert_eq!(wrapped.links.len(), 2);
    assert_eq!(wrapped.links[0].replaced(&read), old);
    assert_eq!(wrapped.links[0].written, fs1);
    assert_eq!(
        row(&read, &wrapped),
        format!("markdown link: {old} \u{2192} {fs1}, \u{a7}FS-2 \u{2192} {fs2}")
    );
}

/// §FS-fmt.3.6: a stale wrap and a fresh citation on one line, each span
/// replaced in source order, and every reading of the record agrees on the count.
#[test]
fn a_stale_wrap_and_a_fresh_citation_are_two_spans_in_source_order() {
    let (config, findings) = project("a_stale_wrap_and_a_fresh_citation_are_two_spans");
    let read = "| [\u{a7}FS-1](specs/FS-1.md#stale) and \u{a7}FS-2 | mixed |";
    let wrapped = pass(&config, &findings, read);
    assert_spans(read, &wrapped);
    assert_eq!(wrapped.links.len(), 2);
    assert_eq!(
        wrapped.links[0].replaced(read),
        "[\u{a7}FS-1](specs/FS-1.md#stale)"
    );
    assert_eq!(wrapped.links[1].replaced(read), "\u{a7}FS-2");
    let mut rewrite = LineRewrite::default();
    wrapped.account(read, &mut rewrite);
    assert_eq!(rewrite.counts.links_written, 2);
    assert_eq!(rewrite.links.len(), 2);
    assert_eq!(rewrite.anchor_detail, None);
}

/// §FS-fmt.3.6: on a line `--marker` marked earlier in the run, the span is
/// the citation as the marking left it, and the row names it after the marking.
#[test]
fn a_line_marked_earlier_in_the_run_names_the_marked_citation() {
    let (config, findings) = project("a_line_marked_earlier_in_the_run_names_the_marked_citation");
    let fs1 = link(&config, &findings, "FS-1");
    let marked = "Read \u{a7}FS-1 first.";
    assert_spans(marked, &pass(&config, &findings, marked));
    let targets = ShorthandTargets::new(config.frame(), Some(&findings), None);
    let (line, label) = fmt_line(
        "Read FS-1 first.",
        DocstringCursor::default(),
        &config.root.join("docs/notes.md"),
        &config,
        true,
        &FmtLineOpts {
            add_marker: true,
            cross_refs: true,
            excluded: false,
            index_entry_ids: None,
            findings: Some(&findings),
            workspace: None,
            shorthand_targets: &targets,
            settled: &SettledHeadings::default(),
        },
        false,
        &mut false,
    );
    assert_eq!(line, format!("Read {fs1} first."));
    assert_eq!(
        label,
        format!("bare \u{2192} marker, markdown link: \u{a7}FS-1 \u{2192} {fs1}")
    );
}
