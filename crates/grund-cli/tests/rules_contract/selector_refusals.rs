//! A refused `list --selector` is answered with a selector a reader can paste
//! back, never with a rule sentence (§FS-rules.8.1), while `check --rule` keeps
//! every byte it printed (§FS-rules.3.5). The fixture is the repository
//! the `list-selector-refused-*` e2e cases share: one kind, `FS`, and `FS-login`
//! with a named `requirements` chapter holding two numbered sections.

use super::support::{assert_run, case_repo, run, scratch_from, text};
use std::fs;
use std::path::PathBuf;

const HINT: &str = "hint: grund show --batch --toc expands each selected unit into its sections\n";

fn repo() -> PathBuf {
    case_repo("list-selector-refused-numbered-section")
}

/// §FS-rules.8.1: every row of the exact table, named sections on.
fn rows() -> Vec<(&'static str, String)> {
    let numbered = "numbered chapter subjects can detach when headings move";
    let wildcard = "section-component wildcards are not accepted in phase 1";
    vec![
        (
            "FS-*.requirements",
            "error: literal subject \"FS-*.requirements\" does not match the configured ID grammar; accepted selector: FS.requirements\n".into(),
        ),
        (
            "FS.requirements.1",
            format!("error: {numbered}; accepted selector: FS.requirements\n{HINT}"),
        ),
        (
            "FS-login.requirements.1",
            format!("error: {numbered}; accepted selector: FS-login.requirements\n{HINT}"),
        ),
        (
            "The requirements.1 chapter of each FS",
            format!(
                "error: {numbered}; accepted selector: The requirements chapter of each FS\n{HINT}"
            ),
        ),
        (
            "FS.*",
            format!("error: {wildcard}; accepted selector: FS\n{HINT}"),
        ),
        (
            "FS-login.*",
            format!("error: {wildcard}; accepted selector: FS-login\n{HINT}"),
        ),
        (
            "Each chapter of each FS",
            format!(
                "error: chapter-quantified subjects are not accepted in phase 1; accepted selector: Each FS\n{HINT}"
            ),
        ),
        (
            "*/FS",
            "error: subject namespaces must be local in phase 1; accepted selector: FS\n".into(),
        ),
        (
            "FS.Requirements",
            "error: named chapter subject \"FS.Requirements\" does not match the configured section grammar; accepted selector: FS\n".into(),
        ),
        (
            "Each POLICY",
            "error: unknown kind \"POLICY\"\nknown kinds: FS\n".into(),
        ),
        (
            "POLICY.requirements",
            "error: literal subject \"POLICY.requirements\" does not match the configured ID grammar\nknown kinds: FS\n".into(),
        ),
        (
            "requirements",
            "error: literal subject \"requirements\" does not match the configured ID grammar\nknown kinds: FS\n".into(),
        ),
        (
            "FS-login must cite at least one GOAL.",
            "error: a rule sentence is not a selector; accepted selector: FS-login\n".into(),
        ),
    ]
}

/// Every row is run before the test decides, so one wrong row never hides
/// another.
#[test]
fn every_refused_selector_is_answered_with_a_selector() {
    let mut wrong = Vec::new();
    for (selector, stderr) in rows() {
        let output = run(&repo(), &["list", ".", "--selector", selector]);
        let (exit, stdout, actual) = (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr),
        );
        if exit != Some(2) || !stdout.is_empty() || actual != stderr {
            wrong.push(format!(
                "--selector {selector:?}: exit {exit:?}, stdout {stdout:?}\n  stderr   {actual:?}\n  expected {stderr:?}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {} refused selectors were not answered with a selector:\n{}",
        wrong.len(),
        rows().len(),
        wrong.join("\n")
    );
}

/// §FS-rules.8.1: the `--size` mode reads the same selector, so it refuses
/// with the same lines.
#[test]
fn list_size_answers_a_refused_selector_with_the_same_lines() {
    let output = run(
        &repo(),
        &[
            "list",
            ".",
            "--size=words",
            "--selector",
            "FS.requirements.1",
        ],
    );
    assert_run(
        &output,
        2,
        "",
        &format!(
            "error: numbered chapter subjects can detach when headings move; accepted selector: FS.requirements\n{HINT}"
        ),
    );
}

/// §FS-rules.8.1: with named sections off the suggestion is the selector as
/// typed, because enabling them is what makes it valid.
#[test]
fn disabled_named_sections_suggest_the_selector_after_enabling_them() {
    let root = scratch_from(&repo(), "selector-named-sections-off");
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    fs::write(
        root.join("grund.toml"),
        config.replace("named_sections = true", "named_sections = false"),
    )
    .expect("disable named sections");
    let output = run(&root, &["list", ".", "--selector", "FS.requirements"]);
    assert_run(
        &output,
        2,
        "",
        "error: named chapter subjects require [id] named_sections = true; accepted selector after enabling it: FS.requirements\n",
    );
}

/// Guard, green before and after §FS-rules.8.1: every selector the table
/// suggests is one `--selector` accepts, so pasting a suggestion back lists
/// units rather than failing again.
#[test]
fn every_suggested_selector_lists_units() {
    for (_, stderr) in rows() {
        let Some((_, rest)) = stderr.split_once("accepted selector: ") else {
            continue;
        };
        let suggestion = rest.lines().next().expect("suggestion line");
        let output = run(&repo(), &["list", ".", "--selector", suggestion]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "suggested selector {suggestion:?} was refused: {}",
            text(&output.stderr)
        );
        assert!(
            !text(&output.stdout).is_empty(),
            "suggested selector {suggestion:?} listed nothing"
        );
    }
}

/// Guard, green before and after §FS-rules.8.1: a rule sentence reaching the
/// same subjects keeps the released §FS-rules.3.5 bytes, so the selector
/// rewrite cannot leak into `check --rule`.
#[test]
fn the_rule_side_keeps_its_released_refusals() {
    for (sentence, stderr) in [
        (
            "FS-*.requirements must cite at least one REQ.",
            "error: literal subject \"FS-*.requirements\" does not match the configured ID grammar; accepted form: FS-login must cite at least one GOAL.\n",
        ),
        (
            "The requirements.1 chapter of each FS must cite at least one REQ.",
            "error: named chapter subject \"The requirements.1 chapter of each FS\" does not match the configured section grammar; accepted form: FS-login.requirements must cite at least one REQ.\n",
        ),
    ] {
        let output = run(&repo(), &["check", ".", "--rule", sentence]);
        assert_run(&output, 2, "", stderr);
    }
}
