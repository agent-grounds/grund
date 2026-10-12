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

/// The suffix a reason carries while §FS-errors.3.8's window keeps the legacy
/// reason ahead of it.
const WINDOW: &str = "; this wording changes in grund 0.19.0";

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
/// later one, and is refused for its own. During §FS-errors.3.8's window a moved
/// reason stands after the legacy one, before the suffix naming `0.19.0`.
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
        let windowed = line.ends_with(&format!(" — {reason}{WINDOW}"));
        if output.status.code() != Some(2) || !(opens || windowed) {
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
