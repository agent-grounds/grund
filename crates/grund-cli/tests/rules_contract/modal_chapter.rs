//! A chapter subject named after a modal word is cut at that name by the split
//! of §FS-rules.3.6, and the subject `The` it leaves is refused with the
//! released ID-grammar reason followed by the name
//! (§FS-rule-refusals.sentences.modal-chapter), on `check --rule`, a configured
//! rule declaration's `invalid-rule` finding and `list --selector`. The
//! fixtures are the `check-rule-modal-chapter-name` e2e case's, one kind `FS`
//! and `FS-login` holding named `must` and `should` chapters, and the
//! `check-rules-modal-chapter-declaration` case's, which adds a `rules = true`
//! declaration written as such a sentence.

use super::support::{assert_run, case_repo, run, text};
use std::path::{Path, PathBuf};

/// The released reason every row opens with.
const RELEASED: &str = "literal subject \"The\" does not match the configured ID grammar";

fn repo() -> PathBuf {
    case_repo("check-rule-modal-chapter-name")
}

/// §FS-rule-refusals.sentences.modal-chapter: the released reason, then the
/// modal word the split took as the chapter's name.
fn modal(name: &str) -> String {
    format!(
        "{RELEASED} — chapter name \"{name}\" is read as the rule's modality, so rename the chapter"
    )
}

/// Every row whose refusal on `args` plus the row's input is not exactly
/// `error: <reason>` then `known kinds: FS`, all run before the caller decides.
fn wrong_rows(root: &Path, args: &[&str], rows: &[(&str, String)]) -> Vec<String> {
    let mut wrong = Vec::new();
    for (input, reason) in rows {
        let output = run(root, &[args, &[*input]].concat());
        let expected = format!("error: {reason}\nknown kinds: FS\n");
        let (exit, stdout, stderr) = (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr),
        );
        if exit != Some(2) || !stdout.is_empty() || stderr != expected {
            wrong.push(format!(
                "{input:?}: exit {exit:?}, stdout {stdout:?}\n  stderr   {stderr:?}\n  expected {expected:?}"
            ));
        }
    }
    wrong
}

/// §FS-rule-refusals.sentences.modal-chapter: the exact `check --rule` rows,
/// and the other modalities that reach a modal name first. The first is the
/// reproducer of agent-grounds/grund#726.
#[test]
fn check_rule_refuses_a_modal_word_chapter_for_its_name() {
    let rows = [
        (
            "The must chapter of each FS must cite at least one FS.",
            modal("must"),
        ),
        (
            "The must chapter of each FS should cite at least one FS.",
            modal("must"),
        ),
        (
            "The should chapter of each FS should cite at least one FS.",
            modal("should"),
        ),
        (
            "The must not chapter of each FS should not cite any FS.",
            modal("must not"),
        ),
        (
            "The should not chapter of each FS should cite at least one FS.",
            modal("should not"),
        ),
        (
            "The must not chapter of each FS must cite at least one FS.",
            modal("must not"),
        ),
        (
            "The should not chapter of each FS should not cite any FS.",
            modal("should not"),
        ),
    ];
    let wrong = wrong_rows(&repo(), &["check", ".", "--rule"], &rows);
    assert!(
        wrong.is_empty(),
        "{} of {} modal-word chapters were not refused for their name:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

/// §FS-rule-refusals.sentences.modal-chapter, §FS-rules.7.1: a configured rule
/// declaration's `invalid-rule` finding carries the same reason.
#[test]
fn the_invalid_rule_finding_names_the_modal_word_chapter() {
    let output = run(
        &case_repo("check-rules-modal-chapter-declaration"),
        &["check", "."],
    );
    assert_run(
        &output,
        1,
        &format!(
            "docs/rules/RULE-modal.md:1: error: RULE-modal is not a valid rule: {}\n",
            modal("must")
        ),
        "",
    );
}

/// §FS-rule-refusals.sentences.modal-chapter, §FS-rule-refusals.selectors: a
/// selector splits where the rule does, so it gives the same reason.
#[test]
fn list_selector_gives_the_same_reason() {
    let rows = [
        (
            "The should chapter of each FS should cite at least one FS.",
            modal("should"),
        ),
        ("The must not chapter of each FS", modal("must not")),
    ];
    let wrong = wrong_rows(&repo(), &["list", ".", "--selector"], &rows);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Guard, green before and after §FS-rule-refusals.sentences.modal-chapter:
/// where the split takes the rule's own modality first the sentence is
/// accepted and reaches the chapter, and the `must` chapter is selectable.
#[test]
fn a_split_that_takes_the_rules_own_modality_first_is_accepted() {
    for (sentence, exit) in [
        (
            "The should chapter of each FS must cite at least one FS.",
            1,
        ),
        ("The must chapter of each FS should not cite any FS.", 0),
        ("The should chapter of each FS must not cite any FS.", 0),
    ] {
        let output = run(&repo(), &["check", ".", "--rule", sentence]);
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{sentence:?}: {}{}",
            text(&output.stdout),
            text(&output.stderr)
        );
    }
    let output = run(
        &repo(),
        &["list", ".", "--selector", "The must chapter of each FS"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert!(
        text(&output.stdout).contains("FS-login"),
        "{}",
        text(&output.stdout)
    );
}
