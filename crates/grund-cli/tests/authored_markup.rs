//! §FS-citations.wrap.enclosing.scope — a citation in every form a Markdown link is
//! written in, and in a citation-only image, is left with the author's markup by
//! `fmt`, and `check` credits only the forms that go somewhere
//! (§FS-citations.wrap.enclosing): from grund 0.20.0 a citation-only image keeps no
//! credit at all (§FS-citations.checks.image-index-entry).
//!
//! These run the real binary, because the defect is a write: `fmt --write`, then
//! `fmt --check`, then `check`, on the bytes a user would hold. They port the
//! fixtures of grund#839's reproducer.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "authored_markup/paragraphs.rs"]
mod paragraphs;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn grund(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn grund")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// A fresh tree holding `files`, each `(path, text)`.
fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = manifest_dir().join("target/authored-markup").join(name);
    let _ = fs::remove_dir_all(&dir);
    for (path, text) in files {
        let file = dir.join(path);
        fs::create_dir_all(file.parent().expect("a parent")).expect("create parent");
        fs::write(file, text).expect("write fixture file");
    }
    dir
}

fn read(dir: &Path, path: &str) -> String {
    fs::read_to_string(dir.join(path)).expect("read fixture file")
}

const V2_TOML: &str = "grund_config_version = 2\nproject_name = \"records\"\n\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\n\
    [schema.kinds.ADR]\nfolders = [\"docs/adr\"]\n";
const POSTGRES: &str = "# ADR-use-postgres: Use Postgres for all data\n\nWe use Postgres.\n";
const REDIS: &str = "# ADR-use-redis: Use Redis for the cache\n\nWe use Redis.\n";
const SRC: &str = "// Stores data in Postgres (§ADR-use-postgres) and caches in Redis (§ADR-use-redis).\n\
    pub fn store() {}\n";
/// The index row for Postgres, already the canonical wrap, so no pass touches it.
const PG_ROW: &str =
    "- [§ADR-use-postgres](use-postgres.md#adr-use-postgres-use-postgres-for-all-data)\n";
const PG_LINK: &str = "adr/use-postgres.md#adr-use-postgres-use-postgres-for-all-data";
const REDIS_LINK: &str = "use-redis.md#adr-use-redis-use-redis-for-the-cache";

/// The project with `notes` as `docs/notes.md` and `index` as the kind's index.
fn project(name: &str, notes: &str, index: &str) -> PathBuf {
    tree(
        name,
        &[
            ("grund.toml", V2_TOML),
            ("docs/adr/use-postgres.md", POSTGRES),
            ("docs/adr/use-redis.md", REDIS),
            ("docs/adr/README.md", index),
            ("docs/notes.md", notes),
            ("src/lib.rs", SRC),
        ],
    )
}

/// The index both declarations are entries of, so a note is all that can move.
fn full_index() -> String {
    format!("{PG_ROW}- [§ADR-use-redis]({REDIS_LINK})\n")
}

/// Every form §FS-citations.wrap.enclosing.scope lists, the citation-only image of
/// §FS-citations.wrap.enclosing.images, and the two shapes of §FS-citations.wrap.enclosing.shape
/// the review of grund#838 raised, each as the whole of `docs/notes.md`.
const AUTHORED: &[(&str, &str)] = &[
    (
        "full-reference",
        "See [the §ADR-use-postgres rationale][pg].\n\n[pg]: adr/use-postgres.md\n",
    ),
    (
        "full-reference-citation-only",
        "See [§ADR-use-postgres][pg].\n\n[pg]: adr/use-postgres.md\n",
    ),
    (
        "full-reference-unresolved",
        "See [the §ADR-use-postgres rationale][pg].\n",
    ),
    (
        "collapsed-reference",
        "See [the §ADR-use-postgres record][].\n\n[the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "collapsed-reference-defined-first",
        "[The §ADR-use-postgres  Record]: adr/use-postgres.md\n\nSee [the §ADR-use-postgres record][].\n",
    ),
    (
        "collapsed-reference-unresolved",
        "See [the §ADR-use-postgres record][].\n",
    ),
    (
        "shortcut-reference",
        "See [the §ADR-use-postgres record].\n\n[the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "definition-label",
        "[the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "definition-in-a-block-quote",
        "> See [the §ADR-use-postgres record][].\n>\n> [the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "definition-in-a-list-item",
        "- See [the §ADR-use-postgres record][].\n- [the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "definition-in-a-nested-container",
        "> 1. [the §ADR-use-postgres record]: adr/use-postgres.md\n",
    ),
    (
        "definition-destination-and-title",
        "[pg]: https://example.com/§ADR-use-postgres \"the §ADR-use-postgres record\"\n",
    ),
    ("autolink", "See <https://example.com/§ADR-use-postgres>.\n"),
    (
        "html-anchor",
        "<a href=\"adr/use-postgres.md\">the §ADR-use-postgres record</a>\n",
    ),
    (
        "html-anchor-uppercase-and-attribute",
        "<A title=\"§ADR-use-postgres\" href=\"adr/use-postgres.md\">the §ADR-use-postgres record</A>\n",
    ),
    (
        "html-anchor-unclosed",
        "<a href=\"adr/use-postgres.md\">the §ADR-use-postgres record\n",
    ),
    (
        "multiline-citation-first",
        "See [the §ADR-use-postgres\nrecord](adr/use-postgres.md).\n",
    ),
    (
        "multiline-citation-second",
        "See [the record of\n§ADR-use-postgres](adr/use-postgres.md).\n",
    ),
    ("citation-only-image", "![§ADR-use-redis](cache.png)\n"),
    (
        "link-inside-the-text",
        "See [a [b](c.md) §ADR-use-postgres d](adr/use-postgres.md).\n",
    ),
    (
        "backtick-that-never-closes",
        "Unclosed code [the §ADR-use-postgres `record](adr/use-postgres.md).\n",
    ),
];

