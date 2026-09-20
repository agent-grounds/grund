//! Contract tests for module-level assigned Python triple-quoted data
//! (§FS-check.1.1.3.1, §FS-refs.2, §FS-fmt.2.3.1.1, §AR-scanner.4.4).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::*;
use crate::queries::{DeclaredId, LspSnapshotOpts, on_type_line_edits};
use crate::testing::{test_root, write};

fn assigned_repo(name: &str, source: &str, scan: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n\
             [reference]\nstrict = false\n\n\
             [id]\nformat = \"{{kind}}-{{number}}-{{slug}}\"\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nindex = false\n\n\
             [scan]\ninclude = [\"docs\", \"src\"]\nextensions = [\"md\", \"py\", \"rs\"]\n{scan}"
        ),
    );
    write(
        &root.join("docs/functional-spec/FS-042-user-login.md"),
        "# FS-042-user-login: User login\n\n## 1. Contract\n\nLead.\n",
    );
    write(&root.join("src/render.py"), source);
    root
}

fn source_sites(root: &Path) -> Vec<(usize, usize, String)> {
    scan(root)
        .expect("scan assigned-data fixture")
        .citations
        .into_iter()
        .filter(|citation| citation.file.ends_with("render.py"))
        .map(|citation| (citation.line, citation.column, citation.text))
        .collect()
}

/// Every approved case-insensitive prefix spelling and both delimiters use the
/// same raw-column span. The citation in data is absent; the same-line tail is
/// ordinary source and keeps its original column (§FS-check.1.1.3.1).
#[test]
fn every_prefix_and_quote_excludes_only_the_assigned_span() {
    let prefixes = [
        "", "r", "U", "b", "F", "t", "br", "RB", "fr", "RF", "tr", "RT",
    ];
    let mut source = String::new();
    let mut expected = Vec::new();
    for (index, prefix) in prefixes.iter().enumerate() {
        let quote = if index % 2 == 0 { "\"\"\"" } else { "'''" };
        let annotation = if index == 1 { ": str" } else { "" };
        let line = format!(
            "DATA_{index}{annotation} = {prefix}{quote}inside §FS-999-missing{quote}; # §FS-042-user-login\n"
        );
        let column = line.find("§FS-042-user-login").expect("tail citation") + 1;
        expected.push((index + 1, column, "§FS-042-user-login".to_string()));
        source.push_str(&line);
    }
    let root = assigned_repo(
        "every_prefix_and_quote_excludes_only_the_assigned_span",
        &source,
        "",
    );
    assert_eq!(source_sites(&root), expected);
}

/// An escaped matching delimiter does not close assigned data. Content on the
/// opening and closing lines stays data, a same-line tail resumes at its raw
/// column, and later function and method docstrings are recognized.
#[test]
fn multiline_data_recovers_tails_and_later_docstrings() {
    let source = concat!(
        "\"\"\"Module documentation.\"\"\"\n",
        "PAYLOAD = \"\"\"opening §FS-999-missing\n",
        "escaped \\\"\"\" still data §FS-999-missing\n",
        "closing data §FS-999-missing\"\"\"; # §FS-042-user-login\n",
        "def function():\n",
        "    \"\"\"Function cites §FS-042-user-login.\"\"\"\n",
        "class Service:\n",
        "    def method(self):\n",
        "        '''Method cites §FS-042-user-login.'''\n",
    );
    let root = assigned_repo(
        "multiline_data_recovers_tails_and_later_docstrings",
        source,
        "",
    );
    assert_eq!(
        source_sites(&root),
        vec![
            (4, 37, "§FS-042-user-login".into()),
            (6, 23, "§FS-042-user-login".into()),
            (9, 25, "§FS-042-user-login".into()),
        ]
    );
}

/// An `=` inside a quoted annotation is not the assignment boundary. The
/// direct assigned value stays data, while both the later docstring and a
/// triple string reached after an expression remain ordinary source
/// (§FS-check.1.1.3.1).
#[test]
fn annotation_equals_selects_only_the_direct_triple_string_rhs() {
    let source = concat!(
        "from typing import Literal\n",
        "\n",
        "DATA: Literal[\"kind=value\"] = \"\"\"stored §FS-999-missing\n",
        "\"\"\"\n",
        "\n",
        "def later():\n",
        "    \"\"\"Cites §FS-042-user-login.\"\"\"\n",
        "EXPRESSION: str = choose(value=\"\"\"§FS-042-user-login\"\"\")\n",
    );
    let root = assigned_repo(
        "annotation_equals_selects_only_the_direct_triple_string_rhs",
        source,
        "",
    );
    assert_eq!(
        source_sites(&root),
        vec![
            (7, 14, "§FS-042-user-login".into()),
            (8, 35, "§FS-042-user-login".into()),
        ]
    );
}

