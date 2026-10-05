//! A finding's rule authority as a selectable fact: `check --only-rule`
//! (§FS-rules.8), the `authority` record field it queries (§FS-rules.7.6,
//! §FS-errors.5.1, §FS-output-shapes.1), and where the two axes of selection
//! meet (§FS-check.1.4, §FS-check.2.1.2). Every assertion invokes the shipped
//! `grund` frontend, so the contract compiles before the field exists.
//!
//! The reason this is a file of its own rather than more cases in
//! `surfaces.rs`: scoping is one question — *what does this sentence find?* —
//! and these cases fail together when the answer moves.

use super::support::{assert_run, fixture, run, scratch, text, write};
use std::fs;

/// The trial sentence used throughout: it authors exactly one finding in the
/// fixture, with the `citation-cardinality` code two declared rules also emit,
/// which is why `--only` cannot separate them (§FS-check.1.4).
const TRIAL: &str = "FS-demo.requirements must cite exactly 3 REQ.";

/// The one line that sentence authors.
const TRIAL_FINDING: &str = "docs/fs/FS-demo.md:6: error: FS-demo.requirements cites REQ 0 times; \
                             --rule requires exactly 3\n";

/// What a bare `--rule` run prints: the trial line sorted into the tree's own
/// three findings. This is the report the ticket says a trial run cannot escape.
const UNSCOPED: &str = concat!(
    "docs/ar/AR-overview.md:3: error: AR-overview.system-overview cites AR-one 2 times; ",
    "RULE-overview requires exactly once\n",
    "docs/ar/AR-overview.md:3: error: AR-overview.system-overview cites AR-one-more 0 times; ",
    "RULE-overview requires exactly once\n",
    "docs/fs/FS-demo.md:6: error: FS-demo.requirements cites REQ 0 times; ",
    "--rule requires exactly 3\n",
    "docs/fs/FS-demo.md:6: error: FS-demo.requirements must cite REQ (RULE-requirements)\n",
);

/// §FS-rules.8: the ticket, restated as an assertion — a scoped report is the
/// trial sentence's findings and nothing else, where the same run unscoped is
/// four lines of which one was asked for.
#[test]
fn a_scoped_report_is_the_trial_sentences_findings_alone() {
    let root = fixture();
    assert_run(
        &run(&root, &["check", ".", "--rule", TRIAL]),
        1,
        UNSCOPED,
        "",
    );
    let scoped = run(&root, &["check", ".", "--rule", TRIAL, "--only-rule"]);
    assert_run(&scoped, 1, TRIAL_FINDING, "");
}

/// §FS-rules.8: bare `--rule` is unchanged — the regression guard on the
/// author's answer that the trial surface keeps its present bytes. This case
/// passes today, and its job is to keep passing.
#[test]
fn bare_rule_keeps_its_bytes_and_its_exit() {
    assert_run(
        &run(&fixture(), &["check", ".", "--rule", TRIAL]),
        1,
        UNSCOPED,
        "",
    );
}

/// §FS-rules.8, §FS-check.1.4: `--only-rule` requires `--rule`. Scoping to no
/// sentence would print `success` and exit 0, which reads as a verdict rather
/// than as the mistake it is, so it is an invocation error decided before any
/// scan.
#[test]
fn only_rule_without_rule_is_an_invocation_error() {
    assert_run(
        &run(&fixture(), &["check", ".", "--only-rule"]),
        2,
        "",
        "error: --only-rule requires --rule\n",
    );
}

/// §FS-check.1.4: the two axes intersect. `--only` on the trial sentence's own
/// code still admits the declared rules that share it — that is the ticket's
/// second probe — while adding `--only-rule` leaves exactly the one line.
#[test]
fn the_code_axis_and_the_authority_axis_intersect() {
    let root = fixture();
    let by_code = run(
        &root,
        &[
            "check",
            ".",
            "--rule",
            TRIAL,
            "--only",
            "citation-cardinality",
        ],
    );
    let expected = concat!(
        "docs/ar/AR-overview.md:3: error: AR-overview.system-overview cites AR-one 2 times; ",
        "RULE-overview requires exactly once\n",
        "docs/ar/AR-overview.md:3: error: AR-overview.system-overview cites AR-one-more 0 times; ",
        "RULE-overview requires exactly once\n",
        "docs/fs/FS-demo.md:6: error: FS-demo.requirements cites REQ 0 times; ",
        "--rule requires exactly 3\n",
    );
    assert_run(&by_code, 1, expected, "");

    let both = run(
        &root,
        &[
            "check",
            ".",
            "--rule",
            TRIAL,
            "--only",
            "citation-cardinality",
            "--only-rule",
        ],
    );
    assert_run(&both, 1, TRIAL_FINDING, "");
}

