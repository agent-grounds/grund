//! §FS-citations.wrap.enclosing.shape — where a paragraph begins, inside a block
//! quote too, and where a definition may start. Link text that runs onto the next
//! line of a quote is the author's link as it is outside one, and a reference whose
//! text runs across quoted lines keeps one key with its definition
//! (§FS-citations.zones.authored-markup). A line that only looks like a definition,
//! because it interrupts a paragraph or holds more than a title after its
//! destination, keeps the reading it had before grund#839
//! (§FS-citations.wrap.enclosing.scope): `fmt` wraps and re-derives there, and
//! `check` says "not a link".

use super::*;

/// Text across the lines of a block quote, in every form that runs on, and the
/// definitions that start their paragraph, each as the whole of `docs/notes.md`.
const ACROSS_LINES: &[(&str, &str)] = &[
    (
        "quoted-multiline-inline",
        "> See [the §ADR-use-postgres\n> record](adr/use-postgres.md).\n",
    ),
    (
        "quoted-multiline-full-reference",
        "> See [the §ADR-use-postgres\n> record][pg].\n>\n> [pg]: adr/use-postgres.md\n",
    ),
    (
        "quoted-multiline-collapsed-reference",
        "> See [the §ADR-use-postgres\n> record][].\n>\n\
         > [the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "quoted-multiline-shortcut-reference",
        "> See [the §ADR-use-postgres\n> record].\n>\n\
         > [the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "quoted-multiline-collapsed-trigger",
        "> See [the $$ADR-use-postgres\n> record][].\n>\n\
         > [the $$ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "quoted-definition",
        "> [the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "list-item-definition",
        "- [the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "titled-definition-after-a-blank-line",
        "Some prose.\n\n[the §ADR-use-postgres record]: adr/use-postgres.md \"the record\"\n",
    ),
    (
        "a-run-of-definitions",
        "[pg]: adr/use-postgres.md\n[the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
];

/// §FS-citations.wrap.enclosing.shape, §FS-citations.zones.authored-markup: each
/// keeps every byte under `fmt --write --marker --cross-refs`, the `$$` trigger
/// included, and `fmt --check` then lists nothing.
#[test]
fn quoted_text_across_lines_and_definitions_keep_their_bytes() {
    let failures = forms_rewritten(ACROSS_LINES);
    assert!(
        failures.is_empty(),
        "authored markup rewritten:\n{}",
        failures.join("\n")
    );
}

/// A `[label]:` line that interrupts a paragraph, in a quote or not, and one with
/// words after its destination, in a list item or not, is prose: `fmt --write`
/// wraps its citation.
#[test]
fn a_line_that_is_no_definition_is_wrapped_as_before() {
    let pg = format!("[§ADR-use-postgres]({PG_LINK})");
    let mut failures = Vec::new();
    for (name, notes, wrapped) in [
        (
            "lazy-quoted-label",
            "> A quoted paragraph runs on\n> [the §ADR-use-postgres record]: adr/use-postgres.md\n",
            format!("> A quoted paragraph runs on\n> [the {pg} record]: adr/use-postgres.md\n"),
        ),
        (
            "lazy-label",
            "A paragraph runs on\n[the §ADR-use-postgres record]: adr/use-postgres.md\n",
            format!("A paragraph runs on\n[the {pg} record]: adr/use-postgres.md\n"),
        ),
        (
            "words-after-the-destination-in-a-list-item",
            "- [WIP]: §ADR-use-postgres needs work\n",
            format!("- [WIP]: {pg} needs work\n"),
        ),
        (
            "words-after-the-destination",
            "[WIP]: §ADR-use-postgres needs work\n",
            format!("[WIP]: {pg} needs work\n"),
        ),
    ] {
        let dir = project(&format!("no-definition-{name}"), notes, &full_index());
        grund(&["fmt", "--write"], &dir);
        if read(&dir, "docs/notes.md") != wrapped {
            failures.push(format!(
                "{name}: {notes:?} became {:?}",
                read(&dir, "docs/notes.md")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// §FS-fmt.6.3: a stale wrap after a `[label]:` with words after it is a wrap like
/// any other, so `fmt --check` lists the line and `fmt --write` re-derives it.
#[test]
fn a_stale_wrap_after_a_label_that_is_no_definition_is_rederived() {
    for (name, prefix) in [("in-a-list-item", "- "), ("at-column-0", "")] {
        let stale = "[§ADR-use-redis](adr/use-redis.md#stale-anchor)";
        let dir = project(
            &format!("stale-after-a-label-{name}"),
            &format!("{prefix}[Stale]: {stale} is stale\n"),
            &full_index(),
        );
        let check = grund(&["fmt", "--check"], &dir);
        assert_eq!(check.status.code(), Some(1), "{name}: {}", stdout(&check));
        assert_eq!(stdout(&check), "docs/notes.md:1: markdown link\n", "{name}");
        grund(&["fmt", "--write"], &dir);
        assert_eq!(
            read(&dir, "docs/notes.md"),
            format!("{prefix}[Stale]: [§ADR-use-redis](adr/{REDIS_LINK}) is stale\n"),
            "{name}"
        );
    }
}

/// §FS-check.3.17: an index row that only looks like a definition holds a bare
/// citation, so `check` says it is not a link and names `fmt`, and the row `fmt`
/// writes is the entry.
#[test]
fn an_index_row_that_is_no_definition_is_not_a_link_fmt_repairs() {
    let mut failures = Vec::new();
    for (name, row, line) in [
        (
            "words-after-the-destination-in-a-list-item",
            "- [Cache]: §ADR-use-redis is the cache\n",
            2,
        ),
        (
            "words-after-the-destination",
            "\n[Cache]: §ADR-use-redis is the cache\n",
            3,
        ),
        (
            "lazy-label",
            "[the §ADR-use-redis record]: use-redis.md\n",
            2,
        ),
    ] {
        let dir = project(
            &format!("not-a-definition-{name}"),
            "",
            &format!("{PG_ROW}{row}"),
        );
        let check = grund(&["check"], &dir);
        let unlinked = format!(
            "docs/adr/README.md:{line}: error: index entry §ADR-use-redis is not a link; \
             unchecked in grund 0.11.0, an error in 0.12.0 — run `grund fmt --write`\n"
        );
        if check.status.code() != Some(1) || stdout(&check) != unlinked {
            failures.push(format!("{name}: check said {:?}", stdout(&check)));
        }
        grund(&["fmt", "--write"], &dir);
        if stdout(&grund(&["check"], &dir)) != "success\n" {
            failures.push(format!(
                "{name}: fmt wrote {:?}",
                read(&dir, "docs/adr/README.md")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
