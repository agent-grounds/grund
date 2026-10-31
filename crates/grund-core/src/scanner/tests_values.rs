//! Focused first-class-value scanner cases: Markdown title boundaries and the
//! exact attempted-binding delimiter and source-context contracts
//! (§FS-values.2.1, §FS-values.3.1, §FS-values.3.2).

use std::path::PathBuf;

use super::values::markdown_component;
use crate::config::{Config, load_config};
use crate::grammar::value_binding_section_shape_is_valid;
use crate::testing::{check_run, codes, graph_codes, scan_findings, test_root, write};

#[test]
fn markdown_component_excludes_the_complete_numeric_coordinate_delimiter() {
    let config = Config::default_for(PathBuf::from("."));

    assert_eq!(
        markdown_component("## 1. 1200", config.frame().grammar()),
        Some(("1200", 7))
    );
    assert_eq!(
        markdown_component("## 1 USD", config.frame().grammar()),
        Some(("USD", 6))
    );
    assert_eq!(
        markdown_component("### 1.2. low / base / high", config.frame().grammar()),
        Some(("low / base / high", 10))
    );
}

fn value_repo(name: &str, binding: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
             [reference]\nstrict = true\n\n\
             [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
             [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n\
             [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
    );
    write(
        &root.join("values/field-price.md"),
        "# CONST-field-price: Reference field price\n## 1. 1200\n",
    );
    write(&root.join("docs/offer.md"), binding);
    root
}

fn source_value_repo(name: &str, source: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
             [reference]\nstrict = true\n\n\
             [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
             [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n\
             [scan]\ninclude = [\"src\"]\nextensions = [\"md\", \"rs\"]\n",
    );
    write(
        &root.join("values/field-price.md"),
        "# CONST-field-price: Reference field price\n## 1. 1.2e3\n",
    );
    write(&root.join("src/lib.rs"), source);
    root
}

#[test]
fn source_bindings_follow_complete_comment_spans() {
    let dereference = source_value_repo(
        "value_binding_rust_dereference_string",
        "pub fn replace(ptr: &mut &'static str) {\n\
             *ptr = \"`999` (§CONST-field-price.1)\";\n\
             }\n",
    );
    let dereference_errors = check_run(&dereference, false).report.errors;
    let dereference_messages = dereference_errors
        .iter()
        .map(|error| format!("{}: {}", error.code.as_str(), error.message))
        .collect::<Vec<_>>();
    assert!(
        dereference_errors.is_empty(),
        "binding-shaped text in a dereference expression's host string is unchecked: {dereference_messages:?}"
    );

    let block = source_value_repo(
        "value_binding_rust_block_interior",
        "pub fn documented() {\n\
             /*\n\
             Checked documentation: `999` (§CONST-field-price.1)\n\
             */ let trailing = \"`998` (§CONST-field-price.1)\";\n\
             let _ = trailing;\n\
             }\n",
    );
    let mismatches = check_run(&block, false)
        .report
        .errors
        .into_iter()
        .filter(|error| error.code == "value-mismatch")
        .map(|error| error.line)
        .collect::<Vec<_>>();
    assert_eq!(
        mismatches,
        vec![Some(3)],
        "the block interior binds, while host bytes after `*/` do not"
    );
}

/// A literal that is never closed, or that runs across two physical lines, is
/// an invalid attempted binding rather than prose (§FS-values.3.1.1).
#[test]
fn unterminated_and_multiline_value_bindings_are_located_errors() {
    for (name, binding, line) in [
        (
            "value_binding_unterminated",
            "`1200 (§CONST-field-price.1)\n",
            1,
        ),
        (
            "value_binding_multiline",
            "`1200\n` (§CONST-field-price.1)\n",
            2,
        ),
    ] {
        let run = check_run(&value_repo(name, binding), false);
        assert!(
            run.report
                .errors
                .iter()
                .any(|error| { error.code == "invalid-value-binding" && error.line == Some(line) }),
            "{name} should report a located invalid binding"
        );
    }
}

/// A missing space or missing parentheses is an invalid *attempted* binding,
/// while an unbackticked adjacent token is never inferred as one and stays
/// ordinary prose with one ordinary citation (§FS-values.3.1.1).
#[test]
fn malformed_delimiters_are_errors_but_unbackticked_adjacency_is_prose() {
    for (name, binding) in [
        (
            "value_binding_missing_space",
            "`1200`(§CONST-field-price.1)\n",
        ),
        (
            "value_binding_missing_parens",
            "`1200` §CONST-field-price.1\n",
        ),
    ] {
        assert_eq!(
            graph_codes(&check_run(&value_repo(name, binding), false)),
            vec!["invalid-value-binding"],
            "{name} should be an invalid attempted binding"
        );
    }

    assert!(
        check_run(
            &value_repo(
                "value_binding_unbackticked_prose",
                "1200 (§CONST-field-price.1)\n",
            ),
            false,
        )
        .report
        .errors
        .is_empty(),
        "unbackticked adjacency remains ordinary prose"
    );
}

/// A literal that closes its line onto a next-line value citation is an invalid
/// attempted binding located at the literal's opening backtick, carrying the
/// join-the-lines message; a blank line between leaves prose (§FS-values.3.1.1.1).
#[test]
fn a_binding_split_by_a_line_break_is_refused_at_its_literal() {
    let joined = "invalid value binding: value binding must be on one physical line; \
                  join the literal and its citation";
    let markdown = [
        (
            "value_binding_split_next_line",
            "The price is `999`\n(§CONST-field-price.1) today.\n",
            14,
        ),
        (
            "value_binding_split_trailing_whitespace",
            "The price is `999` \t\n(§CONST-field-price.1) today.\n",
            14,
        ),
        (
            "value_binding_split_list_continuation",
            "- The price is `999`\n  (§CONST-field-price.1) today.\n",
            16,
        ),
        (
            "value_binding_split_blockquote",
            "> The price is `999`\n> (§CONST-field-price.1) today.\n",
            16,
        ),
    ]
    .map(|(name, binding, column)| (name, value_repo(name, binding), column));
    let source = (
        "value_binding_split_line_comment",
        source_value_repo(
            "value_binding_split_line_comment",
            "// The price is `999`\n// (§CONST-field-price.1) today.\npub fn f() {}\n",
        ),
        17,
    );
    // The point names a Python docstring beside the comment prefixes; the
    // literal sits after the opening quotes, so its column counts them.
    let python = source_value_repo("value_binding_split_python_docstring", "");
    let config = python.join("grund.toml");
    let scans_python = std::fs::read_to_string(&config)
        .unwrap()
        .replace("\"rs\"]", "\"rs\", \"py\"]");
    write(&config, &scans_python);
    write(
        &python.join("src/split.py"),
        "\"\"\"The price is `999`\n(§CONST-field-price.1) today.\n\"\"\"\n",
    );
    let docstring = ("value_binding_split_python_docstring", python, 17);
    for (name, root, column) in markdown.into_iter().chain([source, docstring]) {
        let refused = check_run(&root, false)
            .report
            .errors
            .iter()
            .map(|error| {
                (
                    error.code.as_str(),
                    error.line,
                    error.column,
                    error.message.clone(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            refused,
            vec![(
                "invalid-value-binding",
                Some(1),
                Some(column),
                joined.to_string()
            )],
            "{name}: the split binding is refused once, at its literal"
        );
    }

    assert!(
        check_run(
            &value_repo(
                "value_binding_split_by_a_blank_line",
                "The price is `999`\n\n(§CONST-field-price.1) opens a paragraph.\n",
            ),
            false,
        )
        .report
        .errors
        .is_empty(),
        "a blank line ends the paragraph, so the literal and the citation stay prose"
    );
}

fn mismatch_messages(root: &PathBuf) -> Vec<String> {
    check_run(root, false)
        .report
        .errors
        .into_iter()
        .filter(|error| error.code == "value-mismatch")
        .map(|error| error.message)
        .collect()
}

/// Two components of different kinds can print the same bytes, so the finding
/// names each side's kind after the canonical text (§FS-values.5.2.1).
#[test]
fn a_mixed_kind_mismatch_names_each_sides_kind() {
    assert_eq!(
        mismatch_messages(&value_repo(
            "value_mismatch_mixed_kind",
            "`1,200` (§CONST-field-price.1)\n",
        )),
        vec![
            "value mismatch for CONST-field-price.1: bound `1,200`, declared `1200` \
             at values/field-price.md:2 \u{2014} bound is a string, declared is a number; \
             a number and a string are never equal; change the literal to the declared value, or the declaration to the bound one"
        ]
    );
}

/// A same-kind mismatch keeps the canonical text to the byte, so widening the
/// mixed-kind clause fails here rather than in every golden (§FS-values.5.2.1).
#[test]
fn a_same_kind_mismatch_keeps_the_canonical_text() {
    assert_eq!(
        mismatch_messages(&value_repo(
            "value_mismatch_same_kind",
            "`1250` (§CONST-field-price.1)\n",
        )),
        vec![
            "value mismatch for CONST-field-price.1: bound `1250`, declared `1200` \
             at values/field-price.md:2; change the literal to the declared value, or the declaration to the bound one"
        ]
    );
}

/// A binding's section shape is a valid root path with an optional positive
/// numeric coordinate, so a chapter root's named path is a shape on its own,
/// while a zero, a leading zero, and a numeric segment before a named one stay
/// refused (§FS-values.3.1, §FS-values.3.1.1).
#[test]
fn the_binding_shape_admits_a_root_path_and_keeps_its_refusals() {
    for section in ["1", "1.1", "values.aux-voltage", "values.aux-voltage.1"] {
        assert!(
            value_binding_section_shape_is_valid(section),
            "`{section}` is a binding shape"
        );
    }
    for section in ["0", "01", "1.0", "2.values", "values.aux-voltage.01"] {
        assert!(
            !value_binding_section_shape_is_valid(section),
            "`{section}` is not a binding shape"
        );
    }
}

/// `value-mismatch`'s remedy, which closes the whole message (§FS-errors.5.5.1).
const REMEDY: &str =
    "; change the literal to the declared value, or the declaration to the bound one";

/// A one-component value bound at its root is its `.1` spelling in verdict
/// only: an agreeing literal is silent, a one-part disagreement names the root
/// and its component apart, with none of the `.1` spelling's bytes in front, and
/// a two-part literal names the root because its part count differs
/// (§FS-values.3.1.2, §FS-values.5.2.2, §FS-errors.3.9.1).
#[test]
#[ignore = "red until #1118 drops the root-aimed legacy prefix; implement removes this"]
fn a_one_component_value_bound_at_its_root_is_its_coordinate_spelling_in_verdict() {
    let agrees = check_run(
        &value_repo(
            "value_root_one_component_agrees",
            "`1200.0` (§CONST-field-price)\n",
        ),
        false,
    );
    assert!(
        agrees.report.errors.is_empty(),
        "an agreeing root binding is silent: {:?}",
        codes(&agrees)
    );

    let at_root = mismatch_messages(&value_repo(
        "value_root_one_component_at_root",
        "`1250` (§CONST-field-price)\n",
    ));
    let at_coordinate = mismatch_messages(&value_repo(
        "value_root_one_component_at_coordinate",
        "`1250` (§CONST-field-price.1)\n",
    ));
    assert_eq!(
        at_coordinate,
        vec![
            "value mismatch for CONST-field-price.1: bound `1250`, declared `1200` \
             at values/field-price.md:2; change the literal to the declared value, or the declaration to the bound one"
        ]
    );
    assert_eq!(
        at_root,
        vec![format!(
            "value mismatch for CONST-field-price (component 1): \
             bound `1250`, declared `1200` at values/field-price.md:2{REMEDY}"
        )]
    );

    assert_eq!(
        mismatch_messages(&value_repo(
            "value_root_one_component_two_parts",
            "`1200 USD` (§CONST-field-price)\n",
        )),
        vec![
            "value mismatch for CONST-field-price: bound `1200 USD`, declared `1200` \
             at values/field-price.md:2; change the literal to the declared value, or the declaration to the bound one"
        ]
    );
}

/// A bare whole-value ID is binding grammar now, recorded with no path, while a
/// coordinate is recorded as written: which of them names a root is the
/// checker's question, not the scanner's (§FS-values.3.1, §FS-values.5.1).
#[test]
fn a_bare_value_id_is_a_binding_record_with_no_path() {
    let root = value_repo(
        "value_binding_bare_id_record",
        "`1200 USD` (§CONST-field-price)\n\n`1200` (§CONST-field-price.1)\n",
    );
    let config = load_config(&root).expect("load the value fixture config");
    let findings = scan_findings(&config, &root);
    assert!(
        findings.invalid_value_bindings.is_empty(),
        "neither form is an invalid attempt: {:?}",
        findings.invalid_value_bindings
    );
    assert_eq!(
        findings
            .value_bindings
            .iter()
            .map(|binding| binding.section.as_deref())
            .collect::<Vec<_>>(),
        vec![None, Some("1")]
    );
    let _ = std::fs::remove_dir_all(root);
}