/// §FS-check.1.4: `--ignore` wins over both axes, so ignoring the trial
/// sentence's own code legitimately empties a scoped report.
#[test]
fn ignore_wins_over_the_authority_axis() {
    let output = run(
        &fixture(),
        &[
            "check",
            ".",
            "--rule",
            TRIAL,
            "--only-rule",
            "--ignore",
            "citation-cardinality",
        ],
    );
    assert_run(&output, 0, "success\n", "");
}

/// §FS-rules.6, §FS-rules.8: a finding the trial sentence and a declared rule
/// authored jointly is **in** the scoped report, with its tail exactly as an
/// unscoped run renders it — no marker, no reordering, no second line.
#[test]
fn a_jointly_authored_finding_is_retained_with_its_tail_unchanged() {
    let sentence = "The requirements chapter of each FS must cite at least one REQ.";
    let joint = "docs/fs/FS-demo.md:6: error: FS-demo.requirements must cite REQ \
                 (--rule, RULE-requirements)\n";
    let root = fixture();

    // The merge itself already happens: one finding, both origins in the tail.
    let unscoped = run(&root, &["check", ".", "--rule", sentence]);
    assert!(
        text(&unscoped.stdout).contains(joint),
        "the joint tail is not what this fixture renders:\n{}",
        text(&unscoped.stdout)
    );

    let scoped = run(&root, &["check", ".", "--rule", sentence, "--only-rule"]);
    assert_run(&scoped, 1, joint, "");
}

/// §FS-rules.6, §FS-rules.8: a trial sentence duplicating a `[citations]`
/// direction yields an **empty** scoped report. The config finding wins
/// byte-for-byte and the rule authored nothing, so there is nothing for
/// `authority` to name — and the run says `success` and exits 0 while the
/// tree's own findings still stand. Known behavior, pinned so it cannot change
/// by accident.
#[test]
fn a_sentence_duplicating_a_citation_direction_yields_an_empty_scoped_report() {
    let root = scratch("citations-duplicate");
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    write(
        &root,
        "grund.toml",
        &format!("{config}\n[citations.FS]\nmust = [\"GOAL\"]\n"),
    );
    let sentence = "Each FS must cite at least one GOAL.";
    let direction = "docs/fs/FS-demo.md:1: error: FS-demo must cite GOAL (citation direction)\n";

    // The config finding wins byte-for-byte: no `--rule` reaches the tail.
    let unscoped = run(&root, &["check", ".", "--rule", sentence]);
    assert!(
        text(&unscoped.stdout).contains(direction),
        "the config finding is not what this fixture renders:\n{}",
        text(&unscoped.stdout)
    );

    let scoped = run(&root, &["check", ".", "--rule", sentence, "--only-rule"]);
    assert_run(&scoped, 0, "success\n", "");
}

/// §FS-rules.8, §FS-check.2.3: a `should`-level sentence lands in the
/// suggestions channel, which the default run withholds — so scoping alone
/// prints `success`, and `--suggestions` is what shows it. Suggestions never
/// move the exit, so the scoped run exits 0 either way.
#[test]
fn a_should_level_sentence_needs_suggestions_to_be_seen() {
    let sentence = "Each FS should have exactly one security chapter.";
    let root = fixture();
    assert_run(
        &run(&root, &["check", ".", "--rule", sentence, "--only-rule"]),
        0,
        "success\n",
        "",
    );
    assert_run(
        &run(
            &root,
            &[
                "check",
                ".",
                "--rule",
                sentence,
                "--only-rule",
                "--suggestions",
            ],
        ),
        0,
        "docs/fs/FS-demo.md:1: suggestion: FS-demo has 0 security chapters; \
         --rule requires exactly one; expected display name \"security\" (case-insensitive, not section handle); observed direct chapters: \"goals: Goals\", \"requirements: Requirements\"\n",
        "",
    );
}