/// §FS-citations.wrap.enclosing.scope, §FS-citations.wrap.enclosing.images,
/// §FS-citations.zones.authored-markup, §FS-fmt.6.4: every authored form keeps
/// every byte under the fullest run, `fmt --write --marker --cross-refs`, and
/// `fmt --check` then lists nothing. Today each but the link inside the text
/// gains a wrap, and the image loses `cache.png`.
#[test]
fn every_authored_form_keeps_its_bytes() {
    let failures = forms_rewritten(AUTHORED);
    assert!(
        failures.is_empty(),
        "authored markup rewritten:\n{}",
        failures.join("\n")
    );
}

/// Each of `forms`, as the whole of `docs/notes.md`, that the fullest run
/// rewrites or `fmt --check` then lists.
fn forms_rewritten(forms: &[(&str, &str)]) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, notes) in forms {
        let dir = project(&format!("form-{name}"), notes, &full_index());
        let write = grund(&["fmt", "--write", "--marker", "--cross-refs"], &dir);
        let after = read(&dir, "docs/notes.md");
        if !write.status.success() || after != *notes {
            failures.push(format!("{name}: {notes:?} became {after:?}"));
            continue;
        }
        let check = grund(&["fmt", "--check", "--marker", "--cross-refs"], &dir);
        if !check.status.success() || !stdout(&check).is_empty() {
            failures.push(format!("{name}: fmt --check listed {:?}", stdout(&check)));
        }
    }
    failures
}

/// §FS-citations.wrap.enclosing.shape: each occurrence is read by its own position,
/// so a bare citation beside a reference is still wrapped, and the second run of
/// `fmt --write` writes nothing (§FS-fmt.6.3).
#[test]
fn a_bare_citation_beside_authored_markup_is_still_wrapped() {
    let notes = "See [the §ADR-use-postgres rationale][pg] and §ADR-use-postgres.\n\n\
                 [pg]: adr/use-postgres.md\n";
    let dir = project("bare-beside-reference", notes, &full_index());
    grund(&["fmt", "--write"], &dir);
    assert_eq!(
        read(&dir, "docs/notes.md"),
        format!(
            "See [the §ADR-use-postgres rationale][pg] and [§ADR-use-postgres]({PG_LINK}).\n\n\
             [pg]: adr/use-postgres.md\n"
        )
    );
    assert_eq!(
        stdout(&grund(&["fmt", "--write"], &dir)),
        "rewrote 0 lines\n"
    );
}

/// §FS-citations.wrap.enclosing.shape: a `[^` opens a footnote, not a definition's
/// label, so a citation in a footnote's text is prose: `fmt` wraps it, and a second
/// write is a no-op.
#[test]
fn a_citation_in_a_footnotes_text_is_wrapped() {
    let notes = "A claim.[^1]\n\n[^1]: See §ADR-use-postgres for why.\n";
    let dir = project("footnote-text", notes, &full_index());
    grund(&["fmt", "--write"], &dir);
    assert_eq!(
        read(&dir, "docs/notes.md"),
        format!("A claim.[^1]\n\n[^1]: See [§ADR-use-postgres]({PG_LINK}) for why.\n")
    );
    assert_eq!(
        stdout(&grund(&["fmt", "--write"], &dir)),
        "rewrote 0 lines\n"
    );
}

