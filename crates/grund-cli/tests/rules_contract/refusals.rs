//! Exact production-aware refusal bytes for every common near miss required by
//! §FS-rules.3.5 and the pre-scan lifecycle of §FS-rules.4.

use super::support::{assert_run, case_repo, fixture, run, scratch, scratch_from, text};
use std::fs;
use std::path::{Path, PathBuf};

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

/// §FS-rules.3.5, §FS-rules.12: the deliberate phase-1 absences a sentence can
/// spell - path subjects, wildcard namespaces, component wildcards, chapter
/// quantification - are refused by name, with the typed sentence repaired or,
/// where it does not say what belongs in the gap, the known kinds
/// (§FS-rules.3.5.4.5).
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
            "chapter name must be a non-empty NAME with no surrounding whitespace; NAME forbids whitespace anywhere.",
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
            "path subjects are not accepted in phase 1\nknown kinds: GOAL, REQ, FS, AR, RULE",
        ),
        (
            "Each */FS must cite at least one GOAL.",
            "subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one GOAL.",
        ),
        (
            "FS-demo.* must cite at least one REQ.",
            "section-component wildcards are not accepted in phase 1; accepted form: FS-demo must cite at least one REQ.",
        ),
        (
            "Each chapter of each FS must cite at least one REQ.",
            "chapter-quantified subjects are not accepted in phase 1; accepted form: Each FS must cite at least one REQ.",
        ),
        (
            "FS-demo.2 must cite at least one REQ.",
            "numbered chapter subjects can detach when headings move; accepted form: FS-demo must cite at least one REQ.",
        ),
        (
            "Each POLICY must cite at least one GOAL.",
            "unknown kind \"POLICY\"\nknown kinds: GOAL, REQ, FS, AR, RULE",
        ),
    ];

    let mut wrong = Vec::new();
    for (sentence, refusal) in rows {
        let output = run(&fixture(), &["check", ".", "--rule", sentence]);
        let expected = format!("error: {refusal}\n");
        let stderr = text(&output.stderr);
        if output.status.code() != Some(2) || !output.stdout.is_empty() || stderr != expected {
            wrong.push(format!(
                "--rule {sentence:?}\n  stderr   {stderr:?}\n  expected {expected:?}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {} FS-rules.3.5 rows were not exact:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
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

/// The reason every subject refused for needing named sections gives.
const NAMED_OFF: &str = "named chapter subjects require [id] named_sections = true";

/// §FS-rules.3.5.2's repository: kinds `FS`, `REQ` and `GOAL`, named sections
/// off, and `FS-login` holding a named `requirements` chapter.
fn named_off() -> PathBuf {
    case_repo("check-rule-named-off-rebuilt-subject")
}

/// The same tree with `[id] named_sections = true`.
fn named_on() -> PathBuf {
    let root = scratch_from(&named_off(), "rule-named-sections-on");
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    fs::write(
        root.join("grund.toml"),
        config.replace("named_sections = false", "named_sections = true"),
    )
    .expect("enable named sections");
    root
}

/// §FS-rules.3.5.2: every row of the exact table, the subject and the stderr
/// that follows the reason. The first two are the subjects enabling named
/// sections does make valid, and they print what they printed before.
fn named_off_rows() -> Vec<(&'static str, String)> {
    let after = "; accepted form after enabling it:";
    let chapter = "The requirements chapter of each FS must cite at least one REQ.";
    vec![
        (
            "FS-login.requirements",
            format!("{after} FS-login.requirements must cite at least one REQ.\n"),
        ),
        (
            "The requirements chapter of each FS",
            format!("{after} {chapter}\n"),
        ),
        ("FS-*.requirements", format!("{after} {chapter}\n")),
        ("FS.requirements", format!("{after} {chapter}\n")),
        (
            "The requirements.1 chapter of each FS",
            format!("{after} {chapter}\n"),
        ),
        (
            "FS-login.Requirements",
            "; accepted form: FS-login must cite at least one REQ.\n".into(),
        ),
        (
            "The Requirements chapter of each FS",
            "; accepted form: Each FS must cite at least one REQ.\n".into(),
        ),
        (
            "The * chapter of each FS",
            "; accepted form: Each FS must cite at least one REQ.\n".into(),
        ),
        (
            "POLICY.requirements",
            "\nknown kinds: FS, REQ, GOAL\n".into(),
        ),
    ]
}

/// The rows of `rows` whose `check --rule` refusal in `root` is not exactly
/// theirs, every row run before the caller decides so one never hides another.
fn wrong_named_off_rows(root: &Path, rows: &[(&str, String)]) -> Vec<String> {
    let mut wrong = Vec::new();
    for (subject, tail) in rows {
        let sentence = format!("{subject} must cite at least one REQ.");
        let output = run(root, &["check", ".", "--rule", &sentence]);
        let expected = format!("error: {NAMED_OFF}{tail}");
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

/// §FS-rules.3.5.2: with named sections off, a subject refused for needing them
/// is answered with a subject that turning them on makes valid, labelled
/// `after enabling it` only where the suggestion needs them, and with nothing
/// guessed where no configured kind is recovered.
#[test]
fn a_subject_needing_named_sections_is_answered_with_one_they_make_valid() {
    let rows = named_off_rows();
    let wrong = wrong_named_off_rows(&named_off(), &rows);
    assert!(
        wrong.is_empty(),
        "{} of {} rules refused with named sections off were not answered as FS-rules.3.5.2 says:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

/// Guard, green before and after §FS-rules.3.5.2: the subjects enabling named
/// sections does make valid keep every byte, the §FS-rules.3.5 row included.
#[test]
fn a_subject_enabling_makes_valid_keeps_its_as_typed_suggestion() {
    let wrong = wrong_named_off_rows(&named_off(), &named_off_rows()[..2]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// `check --rule` with `sentence` in `root`: the refusal printed before the
/// scan, or `None` where the sentence was accepted (§FS-rules.4).
fn pre_scan_refusal(root: &Path, sentence: &str) -> Option<String> {
    let output = run(root, &["check", ".", "--rule", sentence]);
    let stderr = text(&output.stderr);
    (output.status.code() == Some(2) || stderr.starts_with("error: ")).then_some(stderr)
}

/// Whether the form `refusal` offers pastes back where its label says, or why not.
fn pastes_back(refusal: &str, off: &Path, on: &Path) -> Result<(), String> {
    let line = refusal.lines().next().unwrap_or_default();
    if let Some((_, form)) = line.split_once("; accepted form after enabling it: ") {
        if let Some(again) = pre_scan_refusal(on, form) {
            return Err(format!("{form:?} is refused once enabled: {again:?}"));
        }
        if pre_scan_refusal(off, form).is_none() {
            return Err(format!(
                "{form:?} is labelled after enabling it but needs no enabling"
            ));
        }
    } else if let Some((_, form)) = line.split_once("; accepted form: ") {
        if let Some(again) = pre_scan_refusal(off, form) {
            return Err(format!("{form:?} is refused as configured: {again:?}"));
        }
    } else if !refusal
        .lines()
        .any(|line| line.starts_with("known kinds: "))
    {
        return Err(format!("offers no form and lists no kinds: {refusal:?}"));
    }
    Ok(())
}

/// §FS-rules.3.5.2, the issue's reproducer: every form offered with named
/// sections off pastes back where its label says. One labelled `after enabling
/// it` is accepted once they are on and refused as configured, a plain one is
/// accepted as configured, and a refusal that offers none lists the kinds.
#[test]
fn every_named_off_rule_suggestion_is_accepted_where_its_label_says() {
    let (off, on) = (named_off(), named_on());
    let wrong = named_off_rows()
        .into_iter()
        .filter_map(|(subject, _)| {
            let sentence = format!("{subject} must cite at least one REQ.");
            let refusal = pre_scan_refusal(&off, &sentence).unwrap_or_default();
            pastes_back(&refusal, &off, &on)
                .err()
                .map(|why| format!("{subject:?}: {why}"))
        })
        .collect::<Vec<_>>();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// §FS-rules.4, §FS-rules.3.5.4.4: each is refused before the scan, and each
/// offers a form but `FSbogus`, from which no configured kind is recovered, so
/// it is answered with the known kinds instead.
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
        if sentence.starts_with("FSbogus") {
            assert!(
                !stderr.contains("accepted form")
                    && stderr.ends_with("\nknown kinds: GOAL, REQ, FS, AR, RULE\n"),
                "an unrecovered subject offers a form or lists no kinds: {stderr}"
            );
        } else {
            assert!(
                stderr.contains("accepted form:"),
                "refusal has no accepted rewrite: {sentence}: {stderr}"
            );
        }
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
/// numeric production. Each stays refused and lands on the reason its two
/// sibling bounds already answer with, so widening the grammar cannot quietly
/// leave a near miss on the generic `count is not accepted` catch-all. The form
/// keeps the typed bound, and a count that is no numeral or is zero gets none
/// (§FS-rules.3.5.4.5).
#[test]
fn at_least_reaches_the_refusals_its_sibling_bounds_already_answer_with() {
    for (sentence, refusal) in [
        (
            "Each FS must cite at least two GOAL.",
            "count must be a canonical positive base-10 integer",
        ),
        (
            "Each FS must cite at least 0 GOAL.",
            "count must be a canonical positive base-10 integer",
        ),
        ("Each FS must cite at least GOAL.", "count has no object"),
        (
            "AR-overview.system-overview must cite each AR at least 2 AR.",
            "per-target counts must end in \"times\"; accepted form: AR-overview.system-overview must cite each AR at least 2 times.",
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
/// spelled some other way names the five canonical counts, appended after the
/// clause it already printed - so `count is not accepted;` stays a verbatim
/// contiguous prefix. The sentence does not say which count it meant, so no
/// form follows the list (§FS-rules.3.5.4.5).
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
        "error: count is not accepted; the canonical counts are \"at least one\", \"at least N\", \"at most N\", \"exactly one\" and \"exactly N\" for a base-10 N\n",
    );
}
