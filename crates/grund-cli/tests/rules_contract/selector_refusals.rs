//! A refused `list --selector` is answered with a selector a reader can paste
//! back, never with a rule sentence (§FS-rules.8.1), and `check --rule` answers
//! the same subject with the same recovery written as a rule subject
//! (§FS-rules.3.5.4.2), after the reason a selector gets for the same component
//! (§FS-rules.3.5.3). The fixture is the repository
//! the `list-selector-refused-*` e2e cases share: one kind, `FS`, and `FS-login`
//! with a named `requirements` chapter holding two numbered sections and a named
//! `should` chapter.

use super::support::{assert_run, case_repo, run, scratch_from, text};
use std::fs;
use std::path::{Path, PathBuf};

const HINT: &str = "hint: grund show --batch --toc expands each selected unit into its sections\n";

/// The reason every selector refused for needing named sections gives.
const NAMED_OFF: &str = "named chapter subjects require [id] named_sections = true";

fn repo() -> PathBuf {
    case_repo("list-selector-refused-numbered-section")
}

/// The shared fixture with `[id] named_sections = false`.
fn named_sections_off() -> PathBuf {
    let root = scratch_from(&repo(), "selector-named-sections-off");
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    fs::write(
        root.join("grund.toml"),
        config.replace("named_sections = true", "named_sections = false"),
    )
    .expect("disable named sections");
    root
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
        // §FS-rules.3.5.3: an empty component is not a number, so no breadcrumb.
        (
            "FS-login.requirements.",
            "error: named chapter subject \"FS-login.requirements.\" does not match the configured section grammar; accepted selector: FS-login.requirements\n".into(),
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
            "Each */FS",
            "error: subject namespaces must be local in phase 1; accepted selector: Each FS\n"
                .into(),
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
            "Each chapter of each POLICY",
            format!(
                "error: chapter-quantified subjects are not accepted in phase 1\nknown kinds: FS\n{HINT}"
            ),
        ),
        (
            "API.requirements.1",
            format!("error: {numbered}\nknown kinds: FS\n{HINT}"),
        ),
        (
            "FS-login must cite at least one GOAL.",
            "error: a rule sentence is not a selector; accepted selector: FS-login\n".into(),
        ),
        (
            "The should chapter of each FS must cite at least one FS.",
            "error: a rule sentence is not a selector; accepted selector: The should chapter of each FS\n".into(),
        ),
    ]
}

/// §FS-rules.8.1: every row of the exact table with named sections off, then
/// the other selectors refused for needing them, each suggested what it would
/// be suggested with them on.
fn named_off_rows() -> Vec<(&'static str, String)> {
    let after = "accepted selector after enabling it";
    vec![
        (
            "FS.requirements",
            format!("error: {NAMED_OFF}; {after}: FS.requirements\n"),
        ),
        (
            "FS-*.requirements",
            format!("error: {NAMED_OFF}; {after}: FS.requirements\n"),
        ),
        (
            "FS.requirements.1",
            format!("error: {NAMED_OFF}; {after}: FS.requirements\n"),
        ),
        (
            "FS.*",
            format!("error: {NAMED_OFF}; accepted selector: FS\n"),
        ),
        (
            "The * chapter of each FS",
            format!("error: {NAMED_OFF}; accepted selector: Each FS\n"),
        ),
        (
            "POLICY.requirements",
            format!("error: {NAMED_OFF}\nknown kinds: FS\n"),
        ),
        // Beyond the table: the same rule over the other spellings.
        (
            "FS-login.requirements",
            format!("error: {NAMED_OFF}; {after}: FS-login.requirements\n"),
        ),
        (
            "The requirements.1 chapter of each FS",
            format!("error: {NAMED_OFF}; {after}: The requirements chapter of each FS\n"),
        ),
        (
            "FS.Requirements",
            format!("error: {NAMED_OFF}; accepted selector: FS\n"),
        ),
    ]
}

/// Every row of `rows` is run in `root` before the caller decides, so one
/// wrong row never hides another.
fn wrong_rows(root: &Path, rows: &[(&str, String)]) -> Vec<String> {
    let mut wrong = Vec::new();
    for (selector, stderr) in rows {
        let output = run(root, &["list", ".", "--selector", selector]);
        let (exit, stdout, actual) = (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr),
        );
        if exit != Some(2) || !stdout.is_empty() || &actual != stderr {
            wrong.push(format!(
                "--selector {selector:?}: exit {exit:?}, stdout {stdout:?}\n  stderr   {actual:?}\n  expected {stderr:?}"
            ));
        }
    }
    wrong
}

/// Whether `selector` lists at least one unit in `root`, or why not.
fn lists_units(root: &Path, selector: &str) -> Result<(), String> {
    let output = run(root, &["list", ".", "--selector", selector]);
    if output.status.code() != Some(0) {
        return Err(format!("refused: {}", text(&output.stderr)));
    }
    if text(&output.stdout).is_empty() {
        return Err("listed nothing".into());
    }
    Ok(())
}

#[test]
fn every_refused_selector_is_answered_with_a_selector() {
    let wrong = wrong_rows(&repo(), &rows());
    assert!(
        wrong.is_empty(),
        "{} of {} refused selectors were not answered with a selector:\n{}",
        wrong.len(),
        rows().len(),
        wrong.join("\n")
    );
}

