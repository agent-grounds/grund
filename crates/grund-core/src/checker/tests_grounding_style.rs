//! Test module: the grounding floor and inline citation style (§FS-check.3.6, §FS-inline-citation-style)

use super::*;
use crate::config::Config;
use crate::scanner::scan_tree;
use crate::testing::{test_root, write};

#[test]
fn require_grounding_off_by_default() {
    let root = test_root("require_grounding_off_by_default");
    write(&root.join("src/util.rs"), "pub fn helper() {}\n");

    let config = Config::default_for(root.clone());
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    assert!(
        !report.errors.iter().any(|e| e.code == "ungrounded"),
        "grounding is opt-in: an uncited source file is not an error by default"
    );
}

#[test]
fn require_grounding_flags_uncited_source_file() {
    let root = test_root("require_grounding_flags_uncited_source_file");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(
        &root.join("src/auth.rs"),
        "// §FS-001-login\npub fn login() {}\n",
    );
    write(&root.join("src/util.rs"), "pub fn helper() {}\n");

    let mut config = Config::default_for(root.clone());
    config.require_grounding = true;
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    let ungrounded: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.code == "ungrounded")
        .map(|e| canonical_test_path(e.path.as_deref().unwrap()))
        .collect();
    assert_eq!(
        ungrounded,
        vec![canonical_test_path(&root.join("src/util.rs"))],
        "only the uncited source file is flagged; the one citing §FS-001-login is grounded"
    );
}

#[test]
fn inline_citation_only_rejects_prose_once_per_site() {
    let root = test_root("inline_citation_only_rejects_prose_once_per_site");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(
        &root.join("src/auth.rs"),
        "// §FS-001-login login branch rationale\n// §FS-001-login\npub fn login() {}\n",
    );

    let mut config = Config::default_for(root.clone());
    config.inline_style = "citation-only".into();
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);
    let style_errors = report
        .errors
        .iter()
        .filter(|error| error.code == "inline-citation-style")
        .collect::<Vec<_>>();

    assert_eq!(style_errors.len(), 1, "one finding per offending site");
    assert_eq!(style_errors[0].line, Some(1));
    assert_eq!(
        style_errors[0].message,
        "inline citation must carry no prose"
    );
}

#[test]
fn inline_note_hard_caps_can_report_multiple_errors() {
    let root = test_root("inline_note_hard_caps_can_report_multiple_errors");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(
        &root.join("src/auth.rs"),
        "// §FS-001-login rationale with a long line that exceeds the cap\n// second line\npub fn login() {}\n",
    );

    let mut config = Config::default_for(root.clone());
    config.inline_note_max_lines = 1;
    config.inline_note_max_columns = 40;
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);
    let messages = report
        .errors
        .iter()
        .filter(|error| error.code == "inline-citation-style")
        .map(|error| error.message.as_str())
        .collect::<Vec<_>>();

    assert!(
            messages.contains(
                &"inline note is 2 lines, over the 1-line maximum: lines 1-2 cite §FS-001-login; a blank line splits a note, an empty comment line does not"
            ),
            "line cap should be reported: {messages:?}"
        );
    assert!(
        messages.contains(
            &"inline note is 64 columns, over the 40-column maximum: lines 1-2 cite §FS-001-login"
        ),
        "column cap should be reported: {messages:?}"
    );
}

/// §FS-inline-citation-style.4.1.2: the site clause names every citation the
/// block carries, each exactly as written — marker and section included —
/// in source order, and only once: a token repeated verbatim contributes
/// nothing past its first occurrence.
#[test]
fn inline_note_line_cap_names_citations_in_source_order_deduplicated() {
    let root = test_root("inline_note_line_cap_names_citations_in_source_order_deduplicated");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n\n## 1. Session\n",
    );
    write(
        &root.join("src/auth.rs"),
        "// §FS-001-login.1 the session rule, and §FS-001-login for the\n// whole feature, and §FS-001-login.1 again for the same rule\npub fn login() {}\n",
    );

    let mut config = Config::default_for(root.clone());
    config.inline_note_max_lines = 1;
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);
    let messages = report
        .errors
        .iter()
        .filter(|error| error.code == "inline-citation-style")
        .map(|error| error.message.as_str())
        .collect::<Vec<_>>();

    assert!(
            messages.contains(
                &"inline note is 2 lines, over the 1-line maximum: lines 1-2 cite §FS-001-login.1, §FS-001-login; a blank line splits a note, an empty comment line does not"
            ),
            "citations are named as written, in source order, deduplicated: {messages:?}"
        );
}

