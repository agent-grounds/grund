//! Black-box regression for §FS-rule-refusals.sentences.every: unsupported
//! quantifiers get bounded, pasteable repairs on all three refusal surfaces.

use super::support::{assert_run, case_repo, run, text, write};
use std::fs;
use std::path::PathBuf;

const REASON: &str = "quantifier \"every\" is not accepted";
const KINDS: &str = "known kinds: SYS, AR, FS, TRACE, RULE\n";

fn repo() -> PathBuf {
    case_repo("check-rule-every-controls")
}

/// Reported cases, the declaration subject, and preservation with configured
/// vocabulary, nested chapter paths, advisory modality and a numeric bound.
fn rows() -> [(&'static str, &'static str); 8] {
    [
        (
            "SYS-overview.components must cite every AR exactly once.",
            "SYS-overview.components must cite each AR exactly once.",
        ),
        (
            "The design chapter of every AR must cite at least one FS.",
            "The design chapter of each AR must cite at least one FS.",
        ),
        (
            "Every TRACE should cite exactly one SYS.",
            "Each TRACE should cite exactly one SYS.",
        ),
        (
            "The every.detail chapter of every SYS should cite at most 2 TRACE.",
            "The every.detail chapter of each SYS should cite at most 2 TRACE.",
        ),
        (
            "SYS-every should cite every TRACE at most 2 times.",
            "SYS-every should cite each TRACE at most 2 times.",
        ),
        (
            "SYS-overview.every.detail should cite every TRACE at least once.",
            "SYS-overview.every.detail should cite each TRACE at least once.",
        ),
        (
            "Every AR must cite at least one FS.",
            "Each AR must cite at least one FS.",
        ),
        (
            "AR-trace.design must cite every FS exactly once.",
            "AR-trace.design must cite each FS exactly once.",
        ),
    ]
}

#[test]
fn accepted_each_controls_pass_in_the_same_fixture() {
    for (_, sentence) in rows() {
        let output = run(&repo(), &["check", ".", "--rule", sentence, "--only-rule"]);
        assert_run(&output, 0, "success\n", "");
        eprintln!("control exit 0: {sentence}");
    }
}

