//! §FS-config-explain.5: a path is read against the version's own tables. One
//! with exactly one reading is answered as its bracketed spelling is
//! (§FS-config-explain.1.1); one with more, and a bare name that tables hold,
//! fails naming every spelling that answers. Reported as agent-grounds/grund#1005.

use super::*;

fn ask(version: u32, target: &str) -> ExplainAnswer {
    answer(version, &ExplainAsk::Target(target.to_string()))
}

fn unknown_of(version: u32, target: &str) -> ExplainUnknown {
    match ask(version, target) {
        ExplainAnswer::Unknown(unknown) => unknown,
        other => panic!("v{version} {target}: answered {other:?}"),
    }
}

/// The bracketed spellings a failure names, in the order it names them: each
/// quoted string of its debug form that opens with `[`, so each one is carried
/// whole, as `'[schema.sources] exclude'` is printed.
fn spellings(unknown: &ExplainUnknown) -> Vec<String> {
    let shown = format!("{unknown:?}");
    let quoted = shown.split('"').skip(1).step_by(2);
    quoted
        .filter(|text| text.starts_with('['))
        .map(str::to_string)
        .collect()
}

/// §FS-config-explain.1.1: a path with exactly one reading answers what its
/// bracketed spelling answers, a single segment from the top included, a name
/// kept for its placeholder and the longest table prefix taken.
#[test]
#[ignore = "red until #1005 answers a path with one reading; implement removes this"]
fn a_path_with_one_reading_is_answered_as_its_bracketed_spelling() {
    for (version, target, bracketed) in [
        (2, "schema", "[schema]"),
        (2, "schema.sources", "[schema.sources]"),
        (2, "schema.sources.exclude", "[schema.sources] exclude"),
        (2, "schema.kinds", "[schema.kinds]"),
        (2, "schema.kinds.FS", "[schema.kinds.FS]"),
        (2, "schema.kinds.ADR", "[schema.kinds.ADR]"),
        (2, "schema.kinds.FS.files", "[schema.kinds.FS] files"),
        (
            2,
            "schema.kinds.FS.id_format",
            "[schema.kinds.FS] id_format",
        ),
        (
            2,
            "rules.citations.grounding",
            "[rules.citations.grounding]",
        ),
        (2, "presentation.fmt", "[presentation.fmt]"),
        (2, "workspace", "[workspace]"),
        (1, "reference", "[reference]"),
        (1, "reference.strict", "[reference] strict"),
        (
            1,
            "reference.lead_size_warning.max",
            "[reference] lead_size_warning.max",
        ),
        (1, "fmt.cross_refs", "[fmt.cross_refs]"),
        (1, "workspace.members", "[workspace] members"),
    ] {
        let expected = ask(version, bracketed);
        assert!(
            !matches!(expected, ExplainAnswer::Unknown(_)),
            "v{version} {bracketed} is no answer: {expected:?}"
        );
        assert_eq!(
            format!("{:?}", ask(version, target)),
            format!("{expected:?}"),
            "v{version} {target} vs {bracketed}"
        );
    }
}

/// §FS-config-explain.5: where the longest run fills a placeholder of a table's
/// header, the path also reads with the name left out, and that reading is
/// named first; neither is answered.
#[test]
#[ignore = "red until #1005 reads a filled placeholder with the name left out; implement removes this"]
fn a_path_with_two_readings_names_both_the_name_left_out_first() {
    for (version, target, named) in [
        (
            2,
            "schema.kinds.id_format",
            [
                "[schema.kinds.<NAME>] id_format",
                "[schema.kinds.id_format]",
            ],
        ),
        (
            2,
            "presentation.kinds.title",
            [
                "[presentation.kinds.<NAME>] title",
                "[presentation.kinds.title]",
            ],
        ),
        (
            2,
            "rules.citations.must",
            ["[rules.citations.<KIND>] must", "[rules.citations.must]"],
        ),
        (
            2,
            "schema.kinds.fields",
            ["[schema.kinds.<NAME>.fields]", "[schema.kinds.fields]"],
        ),
        (
            1,
            "citations.must",
            ["[citations.<KIND>] must", "[citations.must]"],
        ),
    ] {
        let unknown = unknown_of(version, target);
        assert_eq!(
            spellings(&unknown),
            named,
            "v{version} {target}: {unknown:?}"
        );
    }
}

