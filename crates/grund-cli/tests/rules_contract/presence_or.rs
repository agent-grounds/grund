//! A presence `NAME` holding `or` between two words is refused for the `or`,
//! not for its whitespace, with no form offered
//! (§FS-rule-refusals.sentences.presence-or), on `check --rule` and in a
//! configured rule declaration's `invalid-rule` finding; a `NAME` the point
//! does not reach keeps the whitespace refusal's bytes
//! (§FS-rule-refusals.sentences.whitespace). The `check --rule` fixture is the
//! repository of §FS-rule-refusals.sentences.rewrite.by-refusal, one kind `FS`;
//! the declaration fixture is the `check-rules-modal-chapter-declaration`
//! case's, which adds a `rules = true` kind. Reproducer of
//! agent-grounds/grund#1374.

use super::support::{assert_v1_run, case_repo, run, scratch_from, text, write};
use std::path::{Path, PathBuf};

/// §FS-rule-refusals.sentences.presence-or: the reason, with nothing after it.
const OR: &str = "\"or\" between chapter names is not accepted; a presence rule counts one NAME.";

/// §FS-rule-refusals.sentences.whitespace: the released bytes for a `NAME`
/// with whitespace inside it.
const WHITESPACE: &str = "chapter name must be a non-empty NAME with no surrounding whitespace; NAME forbids whitespace anywhere.";

fn repo() -> PathBuf {
    case_repo("list-selector-refused-numbered-section")
}

/// Every row whose `check --rule` refusal is not exactly exit `2`, empty
/// stdout and `error: <reason>` alone on stderr, all run before the caller
/// decides so one never hides another.
fn wrong_rows(root: &Path, rows: &[(&str, &str)]) -> Vec<String> {
    let mut wrong = Vec::new();
    for (sentence, reason) in rows {
        let output = run(root, &["check", ".", "--rule", sentence]);
        let expected = format!("error: {reason}\n");
        let (exit, stdout, stderr) = (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr),
        );
        if exit != Some(2) || !stdout.is_empty() || stderr != expected {
            wrong.push(format!(
                "--rule {sentence:?}: exit {exit:?}, stdout {stdout:?}\n  stderr   {stderr:?}\n  expected {expected:?}"
            ));
        }
    }
    wrong
}

/// §FS-rule-refusals.sentences.presence-or: the point's exact rows, and its
/// `FS-login` row.
#[test]
fn check_rule_refuses_or_between_chapter_names_for_the_or() {
    let rows = [
        (
            "Each FS must have exactly one dearer or unsold chapter.",
            OR,
        ),
        (
            "Each FS must have at least one dearer or unsold chapter.",
            OR,
        ),
        (
            "Each FS must have at least 2 dearer or unsold chapters.",
            OR,
        ),
        (
            "Each FS must have exactly one dearer chapter or unsold chapter.",
            OR,
        ),
        (
            "Each FS must have exactly one  dearer or unsold chapter.",
            OR,
        ),
        (
            "Each FS should have at most 2 dearer or unsold chapters.",
            OR,
        ),
        (
            "Each FS must have exactly one Goal or hypothesis chapter.",
            OR,
        ),
        (
            "FS-login must have exactly one dearer or unsold chapter.",
            OR,
        ),
    ];
    let wrong = wrong_rows(&repo(), &rows);
    assert!(wrong.is_empty(), "wrong refusals:\n{}", wrong.join("\n"));
}

/// §FS-rule-refusals.sentences.whitespace: a `NAME` the `or` point does not
/// reach keeps today's bytes, the trimmed form included.
#[test]
fn a_name_without_or_between_two_words_keeps_the_whitespace_refusal() {
    let trimmed = "chapter name must be a non-empty NAME with no surrounding whitespace; accepted form: FS-login must have exactly one requirements chapter. NAME forbids whitespace anywhere.";
    let rows = [
        (
            "Each FS must have exactly one dearer or chapter.",
            WHITESPACE,
        ),
        (
            "Each FS must have exactly one or unsold chapter.",
            WHITESPACE,
        ),
        (
            "Each FS must have exactly one dearer OR unsold chapter.",
            WHITESPACE,
        ),
        (
            "Each FS must have exactly one dearer order chapter.",
            WHITESPACE,
        ),
        (
            "FS-login must have exactly one Goal and hypothesis chapter.",
            WHITESPACE,
        ),
        ("Each FS must have exactly one  chapter.", WHITESPACE),
        (
            "FS-login must have exactly one  requirements chapter.",
            trimmed,
        ),
    ];
    let wrong = wrong_rows(&repo(), &rows);
    assert!(wrong.is_empty(), "wrong refusals:\n{}", wrong.join("\n"));
}

/// §FS-rule-refusals.sentences.presence-or: a `NAME` that is `or` alone is
/// one token, so the rule is accepted and judged rather than refused.
#[test]
fn a_name_that_is_or_alone_is_still_a_name() {
    let output = run(
        &repo(),
        &[
            "check",
            ".",
            "--rule",
            "Each FS must have exactly one or chapter.",
            "--only-rule",
        ],
    );
    assert_eq!(output.status.code(), Some(1), "the rule is judged");
    let stdout = text(&output.stdout);
    assert!(
        stdout.contains("FS-login has 0 or chapters; --rule requires exactly one"),
        "unexpected stdout {stdout:?}"
    );
    assert!(!text(&output.stderr).contains("error:"));
}

/// §FS-rule-refusals.sentences.presence-or: a configured rule declaration
/// carries the same reason after `is not a valid rule: ` (§FS-rules.7.1).
#[test]
fn a_rule_declaration_naming_two_chapters_with_or_is_invalid_for_the_or() {
    let root = scratch_from(
        &case_repo("check-rules-modal-chapter-declaration"),
        "presence-or-declaration",
    );
    write(
        &root,
        "docs/rules/RULE-modal.md",
        "# RULE-modal: Each FS must have exactly one dearer or unsold chapter.\n\n\
         Each login spec names its price, under \u{a7}FS-login.\n",
    );
    let output = run(&root, &["check", "."]);
    assert_v1_run(
        &output,
        1,
        &format!("docs/rules/RULE-modal.md:1: error: RULE-modal is not a valid rule: {OR}\n"),
        "",
    );
}
