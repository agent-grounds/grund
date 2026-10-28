//! Black-box: a whose clause the grammar refuses is answered for the part that failed, on
//! `check --rule`, a declared rule and `list --selector` alike (§FS-rule-refusals.sentences.whose,
//! §FS-rule-refusals.selectors), alone since the wording window of §FS-errors.3.10 closed; an
//! exact ID or an object kind with a clause appends why (§FS-rule-refusals.sentences.order).

#[path = "support/rule_field_value.rs"]
mod support;

use std::fs;

use support::*;

/// Each row of the §FS-rule-refusals.sentences.whose table the window of §FS-errors.3.10 moved:
/// `(sentence, final reason)`.
const REFUSED: &[(&str, &str)] = &[
    (
        "Each SWR whose verfication is Test must be cited by at least one TC.",
        "\"verfication\" is not a field of kind SWR; its scalar fields: verification",
    ),
    (
        "The evidence chapter of each SWR whose verfication is Test must cite at least one TC.",
        "\"verfication\" is not a field of kind SWR; its scalar fields: verification",
    ),
    (
        "Each SWR whose rationale is Test must be cited by at least one TC.",
        "field \"rationale\" of kind SWR is not a scalar: it sets no one_of; its scalar fields: \
         verification",
    ),
    (
        "Each TC whose verification is Test must cite at least one SWR.",
        "\"verification\" is not a field of kind TC; kind TC declares no scalar field",
    ),
    (
        "Each SWR whose verification is Tset must be cited by at least one TC.",
        "\"Tset\" is not a value of SWR.verification; kind SWR admits Test, Analysis, \
         Inspection, Demonstration",
    ),
    (
        "Each SWR whose verification is Test or Demonstration must be cited by at least one TC.",
        "\"Test or Demonstration\" is not a value of SWR.verification; kind SWR admits Test, \
         Analysis, Inspection, Demonstration; accepted forms: \"Each SWR whose verification is \
         Test must be cited by at least one TC.\" and \"Each SWR whose verification is \
         Demonstration must be cited by at least one TC.\"",
    ),
    (
        "Each AN whose status is fix must wait must cite at least one SWR.",
        "\"fix\" is not a value of AN.status; kind AN admits Draft, Must fix, fix and retest, fix \
         must wait — value \"fix must wait\" holds the modal word \"must\", which ends the \
         subject, so a rule cannot name it",
    ),
    (
        "Each SWR whose verification must be cited by at least one TC.",
        "whose clause \"whose verification\" has no \" is \"; write whose FIELD is VALUE",
    ),
    (
        "Each SWR whose  is Test must be cited by at least one TC.",
        "whose clause \"whose  is Test\" names no field; write whose FIELD is VALUE",
    ),
    (
        "Each SWR whose verification is  must be cited by at least one TC.",
        "whose clause \"whose verification is \" names no value; write whose FIELD is VALUE",
    ),
    (
        "Each SWR whose verification is Test  must be cited by at least one TC.",
        "value \"Test \" of SWR.verification has surrounding whitespace, which no one_of value \
         can hold",
    ),
];

/// On `check --rule` each refusal prints its final reason alone, with no legacy prefix and no
/// suffix, and nothing after it (§FS-errors.3.10).
#[test]
fn check_rule_refuses_a_clause_for_the_part_that_failed() {
    let fx = Fixture::table();
    for (sentence, reason) in REFUSED {
        let run = fx.expect(2, &["check", "--rule", sentence]);
        assert_eq!(run.stderr, format!("error: {reason}\n"), "{sentence}");
        assert_eq!(run.stdout, "", "{sentence}");
    }
}

/// A configured rule declaration's `invalid-rule` carries the same reason (§FS-rules.7.1).
/// The whitespace rows are left to `check --rule`: a heading's own spacing is not theirs.
#[test]
fn a_declared_rule_carries_the_same_reason() {
    let rows: Vec<_> = REFUSED
        .iter()
        .filter(|(sentence, _)| !sentence.contains("  "))
        .collect();
    let files: Vec<String> = (0..rows.len())
        .map(|n| format!("\"rules/RULE-r{n}.md\""))
        .collect();
    let config = format!(
        "{TABLE_CONFIG}\n[schema.kinds.RULE]\nfiles = [{}]\nform = \"rule\"\n",
        files.join(", ")
    );
    let fx = Fixture::table();
    fs::write(fx.0.join("grund.toml"), config).unwrap();
    fs::create_dir_all(fx.0.join("rules")).unwrap();
    for (n, (sentence, _)) in rows.iter().enumerate() {
        fs::write(
            fx.0.join(format!("rules/RULE-r{n}.md")),
            format!("# RULE-r{n}: {sentence}\n\nWhy.\n"),
        )
        .unwrap();
    }
    let run = fx.expect(1, &["check", "--only", "invalid-rule"]);
    for (n, (sentence, reason)) in rows.iter().enumerate() {
        let line =
            format!("rules/RULE-r{n}.md:1: error: RULE-r{n} is not a valid rule: {reason}\n");
        assert!(
            run.stdout.contains(&line),
            "{sentence}: missing {line}in:\n{}",
            run.stdout
        );
    }
}

