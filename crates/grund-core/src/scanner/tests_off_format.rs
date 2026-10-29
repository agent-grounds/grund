//! Test module: the off-format reading (§FS-citations.recognized.off-format) —
//! where its token ends, and which reading claims a marker before it does. Every
//! marker a fixture writes is spelled `\u{a7}`, so this file cites nothing it
//! illustrates.

use super::off_format::off_format_token;
use crate::config::load_config;
use crate::testing::{scan_findings, test_root, write};

/// `FS` and `REQ` under the default `{kind}-{slug}`, as the #1073 fixture has them.
const SLUG: &str = "grund_config_version = 2\nproject_name = \"slug\"\n\n\
                    [schema.kinds.FS]\nfolders = [\"docs/FS\"]\nindex = false\n\n\
                    [schema.kinds.REQ]\nfolders = [\"docs/REQ\"]\nindex = false\n";

/// `FS` under `{kind}-{number}-{slug}`, which has a number-only shorthand.
const NUMBERED: &str = "grund_config_version = 2\nproject_name = \"numbered\"\n\n\
                        [schema]\nid_format = \"{kind}-{number}-{slug}\"\n\n\
                        [schema.kinds.FS]\nfolders = [\"docs/FS\"]\nindex = false\n";

/// The off-format records a scan of `files` under `config` leaves, as `line: token`.
fn records(name: &str, config: &str, files: &[(&str, &str)]) -> Vec<String> {
    let root = test_root(name);
    write(&root.join("grund.toml"), config);
    for (path, text) in files {
        write(&root.join(path), text);
    }
    let config = load_config(&root).expect("load config");
    let findings = scan_findings(&config, &root);
    findings
        .off_format_citations
        .iter()
        .map(|record| format!("{}: {}", record.line, record.token))
        .collect()
}

/// The token runs over ASCII letters, digits, `-`, `_` and `.`, so a section path
/// is part of it, and one `.` that ends the run is sentence punctuation.
#[test]
fn the_token_ends_where_its_characters_do_less_one_trailing_dot() {
    for (tail, token) in [
        ("REQ-999 and more", "REQ-999"),
        ("REQ-999.", "REQ-999"),
        ("REQ-999.2, then", "REQ-999.2"),
        ("REQ-12a.3.1.", "REQ-12a.3.1"),
        ("REQ-1_a-b)", "REQ-1_a-b"),
        ("REQ-999..", "REQ-999."),
        ("REQ-<slug>", "REQ-"),
        ("REQ-\u{2026}", "REQ-"),
        ("REQ-\u{e9}t\u{e9}", "REQ-"),
    ] {
        assert_eq!(off_format_token(tail), token, "{tail:?}");
    }
}

/// A letter or digit must follow the separator, so a prefix mention stays text,
/// and a token no configured kind opens stays text too.
#[test]
fn nothing_after_the_separator_and_no_configured_kind_stay_text() {
    let page = "# FS-a: A\n\nPrefixes \u{a7}REQ- and \u{a7}REQ-<slug> and \u{a7}REQ-_x \
                and \u{a7}REQ-\u{2026}.\nUnconfigured \u{a7}XYZ-1 and \u{a7}REQUIRE-9.\n\
                Reported \u{a7}REQ-9.\n";
    let found = records(
        "off_format_prefix_mentions",
        SLUG,
        &[("docs/FS/FS-a.md", page)],
    );
    assert_eq!(found, ["5: REQ-9"]);
}

/// A full-ID citation, resolved or dangling, a pattern, a backed off-grammar
/// citation and an off-grammar heading's citation of its own ID each claim their
/// marker first, and only the token nothing else read is recorded.
#[test]
fn a_citation_or_a_pattern_at_the_marker_claims_it_first() {
    let page = "# FS-a: A\n\nResolved \u{a7}FS-a, dangling \u{a7}REQ-a12, pattern \u{a7}REQ-*.\n\
                Backed \u{a7}REQ-777, unbacked \u{a7}REQ-999.\n";
    let found = records(
        "off_format_claimed_markers",
        SLUG,
        &[
            ("docs/FS/FS-a.md", page),
            (
                "docs/REQ/REQ-777.md",
                "# REQ-777: Seven, see \u{a7}REQ-777\n",
            ),
        ],
    );
    assert_eq!(found, ["4: REQ-999"]);
}

/// Under a numbered format a number-only shorthand claims its marker and an
/// in-format heading's citation of its own ID is none, while a full ID off the
/// slug pattern is the one record.
#[test]
fn a_shorthand_of_the_format_is_never_off_format() {
    let page = "# FS-001-alpha: Alpha, see \u{a7}FS-001-alpha\n\n\
                Shorthand \u{a7}FS-042, off-pattern \u{a7}FS-042-User-Login.\n";
    let found = records(
        "off_format_numbered_readings",
        NUMBERED,
        &[("docs/FS/FS-001-alpha.md", page)],
    );
    assert_eq!(found, ["3: FS-042-User-Login"]);
}