/// §FS-inline-citation-style.2.3.2: a column is one character, so two notes of
/// equal length are judged alike whatever their prose costs in UTF-8
/// (§DF-note-columns-are-characters).
#[test]
fn inline_note_column_cap_counts_characters_not_bytes() {
    let root = test_root("inline_note_column_cap_counts_characters_not_bytes");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );

    // Identical character counts; only the padding's encoding differs.
    let ascii = format!("// §FS-001-login {}", "x".repeat(60));
    let accented = format!("// §FS-001-login {}", "é".repeat(60));
    assert_eq!(
        ascii.chars().count(),
        accented.chars().count(),
        "the two notes must be the same width in characters"
    );
    assert!(
        accented.len() > ascii.len(),
        "and a different width in bytes, or the test proves nothing"
    );

    write(
        &root.join("src/auth.rs"),
        &format!("{ascii}\npub fn a() {{}}\n\n{accented}\npub fn b() {{}}\n"),
    );

    let mut config = Config::default_for(root.clone());
    // Exactly the width of both notes: at the cap, not over it.
    config.inline_note_max_columns = ascii.chars().count();
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    let columns = report
        .errors
        .iter()
        .filter(|error| {
            error.code == "inline-citation-style" && error.message.contains("-column maximum:")
        })
        .map(|error| (error.line, error.message.as_str()))
        .collect::<Vec<_>>();

    assert!(
        columns.is_empty(),
        "a note at the cap is within it in either alphabet: {columns:?}"
    );
}

/// §FS-inline-citation-style.2.3.2: the cap is exact in non-ASCII prose — one
/// character over is over, and it is the only line reported.
#[test]
fn inline_note_column_cap_is_exact_in_non_ascii_prose() {
    let root = test_root("inline_note_column_cap_is_exact_in_non_ascii_prose");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );

    let at_cap = format!("// §FS-001-login {}", "é".repeat(60));
    let over_by_one = format!("// §FS-001-login {}", "é".repeat(61));

    write(
        &root.join("src/auth.rs"),
        &format!("{at_cap}\npub fn a() {{}}\n\n{over_by_one}\npub fn b() {{}}\n"),
    );

    let mut config = Config::default_for(root.clone());
    config.inline_note_max_columns = at_cap.chars().count();
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    let columns = report
        .errors
        .iter()
        .filter(|error| {
            error.code == "inline-citation-style" && error.message.contains("-column maximum:")
        })
        .map(|error| (error.line, error.message.clone()))
        .collect::<Vec<_>>();

    assert_eq!(
        columns,
        vec![(
            Some(4),
            format!(
                "inline note is {} columns, over the {}-column maximum: line 4 cites §FS-001-login",
                at_cap.chars().count() + 1,
                at_cap.chars().count()
            )
        )],
        "only the line one character over the cap is reported, anchored at itself"
    );
}

#[test]
fn inline_note_soft_cap_is_warning_only_when_enabled() {
    let root = test_root("inline_note_soft_cap_is_warning_only_when_enabled");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(
        &root.join("src/auth.rs"),
        "// §FS-001-login rationale\n// continuation\npub fn login() {}\n",
    );

    let mut config = Config::default_for(root.clone());
    config.warn_on_suggested = true;
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.code == "inline-citation-style"
                    && warning.message == "inline note is 2 lines, over the 1-line preferred limit: lines 1-2 cite §FS-001-login; a blank line splits a note, an empty comment line does not"),
            "soft-cap overrun should be a warning when enabled"
        );
    assert!(
        !report
            .errors
            .iter()
            .any(|error| error.code == "inline-citation-style"),
        "soft-cap overrun within the hard cap must not be an error"
    );
}