/// §FS-config-explain.5: a bare name names each table that holds a key of that
/// name and each table whose header ends in it, in index order, and is not
/// answered even where only one table holds it.
#[test]
#[ignore = "red until #1005 lists the tables that hold a bare name; implement removes this"]
fn a_bare_name_names_every_table_that_holds_it_in_index_order() {
    for (version, target, named) in [
        (
            2,
            "exclude",
            &["[schema.sources] exclude", "[presentation.fmt] exclude"][..],
        ),
        (
            2,
            "id_format",
            &["[schema] id_format", "[schema.kinds.<NAME>] id_format"],
        ),
        (
            2,
            "title",
            &[
                "[schema.kinds.<NAME>.fields.<field>] title",
                "[presentation.kinds.<NAME>] title",
            ],
        ),
        (2, "sources", &["[schema.sources]"]),
        (
            2,
            "grounding",
            &[
                "[rules.citations.grounding]",
                "[rules.citations.<KIND>.grounding]",
            ],
        ),
        (1, "exclude", &["[scan] exclude", "[fmt] exclude"]),
    ] {
        let unknown = unknown_of(version, target);
        assert_eq!(
            spellings(&unknown),
            named,
            "v{version} {target}: {unknown:?}"
        );
    }
}

/// §FS-config-explain.5: a path with an empty segment or one holding a bracket
/// has no reading, so it reaches the catch-all rather than a spelling.
#[test]
#[ignore = "red until #1005 refuses a malformed path a reading; implement removes this"]
fn a_malformed_path_has_no_reading() {
    for target in ["schema.sources]", "schema.kinds."] {
        let other = ExplainUnknown::Other {
            target: target.into(),
        };
        assert_eq!(unknown_of(2, target), other, "{target}");
    }
}

/// §FS-config-explain.5: a placeholder is no name, so a bare name never matches
/// a placeholder key or a header that ends in one.
#[test]
fn a_placeholder_matches_no_bare_name() {
    for target in ["FS", "KIND", "md"] {
        let other = ExplainUnknown::Other {
            target: target.into(),
        };
        assert_eq!(unknown_of(2, target), other, "{target}");
    }
}

/// §FS-config-explain.1.1, §FS-config-explain.5: what the readings must not
/// move: a top-level key and a code still answer, a bare table of the other
/// version gets its bracketed spelling's answer before any own table is listed,
/// and a path naming nothing and a misspelling still take the catch-all.
#[test]
fn a_top_level_key_answers_and_a_path_naming_nothing_is_the_catch_all() {
    assert!(matches!(ask(1, "kinds"), ExplainAnswer::Key(_)));
    assert!(matches!(ask(2, "oversized-lead"), ExplainAnswer::Code(_)));
    assert_eq!(unknown_of(2, "citations"), unknown_of(2, "[citations]"));
    assert_eq!(unknown_of(2, "kinds"), unknown_of(2, "[[kinds]]"));
    assert_eq!(unknown_of(2, "fmt"), unknown_of(2, "[fmt]"));
    for (version, target) in [
        (2, "schema.sorces"),
        (2, "schema.sources.nope"),
        (2, "schema.kinds.FS.nope"),
        (2, "oversized-leed"),
        (1, "scna"),
    ] {
        let other = ExplainUnknown::Other {
            target: target.into(),
        };
        assert_eq!(unknown_of(version, target), other, "v{version} {target}");
    }
}
