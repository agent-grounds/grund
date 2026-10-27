//! Test module: the TOML span tree (§DA-toml-front-matter-reader.5) read through
//! the front-matter pass (§FS-fields.front-matter.reading,
//! §FS-fields.front-matter.keys, §FS-fields.front-matter.values,
//! §FS-fields.front-matter.coordinates): a `+++` block gives the same list of
//! key sites a `---` block does, each at its first contribution, its slice cut
//! at the file's own bytes and lines.

use super::front_matter::{Entry, read_block};
use crate::grammar::front_matter_extent;
use crate::model::{FrontMatterBlock, FrontMatterUnreadable};

fn read(text: &str) -> (FrontMatterBlock, Vec<Entry>) {
    read_block(text, front_matter_extent(text.lines()))
}

/// Each site's name, line and one-line value, a refused contribution included.
fn sites(text: &str) -> Vec<(String, usize, Option<String>)> {
    match read(text).0 {
        FrontMatterBlock::Read(keys) => keys
            .into_iter()
            .map(|key| (key.name, key.line, key.one_line))
            .collect(),
        other => panic!("expected a readable block, got {other:?}"),
    }
}

/// Each site's name and line.
fn lines(text: &str) -> Vec<(String, usize)> {
    sites(text)
        .into_iter()
        .map(|(name, line, _)| (name, line))
        .collect()
}

fn named(pairs: &[(&str, usize)]) -> Vec<(String, usize)> {
    pairs
        .iter()
        .map(|(name, line)| (name.to_string(), *line))
        .collect()
}

/// Each key coordinate's path, line, slice and last line, the whole block left
/// out; a second site of a key comes after its first, as the section map takes it.
fn slices(text: &str) -> Vec<(String, usize, String, usize)> {
    read(text)
        .1
        .into_iter()
        .skip(1)
        .map(|(path, info)| {
            let slice = info.front_matter.expect("a front-matter entry");
            (path, info.line, slice.text, slice.end_line)
        })
        .collect()
}

fn slice(path: &str, line: usize, text: &str, end: usize) -> (String, usize, String, usize) {
    (format!("front-matter.{path}"), line, text.to_string(), end)
}

fn unreadable(text: &str) -> FrontMatterUnreadable {
    match read(text).0 {
        FrontMatterBlock::Unreadable(reason) => reason,
        other => panic!("expected an unreadable block, got {other:?}"),
    }
}

fn toml(reason: &str, line: usize) -> FrontMatterUnreadable {
    FrontMatterUnreadable::Toml {
        reason: reason.to_string(),
        line,
    }
}

/// §FS-fields.front-matter.keys: a dotted key, a header and an array of tables
/// each write the key their first component names, at its first contribution.
#[test]
fn contributions_fold_into_one_key_at_its_first_site() {
    let text = "+++\nlinks.next = 'a'\nlinks.prev = 'b'\n\n[[authors]]\nname = 'x'\n\n\
                [[authors]]\nname = 'y'\n\n[meta.a]\nx = 1\n\n[meta]\ny = 2\n\n\
                [authors.extra]\nz = 3\n+++\n";
    assert_eq!(
        lines(text),
        named(&[("links", 2), ("authors", 5), ("meta", 11)])
    );
}

/// §FS-fields.front-matter.keys: quoted and bare names are one key, and a
/// quoted name holding a dot is one key that is no section.
#[test]
fn quoted_and_bare_names_are_one_key() {
    let text = "+++\n\"status\" = 'a'\n'date' = 1\n\"links.next\" = 'x'\n+++\n";
    assert_eq!(
        lines(text),
        named(&[("status", 2), ("date", 3), ("links.next", 4)])
    );
    let paths: Vec<String> = read(text).1.into_iter().map(|(path, _)| path).collect();
    assert_eq!(
        paths,
        ["front-matter", "front-matter.status", "front-matter.date"]
    );
    assert_eq!(
        lines("+++\nstatus = 'a'\n'status' = 'b'\n+++\n"),
        named(&[("status", 2), ("status", 3)])
    );
}

