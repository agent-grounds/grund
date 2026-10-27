//! Test module: the front-matter pass (§FS-fields.front-matter.reading,
//! §FS-fields.front-matter.keys, §FS-fields.front-matter.values,
//! §FS-fields.front-matter.coordinates): a block read through the one span
//! tree gives its top-level keys, decoded, in source order, each slice cut at
//! the file's own bytes and lines.

use super::front_matter::{Entry, read_block};
use crate::grammar::front_matter_extent;
use crate::model::{FrontMatterBlock, FrontMatterKeySite, FrontMatterUnreadable};

fn read(text: &str) -> (FrontMatterBlock, Vec<Entry>) {
    read_block(text, front_matter_extent(text.lines()))
}

fn keys(text: &str) -> Vec<FrontMatterKeySite> {
    match read(text).0 {
        FrontMatterBlock::Read(keys) => keys,
        other => panic!("expected a readable block, got {other:?}"),
    }
}

/// Each coordinate's path, line, title, slice and last line.
fn coordinates(text: &str) -> Vec<(String, usize, String, String, usize)> {
    read(text)
        .1
        .into_iter()
        .map(|(path, info)| {
            let slice = info.front_matter.expect("a front-matter entry");
            assert_eq!(info.heading_level, 0, "{path} has no heading");
            (path, info.line, info.title, slice.text, slice.end_line)
        })
        .collect()
}

fn unreadable(text: &str) -> FrontMatterUnreadable {
    match read(text).0 {
        FrontMatterBlock::Unreadable(reason) => reason,
        other => panic!("expected an unreadable block, got {other:?}"),
    }
}

/// §FS-fields.front-matter.values: quotes and escapes resolved, no YAML type
/// applied, a tag or anchor ignored, an empty value `""`.
#[test]
fn a_value_is_its_decoded_text_and_never_a_yaml_type() {
    let text = "---\na: 'accepted'\nb: \"acc\\u0065pted\"\nc: 1.10\nd: no\ne: !!str accepted\nf: &s accepted\ng:\n---\n";
    let values: Vec<(String, Option<String>)> = keys(text)
        .into_iter()
        .map(|key| (key.name, key.one_line))
        .collect();
    let expected = [
        ("a", "accepted"),
        ("b", "accepted"),
        ("c", "1.10"),
        ("d", "no"),
        ("e", "accepted"),
        ("f", "accepted"),
        ("g", ""),
    ];
    let expected: Vec<(String, Option<String>)> = expected
        .iter()
        .map(|(key, value)| (key.to_string(), Some(value.to_string())))
        .collect();
    assert_eq!(values, expected);
}

/// §FS-fields.front-matter.values: only a plain or quoted scalar that starts and
/// ends on its key's line is a one-line value.
#[test]
fn only_a_scalar_on_its_keys_line_is_one_line() {
    let text = "---\nblock: |\n  accepted\nfolded: >\n  accepted\nwrapped: accepted\n  later\nnext:\n  accepted\nseq: [accepted]\nmap: {a: b}\nalias: *x\n---\n";
    let text = text.replace("alias: *x", "anchor: &x accepted\nalias: *x");
    let one_line: Vec<(String, Option<String>)> = keys(&text)
        .into_iter()
        .map(|key| (key.name, key.one_line))
        .collect();
    assert_eq!(
        one_line,
        vec![
            ("block".to_string(), None),
            ("folded".to_string(), None),
            ("wrapped".to_string(), None),
            ("next".to_string(), None),
            ("seq".to_string(), None),
            ("map".to_string(), None),
            ("anchor".to_string(), Some("accepted".to_string())),
            ("alias".to_string(), None),
        ]
    );
}

/// §FS-fields.front-matter.keys: a key nested in a value is part of that value,
/// and a key that is no section name has no coordinate.
#[test]
fn only_top_level_keys_are_keys_and_only_section_names_are_coordinates() {
    let text = "---\nsources:\n  status: nested\nReviewer: Ana\n\"quoted\": x\n---\n";
    let names: Vec<(String, bool)> = keys(text)
        .into_iter()
        .map(|key| (key.name, key.section))
        .collect();
    assert_eq!(
        names,
        vec![
            ("sources".to_string(), true),
            ("Reviewer".to_string(), false),
            ("quoted".to_string(), true),
        ]
    );
    let paths: Vec<String> = coordinates(text).into_iter().map(|c| c.0).collect();
    assert_eq!(
        paths,
        [
            "front-matter",
            "front-matter.sources",
            "front-matter.quoted"
        ]
    );
}