/// An escaped candidate can overlap the matching close by two quote bytes.
/// Assigned bytes remain excluded, and the close's tail and later docstring
/// resume at their exact raw sites (§FS-check.1.1.3.1).
#[test]
fn escaped_candidate_does_not_skip_an_overlapping_close() {
    let source = r#"PAYLOAD = """stored §FS-999-missing
escaped quote then close: \""""  # §FS-042-user-login

def later():
    """Cites §FS-042-user-login."""
"#;
    let root = assigned_repo(
        "escaped_candidate_does_not_skip_an_overlapping_close",
        source,
        "",
    );
    assert_eq!(
        source_sites(&root),
        vec![
            (2, 36, "§FS-042-user-login".into()),
            (5, 14, "§FS-042-user-login".into()),
        ]
    );
}

/// The exception removes all citation forms from assigned data without merging
/// their existing rules: marked and bare citations in the tail remain live in
/// non-strict mode, as does a qualified citation outside the data span.
#[test]
fn marked_bare_and_qualified_forms_keep_distinct_rules() {
    let root = assigned_repo(
        "marked_bare_and_qualified_forms_keep_distinct_rules",
        concat!(
            "DATA = \"\"\"§FS-042-user-login FS-042-user-login ",
            "§api/FS-042-user-login\"\"\"; # §FS-042-user-login ",
            "FS-042-user-login §api/FS-042-user-login\n",
        ),
        "",
    );
    assert_eq!(
        source_sites(&root)
            .into_iter()
            .map(|(_, _, text)| text)
            .collect::<Vec<_>>(),
        vec![
            "§FS-042-user-login",
            "FS-042-user-login",
            "§api/FS-042-user-login",
        ]
    );
}

/// `check`, `refs`, `cover`, and the editor snapshot all consume one recognized
/// citation set: assigned data cannot dangle or navigate, while a real later
/// docstring remains visible on every surface.
#[test]
fn check_refs_cover_and_lsp_share_the_assigned_data_boundary() {
    let root = assigned_repo(
        "check_refs_cover_and_lsp_share_the_assigned_data_boundary",
        concat!(
            "PAYLOAD = \"\"\"§FS-999-missing\"\"\"\n",
            "def login():\n",
            "    \"\"\"Uses §FS-042-user-login.\"\"\"\n",
        ),
        "",
    );
    let report = check(&root).expect("check");
    let dangling = report
        .errors
        .iter()
        .filter(|finding| finding.code == "dangling")
        .map(|finding| finding.line)
        .collect::<Vec<_>>();
    let refs_lines = refs(RefsOpts {
        path: root.clone(),
        path_provided: true,
        id: "FS-042-user-login".into(),
        section: None,
    })
    .expect("refs")
    .hits
    .into_iter()
    .filter(|hit| hit.path == "src/render.py")
    .map(|hit| hit.line)
    .collect::<Vec<_>>();
    let cover_lines = cover(CoverOpts {
        path: root.clone(),
        path_provided: true,
    })
    .expect("cover")
    .entries
    .into_iter()
    .find(|entry| entry.path == "src/render.py")
    .expect("source cover row")
    .citations
    .into_iter()
    .map(|citation| citation.line)
    .collect::<Vec<_>>();
    let lsp_lines = lsp_snapshot(LspSnapshotOpts {
        path: root,
        path_provided: true,
        open_documents: BTreeMap::new(),
    })
    .expect("LSP snapshot")
    .citations
    .into_iter()
    .filter(|citation| citation.display_path == "src/render.py")
    .map(|citation| citation.line)
    .collect::<Vec<_>>();

    assert_eq!(
        (dangling, refs_lines, cover_lines, lsp_lines),
        (vec![], vec![3], vec![3], vec![3])
    );
}

fn type_after(root: &Path, before: &str, typed: &str) -> String {
    let path = root.join("src/render.py");
    let declarations = [DeclaredId {
        path: root,
        id: "FS-042-user-login",
    }];
    let line_index = before.lines().count();
    let mut line = String::new();
    for ch in typed.chars() {
        line.push(ch);
        let text = format!("{before}{line}");
        for edit in on_type_line_edits(&path, &text, line_index, line.len(), &declarations)
            .expect("on-type edits")
            .iter()
            .rev()
        {
            line.replace_range(edit.start..edit.end, &edit.text);
        }
    }
    line
}

/// Bulk and live formatting preserve assigned bytes but recover before the next
/// real docstring, where both expand the shorthand (§FS-fmt.2.3.1.1).
#[test]
fn fmt_and_on_type_preserve_data_and_recover_for_a_later_docstring() {
    let source = concat!(
        "PAYLOAD = \"\"\"Stored §FS-042 bytes.\n",
        "Also $$FS-042 and §FS-042.\n",
        "\"\"\"\n",
        "def login():\n",
        "    \"\"\"Uses §FS-042.\"\"\"\n",
    );
    let root = assigned_repo(
        "fmt_and_on_type_preserve_data_and_recover_for_a_later_docstring",
        source,
        "",
    );
    format_references(FmtOpts {
        path: root.clone(),
        path_provided: true,
        write: true,
        ..FmtOpts::default()
    })
    .expect("fmt --write");
    let written = fs::read_to_string(root.join("src/render.py")).expect("formatted source");
    let live = type_after(
        &root,
        "PAYLOAD = \"\"\"stored\n\"\"\"\ndef login():\n",
        "    \"\"\"Uses $$FS-042 here.",
    );
    assert_eq!(
        (written, live),
        (
            source.replace("Uses §FS-042.", "Uses §FS-042-user-login."),
            "    \"\"\"Uses §FS-042-user-login here.".to_string(),
        )
    );
}

/// The config gate is compatibility behavior: with Python docstring scanning
/// disabled, both marked strings are read by the old raw-line rule.
#[test]
fn docstring_python_false_keeps_raw_line_behavior() {
    let root = assigned_repo(
        "docstring_python_false_keeps_raw_line_behavior",
        concat!(
            "PAYLOAD = \"\"\"§FS-042-user-login\"\"\"\n",
            "def login():\n",
            "    \"\"\"§FS-042-user-login\"\"\"\n",
        ),
        "docstring_python = false\n",
    );
    assert_eq!(
        source_sites(&root),
        vec![
            (1, 14, "§FS-042-user-login".into()),
            (3, 8, "§FS-042-user-login".into()),
        ]
    );
}

/// Parenthesized and indented assignments are deliberately outside the bounded
/// exception, while ordinary Rust and Markdown strings retain their citations.
#[test]
fn out_of_scope_and_other_language_controls_keep_existing_rules() {
    let root = assigned_repo(
        "out_of_scope_and_other_language_controls_keep_existing_rules",
        concat!(
            "PAREN = (\"\"\"§FS-042-user-login\"\"\")\n",
            "def nested():\n",
            "    DATA = \"\"\"§FS-042-user-login\"\"\"\n",
            "PLAIN = \"§FS-042-user-login\"\n",
        ),
        "",
    );
    write(
        &root.join("src/other.rs"),
        "const TEXT: &str = \"§FS-042-user-login\";\n",
    );
    write(
        &root.join("docs/control.md"),
        "Markdown cites §FS-042-user-login.\n",
    );
    let findings = scan(&root).expect("scan controls");
    let sites = findings
        .citations
        .iter()
        .map(|citation| {
            (
                citation.file.file_name().unwrap().to_string_lossy(),
                citation.line,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        sites,
        vec![
            ("control.md".into(), 1),
            ("other.rs".into(), 1),
            ("render.py".into(), 1),
            ("render.py".into(), 3),
            ("render.py".into(), 4),
        ]
    );
}

/// The shared classifier must not let assigned data become a declaration,
/// section, value binding, or inline site, and it must recover in time for the
/// real docstring that follows (§AR-scanner.4.4).
#[test]
fn declaration_section_value_and_inline_consumers_share_recovery() {
    let root = test_root("declaration_section_value_and_inline_consumers_share_recovery");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [id]\nformat = \"{kind}-{slug}\"\n\n\
         [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n\
         [scan]\ninclude = [\"values\", \"src\"]\nextensions = [\"md\", \"py\"]\n",
    );
    write(&root.join("values/README.md"), "# Values\n");
    write(
        &root.join("src/catalog.py"),
        concat!(
            "PAYLOAD = \"\"\"CONST-fake: Fake\n",
            "## 1. Stored <!-- grund:value -->\n",
            "### 1.1. 999\n",
            "Bound: `999` (§CONST-fake.1.1)\n",
            "\"\"\"\n",
            "\"\"\"CONST-price: Price\n",
            "## 1. Live <!-- grund:value -->\n",
            "### 1.1. 10\n",
            "## 2. Use\n",
            "Bound: `10` (§CONST-price.1.1)\n",
            "\"\"\"\n",
        ),
    );
    let findings = scan(&root).expect("scan declaration/value recovery");
    let declarations = findings
        .declarations
        .values()
        .flatten()
        .map(|declaration| {
            (
                declaration.title.clone(),
                declaration.sections.keys().cloned().collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let citations = findings
        .citations
        .iter()
        .map(|citation| (citation.text.clone(), citation.inline_site.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        (
            declarations,
            findings.value_bindings.len(),
            findings.invalid_value_declarations.len(),
            citations,
        ),
        (
            vec![(
                Some("Price".into()),
                vec!["1".into(), "1.1".into(), "2".into()],
            )],
            1,
            0,
            vec![("§CONST-price.1.1".into(), None)],
        )
    );
}