/// The controls #839 must not move: a bare citation is wrapped, a stale wrap is
/// re-derived (§FS-fmt.6.3), brackets no definition matches are text
/// (§FS-citations.wrap.enclosing.shape), a citation in a longer image description is
/// wrapped (§FS-citations.wrap.enclosing.images), and a second write is a no-op.
#[test]
fn bare_citations_and_managed_wraps_are_maintained_as_before() {
    let notes = "See §ADR-use-postgres.\n\nSee [§ADR-use-postgres](adr/old.md).\n\n\
                 See [the §ADR-use-postgres record].\n\n![a diagram of §ADR-use-redis](d.png)\n";
    let dir = project("controls", notes, &full_index());
    grund(&["fmt", "--write"], &dir);
    assert_eq!(
        read(&dir, "docs/notes.md"),
        format!(
            "See [§ADR-use-postgres]({PG_LINK}).\n\nSee [§ADR-use-postgres]({PG_LINK}).\n\n\
             See [the [§ADR-use-postgres]({PG_LINK}) record].\n\n\
             ![a diagram of [§ADR-use-redis](adr/{REDIS_LINK})](d.png)\n"
        )
    );
    assert_eq!(
        stdout(&grund(&["fmt", "--write"], &dir)),
        "rewrote 0 lines\n"
    );
    let check = grund(&["fmt", "--check"], &dir);
    assert!(
        check.status.success() && stdout(&check).is_empty(),
        "{}",
        stdout(&check)
    );
}

/// §FS-citations.zones.authored-markup: a collapsed reference's text and its
/// definition's label are one key, so no pass rewrites either — not the shorthand
/// expansion, not `--marker` off strict mode, not the trigger — and `check` withholds
/// the shorthand demand there (§FS-citations.rewrite-exemption). A full reference's
/// visible text is no key, and its shorthand is still expanded.
#[test]
fn a_collapsed_reference_and_its_definition_keep_one_key() {
    let keys = "See [the §ADR-001 record][].\n\n[the §ADR-001 record]: adr/ADR-001-postgres.md\n\n\
                And [the ADR-001-postgres record][].\n\n\
                [the ADR-001-postgres record]: adr/ADR-001-postgres.md\n\n\
                And [the $$ADR-001-postgres record][].\n\n\
                [the $$ADR-001-postgres record]: adr/ADR-001-postgres.md\n";
    let visible = "See [the §ADR-001 rationale][pg].\n\n[pg]: adr/ADR-001-postgres.md\n";
    let dir = tree(
        "collapsed-key",
        &[
            (
                "grund.toml",
                "grund_config_version = 1\n\n[reference]\nstrict = false\n\n\
                 [[kinds]]\nkind = \"ADR\"\nfolder = \"docs/adr\"\n\n\
                 [scan]\ninclude = [\"docs\", \"src\"]\n",
            ),
            (
                "docs/adr/ADR-001-postgres.md",
                "# ADR-001-postgres: Use Postgres for all data\n\nWe use Postgres.\n",
            ),
            (
                "docs/adr/README.md",
                "- [§ADR-001-postgres: Use Postgres](ADR-001-postgres.md)\n",
            ),
            ("docs/keys.md", keys),
            ("docs/visible.md", visible),
            ("src/lib.rs", "// §ADR-001-postgres\npub fn store() {}\n"),
        ],
    );
    let check = grund(&["check"], &dir);
    assert_eq!(
        stdout(&check),
        "docs/visible.md:1: error: shorthand citation §ADR-001; write §ADR-001-postgres\n",
        "only the visible text owes the expansion"
    );
    grund(&["fmt", "--write", "--marker", "--cross-refs"], &dir);
    assert_eq!(
        read(&dir, "docs/keys.md"),
        keys,
        "both spellings of each key keep every byte"
    );
    assert_eq!(
        read(&dir, "docs/visible.md"),
        "See [the §ADR-001-postgres rationale][pg].\n\n[pg]: adr/ADR-001-postgres.md\n"
    );
    let dry = grund(&["fmt", "--check", "--marker", "--cross-refs"], &dir);
    assert!(
        dry.status.success() && stdout(&dry).is_empty(),
        "{}",
        stdout(&dry)
    );
}

