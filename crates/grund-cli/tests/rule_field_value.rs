//! Black-box: a rule subject may end in one whose clause, `Each KIND whose FIELD is VALUE`,
//! that narrows it to the declarations whose scalar field holds VALUE (§FS-rules.2.2). The
//! same subject reads alike on `check --rule`, a declared rule and `list --selector`
//! (§FS-rules.8), a declaration it leaves out is outside the rule (§FS-rules.5.2,
//! §FS-rules.requirements.unseen), and a filtered rule never folds into a direction (§FS-rules.6).
//! Ports the #1047 reproducer; the refusals are `rule_field_value_refusals.rs`.

#[path = "support/rule_field_value.rs"]
mod support;

use std::fs;

use support::*;

/// The issue's own run: the filtered rule reports only the requirement verified by test.
#[test]
fn the_issue_rule_reports_only_the_test_requirement() {
    let fx = Fixture::issue(CONFIG);
    let run = fx.expect(1, &["check", "--rule", RULE, "--only-rule"]);
    assert_eq!(run.stdout, BRAKE_UNCITED);
    assert_eq!(run.stderr, "");
}

/// The same sentence as a declared version-2 rule selects the same declarations (§FS-rules.1).
#[test]
fn a_declared_rule_selects_what_the_trial_sentence_selects() {
    let config = format!(
        "{CONFIG}\n[schema.kinds.RULE]\nfiles = [\"rules/RULE-tested.md\"]\nform = \"rule\"\n"
    );
    let fx = Fixture::issue(&config);
    fs::create_dir_all(fx.0.join("rules")).unwrap();
    fs::write(
        fx.0.join("rules/RULE-tested.md"),
        format!("# RULE-tested: {RULE}\n\nA requirement verified by test needs a test case.\n"),
    )
    .unwrap();
    let run = fx.expect(1, &["check", "--only", "uncited-unit"]);
    assert_eq!(
        run.stdout,
        BRAKE_UNCITED.replace("--rule requires", "RULE-tested requires")
    );
}

/// `list --selector` and `list --size --selector` list only the selected declaration.
#[test]
fn the_selector_lists_only_the_selected_declaration() {
    let fx = Fixture::issue(CONFIG);
    let selector = "Each SWR whose verification is Test";
    assert_eq!(
        fx.expect(0, &["list", "--selector", selector]).stdout,
        "SWR-brake-latency  req/SWR-brake-latency.md:1  The brake command reaches the actuator \
         within 10 ms\n"
    );
    assert_eq!(
        fx.expect(0, &["list", "--size", "--selector", selector])
            .stdout,
        "SWR-brake-latency  req/SWR-brake-latency.md:1  lines=0/2 words=0/12 bytes=0/75\n"
    );
}

/// The chapter form: a selected declaration without the chapter is unreached, and one the
/// clause leaves out is silent (§FS-rules.checks.unreached-declaration).
#[test]
fn the_chapter_form_reports_only_selected_declarations_as_unreached() {
    let fx = Fixture::with(&[
        (
            "grund.toml",
            &CONFIG
                .replace(
                    "\"req/SWR-fault-log.md\"]",
                    "\"req/SWR-fault-log.md\", \"req/SWR-wheel-speed.md\"]",
                )
                .replace("tc/TC-placeholder.md", "tc/TC-bench.md"),
        ),
        (
            "req/SWR-brake-latency.md",
            &format!("{BRAKE}\n## evidence: Evidence\n\nTested by §TC-bench.\n"),
        ),
        ("req/SWR-fault-log.md", FAULT),
        (
            "req/SWR-wheel-speed.md",
            "# SWR-wheel-speed: Wheel speed is sampled at 1 kHz\n\n## verification: Test\n\n\
             Sampled on the rig.\n",
        ),
        (
            "tc/TC-bench.md",
            "# TC-bench: Bench latency test\n\nCovers §SWR-wheel-speed.\n",
        ),
        ("an/AN-brake-latency.md", ANALYSIS),
    ]);
    let run = fx.expect(
        1,
        &[
            "check",
            "--rule",
            "The evidence chapter of each SWR whose verification is Test must cite at least \
             one TC.",
            "--only-rule",
        ],
    );
    assert_eq!(
        run.stdout,
        "req/SWR-wheel-speed.md:1: error: SWR-wheel-speed has no evidence chapter, so --rule \
         cannot reach it; add the chapter, or narrow the rule to the declarations that have \
         one; this became an error in grund 0.16.0\n"
    );
}

