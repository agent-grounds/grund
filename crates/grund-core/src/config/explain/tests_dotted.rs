//! §FS-config-explain.5: a dotted path to a table or a key the version admits
//! is not answered (§FS-config-explain.1.1), and its failure names the
//! bracketed spelling that would answer.

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

/// §FS-config-explain.5: the failure for each dotted path is not the catch-all
/// and carries the bracketed spelling, with a name kept as written and the
/// longest table prefix taken.
#[test]
fn a_dotted_path_names_its_bracketed_spelling() {
    for (version, target, spelling) in [
        (2, "schema.sources", "[schema.sources]"),
        (2, "schema.sources.exclude", "[schema.sources] exclude"),
        (2, "schema.kinds", "[schema.kinds]"),
        (2, "schema.kinds.FS", "[schema.kinds.FS]"),
        (2, "schema.kinds.FS.files", "[schema.kinds.FS] files"),
        (2, "presentation.fmt", "[presentation.fmt]"),
        (1, "reference", "[reference]"),
        (1, "reference.strict", "[reference] strict"),
        (
            1,
            "reference.lead_size_warning.max",
            "[reference] lead_size_warning.max",
        ),
        (1, "fmt.cross_refs", "[fmt.cross_refs]"),
    ] {
        let unknown = unknown_of(version, target);
        assert!(
            !matches!(unknown, ExplainUnknown::Other { .. }),
            "v{version} {target}: the catch-all {unknown:?}"
        );
        let shown = format!("{unknown:?}");
        assert!(
            shown.contains(&format!("\"{spelling}\"")),
            "v{version} {target}: names no {spelling}: {shown}"
        );
    }
}

/// §FS-config-explain.1.1, §FS-config-explain.5: what the dotted reading must
/// not move: a top-level key still answers, and a path naming nothing, a bare
/// leaf and a misspelling still take the catch-all; a bare table of the other
/// version gets its bracketed spelling's answer instead.
#[test]
fn a_top_level_key_answers_and_a_path_naming_nothing_is_the_catch_all() {
    assert!(matches!(ask(1, "kinds"), ExplainAnswer::Key(_)));
    assert!(matches!(ask(2, "oversized-lead"), ExplainAnswer::Code(_)));
    assert_eq!(unknown_of(2, "citations"), unknown_of(2, "[citations]"));
    for (version, target) in [
        (2, "sources"),
        (2, "schema.sorces"),
        (2, "schema.sources.nope"),
        (1, "scna"),
    ] {
        let other = ExplainUnknown::Other {
            target: target.into(),
        };
        assert_eq!(unknown_of(version, target), other, "v{version} {target}");
    }
}