/// §FS-rules.7.6, §FS-rules.7.1: **the blocker.** A syntactically valid
/// sentence whose literal subject does not resolve is a report finding, not a
/// rule evaluation, so it never passes through the group/origin join. A
/// selector written as "keep what the evaluator attributed to `--rule`" drops
/// it and renders a typo as `success` — reporting nothing, silently, which is
/// the failure this whole surface exists to remove. It is retained because the
/// diagnostic carries its one rule's origin as its authority.
#[test]
fn a_typod_sentence_is_retained_rather_than_read_as_success() {
    let sentence = "FS-nosuch.requirements must cite at least one REQ.";
    let invalid = "error: --rule is not a valid rule: literal subject FS-nosuch.requirements \
                   does not resolve\n";
    let root = fixture();

    // It is a located report finding on stdout at exit 1, not a pre-scan refusal.
    let unscoped = run(&root, &["check", ".", "--rule", sentence]);
    assert_eq!(unscoped.status.code(), Some(1));
    assert!(
        text(&unscoped.stdout).starts_with(invalid),
        "the invalid-rule finding is not on stdout:\n{}",
        text(&unscoped.stdout)
    );

    let scoped = run(&root, &["check", ".", "--rule", sentence, "--only-rule"]);
    assert_run(&scoped, 1, invalid, "");
}

/// §FS-rules.7.6, §FS-rules.7.1: the same diagnostic in JSON names the one
/// rule that produced it, with `path` null and `line` 1.
#[test]
fn a_typod_sentences_json_names_the_rule_that_produced_it() {
    let sentence = "FS-nosuch.requirements must cite at least one REQ.";
    let output = run(
        &fixture(),
        &[
            "check",
            ".",
            "--rule",
            sentence,
            "--only-rule",
            "--format",
            "json",
        ],
    );
    let expected = concat!(
        "{\"severity\":\"error\",\"path\":null,\"line\":1,\"code\":\"invalid-rule\",",
        "\"message\":\"--rule is not a valid rule: literal subject FS-nosuch.requirements ",
        "does not resolve\",\"sites\":null,\"authority\":[\"--rule\"]}\n",
    );
    assert_run(&output, 1, expected, "");
}

/// §FS-errors.5.1, §FS-output-shapes.1, §FS-distribution.3.0.1: `authority` is
/// the last key of every finding record — a bytewise-sorted list of the rule
/// origins that authored the finding, `null` where none did. The four rows here
/// cover all three shapes the field takes: one declared rule, the trial
/// sentence alone, and the joined pair.
#[test]
fn every_finding_record_carries_the_authority_that_produced_it() {
    let output = run(
        &fixture(),
        &["check", ".", "--rule", TRIAL, "--format", "json"],
    );
    let expected = concat!(
        "{\"severity\":\"error\",\"path\":\"docs/ar/AR-overview.md\",\"line\":3,",
        "\"code\":\"citation-cardinality\",\"message\":\"AR-overview.system-overview cites ",
        "AR-one 2 times; RULE-overview requires exactly once\",\"sites\":null,",
        "\"authority\":[\"RULE-overview\"]}\n",
        "{\"severity\":\"error\",\"path\":\"docs/ar/AR-overview.md\",\"line\":3,",
        "\"code\":\"citation-cardinality\",\"message\":\"AR-overview.system-overview cites ",
        "AR-one-more 0 times; RULE-overview requires exactly once\",\"sites\":null,",
        "\"authority\":[\"RULE-overview\"]}\n",
        "{\"severity\":\"error\",\"path\":\"docs/fs/FS-demo.md\",\"line\":6,",
        "\"code\":\"citation-cardinality\",\"message\":\"FS-demo.requirements cites REQ 0 times; ",
        "--rule requires exactly 3\",\"sites\":null,\"authority\":[\"--rule\"]}\n",
        "{\"severity\":\"error\",\"path\":\"docs/fs/FS-demo.md\",\"line\":6,",
        "\"code\":\"missing-citation\",\"message\":\"FS-demo.requirements must cite REQ ",
        "(RULE-requirements)\",\"sites\":null,\"authority\":[\"RULE-requirements\"]}\n",
    );
    assert_run(&output, 1, expected, "");
}

