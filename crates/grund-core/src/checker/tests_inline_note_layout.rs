//! Test module: the opt-in inline note layout check — its two config keys, the
//! channel each level speaks through, and the sentence the managed entrypoint
//! block teaches (§FS-inline-citation-style.4.4, §FS-inline-citation-style.5)

use std::path::{Path, PathBuf};

use super::*;
use crate::config::{Config, load_config};
use crate::model::Diagnostic;
use crate::scanner::scan_tree;
use crate::templates::inline_citation_style_sentence;
use crate::testing::{layout_config, legacy_fs_folder_config, test_root, write};

fn layout_fixture(name: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(
        &root.join("src/auth.rs"),
        concat!(
            "// §FS-001-login: reject an expired credential\n",
            "pub fn login() {}\n",
            "\n",
            "// §FS-001-login reject an expired credential\n",
            "pub fn relogin() {}\n",
            "\n",
            "// Walks the credential store.\n",
            "// §FS-001-login: one error per expired credential.\n",
            "// §FS-001-login — and one more, laid out wrong.\n",
            "pub fn sweep() {}\n",
        ),
    );
    root
}

fn style_findings(config: &Config, root: &Path) -> (Vec<usize>, Vec<usize>) {
    let (findings, _) = scan_tree(config, Some(root), true).expect("scan root");
    let report = check_findings(&findings, config);
    let lines = |diagnostics: &[Diagnostic]| {
        diagnostics
            .iter()
            .filter(|finding| {
                finding.code == "inline-citation-style"
                    && finding.message.starts_with("inline note must open")
            })
            .filter_map(|finding| finding.line)
            .collect::<Vec<_>>()
    };
    (lines(&report.errors), lines(&report.warnings))
}

/// §FS-inline-citation-style.4.4.1: one error per nonconforming line, anchored
/// at the line — never at the site's opener, and never on a conforming sibling.
///
/// §FS-inline-citation-style.7.3: the rule is a pure pass over `Findings`. The
/// verdicts arrive on the inline citation site the scanner recorded, so
/// `check_findings` is handed no root and no file and could not re-read a line
/// to decide its shape even if it wanted to.
#[test]
fn layout_error_reports_one_finding_per_offending_line() {
    let root = layout_fixture("layout_error_reports_one_finding_per_offending_line");
    let mut config = layout_config(root.clone(), "citation-first-colon");
    config.inline_note_layout_check = "error".into();

    let (errors, warnings) = style_findings(&config, &root);
    assert_eq!(errors, vec![4, 9], "the two deviating lines, in file order");
    assert!(
        warnings.is_empty(),
        "`error` speaks through one channel only"
    );
}

/// §FS-inline-citation-style.4.4: `warn` reports the same lines with the same
/// message on the warning channel, which never moves the exit code.
///
/// §FS-inline-citation-style.4.4.3: the level chooses the channel and nothing
/// else — the same two lines come back in the same deterministic order as at
/// `error`, on the other channel, with neither leaking into the first.
#[test]
fn layout_warn_reports_the_same_lines_as_warnings() {
    let root = layout_fixture("layout_warn_reports_the_same_lines_as_warnings");
    let mut config = layout_config(root.clone(), "citation-first-colon");
    config.inline_note_layout_check = "warn".into();

    let (errors, warnings) = style_findings(&config, &root);
    assert!(errors.is_empty(), "a warning never becomes an error");
    assert_eq!(warnings, vec![4, 9]);
}

// §FS-inline-citation-style.4.4.2: the message is identical at both levels, so a
// project moving from `warn` to `error` changes the exit code and nothing else.
#[test]
fn layout_message_names_the_form_with_the_configured_marker() {
    let root = layout_fixture("layout_message_names_the_form_with_the_configured_marker");
    let mut config = layout_config(root.clone(), "citation-first-colon");
    config.inline_note_layout_check = "error".into();

    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);
    let messages = report
        .errors
        .iter()
        .filter(|finding| finding.code == "inline-citation-style")
        .map(|finding| finding.message.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec!["inline note must open with its citations and a colon (§<ID>: note)".to_string(); 2]
    );
}

/// §FS-inline-citation-style.4.4: both keys default to the inert value, and the
/// check key is inert on its own under `any` — an upgrade turns nothing red.
///
/// §FS-inline-citation-style.3.3.10: `any` is the default, and it imposes
/// nothing — the deviating lines this fixture holds are exactly the comments a
/// tree that adopts `grund` mid-life already had, and they earn no finding.
#[test]
fn layout_is_silent_when_either_key_is_inert() {
    let root = layout_fixture("layout_is_silent_when_either_key_is_inert");

    let off = layout_config(root.clone(), "citation-first-colon");
    assert_eq!(style_findings(&off, &root), (Vec::new(), Vec::new()));

    let mut any = layout_config(root.clone(), "any");
    any.inline_note_layout_check = "error".into();
    assert_eq!(style_findings(&any, &root), (Vec::new(), Vec::new()));

    let defaults = legacy_fs_folder_config(root.clone());
    assert_eq!(defaults.inline_note_layout, "any");
    assert_eq!(defaults.inline_note_layout_check, "off");
    assert_eq!(style_findings(&defaults, &root), (Vec::new(), Vec::new()));
}