/// §FS-fields.front-matter.keys: every contribution TOML refuses because of
/// what the key already is is a second site of the key, at its own line, and an
/// allowed one is none.
#[test]
fn a_refused_contribution_is_a_second_site() {
    let refused = [
        ("k = 1\nk = 2\n", 3),
        ("k = 1\nk.x = 2\n", 3),
        ("k = 1\n[k]\n", 3),
        ("k = {a = 1}\n[k.x]\n", 3),
        ("k = {a = 1}\nk.y = 2\n", 3),
        ("k = [1]\n[[k]]\n", 3),
        ("k.x = 1\nk = 2\n", 3),
        ("k.x = 1\n[k]\n", 3),
        ("k.x = 1\n[[k]]\n", 3),
        ("[k]\n[k]\n", 3),
        ("[k]\n[[k]]\n", 3),
        ("[k.x]\n[[k]]\n", 3),
        ("[[k.x]]\n[[k]]\n", 3),
        ("[[k]]\n[k]\n", 3),
        ("[k.x]\n[k]\n[k]\n", 4),
    ];
    for (content, second) in refused {
        let text = format!("+++\n{content}+++\n");
        assert_eq!(
            lines(&text),
            named(&[("k", 2), ("k", second)]),
            "{content:?}"
        );
    }
    let allowed = [
        "k.x = 1\nk.y = 2\n",
        "k.x = 1\n[k.y]\n",
        "k.x = 1\n[[k.y]]\n",
        "[k]\n[k.y]\n",
        "[k.x]\n[k.y]\n",
        "[k.x]\n[k]\n",
        "[[k.x]]\n[k]\n",
        "[[k]]\n[[k]]\n",
        "[[k]]\n[k.x]\n",
        "[k.x]\n[k.x]\n",
    ];
    for content in allowed {
        let text = format!("+++\n{content}+++\n");
        assert_eq!(lines(&text), named(&[("k", 2)]), "{content:?}");
    }
}

/// §FS-fields.front-matter.values: a string is its decoded text, every other
/// scalar its raw spelling; a multi-line string, an array, an inline table and
/// a table have no one-line value.
#[test]
fn a_value_keeps_its_spelling_and_a_string_is_decoded() {
    let text = "+++\na = 'x'\nb = \"acc\\u0065pted\"\nc = 1_000\nd = 0x1F\ne = 1.10\n\
                f = true\ng = 1979-05-27T07:32:00Z\nh = \"\"\ni = \"\"\"y\"\"\"\n\
                j = \"\"\"\nz\"\"\"\nk = [1]\nl = {m = 1}\nn.o = 'p'\n[q]\nr = 's'\n+++\n";
    let values: Vec<(String, Option<String>)> = sites(text)
        .into_iter()
        .map(|(name, _, value)| (name, value))
        .collect();
    let expected = [
        ("a", Some("x")),
        ("b", Some("accepted")),
        ("c", Some("1_000")),
        ("d", Some("0x1F")),
        ("e", Some("1.10")),
        ("f", Some("true")),
        ("g", Some("1979-05-27T07:32:00Z")),
        ("h", Some("")),
        ("i", Some("y")),
        ("j", None),
        ("k", None),
        ("l", None),
        ("n", None),
        ("q", None),
    ];
    let expected: Vec<(String, Option<String>)> = expected
        .iter()
        .map(|(name, value)| (name.to_string(), value.map(str::to_string)))
        .collect();
    assert_eq!(values, expected);
}

/// §FS-fields.front-matter.coordinates: a slice starts at the key's first byte,
/// its opening quote, or a header's `[` or `[[`, runs over the key's first run,
/// and ends at its last value, before a trailing comment or blank line.
#[test]
fn a_slice_runs_over_the_first_run_and_ends_at_its_last_value() {
    let text = "+++\n\"status\" = 'a' # after review\nlinks.next = 'x'\nlinks.prev = 'y'\n\n\
                [meta]\nk = 1 # first\n\n# trailing\n\n[[authors]]\nname = 'p'\n\n\
                [[authors]]\nname = 'q'\n\n[meta.extra]\nh = 2\n\n[empty]\n+++\n";
    assert_eq!(
        slices(text),
        [
            slice("status", 2, "\"status\" = 'a'", 2),
            slice("links", 3, "links.next = 'x'\nlinks.prev = 'y'", 4),
            slice("meta", 6, "[meta]\nk = 1", 7),
            slice(
                "authors",
                11,
                "[[authors]]\nname = 'p'\n\n[[authors]]\nname = 'q'",
                15
            ),
            slice("empty", 20, "[empty]", 20),
        ]
    );
}

