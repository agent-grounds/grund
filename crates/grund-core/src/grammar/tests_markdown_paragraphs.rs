//! Test module: where a paragraph of a Markdown document begins, which the
//! document reading of §FS-citations.wrap.enclosing.shape reads a link's text and
//! a definition by. A quoted line continues its quote's paragraph, a lazy line
//! continues the paragraph it follows, a definition only starts a paragraph, and
//! only a title closed on its line may follow a definition's destination.

use super::*;

/// The reading of the citation `FS-x` on line `index` of `doc`, the document
/// placing that line.
fn reading(doc: &str, index: usize) -> CitationLink {
    let links = MarkdownLinks::new(doc);
    let line = doc.lines().nth(index).expect("the line");
    let start = line.find('\u{a7}').expect("a citation");
    citation_link(links.line(index), line, start, start + "\u{a7}FS-x".len())
}

const LINKED: CitationLink = CitationLink::Enclosing { entry: true };
const LABEL: CitationLink = CitationLink::Enclosing { entry: false };

/// A link's text runs onto the next line of its paragraph inside a block quote as
/// outside one: a quoted line of the same quote, a line with fewer `>` markers and
/// a line with none continue it.
#[test]
fn a_quoted_line_continues_its_quotes_paragraph() {
    for doc in [
        "> See [the \u{a7}FS-x\n> record](x.md).",
        "> > See [the \u{a7}FS-x\n> > record](x.md).",
        "> > See [the \u{a7}FS-x\n> record](x.md).",
        "> See [the \u{a7}FS-x\nrecord](x.md).",
        "> - See [the \u{a7}FS-x\n>   record](x.md).",
    ] {
        assert_eq!(reading(doc, 0), LINKED, "{doc:?}");
    }
}

/// What ends a paragraph ends it inside a quote too — a blank quoted line, a list
/// item, a heading — and a quote of its own, nested or after plain text, opens one.
#[test]
fn a_quoted_line_that_opens_a_block_ends_the_paragraph() {
    for doc in [
        "> See [the \u{a7}FS-x\n>\n> record](x.md).",
        "> See [the \u{a7}FS-x\n> - record](x.md).",
        "> See [the \u{a7}FS-x\n> # record](x.md).",
        "> See [the \u{a7}FS-x\n> > record](x.md).",
        "See [the \u{a7}FS-x\n> record](x.md).",
    ] {
        assert_eq!(reading(doc, 0), CitationLink::Unwrapped, "{doc:?}");
    }
}

/// A collapsed reference whose text runs across quoted lines spells the key its
/// definition in the quote spells on one line: the `>` markers are the quote's,
/// not the key's (§FS-citations.zones.authored-markup).
#[test]
fn a_key_spelled_across_quoted_lines_is_its_definitions_key() {
    let doc = "> See [the \u{a7}FS-x\n> record][].\n>\n> [the \u{a7}FS-x record]: x.md";
    assert_eq!(reading(doc, 0), LINKED);
    assert_eq!(reading(doc, 3), LABEL);
}

/// A definition never interrupts a paragraph: after a line of text, in a quote or
/// lazily after a list item, `[label]: …` is prose. It starts a paragraph after a
/// blank line, a heading, as a list item or a quote, and after another definition.
#[test]
fn a_definition_only_starts_a_paragraph() {
    for doc in [
        "Text runs on\n[the \u{a7}FS-x record]: x.md",
        "> Text runs on\n> [the \u{a7}FS-x record]: x.md",
        "- An item\n[the \u{a7}FS-x record]: x.md",
    ] {
        assert_eq!(reading(doc, 1), CitationLink::Unwrapped, "{doc:?}");
    }
    for doc in [
        "Text.\n\n[the \u{a7}FS-x record]: x.md",
        "# Title\n[the \u{a7}FS-x record]: x.md",
        "Text.\n- [the \u{a7}FS-x record]: x.md",
        "Text.\n> [the \u{a7}FS-x record]: x.md",
        "[a]: a.md\n[the \u{a7}FS-x record]: x.md",
        "> [a]: a.md\n> [the \u{a7}FS-x record]: x.md",
    ] {
        let last = doc.lines().count() - 1;
        assert_eq!(reading(doc, last), LABEL, "{doc:?}");
    }
}

/// After its destination a definition holds at most a title closed on its line;
/// a line with anything else, or with no destination, is no definition, and its
/// citations are read as anywhere else.
#[test]
fn only_a_title_may_follow_a_definitions_destination() {
    for tail in [
        "x.md",
        "x.md \"the title\"",
        "x.md 'the title'",
        "x.md (the title)",
        "<x y.md> \"the title\"",
        "x(1).md",
        "<>",
    ] {
        let doc = format!("[the \u{a7}FS-x record]: {tail}");
        assert_eq!(reading(&doc, 0), LABEL, "{doc:?}");
    }
    for tail in [
        "",
        "x.md and words",
        "x.md \"unclosed",
        "x.md \"the title\" and words",
        "x.md (a (nested) title)",
        "<x.md>\"no space\"",
        "x.md)",
    ] {
        let doc = format!("[the \u{a7}FS-x record]: {tail}");
        assert_eq!(reading(&doc, 0), CitationLink::Unwrapped, "{doc:?}");
    }
    assert_eq!(
        reading("- [WIP]: \u{a7}FS-x needs work", 0),
        CitationLink::Unwrapped
    );
    assert!(matches!(
        reading("- [Stale]: [\u{a7}FS-x](x.md#stale) is stale", 0),
        CitationLink::Wrap(_)
    ));
}