// §FS-inline-citation-style.2.2: both keys are closed enums read from the
// project's own config, and an unrecognized value fails on load.
#[test]
fn both_layout_keys_load_and_reject_unknown_values() {
    let root = test_root("both_layout_keys_load_and_reject_unknown_values");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[reference]\ninline_note_layout = \"citation-first-colon\"\ninline_note_layout_check = \"warn\"\n",
    );
    let config = load_config(&root).expect("load config");
    assert_eq!(config.inline_note_layout, "citation-first-colon");
    assert_eq!(config.inline_note_layout_check, "warn");

    for (key, value, expected) in [
        (
            "inline_note_layout",
            "citation-first",
            "unknown [reference] inline_note_layout `citation-first` (expected any or citation-first-colon)",
        ),
        (
            "inline_note_layout_check",
            "errors",
            "unknown [reference] inline_note_layout_check `errors` (expected off, warn, or error)",
        ),
    ] {
        write(
            &root.join("grund.toml"),
            &format!("grund_config_version = 1\n[reference]\n{key} = \"{value}\"\n"),
        );
        let error = match load_config(&root) {
            Ok(_) => panic!("{key} = \"{value}\" must be rejected"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains(expected),
            "expected `{expected}`, got `{error}`"
        );
    }
}

/// §FS-inline-citation-style.5: the block sentence appends to the budget
/// sentence and precedes the layout sentence, which is written with the
/// project's marker and absent under `any` so no existing managed block
/// drifts on that key. The doc-comment sentence closes the copy at every
/// `inline_style`, after whatever the other keys produced.
///
/// §FS-inline-citation-style.5.1: the budgets line opens the copy, in the
/// `suggested < max` form here and in the `citation-only` form below.
/// §FS-inline-citation-style.5.2: the block sentence follows it under
/// `citation-with-note` only, and no such sentence is added under
/// `citation-only`.
/// §FS-inline-citation-style.5.3.2: the enforcement level is not part of the
/// house style, so `off` renders byte-for-byte what `error` renders.
#[test]
fn agents_sentence_teaches_the_configured_layout() {
    let root = test_root("agents_sentence_teaches_the_configured_layout");
    let any = layout_config(root.clone(), "any");
    assert_eq!(
        inline_citation_style_sentence(&any),
        "Inline notes: ≤ 1 line preferred, hard cap 3 lines; ≤ 100 columns. A note is one comment block: a blank line splits it, an empty comment line does not. Doc-comments (`///`, `//!`, `/** */`, a docstring, a Go, Ruby, shell or SQL comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there."
    );

    let mut colon = layout_config(root.clone(), "citation-first-colon");
    colon.inline_note_layout_check = "error".into();
    assert_eq!(
        inline_citation_style_sentence(&colon),
        "Inline notes: ≤ 1 line preferred, hard cap 3 lines; ≤ 100 columns. A note is one comment block: a blank line splits it, an empty comment line does not. Lay each note out citation-first: `// §<ID>: <note>` (several citations: `// §<ID>, §<ID>: <note>`). Doc-comments (`///`, `//!`, `/** */`, a docstring, a Go, Ruby, shell or SQL comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there."
    );

    // The enforcement level is not an instruction: `off` renders the same
    // sentence `error` does.
    let mut off = layout_config(root.clone(), "citation-first-colon");
    off.marker = "@".into();
    assert!(inline_citation_style_sentence(&off).contains("`// @<ID>: <note>`"));

    // A style that permits no note at all has no layout to teach — but the
    // doc-comment sentence closes this style too (§FS-inline-citation-style.5.4).
    let mut citation_only = layout_config(root, "citation-first-colon");
    citation_only.inline_style = "citation-only".into();
    assert_eq!(
        inline_citation_style_sentence(&citation_only),
        "Inline citations carry no prose — put rationale in the spec. Doc-comments (`///`, `//!`, `/** */`, a docstring, a Go, Ruby, shell or SQL comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there."
    );
    assert!(
        inline_citation_style_sentence(&citation_only).contains("are documentation, not notes"),
        "the doc-comment sentence must close `citation-only` too"
    );
}

/// §FS-inline-citation-style.3.3.7: layout and size are judged independently.
/// One line can deviate from the layout *and* exceed the column budget, and
/// then it earns one finding for each — neither verdict absorbs the other, and
/// neither is withheld because the other already fired.
///
/// Its own fixture rather than a case appended to `layout_fixture`: the two
/// suites above assert the exact line list a run produces, so a third
/// offending line there would be a change to their assertions rather than an
/// addition to the module.
#[test]
fn a_line_can_break_the_layout_and_the_column_budget_at_once() {
    let root = test_root("a_line_can_break_the_layout_and_the_column_budget_at_once");
    write(
        &root.join("docs/functional-spec/FS-001-login.md"),
        "# FS-001-login: Login\n",
    );
    write(
        &root.join("src/auth.rs"),
        concat!(
            "// §FS-001-login: short, and laid out the way the project asked\n",
            "pub fn login() {}\n",
            "\n",
            "// §FS-001-login and this one note opens with no colon at all and \
             also runs well past the hundredth column on purpose\n",
            "pub fn sweep() {}\n",
        ),
    );
    let mut config = layout_config(root.clone(), "citation-first-colon");
    config.inline_note_layout_check = "error".into();

    let (findings, _) = scan_tree(&config, Some(&root), true).expect("scan root");
    let report = check_findings(&findings, &config);
    let style = report
        .errors
        .iter()
        .filter(|finding| finding.code == "inline-citation-style")
        .map(|finding| (finding.line, finding.message.clone()))
        .collect::<Vec<_>>();

    assert_eq!(
        style.len(),
        2,
        "one layout finding and one budget finding: {style:?}"
    );
    assert!(
        style.iter().all(|(line, _)| *line == Some(4)),
        "both findings are about the same line: {style:?}"
    );
    assert_eq!(
        style
            .iter()
            .filter(|(_, message)| message.starts_with("inline note must open"))
            .count(),
        1,
        "{style:?}"
    );
    assert_eq!(
        style
            .iter()
            .filter(|(_, message)| message.contains("over the 100-column maximum"))
            .count(),
        1,
        "{style:?}"
    );
}