#[test]
#[ignore = "red until #833 diagnoses every and reparses each offers; implement removes this"]
fn check_rule_names_every_and_pastes_the_complete_each_offer() {
    let mut wrong = Vec::new();
    for (sentence, repaired) in rows() {
        let output = run(&repo(), &["check", ".", "--rule", sentence, "--only-rule"]);
        let stderr = text(&output.stderr);
        let expected = format!("error: {REASON}; accepted form: {repaired}\n");
        if output.status.code() != Some(2) || !output.stdout.is_empty() || stderr != expected {
            wrong.push(format!(
                "{sentence}\nexit {:?}\nactual: {stderr}expected: {expected}",
                output.status.code()
            ));
        }
        if let Some((_, offer)) = stderr.trim_end().split_once("; accepted form: ") {
            let pasted = run(&repo(), &["check", ".", "--rule", offer, "--only-rule"]);
            assert_run(&pasted, 0, "success\n", "");
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
#[ignore = "red until #833 gives configured rules the same every repair; implement removes this"]
fn configured_rules_name_every_and_match_the_ad_hoc_repair() {
    let root = std::env::temp_dir().join(format!(
        "grund-833-configured-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).expect("create isolated configured fixture");
    for file in [
        "grund.toml",
        "docs/sys/SYS-overview.md",
        "docs/sys/SYS-every.md",
        "docs/ar/AR-trace.md",
        "docs/fs/FS-trace.md",
        "docs/trace/TRACE-link.md",
    ] {
        write(&root, file, &fs::read_to_string(repo().join(file)).unwrap());
    }
    for (index, (sentence, _)) in rows().iter().enumerate() {
        write(
            &root,
            &format!("docs/rules/RULE-every-{index}.md"),
            &format!("# RULE-every-{index}: {sentence}\n\n\u{a7}FS-trace\n"),
        );
    }
    let output = run(&root, &["check", "."]);
    fs::remove_dir_all(&root).expect("remove configured fixture");
    assert_eq!(
        output.status.code(),
        Some(1),
        "a finding, not a CLI refusal"
    );
    let stdout = text(&output.stdout);
    let mut wrong = Vec::new();
    for (index, (sentence, repaired)) in rows().iter().enumerate() {
        let expected =
            format!("RULE-every-{index} is not a valid rule: {REASON}; accepted form: {repaired}");
        let actual = stdout
            .lines()
            .find(|line| line.contains(&format!("RULE-every-{index} is not a valid rule:")))
            .unwrap_or("missing invalid-rule finding");
        let ad_hoc = run(&repo(), &["check", ".", "--rule", sentence, "--only-rule"]);
        let expected_ad_hoc = format!("error: {REASON}; accepted form: {repaired}\n");
        if !actual.ends_with(&expected) || text(&ad_hoc.stderr) != expected_ad_hoc {
            wrong.push(format!("actual: {actual}\nexpected suffix: {expected}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
#[ignore = "red until #833 gives selectors the every reason and each repair; implement removes this"]
fn selectors_name_every_and_paste_selector_offers() {
    let mut wrong = Vec::new();
    for (selector, repaired) in [
        ("Every TRACE", "Each TRACE"),
        (
            "The design chapter of every AR",
            "The design chapter of each AR",
        ),
        (
            "The every.detail chapter of every SYS",
            "The every.detail chapter of each SYS",
        ),
    ] {
        let output = run(&repo(), &["list", ".", "--selector", selector]);
        let stderr = text(&output.stderr);
        let expected = format!("error: {REASON}; accepted selector: {repaired}\n");
        if output.status.code() != Some(2) || !output.stdout.is_empty() || stderr != expected {
            wrong.push(format!("{selector}\nactual: {stderr}expected: {expected}"));
        }
        if let Some((_, offer)) = stderr.trim_end().split_once("; accepted selector: ") {
            let pasted = run(&repo(), &["list", ".", "--selector", offer]);
            assert_eq!(
                pasted.status.code(),
                Some(0),
                "{offer}: {}",
                text(&pasted.stderr)
            );
            assert!(
                !pasted.stdout.is_empty(),
                "the selector selects fixture units"
            );
        }
    }
    let output = run(&repo(), &["list", ".", "--selector", "Every MISSING"]);
    let expected = format!("error: {REASON}\n{KINDS}");
    if output.status.code() != Some(2) || text(&output.stderr) != expected {
        wrong.push(format!(
            "unknown kind: {}expected: {expected}",
            text(&output.stderr)
        ));
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
#[ignore = "red until #833 repairs every through existing refusal stages; implement removes this"]
fn every_repair_respects_stage_precedence_and_reparse_limits() {
    let mut wrong = Vec::new();
    for (sentence, reason, offer, kinds) in [
        (
            "Every TRACE should cite exactly one SYS",
            "rule must end with \".\"",
            Some("Each TRACE should cite exactly one SYS."),
            false,
        ),
        (
            "Every SYS may not cite any AR.",
            "modality \"may not\" is not accepted",
            Some("Each SYS must not cite any AR."),
            false,
        ),
        (
            "Every TRACE must cite exactly one SYS and must cite exactly one SYS.",
            "conjunctions are not accepted; accepted forms: \"Each TRACE must cite exactly one SYS.\" and \"Each TRACE must cite exactly one SYS.\"",
            None,
            false,
        ),
        (
            "Each MISSING must cite every SYS exactly once.",
            "unknown kind \"MISSING\"",
            None,
            true,
        ),
        (
            "Every MISSING must cite at least one SYS.",
            REASON,
            None,
            true,
        ),
        (
            "The design chapter of every AR must cite every FS exactly once.",
            REASON,
            Some("The design chapter of each AR must cite each FS exactly once."),
            false,
        ),
        (
            "SYS-every must cite every TRACE at most one times.",
            REASON,
            None,
            false,
        ),
        (
            "SYS-every must cite every MISSING exactly once.",
            REASON,
            None,
            true,
        ),
    ] {
        let output = run(&repo(), &["check", ".", "--rule", sentence, "--only-rule"]);
        let suffix = offer.map_or(String::new(), |offer| format!("; accepted form: {offer}"));
        let expected = format!(
            "error: {reason}{suffix}\n{}",
            if kinds { KINDS } else { "" }
        );
        let stderr = text(&output.stderr);
        if output.status.code() != Some(2) || stderr != expected {
            wrong.push(format!("{sentence}\nactual: {stderr}expected: {expected}"));
        }
        if let Some((_, offer)) = stderr.trim_end().split_once("; accepted form: ") {
            let pasted = run(&repo(), &["check", ".", "--rule", offer, "--only-rule"]);
            assert!(
                matches!(pasted.status.code(), Some(0 | 1)),
                "offer was refused: {}",
                text(&pasted.stderr)
            );
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn genuine_every_chapter_names_and_id_components_stay_accepted() {
    for selector in [
        "SYS-every",
        "SYS-overview.every.detail",
        "The every.detail chapter of each SYS",
    ] {
        let output = run(&repo(), &["list", ".", "--selector", selector]);
        assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
        assert!(!output.stdout.is_empty());
    }
    for sentence in [
        "SYS-every must cite each TRACE exactly once.",
        "SYS-overview.every.detail must cite exactly one TRACE.",
        "The every.detail chapter of each SYS must cite at most 2 TRACE.",
        "Each SYS should have at most 2 every chapters.",
    ] {
        assert_run(
            &run(&repo(), &["check", ".", "--rule", sentence, "--only-rule"]),
            0,
            "success\n",
            "",
        );
    }
}