/// §FS-fields.front-matter.keys: a key written twice is kept twice, in source
/// order, compared decoded.
#[test]
fn a_repeated_key_is_kept_in_order() {
    let text = "---\nstatus: a\ndate: x\n\"status\": b\n---\n";
    let sites: Vec<(String, usize)> = keys(text)
        .into_iter()
        .map(|key| (key.name, key.line))
        .collect();
    assert_eq!(
        sites,
        vec![
            ("status".to_string(), 2),
            ("date".to_string(), 3),
            ("status".to_string(), 4),
        ]
    );
}

/// §FS-fields.front-matter.coordinates: a slice is the file's own bytes, so a
/// comment of multi-byte characters before a key moves neither its line nor its
/// slice; a comment after the value is outside it, and the whole block runs
/// from delimiter to delimiter.
#[test]
fn slices_stay_exact_after_a_non_ascii_comment() {
    let text = "---\n# Entscheidung für Redis — geprüft\ndate: 2026-02-03\nstatus: accepted # nach Prüfung\n---\n\n# ADR-cache: Cache\n";
    let block = "---\n# Entscheidung für Redis — geprüft\ndate: 2026-02-03\nstatus: accepted # nach Prüfung\n---";
    assert_eq!(
        coordinates(text),
        vec![
            (
                "front-matter".to_string(),
                1,
                String::new(),
                block.to_string(),
                5
            ),
            (
                "front-matter.date".to_string(),
                3,
                "2026-02-03".to_string(),
                "date: 2026-02-03".to_string(),
                3
            ),
            (
                "front-matter.status".to_string(),
                4,
                "accepted".to_string(),
                "status: accepted".to_string(),
                4
            ),
        ]
    );
}

/// §FS-fields.front-matter.coordinates: a key's slice runs to its value's last
/// byte across lines, and a key with no value ends at its colon.
#[test]
fn a_slice_runs_to_its_values_end_or_its_colon() {
    let text = "---\nsources:\n  - a\n  - b\nempty:\n---\n";
    let slices: Vec<(String, String, usize)> = coordinates(text)
        .into_iter()
        .skip(1)
        .map(|(path, _, _, slice, end)| (path, slice, end))
        .collect();
    assert_eq!(
        slices,
        vec![
            (
                "front-matter.sources".to_string(),
                "sources:\n  - a\n  - b".to_string(),
                4
            ),
            ("front-matter.empty".to_string(), "empty:".to_string(), 5),
        ]
    );
}

/// §FS-fields.front-matter.reading: a CRLF file reads as its LF form does, and
/// no value or slice ends in a carriage return.
#[test]
fn a_crlf_block_reads_as_its_lf_form() {
    let lf = "---\nstatus: accepted\nsources:\n  - a\n---\n";
    let crlf = lf.replace('\n', "\r\n");
    assert_eq!(keys(&crlf), keys(lf));
    let lf_coordinates = coordinates(lf);
    let crlf_coordinates: Vec<_> = coordinates(&crlf)
        .into_iter()
        .map(|(path, line, title, slice, end)| {
            (path, line, title, slice.replace("\r\n", "\n"), end)
        })
        .collect();
    assert_eq!(crlf_coordinates, lf_coordinates);
    for (path, _, title, slice, _) in coordinates(&crlf) {
        assert!(
            !slice.ends_with('\r') && !title.ends_with('\r'),
            "{path}: {slice:?}"
        );
    }
}

/// §FS-fields.front-matter.reading: each reason a block cannot be read, at the
/// file line it is at; empty content is a block with no keys.
#[test]
fn an_unreadable_block_names_its_reason_and_line() {
    let yaml = |reason: &str, line: usize| FrontMatterUnreadable::Yaml {
        reason: reason.to_string(),
        line,
    };
    assert_eq!(
        unreadable("---\n- a\n---\n"),
        yaml("its top level is not a mapping", 2)
    );
    assert_eq!(
        unreadable("---\nstatus: a\n--- # x\ndate: b\n---\n"),
        yaml("a second document starts", 3)
    );
    assert_eq!(
        unreadable("+++\ntitle = 'x'\n+++\n"),
        FrontMatterUnreadable::Toml
    );
    let deep = format!("---\nk: {}{}\n---\n", "[".repeat(200), "]".repeat(200));
    assert!(matches!(
        unreadable(&deep),
        FrontMatterUnreadable::Yaml { reason, line: 2 } if reason.contains("nested more than 128")
    ));
    assert!(keys("---\n# only a comment\n\n---\n").is_empty());
    assert!(keys("---\n---\n").is_empty());
}