/// §FS-rules.6, §FS-errors.5.1: a jointly authored record names both origins in
/// bytewise order — the same order and the same set the message tail joins, so
/// text and JSON cannot disagree about who said it.
#[test]
fn a_jointly_authored_record_names_both_origins_in_bytewise_order() {
    let sentence = "The requirements chapter of each FS must cite at least one REQ.";
    let output = run(
        &fixture(),
        &[
            "check",
            ".",
            "--rule",
            sentence,
            "--only-rule",
            "--format",
            "json",
        ],
    );
    let expected = concat!(
        "{\"severity\":\"error\",\"path\":\"docs/fs/FS-demo.md\",\"line\":6,",
        "\"code\":\"missing-citation\",\"message\":\"FS-demo.requirements must cite REQ ",
        "(--rule, RULE-requirements)\",\"sites\":null,",
        "\"authority\":[\"--rule\",\"RULE-requirements\"]}\n",
    );
    assert_run(&output, 1, expected, "");
}

/// §FS-errors.5.2: the key set is one set. A failed ID query is rendered in the
/// same shape on stderr and carries `authority` present and `null` — no query
/// failure is a chapter rule's, and a conditional key would make every consumer
/// branch on the shape it got.
#[test]
fn a_failed_id_query_carries_a_null_authority() {
    let output = run(&fixture(), &["FS-nosuch", "--format", "json"]);
    let expected = concat!(
        "{\"severity\":\"error\",\"path\":null,\"line\":null,\"code\":\"not-found\",",
        "\"message\":\"ID not found: FS-nosuch\",\"sites\":null,\"authority\":null}\n",
    );
    assert_run(&output, 1, "", expected);
}

/// §FS-errors.5.2.3: a run-level finding is about the run rather than about a
/// unit, so no rule authored it and it carries a null authority. What selection
/// then does with it is
/// `a_scoped_run_drops_a_run_level_warning_as_the_code_axis_does`.
#[test]
fn a_run_level_warning_carries_a_null_authority() {
    let root = scratch("run-level-authority");
    fs::remove_dir_all(root.join("docs")).expect("empty the scan");
    fs::create_dir_all(root.join("docs")).expect("recreate the scanned folder");
    let output = run(&root, &["check", ".", "--format", "json"]);
    assert_eq!(output.status.code(), Some(0), "unexpected exit");
    assert_eq!(text(&output.stdout), "", "a warning is not stdout's");
    assert!(
        text(&output.stderr).contains("\"sites\":null,\"authority\":null}"),
        "the run-level warning does not carry a null authority:\n{}",
        text(&output.stderr)
    );
}

/// §FS-check.2.1.2: the authority axis inherits selection's powerlessness over
/// an incomplete run exactly as the code axis has it — the `io` finding stays
/// visible and the run still exits 2.
#[cfg(unix)]
#[test]
fn an_incomplete_run_survives_the_authority_axis() {
    use std::os::unix::fs::symlink;

    let root = scratch("incomplete-scope");
    symlink("missing-target.md", root.join("docs/fs/FS-gone.md")).expect("create broken symlink");
    let output = run(&root, &["check", ".", "--rule", TRIAL, "--only-rule"]);
    assert_run(
        &output,
        2,
        "",
        "error: docs/fs/FS-gone.md: broken symlink: the target does not exist\n",
    );
}

/// §FS-errors.5.2.3, §FS-check.2.1.3: what a scoped run costs an author whose
/// tree scanned nothing. No rule authored the empty-scan warning, so the
/// authority axis drops it and the run prints `success` at exit 0 — the code
/// axis drops it the same way, which is why this is selection working rather
/// than `--only-rule` diverging. The half that passes before this case existed
/// is the premise: the plain run does print the warning.
#[test]
fn a_scoped_run_drops_a_run_level_warning_as_the_code_axis_does() {
    const FAMILY: &str = "Each FS must have exactly one security chapter.";

    let root = scratch("run-level-scoped");
    fs::remove_dir_all(root.join("docs")).expect("empty the scan");
    fs::create_dir_all(root.join("docs")).expect("recreate the scanned folder");

    let plain = run(&root, &["check", ".", "--rule", FAMILY]);
    assert_eq!(
        plain.status.code(),
        Some(0),
        "a warning does not move the exit"
    );
    assert_eq!(text(&plain.stdout), "", "a warning is not stdout's");
    assert!(
        text(&plain.stderr).starts_with("warning: nothing to scan"),
        "the premise does not hold — the plain run says no such warning:\n{}",
        text(&plain.stderr)
    );

    let scoped = run(&root, &["check", ".", "--rule", FAMILY, "--only-rule"]);
    assert_run(&scoped, 0, "success\n", "");

    let by_code = run(&root, &["check", ".", "--only", "missing-citation"]);
    assert_run(&by_code, 0, "success\n", "");
}