/// On `list --selector` the reason is the rule's, no selector is offered, and no `known kinds:`
/// line follows, because the kind was recognised (§FS-rule-refusals.selectors, §FS-errors.3.10).
#[test]
fn the_selector_refuses_a_clause_for_the_part_that_failed() {
    let fx = Fixture::table();
    for (selector, reason) in [
        (
            "Each SWR whose verfication is Test",
            "\"verfication\" is not a field of kind SWR; its scalar fields: verification",
        ),
        (
            "The evidence chapter of each SWR whose verification is Tset",
            "\"Tset\" is not a value of SWR.verification; kind SWR admits Test, Analysis, \
             Inspection, Demonstration",
        ),
        (
            "Each SWR whose verification is Test or Demonstration",
            "\"Test or Demonstration\" is not a value of SWR.verification; kind SWR admits Test, \
             Analysis, Inspection, Demonstration",
        ),
    ] {
        for mode in [&["list"][..], &["list", "--size"][..]] {
            let args = [mode, &["--selector", selector][..]].concat();
            let run = fx.expect(2, &args);
            assert_eq!(run.stderr, format!("error: {reason}\n"), "{selector}");
        }
    }
}

/// An unknown kind is refused before its clause, naming the kind alone and keeping the
/// `known kinds:` line, on `check --rule` and every `list --selector` mode
/// (§FS-rule-refusals.sentences.order stage 5, §FS-errors.3.10).
#[test]
fn an_unknown_kind_is_refused_before_its_clause() {
    let fx = Fixture::table();
    let run = fx.expect(
        2,
        &[
            "check",
            "--rule",
            "Each POLICY whose status is Open must cite at least one SWR.",
        ],
    );
    assert_eq!(
        run.stderr,
        format!("error: unknown kind \"POLICY\"\n{KNOWN}")
    );
    for mode in [&["list"][..], &["list", "--size"][..]] {
        let args = [
            mode,
            &["--selector", "Each POLICY whose status is Open"][..],
        ]
        .concat();
        let run = fx.expect(2, &args);
        assert_eq!(
            run.stderr,
            format!("error: unknown kind \"POLICY\"\n{KNOWN}")
        );
    }
}

/// A version-1 config declares no scalar field, so any whose clause is refused for that, and
/// the recognised kind adds no `known kinds:` line.
#[test]
fn a_version_one_config_refuses_every_clause() {
    let fx = Fixture::with(&[
        (
            "grund.toml",
            "grund_config_version = 1\nproject_name = \"swr\"\n\n[id]\nformat = \
             \"{kind}-{slug}\"\nnamed_sections = true\n\n[[kinds]]\nkind = \"SWR\"\nfolder = \
             \"req\"\nindex = false\n\n[[kinds]]\nkind = \"TC\"\nfolder = \"tc\"\nindex = \
             false\n",
        ),
        ("req/SWR-brake-latency.md", BRAKE),
    ]);
    let reason = "whose clauses require grund_config_version = 2";
    let run = fx.expect(2, &["check", "--rule", RULE]);
    assert!(
        run.stderr.contains(&format!("error: {reason}\n")),
        "{}",
        run.stderr
    );
    let run = fx.expect(
        2,
        &["list", "--selector", "Each SWR whose verification is Test"],
    );
    assert!(
        run.stderr.ends_with(&format!("error: {reason}\n")),
        "{}",
        run.stderr
    );
    assert!(!run.stderr.contains("known kinds:"), "{}", run.stderr);
}

