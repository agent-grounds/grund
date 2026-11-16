//! Canonical parsing selects a format independently of malformed-token name
//! boundaries (§FS-config.3.2); attribution remains unique
//! (§FS-citations.recognized.off-format, §FS-declarations.checks.declaration-near-miss.1).

use super::id_format::id_token_end_at;
use super::settings::GrammarKind;
use super::{Grammar, near_miss_heading, parse_id_arg};

fn grammar(default: &str, rows: &[(&str, &str)]) -> Grammar {
    let kinds = rows
        .iter()
        .map(|(name, format)| GrammarKind {
            name: (*name).into(),
            format: Some((*format).into()),
        })
        .collect::<Vec<_>>();
    Grammar::build(
        default,
        &kinds,
        r"\d+",
        "[a-z][a-z0-9-]*",
        ".",
        false,
        &["///".into()],
    )
    .expect("test grammar")
}

/// This bypasses config validation to expose the format-selection bug directly:
/// the shorter GR name sorts before GRUND but has different literals.
#[test]
#[ignore = "red until #1357 parsed-kind boundary selection; implement removes this"]
fn kind_heads_canonical_boundary_uses_longer_parsed_kind() {
    for reverse in [false, true] {
        let mut rows = [
            ("GR", "{kind}-{number}-{slug}"),
            ("GRUND", "{kind}:{number}:{slug}"),
        ];
        if reverse {
            rows.reverse();
        }
        let grammar = grammar("{kind}-{number}-{slug}", &rows);
        for (text, end, clean) in [
            ("GRUND:7:Upper", 7, false),
            ("GRUND:7.", 7, true),
            ("GRUND:7:why", 11, true),
            ("GR-7-Upper", 4, false),
            ("GR-7.", 4, true),
        ] {
            assert_eq!(grammar.id_token_ends_cleanly(text, end), clean, "{text}");
        }
    }
}

/// Already-loadable unrelated names isolate the author's alias case. Both
/// fmt's trigger and LSP's live transform call this exact shared entry point.
#[test]
#[ignore = "red until #1357 alias-qualified parsed-kind literals; implement removes this"]
fn kind_heads_alias_trigger_and_lsp_use_override_literals() {
    let grammar = grammar(
        "{kind}-{number}-{slug}",
        &[("ADR", "{kind}:{number}:{slug}")],
    );
    assert_eq!(id_token_end_at("ADR:7:Upper", 0, &grammar), None);
    assert_eq!(id_token_end_at("api/ADR:7:Upper", 0, &grammar), None);
}

/// A canonical grammar match needs no punctuation after the kind name, and
/// the name need not lead the token. The default's literals are not a fallback
/// for either supported shape.
#[test]
#[ignore = "red until #1357 parsed-kind selection for every format; implement removes this"]
fn kind_heads_override_boundary_supports_separator_free_and_non_leading() {
    for (format, token, end) in [
        ("{kind}{number}:{slug}", "ADR7:Upper", 4),
        ("{number}:{kind}:{slug}", "7:ADR:Upper", 5),
    ] {
        let grammar = grammar("{kind}-{number}-{slug}", &[("ADR", format)]);
        assert!(!grammar.id_token_ends_cleanly(token, end), "{token}");
    }
}

/// Controls for a valid qualified shorthand, sentence punctuation and full ID.
/// Separator-free and non-leading formats must use canonical parsing, not the
/// punctuation boundary used for malformed IDs.
#[test]
fn kind_heads_preserve_separator_free_non_leading_and_shorthand_parsing() {
    for (format, full, shorthand, off) in [
        ("{kind}{number}:{slug}", "ADR7:why", "ADR7", "ADR7:Upper"),
        (
            "{number}:{kind}:{slug}",
            "7:ADR:why",
            "7:ADR",
            "7:ADR:Upper",
        ),
        (
            "{kind}:{number}:{slug}",
            "ADR:7:why",
            "ADR:7",
            "ADR:7:Upper",
        ),
    ] {
        // The default deliberately agrees here: this is a passing safety control.
        let grammar = grammar(format, &[("ADR", format)]);
        assert_eq!(parse_id_arg(full, &grammar).unwrap().0.kind, "ADR");
        assert!(id_token_end_at(full, 0, &grammar).is_some());
        assert_eq!(
            id_token_end_at(shorthand, 0, &grammar),
            Some(shorthand.len())
        );
        assert_eq!(
            id_token_end_at(&format!("api/{shorthand}."), 0, &grammar),
            Some(shorthand.len() + 4)
        );
        assert_eq!(id_token_end_at(off, 0, &grammar), None);
    }
}

/// Off-format names are attributed once even when the short kind comes first.
#[test]
fn kind_heads_malformed_names_have_exactly_one_owner() {
    let grammar = grammar(
        "{kind}-{slug}",
        &[("GR", "{kind}-{number}"), ("GRUND", "{kind}-{slug}")],
    );
    for (token, owner) in [("GR_80923", "GR"), ("GRUND_why", "GRUND")] {
        assert_eq!(grammar.legacy_kind_and_format(token).unwrap().0, owner);
        for (prefix, docstring, markdown) in [
            ("# ", false, true),
            ("/// ", false, false),
            ("", true, false),
        ] {
            let line = format!("{prefix}{token}: Wrong separator");
            let found = near_miss_heading(&grammar, &line, docstring, markdown).expect("near miss");
            assert_eq!(found.0, token);
            assert_eq!(found.2, owner);
        }
    }
}