/// An absent `may` field puts a declaration outside the rule, and nothing reports it.
#[test]
fn an_absent_optional_field_is_outside_the_rule() {
    let fx = Fixture::with(&[
        (
            "grund.toml",
            &CONFIG.replace("presence = \"must\"", "presence = \"may\""),
        ),
        ("req/SWR-brake-latency.md", BRAKE),
        (
            "req/SWR-fault-log.md",
            "# SWR-fault-log: Every actuator fault is written to the fault log\n\nNo method yet.\n",
        ),
        ("an/AN-brake-latency.md", ANALYSIS),
    ]);
    let run = fx.expect(1, &["check", "--rule", RULE]);
    assert!(run.stdout.contains(BRAKE_UNCITED), "{}", run.stdout);
    assert!(!run.stdout.contains("SWR-fault-log"), "{}", run.stdout);
}

/// A value outside `one_of` is `invalid-field-value` unscoped, and outside the rule, so a
/// run scoped to the sentence shows nothing for it (§FS-rules.8).
#[test]
fn an_invalid_value_is_reported_by_the_field_and_not_by_the_rule() {
    let fx = Fixture::with(&[
        ("grund.toml", CONFIG),
        ("req/SWR-brake-latency.md", BRAKE),
        ("req/SWR-fault-log.md", &FAULT.replace("Inspection", "Tset")),
        ("an/AN-brake-latency.md", ANALYSIS),
    ]);
    let run = fx.expect(1, &["check", "--rule", RULE]);
    assert!(run.stdout.contains(BRAKE_UNCITED), "{}", run.stdout);
    assert!(
        run.stdout.contains(
            "req/SWR-fault-log.md:3: error: SWR-fault-log.verification is \"Tset\"; kind SWR \
             admits Test, Analysis, Inspection, Demonstration; retitle the heading to one of them\n"
        ),
        "{}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("SWR-fault-log is cited by"),
        "{}",
        run.stdout
    );
    assert_eq!(
        fx.expect(1, &["check", "--rule", RULE, "--only-rule"])
            .stdout,
        BRAKE_UNCITED
    );
}

/// A filtered subject is never kind-wide, so it never folds into a direction: both report,
/// as an exact-ID subject does today (§FS-rules.6).
#[test]
fn a_filtered_rule_beside_a_direction_reports_both() {
    let fx = Fixture::issue(&format!(
        "{CONFIG}\n[rules.citations.SWR]\nmust = [\"TC\"]\n"
    ));
    let run = fx.expect(
        1,
        &[
            "check",
            "--rule",
            "Each SWR whose verification is Test must cite at least one TC.",
        ],
    );
    for line in [
        "req/SWR-brake-latency.md:1: error: SWR-brake-latency must cite TC (--rule); add such a \
         citation within it\n",
        "req/SWR-brake-latency.md:1: error: SWR-brake-latency must cite TC (citation direction); \
         add such a citation within it\n",
        "req/SWR-fault-log.md:1: error: SWR-fault-log must cite TC (citation direction); add such \
         a citation within it\n",
    ] {
        assert!(
            run.stdout.contains(line),
            "missing {line}in:\n{}",
            run.stdout
        );
    }
    assert!(
        !run.stdout.contains("SWR-fault-log must cite TC (--rule)"),
        "{}",
        run.stdout
    );
}

/// The modality split leaves a capitalised `Must fix` and a value holding ` and ` whole
/// (§FS-rules.3.6).
#[test]
fn values_with_spaces_and_capitalised_modal_words_parse_whole() {
    let fx = Fixture::table();
    let run = fx.expect(
        1,
        &[
            "check",
            "--rule",
            "Each AN whose status is Must fix must cite at least one TC.",
            "--only-rule",
        ],
    );
    assert_eq!(
        run.stdout,
        "an/AN-brake-latency.md:1: error: AN-brake-latency must cite TC (--rule); add such a \
         citation within it\n"
    );
    assert_eq!(
        fx.expect(
            0,
            &[
                "list",
                "--selector",
                "Each AN whose status is fix and retest"
            ]
        )
        .stdout,
        ""
    );
}