/// Guard, green today: an exact ID or an object kind followed by a clause keeps its released
/// reason and appends why, and never took the window (§FS-rule-refusals.sentences.whose).
#[test]
fn an_exact_id_or_an_object_with_a_clause_appends_why() {
    let fx = Fixture::table();
    let id = "literal subject \"SWR-brake-latency whose verification is Test\" does not match \
              the configured ID grammar — a whose clause narrows only Each KIND and The NAME \
              chapter of each KIND";
    let run = fx.expect(
        2,
        &[
            "check",
            "--rule",
            "SWR-brake-latency whose verification is Test must be cited by at least one TC.",
        ],
    );
    assert_eq!(run.stderr, format!("error: {id}\n"));
    let run = fx.expect(
        2,
        &[
            "list",
            "--selector",
            "SWR-brake-latency whose verification is Test",
        ],
    );
    assert_eq!(run.stderr, format!("error: {id}\n{KNOWN}"));
    let run = fx.expect(
        2,
        &[
            "check",
            "--rule",
            "Each AN must cite at least one SWR whose verification is Test.",
        ],
    );
    assert_eq!(
        run.stderr,
        format!(
            "error: unknown kind \"SWR whose verification is Test\" — a whose clause narrows \
             only a rule's subject\n{KNOWN}"
        )
    );
}

/// Guard, green today: `KIND.NAME` followed by a clause keeps its section-grammar reason and
/// accepted selector, and moves through no window (§FS-rule-refusals.selectors).
#[test]
fn the_shorthand_with_a_clause_keeps_its_section_grammar_reason() {
    let fx = Fixture::table();
    let run = fx.expect(
        2,
        &[
            "list",
            "--selector",
            "SWR.evidence whose verification is Test",
        ],
    );
    assert_eq!(
        run.stderr,
        "error: named chapter subject \"SWR.evidence whose verification is Test\" does not match \
         the configured section grammar; accepted selector: SWR\n"
    );
}

/// Guard, green today: a conjunction is refused before the subject, and so before its clause
/// (§FS-rule-refusals.sentences.order stage 3).
#[test]
fn a_conjunction_is_refused_before_the_clause() {
    let fx = Fixture::table();
    let run = fx.expect(
        2,
        &[
            "check",
            "--rule",
            "Each SWR whose verification is Tset must cite at least one TC and must not cite any \
             AN.",
        ],
    );
    assert_eq!(run.stderr, "error: conjunctions are not accepted\n");
}

/// Refusals a later production raises after an accepted clause: `(sentence, final reason,
/// what follows on `check --rule`)`. Before clauses existed each was an unknown kind.
const LATER: &[(&str, &str, &str)] = &[
    (
        "Each SWR whose verification is Test must cite every TC.",
        "quantifier \"every\" is not accepted",
        "",
    ),
    (
        "Each SWR whose verification is Test must cite at least one FOO.",
        "unknown kind \"FOO\"",
        KNOWN,
    ),
    (
        "The evidence chapter of each SWR whose verification is Test must have at least one \
         notes chapter.",
        // §FS-rule-refusals.sentences.rewrite.same-units: no sentence counts under a chapter.
        "chapter subjects cannot have chapters",
        "",
    ),
];

/// A sentence whose clause is accepted and which a later production refuses prints that
/// production's reason alone, on `check --rule` and as a declared rule (§FS-errors.3.10).
#[test]
fn a_later_refusal_after_an_accepted_clause_prints_its_reason_alone() {
    let fx = Fixture::table();
    for (sentence, reason, after) in LATER {
        let run = fx.expect(2, &["check", "--rule", sentence]);
        assert_eq!(
            run.stderr,
            format!("error: {reason}\n{after}"),
            "{sentence}"
        );
    }
    let files: Vec<String> = (0..LATER.len())
        .map(|n| format!("\"rules/RULE-r{n}.md\""))
        .collect();
    let config = format!(
        "{TABLE_CONFIG}\n[schema.kinds.RULE]\nfiles = [{}]\nform = \"rule\"\n",
        files.join(", ")
    );
    fs::write(fx.0.join("grund.toml"), config).unwrap();
    fs::create_dir_all(fx.0.join("rules")).unwrap();
    for (n, (sentence, _, _)) in LATER.iter().enumerate() {
        fs::write(
            fx.0.join(format!("rules/RULE-r{n}.md")),
            format!("# RULE-r{n}: {sentence}\n\nWhy.\n"),
        )
        .unwrap();
    }
    let run = fx.expect(1, &["check", "--only", "invalid-rule"]);
    for (n, (sentence, reason, _)) in LATER.iter().enumerate() {
        let line =
            format!("rules/RULE-r{n}.md:1: error: RULE-r{n} is not a valid rule: {reason}\n");
        assert!(
            run.stdout.contains(&line),
            "{sentence}: missing {line}in:\n{}",
            run.stdout
        );
    }
}
