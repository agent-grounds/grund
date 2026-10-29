//! §FS-rule-refusals.sentences.order: a sentence is refused for the first stage it
//! fails. The table is read from the specification itself, so the order the
//! documentation states and the order `check --rule` follows cannot drift apart.

use super::support::{case_repo, repo_root, run, text};
use std::fs;

/// The stages, in order, as the table must name them.
const STAGES: [&str; 6] = [
    "1. terminal period",
    "2. fixed word and phase-1 subject",
    "3. conjunction",
    "4. modality",
    "5. the rest of the subject",
    "6. the predicate",
];

/// The rows of the `.order` table: stage, example sentence, opening reason.
fn order_rows() -> Vec<(String, String, String)> {
    let spec = fs::read_to_string(repo_root().join("docs/functional-spec/FS-rule-refusals.md"))
        .expect("rule refusals spec");
    let start = spec
        .find("### sentences.order:")
        .expect("the sentences.order point");
    let section = &spec[start..];
    let section = &section[..section[1..]
        .find("\n#")
        .map_or(section.len(), |end| end + 1)];
    let unquote = |cell: &str| {
        let cell = cell.trim();
        cell.strip_prefix('`')
            .and_then(|cell| cell.strip_suffix('`'))
            .unwrap_or(cell)
            .to_owned()
    };
    section
        .lines()
        .filter(|line| line.starts_with("| ") && !line.starts_with("| Stage"))
        .map(|line| {
            let cells: Vec<&str> = line.trim_matches('|').split(" | ").collect();
            assert_eq!(cells.len(), 4, "an order row has four cells: {line}");
            (
                cells[0].trim().to_owned(),
                unquote(cells[2]),
                unquote(cells[3]),
            )
        })
        .collect()
}

/// §FS-rule-refusals.sentences.order: the table holds the six stages of
/// §DF-one-answer-rulings-grammar's D20, in order.
#[test]
fn the_order_table_names_the_documented_stages() {
    let stages: Vec<String> = order_rows().into_iter().map(|(stage, ..)| stage).collect();
    assert_eq!(
        stages, STAGES,
        "the refusal order is not the documented one"
    );
}

/// §FS-rule-refusals.sentences.order: each example fails its own stage and a
/// later one, and is refused for its own. Since §FS-errors.3.8's window closed
/// in `0.19.0`, a moved reason opens the line with no legacy reason before it.
#[test]
fn a_sentence_is_refused_for_the_first_stage_it_fails() {
    let root = case_repo("check-rule-conjunction-one-kind");
    let mut wrong = Vec::new();
    for (stage, sentence, reason) in order_rows() {
        let output = run(&root, &["check", ".", "--rule", &sentence]);
        let stderr = text(&output.stderr);
        let line = stderr.lines().next().unwrap_or_default();
        let opens = line
            .strip_prefix("error: ")
            .and_then(|rest| rest.strip_prefix(reason.as_str()))
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(';'));
        if output.status.code() != Some(2) || !opens {
            wrong.push(format!(
                "{stage}: {sentence}\n  exit {:?}\n  first line {line:?}\n  expected the reason {reason:?}",
                output.status.code()
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "refused at the wrong stage:\n{}",
        wrong.join("\n")
    );
}

/// §FS-rule-refusals.sentences.order: stage 2 reads the wildcard from the first
/// clause's subject, so the modality of a later clause cannot hide it — `must
/// not` and `should not` are read before `must` and `should` when a subject is
/// split off a whole sentence (§FS-rules.3.6). A modality the first clause does
/// not accept leaves no subject, and the sentence is refused as before. A
/// subject rebuilt from the wildcard leaves a conjunction, so no form is offered
/// (§FS-errors.3.8.1).
#[test]
fn a_wildcard_is_read_from_the_subject_of_the_first_clause() {
    let root = case_repo("check-rule-conjunction-one-kind");
    let wildcard = "section-component wildcards are not accepted in phase 1";
    let rows = [
        (
            "The * chapter of each FS must cite at least one FS and must not cite any FS.",
            format!("error: {wildcard}"),
        ),
        (
            "The * chapter of each FS should cite at least one FS and should not cite any FS.",
            format!("error: {wildcard}"),
        ),
        // §FS-rule-refusals.sentences.rewrite.same-units: `FS-x` would drop the typed `*`.
        (
            "FS-x.* may not cite any FS.",
            "error: modality \"may not\" is not accepted".to_string(),
        ),
    ];
    let mut wrong = Vec::new();
    for (sentence, expected) in rows {
        let output = run(&root, &["check", ".", "--rule", sentence]);
        let stderr = text(&output.stderr);
        let line = stderr.lines().next().unwrap_or_default();
        if output.status.code() != Some(2) || line != expected {
            wrong.push(format!(
                "{sentence}\n  exit {:?}\n  first line {line:?}\n  expected   {expected:?}",
                output.status.code()
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