/// §FS-fields.front-matter.coordinates: a refused contribution ends its key's
/// first run, so the first slice never holds the second site, which has its own.
#[test]
fn a_refused_contribution_ends_the_first_run() {
    let text = "+++\n[k]\na = 1\n\n[k]\nb = 2\n+++\n";
    assert_eq!(
        slices(text),
        [
            slice("k", 2, "[k]\na = 1", 3),
            slice("k", 5, "[k]\nb = 2", 6)
        ]
    );
}

/// §DA-toml-front-matter-reader.2.3: spans are bytes, so a non-ASCII comment or
/// string above a key moves no later slice.
#[test]
fn slices_stay_exact_after_non_ascii_text() {
    let text = "+++\n# Überblick — ü\nauthor = 'José Müller'\nstatus = 'accepted'\n\
                [links]\nnext = 'ñ'\n+++\n";
    assert_eq!(
        slices(text),
        [
            slice("author", 3, "author = 'José Müller'", 3),
            slice("status", 4, "status = 'accepted'", 4),
            slice("links", 5, "[links]\nnext = 'ñ'", 6),
        ]
    );
}

/// §FS-fields.front-matter.reading: a CRLF block reads as its LF form does, and
/// no value or slice ends in a carriage return.
#[test]
fn a_crlf_block_reads_as_its_lf_form() {
    let lf = "+++\nstatus = 'accepted' # x\ndate = 2026-02-03\n\n[links]\nnext = 'a'\n\n+++\n";
    let crlf = lf.replace('\n', "\r\n");
    assert_eq!(sites(&crlf), sites(lf));
    let folded: Vec<_> = slices(&crlf)
        .into_iter()
        .map(|(path, line, slice, end)| (path, line, slice.replace("\r\n", "\n"), end))
        .collect();
    assert_eq!(folded, slices(lf));
    for (path, _, slice, _) in slices(&crlf) {
        assert!(!slice.ends_with('\r'), "{path}: {slice:?}");
    }
}

/// §FS-fields.front-matter.reading: the first parse or decoder error in source
/// order, at the file line of the first byte it did not expect; a string still
/// open when the content ends is at the closing delimiter's line.
#[test]
fn an_unreadable_block_names_the_first_error_and_its_line() {
    assert_eq!(
        unreadable("+++\nstatus = 'a'\n\ndate = accepted\nnext = \n+++\n"),
        toml("string values must be quoted", 4)
    );
    assert_eq!(
        unreadable("+++\n# Überblick\nnotes = \"\"\"\nopen\n+++\n"),
        toml("invalid multi-line basic string", 5)
    );
    assert!(sites("+++\n# only a comment\n\n+++\n").is_empty());
    assert!(sites("+++\n+++\n").is_empty());
}

/// §FS-fields.front-matter.reading, §DA-toml-front-matter-reader.2.4: arrays
/// and inline tables nest to the YAML tree's one bound, counted with the top
/// level, and past it are refused at the line that goes past, never overflowed.
#[test]
fn nesting_past_the_bound_is_refused_and_never_overflows() {
    let nested = |open: &str, close: &str, depth: usize| {
        format!(
            "+++\nstatus = 'a'\nnotes = {}{}\n+++\n",
            open.repeat(depth),
            close.repeat(depth)
        )
    };
    let reason = "collections are nested more than 128 deep";
    assert_eq!(
        lines(&nested("[", "]", 127)),
        named(&[("status", 2), ("notes", 3)])
    );
    assert_eq!(unreadable(&nested("[", "]", 128)), toml(reason, 3));
    assert_eq!(unreadable(&nested("[", "]", 10_000)), toml(reason, 3));
    let tables = format!(
        "+++\nnotes = {}1{}\n+++\n",
        "{ a = ".repeat(10_000),
        " }".repeat(10_000)
    );
    assert_eq!(unreadable(&tables), toml(reason, 2));
}

/// §FS-fields.front-matter.keys, §REQ-never-crashes.1: only a key's first
/// component is kept, so a dotted key or a header of 10 000 components is read
/// in one pass, adds no nesting level, and overflows nothing.
#[test]
fn a_key_of_many_components_is_read_flat() {
    let path = vec!["a"; 10_000].join(".");
    let text = format!("+++\n{path} = 1\n[{path}x]\n[[b.{path}]]\n+++\n");
    assert_eq!(lines(&text), named(&[("a", 2), ("b", 4)]));
}
