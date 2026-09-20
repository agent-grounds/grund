//! Exact production-aware refusal bytes for every common near miss required by
//! §FS-rules.3.5 and the pre-scan lifecycle of §FS-rules.4.

use super::support::{assert_run, fixture, run, scratch, text};
use std::fs;

#[test]
fn every_released_family_subject_modality_and_count_spelling_is_accepted() {
    let sentences = [
        "Each FS must have at least one requirements chapter.",
        "FS-demo should have at most 2 requirements chapters.",
        "Each FS must have exactly one requirements chapter.",
        "Each FS should have exactly 2 requirements chapters.",
        "Each FS must cite at least one GOAL or REQ.",
        "The requirements chapter of each FS should cite at most 2 REQ.",
        "FS-demo.requirements must cite exactly one REQ.",
        "AR-overview.system-overview must cite each AR at least once.",
        "AR-overview.system-overview should cite each AR at most 2 times.",
        "AR-overview.system-overview must cite each AR exactly once.",
        "AR-overview.system-overview should cite each AR exactly 2 times.",
        "Each FS must be cited by at least one AR.",
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
