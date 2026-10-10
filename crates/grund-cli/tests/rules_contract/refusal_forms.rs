//! A refused rule's accepted form is the sentence its author typed with only
//! the refused part replaced (§FS-rules.3.5.4), pinned by exact rows. The
//! fixture is the repository the `list-selector-refused-*` e2e cases share: one
//! kind, `FS`, named sections on, and `FS-login` with named `requirements` and
//! `should` chapters.

use super::support::{case_repo, run, text};
use std::path::{Path, PathBuf};

/// The reason every subject refused for needing named sections gives.
const NAMED_OFF: &str = "named chapter subjects require [id] named_sections = true";

fn repo() -> PathBuf {
    case_repo("list-selector-refused-numbered-section")
}

/// §FS-rules.3.5.2's repository: kinds `FS`, `REQ` and `GOAL`, named sections
/// off, and `FS-login` holding a named `requirements` chapter.
fn named_off() -> PathBuf {
    case_repo("check-rule-named-off-rebuilt-subject")
}

/// The rows of `rows` whose `check --rule` refusal in `root` is not exactly
/// `error: <text>` and then `<after>`, every row run before the caller decides
/// so one never hides another.
fn wrong_rows(root: &Path, rows: &[(&str, &str, &str)]) -> Vec<String> {
    let mut wrong = Vec::new();
    for (sentence, refusal, after) in rows {
        let output = run(root, &["check", ".", "--rule", sentence]);
        let expected = format!("error: {refusal}\n{after}");
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

/// §FS-rules.3.5.4.6: every row of the exact table, the refused sentence, the
/// `error:` line's text and the line after it.
fn fs_only_rows() -> Vec<(&'static str, &'static str, &'static str)> {
    let kinds = "known kinds: FS\n";
    vec![
        (
            "Each FS may not cite any FS.",
            "modality \"may not\" is not accepted; accepted form: Each FS must not cite any FS.",
            "",
        ),
        (
            "Each FS should cite no FS.",
            "\"cite no\" is not accepted; accepted form: Each FS should not cite any FS.",
            "",
        ),
        (
            "each FS must cite at least one FS.",
            "fixed word \"Each\" is case-sensitive; accepted form: Each FS must cite at least one FS.",
            "",
        ),
        (
            "Each FS must cite at least 1 FS",
            "rule must end with \".\"; accepted form: Each FS must cite at least one FS.",
            "",
        ),
        (
            "Each FS must cite at least 1 FS.",
            "numeric \"at least 1\" is not canonical; accepted form: Each FS must cite at least one FS.",
            "",
        ),
        (
            "Each FS should be cited by exactly 1 FS.",
            "numeric \"exactly 1\" is not canonical; accepted form: Each FS should be cited by exactly one FS.",
            "",
        ),
        (
            "FS-login.requirements must cite each FS at least 1 times.",
            "numeric \"at least 1 times\" is not canonical; accepted form: FS-login.requirements must cite each FS at least once.",
            "",
        ),
        (
            "FS-login must cite each FS exactly 2.",
            "per-target counts must end in \"times\"; accepted form: FS-login must cite each FS exactly 2 times.",
            "",
        ),
        (
            "Each FS must cite exactly 02 FS.",
            "count must be a canonical positive base-10 integer; accepted form: Each FS must cite exactly 2 FS.",
            "",
        ),
        (
            "Each FS must cite a FS.",
            "quantifier \"a\" is ambiguous; accepted forms: \"Each FS must cite at least one FS.\" or \"Each FS must cite exactly one FS.\"",
            "",
        ),
        (
            "Each FS must cite at least one GOAL and must not cite any AR.",
            "conjunctions are not accepted",
            kinds,
        ),
        (
            "Each FS must cite at least one FS and must not cite any FS.",
            "unknown kind \"FS must cite at least one FS and\"",
            kinds,
        ),
        (
            "Each FS should not cite at least one FS.",
            "a prohibition must use \"cite any\"; accepted form: Each FS should not cite any FS.",
            "",
        ),
        (
            "FS-login must have exactly one requirements section.",
            "chapter presence must end in \"chapter\"; accepted form: FS-login must have exactly one requirements chapter.",
            "",
        ),
        (
            "FS-login should have at most 2 requirements chapter.",
            "chapter count has the wrong singular/plural spelling; accepted form: FS-login should have at most 2 requirements chapters.",
            "",
        ),
        (
            "FS-login must have exactly one  requirements chapter.",
            "chapter name must be a non-empty NAME with no surrounding whitespace; accepted form: FS-login must have exactly one requirements chapter. NAME forbids whitespace anywhere.",
            "",
        ),
        (
            "FS-login must have exactly one Goal and hypothesis chapter.",
            "chapter name must be a non-empty NAME with no surrounding whitespace; NAME forbids whitespace anywhere.",
            "",
        ),
        (
            "FS-login.requirements must have exactly one should chapter.",
            "chapter subjects cannot have chapters; accepted form: FS-login must have exactly one should chapter.",
            "",
        ),
        (
            "The requirements chapter of each FS should have at least one should chapter.",
            "chapter subjects cannot have chapters; accepted form: Each FS should have at least one should chapter.",
            "",
        ),
        (
            "FS-*.requirements must cite at least one FS.",
            "literal subject \"FS-*.requirements\" does not match the configured ID grammar; accepted form: The requirements chapter of each FS must cite at least one FS.",
            "",
        ),
        (
            "FS.* must cite at least one FS.",
            "section-component wildcards are not accepted in phase 1; accepted form: Each FS must cite at least one FS.",
            "",
        ),
        (
            "FS-login.2 should cite at most 2 FS.",
            "numbered chapter subjects can detach when headings move; accepted form: FS-login should cite at most 2 FS.",
            "",
        ),
        (
            "Each chapter of each FS must be cited by at least one FS.",
            "chapter-quantified subjects are not accepted in phase 1; accepted form: Each FS must be cited by at least one FS.",
            "",
        ),
        (
            "Each */FS must cite at least one FS.",
            "subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one FS.",
            "",
        ),
        (
            "Each POLICY must cite at least one FS.",
            "unknown kind \"POLICY\"",
            kinds,
        ),
        (
            "Each file in vendor/ must cite at least one FS.",
            "path subjects are not accepted in phase 1",
            kinds,
        ),
        (
            "Each FS must cite at least one POLICY.",
            "unknown kind \"POLICY\"",
            kinds,
        ),
        (
            "FS-*.requirements must cite at least one GOAL.",
            "literal subject \"FS-*.requirements\" does not match the configured ID grammar",
            kinds,
        ),
        (
            "Each FS cites at least one FS.",
            "rule has no accepted modality",
            "",
        ),
        (
            "Each FS must reference at least one FS.",
            "verb is not accepted",
            "",
        ),
        (
            "Each FS must cite some FS.",
            "count is not accepted; the canonical counts are \"at least one\", \"at least N\", \"at most N\", \"exactly one\" and \"exactly N\" for a base-10 N",
            "",
        ),
        (
            "Each FS must cite at least two FS.",
            "count must be a canonical positive base-10 integer",
            "",
        ),
    ]
}

/// §FS-rules.3.5.4.6, the issue's reproducer: in a repository that configures
/// only `FS`, each refusal keeps what was typed, replaces the refused part, and
/// offers nothing, followed by the kinds, where the part missing is a kind.
#[test]
fn every_refusal_where_fs_is_the_only_kind_offers_the_typed_sentence_repaired() {
    let rows = fs_only_rows();
    let wrong = wrong_rows(&repo(), &rows);
    assert!(
        wrong.is_empty(),
        "{} of {} refusals in an FS-only repository were not as FS-rules.3.5.4.6 says:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

/// §FS-rules.3.5.4.3: the rows a single replacement cannot repair. A missing
/// `.` is offered only once the second pass has rewritten `at least 1`, and a
/// form that would name a kind the repository lacks is never offered.
#[test]
fn a_form_is_parsed_again_before_it_is_offered() {
    let rows: Vec<_> = fs_only_rows()
        .into_iter()
        .filter(|(sentence, ..)| {
            [
                "Each FS must cite at least 1 FS",
                "Each FS must cite at least one GOAL and must not cite any AR.",
                "FS-*.requirements must cite at least one GOAL.",
            ]
            .contains(sentence)
        })
        .collect();
    assert_eq!(rows.len(), 3, "the three multi-pass rows are in the table");
    let wrong = wrong_rows(&repo(), &rows);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// §FS-rules.3.5.2, item 5: with named sections off, the subject is answered
/// as that point says, and the modality and predicate are the ones the author
/// typed, rewritten where they are refused too. The label is decided on the
/// whole sentence, and where the predicate has no form nothing is suggested.
#[test]
fn a_named_off_subject_keeps_the_typed_predicate() {
    let after = format!("{NAMED_OFF}; accepted form after enabling it:");
    let rows = [
        (
            "FS-*.requirements must cite at least one GOAL.",
            format!("{after} The requirements chapter of each FS must cite at least one GOAL."),
            "",
        ),
        (
            "FS-login.Requirements should be cited by at most 2 GOAL.",
            format!("{NAMED_OFF}; accepted form: FS-login should be cited by at most 2 GOAL."),
            "",
        ),
        (
            "FS-login.requirements must cite at least 1 GOAL.",
            format!("{after} FS-login.requirements must cite at least one GOAL."),
            "",
        ),
        (
            "FS-login.requirements must cite at least one POLICY.",
            NAMED_OFF.to_string(),
            "known kinds: FS, REQ, GOAL\n",
        ),
        (
            "FS-login.requirements must reference at least one REQ.",
            NAMED_OFF.to_string(),
            "",
        ),
        (
            "FS-login.requirements must cite some REQ.",
            NAMED_OFF.to_string(),
            "",
        ),
    ];
    let rows: Vec<_> = rows
        .iter()
        .map(|(sentence, refusal, tail)| (*sentence, refusal.as_str(), *tail))
        .collect();
    let wrong = wrong_rows(&named_off(), &rows);
    assert!(
        wrong.is_empty(),
        "{} of {} named-off refusals did not keep the typed predicate:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

/// §FS-rules.3.5.2 step 3: only a reason about named sections may carry the
/// `after enabling it` label. Where another part is refused first and only its
/// form would need named sections, the refusal is the reason alone.
#[test]
fn only_a_named_sections_reason_carries_the_after_enabling_label() {
    let rows = [
        (
            "FS-login.requirements may not cite any REQ.",
            "modality \"may not\" is not accepted",
            "",
        ),
        (
            "FS-login.requirements must cite each REQ exactly 1 times",
            "rule must end with \".\"",
            "",
        ),
    ];
    let wrong = wrong_rows(&named_off(), &rows);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// §FS-rules.3.5.4.2: a refused subject holding a clause of its own, an
/// undocumented conjunction split at its second modality, is offered no form,
/// never one that drops the first clause.
#[test]
fn a_subject_holding_a_clause_is_offered_no_form() {
    let rows: Vec<_> = fs_only_rows()
        .into_iter()
        .filter(|(sentence, ..)| {
            *sentence == "Each FS must cite at least one FS and must not cite any FS."
        })
        .collect();
    assert_eq!(rows.len(), 1, "the clause row is in the table");
    let wrong = wrong_rows(&repo(), &rows);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