/// The §FS-check.3.18 finding for `ADR-use-redis` where the index mentions it and holds
/// no entry for it: it appears, but not as an entry (§FS-check.3.18.5).
const REDIS_APPEARS: &str = "docs/adr/use-redis.md:1: error: ADR-use-redis appears in docs/adr/README.md \
     but not as an entry: an entry is a `§`-marked Markdown link to the declaration — became an \
     error in grund 0.13.0\n";

/// §FS-check.3.18.5 by §FS-citations.wrap.enclosing: in a kind's index, a reference
/// whose key a definition matches and a complete link whose text runs onto a second
/// line go somewhere, so each is the entry; `check` never says "is not a link"
/// and `fmt --write` writes nothing. Today each is reported as not a link, and the
/// command it names nests a second link into it.
#[test]
fn index_rows_that_go_somewhere_are_entries() {
    let mut failures = Vec::new();
    for (name, row) in [
        (
            "full",
            "- [the §ADR-use-redis record][redis]\n\n[redis]: use-redis.md\n",
        ),
        (
            "collapsed",
            "- [the §ADR-use-redis record][]\n\n[the §ADR-use-redis record]: use-redis.md\n",
        ),
        (
            "shortcut",
            "- [the §ADR-use-redis record]\n\n[the §ADR-use-redis record]: use-redis.md\n",
        ),
        (
            "multiline-first",
            "- [the §ADR-use-redis\n  record](use-redis.md)\n",
        ),
        (
            "multiline-second",
            "- [the record of\n  §ADR-use-redis](use-redis.md)\n",
        ),
    ] {
        let index = format!("{PG_ROW}{row}");
        let dir = project(&format!("entry-{name}"), "", &index);
        let check = grund(&["check"], &dir);
        if !check.status.success() || stdout(&check) != "success\n" {
            failures.push(format!("{name}: check said {:?}", stdout(&check)));
        }
        grund(&["fmt", "--write"], &dir);
        if read(&dir, "docs/adr/README.md") != index {
            failures.push(format!(
                "{name}: fmt rewrote it to {:?}",
                read(&dir, "docs/adr/README.md")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// §FS-citations.wrap.enclosing.shape: the link inside the link's text keeps the
/// balanced-bracket reading grund#838 gave it, an entry `fmt` leaves.
#[test]
fn a_link_inside_an_index_rows_text_stays_an_entry() {
    let index = format!("{PG_ROW}- [a [b](c.md) §ADR-use-redis d](use-redis.md)\n");
    let dir = project("entry-link-inside-the-text", "", &index);
    assert_eq!(stdout(&grund(&["check"], &dir)), "success\n");
    assert_eq!(
        stdout(&grund(&["fmt", "--write"], &dir)),
        "rewrote 0 lines\n"
    );
}

/// §FS-citations.wrap.enclosing, §FS-check.3.18: a protected form that goes
/// nowhere — a reference no definition matches, a definition's label, which starts
/// its paragraph, in a list item too, an autolink, an HTML anchor, a label a lone
/// backtick leaves unclosed — is left by `fmt` and is no entry, so the ID is reported missing, at the declaration, in the bytes that
/// finding has shipped in, and never as "is not a link".
#[test]
fn index_rows_that_go_nowhere_are_missing_entries() {
    let missing = REDIS_APPEARS;
    let mut failures = Vec::new();
    for (name, row) in [
        (
            "unresolved-reference",
            "- [the §ADR-use-redis record][redis]\n",
        ),
        (
            "definition-label",
            "\n[the §ADR-use-redis record]: use-redis.md\n",
        ),
        (
            "list-item-definition-label",
            "- [the §ADR-use-redis record]: use-redis.md\n",
        ),
        ("autolink", "- <https://example.com/§ADR-use-redis>\n"),
        (
            "html-anchor",
            "- <a href=\"use-redis.md\">the §ADR-use-redis record</a>\n",
        ),
        ("backtick", "- [the §ADR-use-redis `record](use-redis.md)\n"),
    ] {
        let index = format!("{PG_ROW}{row}");
        let dir = project(&format!("nowhere-{name}"), "", &index);
        let check = grund(&["check"], &dir);
        if check.status.code() != Some(1) || stdout(&check) != missing {
            failures.push(format!("{name}: check said {:?}", stdout(&check)));
        }
        grund(&["fmt", "--write"], &dir);
        if read(&dir, "docs/adr/README.md") != index {
            failures.push(format!(
                "{name}: fmt rewrote it to {:?}",
                read(&dir, "docs/adr/README.md")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// §FS-citations.checks.image-index-entry, §FS-citations.wrap.enclosing.images: from
/// grund 0.20.0 a citation-only image that was the ID's only entry is no entry, so the ID
/// is reported missing, an error, in the "appears" words of §FS-check.3.18.5, and never
/// as `image-index-entry` or as not a link; `fmt` keeps `cache.png`; and the link the
/// 0.19.0 warning named, written beside the image, is the entry.
#[test]
fn a_citation_only_image_is_no_entry_and_a_link_beside_it_is() {
    let index = format!("{PG_ROW}- ![§ADR-use-redis](cache.png)\n");
    let dir = project("image-entry", "", &index);
    let check = grund(&["check"], &dir);
    assert_eq!(
        check.status.code(),
        Some(1),
        "an error, exit 1: {}",
        stdout(&check)
    );
    assert_eq!(stdout(&check), REDIS_APPEARS);
    let json = stdout(&grund(&["check", "--format", "json"], &dir));
    assert!(
        json.contains("\"code\":\"missing-index-entry\"")
            && json.contains("\"severity\":\"error\""),
        "{json}"
    );
    assert!(
        !json.contains("image-index-entry") && !json.contains("unlinked-index-entry"),
        "{json}"
    );
    assert_eq!(
        stdout(&grund(&["fmt", "--write"], &dir)),
        "rewrote 0 lines\n"
    );
    assert_eq!(
        read(&dir, "docs/adr/README.md"),
        index,
        "the image keeps cache.png"
    );
    fs::write(
        dir.join("docs/adr/README.md"),
        format!("{index}- [§ADR-use-redis]({REDIS_LINK})\n"),
    )
    .expect("write the named entry");
    assert_eq!(stdout(&grund(&["check"], &dir)), "success\n");
}

/// §FS-citations.checks.image-index-entry: from grund 0.20.0 a citation-only image
/// whose destination is the canonical link to an external source declaration no longer
/// enrolls it (§FS-check.3.18.4). The declaration is no member, so the image is an
/// ordinary citation of it and nothing is reported: no warning, and no uncited note
/// naming an index entry.
#[test]
fn a_citation_only_image_enrolls_nothing() {
    let dir = tree(
        "image-enrollment",
        &[
            ("grund.toml", V2_TOML),
            ("docs/adr/use-postgres.md", POSTGRES),
            (
                "docs/adr/README.md",
                &format!("{PG_ROW}- ![§ADR-inline](../../src/lib.rs)\n"),
            ),
            (
                "src/lib.rs",
                "/// ADR-inline: Keep the inline record\n///\n/// Stores in Postgres (§ADR-use-postgres).\n\
                 pub fn store() {}\n",
            ),
        ],
    );
    let check = grund(&["check"], &dir);
    assert!(check.status.success(), "{}", stdout(&check));
    assert_eq!(stdout(&check), "success\n", "ADR-inline is still enrolled");
}

/// §FS-citations.wrap.enclosing.images: an index row a `fmt` already wrapped inside a
/// longer image description keeps its credit through the inner wrap, and the image
/// beside another entry draws no warning.
#[test]
fn a_wrapped_image_description_keeps_its_credit() {
    let index = format!("{PG_ROW}- ![a diagram of [§ADR-use-redis]({REDIS_LINK})](d.png)\n");
    let dir = project("image-description-credit", "", &index);
    assert_eq!(stdout(&grund(&["check"], &dir)), "success\n");
    assert_eq!(
        stdout(&grund(&["fmt", "--write"], &dir)),
        "rewrote 0 lines\n"
    );
}

/// §FS-show.3.2.1: `show` collapses only a wrap, so a reference, a collapsed
/// reference and a citation-only image in a body print as written.
#[test]
fn show_prints_authored_markup_as_written() {
    let body = "See [the §ADR-use-postgres rationale][pg].\n\n\
                See [the §ADR-use-postgres record][].\n\n\
                ![§ADR-use-postgres](cache.png)\n\n\
                [pg]: use-postgres.md\n\n[the §ADR-use-postgres record]: use-postgres.md\n";
    let dir = tree(
        "show-authored",
        &[
            ("grund.toml", V2_TOML),
            ("docs/adr/use-postgres.md", POSTGRES),
            ("docs/adr/use-redis.md", &format!("{REDIS}\n{body}")),
            ("docs/adr/README.md", &full_index()),
            ("src/lib.rs", SRC),
        ],
    );
    let shown = stdout(&grund(&["ADR-use-redis", "--full"], &dir));
    for line in body.lines().filter(|line| !line.is_empty()) {
        assert!(
            shown.contains(line),
            "`{line}` is printed as written: {shown}"
        );
    }
}
