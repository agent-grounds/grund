//! Exact production-aware refusal bytes for every common near miss required by
//! §FS-rules.3.5 and the pre-scan lifecycle of §FS-rules.4.

use super::support::{assert_run, fixture, run, scratch, text};
use std::fs;

#[test]
fn every_released_family_subject_modality_and_count_spelling_is_accepted() {
    let sentences = [
        "Each FS must have at least one requirements chapter.",
        "Each FS must have at least 2 requirements chapters.",
        "FS-demo should have at most 2 requirements chapters.",
        "Each FS must have exactly one requirements chapter.",
        "Each FS should have exactly 2 requirements chapters.",
        "Each FS must cite at least one GOAL or REQ.",
        "FS-demo.requirements must cite at least 2 REQ.",
        "The requirements chapter of each FS should cite at most 2 REQ.",
        "FS-demo.requirements must cite exactly one REQ.",
        "AR-overview.system-overview must cite each AR at least once.",
        "AR-overview.system-overview must cite each AR at least 2 times.",
        "AR-overview.system-overview should cite each AR at most 2 times.",
        "AR-overview.system-overview must cite each AR exactly once.",
        "AR-overview.system-overview should cite each AR exactly 2 times.",
        "Each FS must be cited by at least one AR.",
        "Each FS must be cited by at least 2 AR.",
        "FS-demo.requirements should be cited by at most 2 AR or GOAL.",
        "Each FS must be cited by exactly one AR.",
        "Each FS must not cite any AR.",
        "FS-demo.requirements should not cite any AR or GOAL.",
    ];
    for sentence in sentences {
        let output = run(&fixture(), &["check", ".", "--rule", sentence]);
        assert_ne!(
            output.status.code(),
            Some(2),
            "accepted sentence was refused: {sentence}; stderr was {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// §FS-rules.12: the deliberate phase-1 absences a sentence can spell - path
/// subjects, wildcard namespaces, component wildcards, chapter quantification -
/// are refused by name with the accepted rewrite.
#[test]
fn every_listed_refusal_has_its_exact_rewrite_and_exit_two() {
    let rows = [
        (
            "Each FS may not cite any AR.",
            "modality \"may not\" is not accepted; accepted form: Each FS must not cite any AR.",
        ),
        (
            "Each FS must cite no AR.",
            "\"cite no\" is not accepted; accepted form: Each FS must not cite any AR.",
        ),
        (
            "Each FS must cite a GOAL.",
            "quantifier \"a\" is ambiguous; accepted forms: \"Each FS must cite at least one GOAL.\" or \"Each FS must cite exactly one GOAL.\"",
        ),
        (
            "Each FS must cite at least one GOAL and must not cite any AR.",
            "conjunctions are not accepted; accepted forms: \"Each FS must cite at least one GOAL.\" and \"Each FS must not cite any AR.\"",
        ),
        (
            "Each FS must have exactly one  chapter.",
            "chapter name must be a non-empty NAME with no surrounding whitespace; accepted form: Each FS must have exactly one requirements chapter.",
        ),
        (
            "Each FS must cite at least one GOAL",
            "rule must end with \".\"; accepted form: Each FS must cite at least one GOAL.",
        ),
        (
            "each FS must cite at least one GOAL.",
            "fixed word \"Each\" is case-sensitive; accepted form: Each FS must cite at least one GOAL.",
        ),
        (
            "Each file in vendor/ must cite at least one FS.",
            "path subjects are not accepted in phase 1; accepted form: Each FS must cite at least one GOAL.",
        ),
        (
            "Each */FS must cite at least one GOAL.",
            "subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one GOAL.",
        ),
        (
            "FS-demo.* must cite at least one REQ.",
            "section-component wildcards are not accepted in phase 1; accepted form: FS-demo.requirements must cite at least one REQ.",
        ),
        (
            "Each chapter of each FS must cite at least one REQ.",
            "chapter-quantified subjects are not accepted in phase 1; accepted form: The requirements chapter of each FS must cite at least one REQ.",
        ),
        (
            "FS-demo.2 must cite at least one REQ.",
            "numbered chapter subjects can detach when headings move; accepted form: FS-demo.requirements must cite at least one REQ.",
        ),
        (
            "Each POLICY must cite at least one GOAL.",
            "unknown kind \"POLICY\"; accepted form: Each FS must cite at least one GOAL.",
        ),
    ];

    for (sentence, refusal) in rows {
        let output = run(&fixture(), &["check", ".", "--rule", sentence]);
        assert_run(&output, 2, "", &format!("error: {refusal}\n"));
    }
}

#[test]
fn named_chapter_subject_requires_the_named_sections_gate() {
    let root = scratch("named-sections-off");
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    fs::write(
        root.join("grund.toml"),
        config.replace("named_sections = true", "named_sections = false"),
    )
    .expect("disable named sections");
    let sentence = "FS-demo.requirements must cite at least one REQ.";
    let output = run(&root, &["check", ".", "--rule", sentence]);
    assert_run(
        &output,
        2,
        "",
        "error: named chapter subjects require [id] named_sections = true; accepted form after enabling it: FS-demo.requirements must cite at least one REQ.\n",
    );
}

#[test]
fn malformed_counts_ids_and_named_paths_are_pre_scan_refusals() {
    for sentence in [
        "Each FS must have exactly 2 requirements chapter.",
        "Each FS must have exactly one requirements chapters.",
        "Each FS must have exactly 02 requirements chapters.",
        "Each FS must have exactly one  requirements chapter.",
        "Each FS must cite exactly 1 GOAL.",
        "AR-overview.system-overview must cite each AR exactly 1 times.",
        "FSbogus must cite at least one GOAL.",
        "FS-demo.bad_path must cite at least one REQ.",
    ] {
        let output = run(&fixture(), &["check", ".", "--rule", sentence]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "non-production was accepted: {sentence}"
        );
        let stderr = text(&output.stderr);
        assert!(
            stderr.contains("accepted form:"),
            "refusal has no accepted rewrite: {sentence}: {stderr}"
        );
    }
}

/// §FS-rules.3, §FS-rules.3.5: one meaning keeps one spelling, so a floor of one
/// is the word `one` in both count-bearing shapes and the numeral is refused by
/// name - the rule the new `at least N` production has to obey.
#[test]
fn numeric_at_least_one_is_refused_in_both_count_shapes() {
    for (sentence, refusal) in [
        (
            "Each FS must cite at least 1 GOAL.",
            "numeric \"at least 1\" is not canonical; accepted form: Each FS must cite at least one GOAL.",
        ),
        (
            "AR-overview.system-overview must cite each AR at least 1 times.",
            "numeric \"at least 1 times\" is not canonical; accepted form: AR-overview.system-overview must cite each AR at least once.",
        ),
    ] {
        let output = run(&fixture(), &["check", ".", "--rule", sentence]);
        assert_run(&output, 2, "", &format!("error: {refusal}\n"));
    }
}

/// §FS-rules.3.5: the four sentences `at least` could not reach before it had a
/// numeric production. Each stays refused and lands on the message its two
/// sibling bounds already answer with, so widening the grammar cannot quietly
/// leave a near miss on the generic `count is not accepted` catch-all.
#[test]
fn at_least_reaches_the_refusals_its_sibling_bounds_already_answer_with() {
    for (sentence, refusal) in [
        (
            "Each FS must cite at least two GOAL.",
            "count must be a canonical positive base-10 integer; accepted form: Each FS must cite exactly 2 GOAL.",
        ),
        (
            "Each FS must cite at least 0 GOAL.",
            "count must be a canonical positive base-10 integer; accepted form: Each FS must cite exactly 2 GOAL.",
        ),
        ("Each FS must cite at least GOAL.", "count has no object"),
        (
            "AR-overview.system-overview must cite each AR at least 2 AR.",
            "per-target counts must end in \"times\"; accepted form: AR-overview.system-overview must cite each AR exactly 2 times.",
        ),
    ] {
        let output = run(&fixture(), &["check", ".", "--rule", sentence]);
        assert_run(&output, 2, "", &format!("error: {refusal}\n"));
    }
}

/// §FS-rules.3, §FS-rules.3.1: the new lower bound takes the plural noun like
/// every other numeric count, so the singular spelling gets the existing
/// wrong-spelling refusal with the right rewrite rather than a count refusal.
#[test]
fn a_chapter_floor_above_one_takes_the_plural_noun() {
    let output = run(
        &fixture(),
        &[
            "check",
            ".",
            "--rule",
            "Each FS must have at least 2 requirements chapter.",
        ],
    );
    assert_run(
        &output,
        2,
        "",
        "error: chapter count has the wrong singular/plural spelling; accepted form: Each FS must have at least 2 requirements chapters.\n",
    );
}

/// §FS-rules.3, §FS-errors.3: the catch-all a sentence reaches when its count is
/// spelled some other way now names the five canonical counts, appended after
/// the clause it already printed - so `count is not accepted;` stays a verbatim
/// contiguous prefix and no shipped bytes are rewritten.
#[test]
fn the_catch_all_count_refusal_names_every_canonical_count() {
    let output = run(
        &fixture(),
        &["check", ".", "--rule", "Each FS must cite some GOAL."],
    );
    assert_run(
        &output,
        2,
        "",
        "error: count is not accepted; the canonical counts are \"at least one\", \"at least N\", \"at most N\", \"exactly one\" and \"exactly N\" for a base-10 N; accepted form: Each FS must cite at least one GOAL.\n",
    );
}