/// §FS-rules.8.1: the `--size` mode reads the same selector, so it refuses
/// with the same lines, the hint after `known kinds:` included.
#[test]
fn list_size_answers_a_refused_selector_with_the_same_lines() {
    for (selector, stderr) in [
        (
            "FS.requirements.1",
            format!(
                "error: numbered chapter subjects can detach when headings move; accepted selector: FS.requirements\n{HINT}"
            ),
        ),
        (
            "Each chapter of each POLICY",
            format!(
                "error: chapter-quantified subjects are not accepted in phase 1\nknown kinds: FS\n{HINT}"
            ),
        ),
    ] {
        let output = run(
            &repo(),
            &["list", ".", "--size=words", "--selector", selector],
        );
        assert_run(&output, 2, "", &stderr);
    }
}

/// §FS-rules.8.1: with named sections off, a selector refused for needing them
/// is suggested what it would be suggested with them on, labelled
/// `after enabling it` only where the suggestion needs them.
#[test]
fn disabled_named_sections_suggest_what_enabling_them_would_accept() {
    let rows = named_off_rows();
    let wrong = wrong_rows(&named_sections_off(), &rows);
    assert!(
        wrong.is_empty(),
        "{} of {} selectors refused with named sections off were not answered with a selector that pastes back:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

/// Guard, green before and after §FS-rules.8.1: a suggestion labelled
/// `after enabling it` lists units once named sections are on, and a plain
/// one lists units as configured, so no named-off suggestion is refused again.
#[test]
fn every_named_off_suggestion_lists_units_where_its_label_says() {
    let off = named_sections_off();
    for (selector, stderr) in named_off_rows() {
        let line = stderr.lines().next().expect("error line");
        let (root, suggestion) =
            if let Some((_, rest)) = line.split_once("accepted selector after enabling it: ") {
                (repo(), rest)
            } else if let Some((_, rest)) = line.split_once("accepted selector: ") {
                (off.clone(), rest)
            } else {
                continue;
            };
        if let Err(why) = lists_units(&root, suggestion) {
            panic!("{selector:?} suggests {suggestion:?}, which {why}");
        }
    }
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
        if let Err(why) = lists_units(&repo(), suggestion) {
            panic!("suggested selector {suggestion:?} {why}");
        }
    }
}

/// The rule subject the selector `selector` names: `KIND.NAME[.NAME…]` is
/// selector-only, so it becomes `The NAME[.NAME…] chapter of each KIND`, and a
/// bare `KIND` becomes `Each KIND` (§FS-rules.3.5.4.2).
fn as_rule_subject(selector: &str) -> String {
    match selector.split_once('.') {
        Some(("FS", names)) => format!("The {names} chapter of each FS"),
        None if selector == "FS" => "Each FS".into(),
        _ => selector.into(),
    }
}

/// §FS-rules.8.1, §FS-rules.3.5.4.2: a rule sentence over a refused subject is
/// answered with the subject `list --selector` gives it, written as a rule
/// subject under the typed predicate, and with no form and `known kinds:` where
/// the selector has none. Only what follows the reason is compared, because the
/// reasons are agent-grounds/grund#507's to correct.
#[test]
fn the_rule_side_answers_a_refused_subject_as_the_selector_does() {
    let predicate = "must cite at least one FS.";
    let mut wrong = Vec::new();
    for subject in [
        "FS-*.requirements",
        "The requirements.1 chapter of each FS",
        "Each */FS",
        "FS.*",
        "FS-login.*",
        "Each chapter of each FS",
        "FS-login.requirements.1",
        "FS.requirements.1",
        "FS-login.requirements.",
        "FS-login..requirements",
        "Each POLICY",
        "Each chapter of each POLICY",
        "FSbogus",
    ] {
        let listed = text(&run(&repo(), &["list", ".", "--selector", subject]).stderr);
        let first = listed.lines().next().unwrap_or_default();
        let expected = match first.split_once("; accepted selector: ") {
            Some((_, selector)) => {
                format!(
                    "; accepted form: {} {predicate}\n",
                    as_rule_subject(selector)
                )
            }
            None => "\nknown kinds: FS\n".into(),
        };
        let sentence = format!("{subject} {predicate}");
        let output = run(&repo(), &["check", ".", "--rule", &sentence]);
        let stderr = text(&output.stderr);
        let after = stderr.find([';', '\n']).map_or("", |at| &stderr[at..]);
        if output.status.code() != Some(2) || after != expected {
            wrong.push(format!(
                "--rule {sentence:?}: {stderr:?}\n  expected after the reason {expected:?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Guard, green before and after §FS-rules.8.1: the sentence a selector now
/// splits where the rule does (§FS-rules.3.6) is the rule `check --rule` has
/// always read, with the subject `The should chapter of each FS`.
#[test]
fn the_rule_side_keeps_its_reading_of_a_sentence_with_a_modality_named_chapter() {
    let output = run(
        &repo(),
        &[
            "check",
            ".",
            "--rule",
            "The should chapter of each FS must cite at least one FS.",
        ],
    );
    assert_run(
        &output,
        1,
        "docs/functional-spec/FS-login.md:11: error: FS-login.should must cite FS (--rule)\n\
         docs/functional-spec/FS-login.md:1: warning: declared but never cited: FS-login\n",
        "",
    );
}