#[test]
fn inline_declaration_blocks_are_not_citation_style_sites() {
    let root = test_root("inline_declaration_blocks_are_not_citation_style_sites");
    write(
        &root.join("docs/functional-spec/FS-002-beta.md"),
        "# FS-002-beta: Beta\n",
    );
    write(
        &root.join("src/auth.rs"),
        "// FS-001-login: Login\n// §FS-002-beta body citation with prose\npub fn login() {}\n",
    );

    let mut config = Config::default_for(root.clone());
    config.inline_style = "citation-only".into();
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    assert!(
        !report
            .errors
            .iter()
            .any(|error| error.code == "inline-citation-style"),
        "inline spec declaration bodies are not style-checked"
    );
}

#[test]
fn inline_style_respects_disabled_python_docstring_scanning() {
    let root = test_root("inline_style_respects_disabled_python_docstring_scanning");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(
        &root.join("src/auth.py"),
        "\"\"\"\n§FS-001-login\nsecond line\nthird line\n\"\"\"\n",
    );

    let mut config = Config::default_for(root.clone());
    config.docstring_python = false;
    config.inline_note_max_lines = 1;
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    assert!(
        !report
            .errors
            .iter()
            .any(|error| error.code == "inline-citation-style"),
        "triple-quoted strings are not inline citation sites when docstring scanning is disabled"
    );
}

#[test]
fn python_docstring_citations_keep_source_columns() {
    let root = test_root("python_docstring_citations_keep_source_columns");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(
        &root.join("src/auth.py"),
        "class Auth:\n    \"\"\"\n    Uses §FS-001-login.\n    \"\"\"\n\n\
             def inline():\n    \"\"\"Inline §FS-001-login.\"\"\"\n",
    );

    let config = Config::default_for(root.clone());
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let mut citations = findings
        .citations
        .iter()
        .filter(|citation| citation.file.ends_with("src/auth.py"))
        .collect::<Vec<_>>();
    citations.sort_by_key(|citation| (citation.line, citation.column));

    assert_eq!(
        citations.len(),
        2,
        "expected both Python docstring citations"
    );
    assert_eq!(
        (
            citations[0].line,
            citations[0].column,
            citations[0].text.as_str()
        ),
        (3, 10, "§FS-001-login"),
        "indented docstring body citation must use original source column (§AR-scanner.4.4)"
    );
    assert_eq!(
        (
            citations[1].line,
            citations[1].column,
            citations[1].text.as_str()
        ),
        (7, 15, "§FS-001-login"),
        "same-line triple-quoted docstring citation must be scanned after the opening delimiter (§AR-scanner.4.4)"
    );
}

#[test]
fn inline_style_strips_configured_comment_prefixes_for_note_detection() {
    let root = test_root("inline_style_strips_configured_comment_prefixes_for_note_detection");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(&root.join("src/auth.rs"), "% §FS-001-login\n");

    let mut config = Config::default_for(root.clone());
    config.inline_style = "citation-only".into();
    config.comment_prefixes = vec!["%".into()];
    config.rebuild_grammar().expect("rebuild grammar");
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    assert!(
        !report
            .errors
            .iter()
            .any(|error| error.code == "inline-citation-style"),
        "a pure citation with a configured custom comment prefix must not count as prose"
    );
}

#[test]
fn inline_style_strips_block_comment_continuation_prefix() {
    let root = test_root("inline_style_strips_block_comment_continuation_prefix");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(
        &root.join("src/auth.rs"),
        "/*\n * §FS-001-login\n */\npub fn login() {}\n",
    );

    let mut config = Config::default_for(root.clone());
    config.inline_style = "citation-only".into();
    config.comment_prefixes = vec!["/*".into()];
    config.rebuild_grammar().expect("rebuild grammar");
    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);

    assert!(
        !report
            .errors
            .iter()
            .any(|error| error.code == "inline-citation-style"),
        "block-comment `*` continuation markers must be stripped when `/*` is configured"
    );
}
