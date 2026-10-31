// §AR-bindings.3: the e2e tests exercise the dedicated CLI frontend crate.
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

#[path = "support/case_runner.rs"]
mod case_runner;
#[path = "../../../tests/integration/ordered_pool.rs"]
mod ordered_pool;

// §FS-init.2.3.5.10: the generated chapter rules survive a whole init/fmt cycle.
#[path = "support/chapter_rule_workflow.rs"]
mod chapter_rule_workflow;

#[path = "support/chapter_diagnostics.rs"]
mod chapter_diagnostics;

#[path = "support/configured_star_slug.rs"]
mod configured_star_slug;

// §FS-declarations.stubs.target.3: the extension-refusal explanation and its controls.
#[path = "support/stub_extension_diagnostic.rs"]
mod stub_extension_diagnostic;

#[path = "support/case_isolation.rs"]
mod case_isolation;

// A copied fixture script is never busy because of a sibling spawn (§FS-examples.5.6.3).
// Declared here rather than in the shared runner so it runs in this one binary.
#[cfg(unix)]
#[path = "support/case_fixture_exec_tests.rs"]
mod fixture_exec_tests;
#[cfg(unix)]
use case_runner::{copy_dir, run_grund};

#[path = "support/init_marker_guidance.rs"]
mod init_marker_guidance;

#[path = "support/mixed_path_contract.rs"]
mod mixed_path_contract;

#[path = "support/fmt_cross_refs_sections.rs"]
mod fmt_cross_refs_sections;

// §FS-check.4.1.2 / §FS-refs.3.1.1 / §FS-refs.3.3.1: issue #909.
#[path = "support/index_use_explanations.rs"]
mod index_use_explanations;

// §FS-config.3.9.3: the member recipe the guide teaches is the one the example runs.
#[path = "support/isolated_member_example.rs"]
mod isolated_member_example;

use case_runner::CaseKind::{E2e, Example};
use case_runner::{
    CaseOutcome, assert_case_is_deterministic, assert_every_case_passed, discover_e2e_cases,
    discover_examples, golden_form_violations, run_case,
};

/// A clean run's line on a version 1 config (§FS-check.checks.config-version-1).
const VERSION_1_CAUTION: &str =
    "warning: grund.toml is version 1; migrate it with grund config migrate --to 2\n";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// §FS-init.2.3.4.16: standalone omission and existing workspace-root/member controls.
#[test]
fn project_namespaces_golden_contract() {
    let root = repo_root();
    let outcomes = [
        "init-no-workspace-members",
        "init-workspace-root",
        "init-workspace-member",
    ]
    .iter()
    .map(|name| run_case(&root, &root.join("tests/e2e/cases").join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("project namespaces", &outcomes);
}

// Every pass collects its per-case outcomes and hands them to
// `assert_every_case_passed`: a case that mismatched its goldens or that the
// platform could not build is counted and named there, never left to look
// like one of the passes libtest reports.

#[test]
fn e2e_cases_match_expected_reports() {
    let started = Instant::now();
    let manifest_dir = repo_root();
    // §REQ-one-answer.proof.delivery.payment: the cases overlap, each one run and compared
    // on one worker, outcomes in corpus order (§AR-workspace.9.3).
    let outcomes = ordered_pool::in_order(&discover_e2e_cases(&manifest_dir), |case| {
        run_case(&manifest_dir, case, E2e)
    });
    let compared = outcomes
        .iter()
        .filter(|outcome| matches!(outcome, CaseOutcome::Ran | CaseOutcome::Failed { .. }))
        .count();
    // §AR-ci.requirements.5.1: past libtest's capture, the seconds this sweep costs.
    let _ = writeln!(
        std::io::stderr(),
        "gate time: e2e cases {compared} case(s) on {} worker(s) in {:.1}s",
        ordered_pool::workers(),
        started.elapsed().as_secs_f64()
    );
    assert_every_case_passed("e2e cases", &outcomes);
}

/// §FS-check.1.1.10 (§FS-citations.recognized.marker-start): port the control/subject reproducer across all graph consumers.
#[test]
fn word_character_marker_cli_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let mut outcomes = Vec::new();
    for marker in ["default", "underscore"] {
        for surface in ["refs", "cover", "check", "missing"] {
            let name = format!("word-character-marker-{marker}-{surface}");
            outcomes.push(run_case(&root, &cases.join(name), E2e));
        }
    }
    assert_every_case_passed("configured marker graph", &outcomes);
}

/// §FS-config-explain.1.1, §FS-config-explain.5: issue #1005's cases, run alone. Every
/// one is also in the full pass: a path with one reading answered, and the failures
/// that name every reading of a bare name or of a path with two.
#[test]
fn config_explain_readings_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "config-explain-v2-dotted-table",
        "config-explain-v1-dotted-key",
        "config-explain-v2-dotted-placeholder",
        "config-explain-v2-dotted-placeholder-key",
        "config-explain-v2-bare-key",
        "config-explain-v2-placeholder-key",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("config explain readings", &outcomes);
}

/// §FS-config-v2: the version-2 reader's contract, run alone. Every case is also
/// in the full pass; the two `config-v1-*` cases are the v1 halves that must not move.
#[test]
fn v2_config_reader_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "config-unsupported-version",
        "config-v2-citation-warn",
        "config-v2-citation-warn-json",
        "config-v2-citation-warn-cleared",
        "config-v2-validate-worked-example",
        "config-v2-citation-must",
        "config-v2-citation-should",
        "config-v2-citation-may",
        "config-v2-citation-any-of",
        "config-v2-citation-prohibitions",
        "config-v2-citation-default-must-refused",
        "config-v1-citation-default-must-loads",
        "config-v2-resolution-warn",
        "config-v2-resolution-without-fetch-refused",
        "config-v2-grounding-ladder-replaced-whole",
        "config-v1-grounding-keys-inherited-independently",
        "config-v2-grounding-ladder-order-refused",
        "config-v2-unsupported-clause-refused",
        "config-v2-languages-omitted-epoch",
        "config-v2-repeated-table-refused",
        "config-v2-v1-key-refused",
        "config-v2-strength-table-collision-refused",
        "config-v2-defaults-epoch",
        "config-v2-workspace-configless-member-keeps-v1",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("v2 config reader", &outcomes);
}

/// §FS-config-v2.schema.sources, §FS-config-v2.schema.sources.declarations: v2 sources and
/// the doc-comment declaration gate, run alone. Every case is also in the full
/// pass; the `*-v1-*` cases are the v1 halves that must not move.
#[test]
fn v2_sources_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "declarations-ordinary-comment-v1-list",
        "declarations-ordinary-comment-v1-check",
        "declarations-ordinary-comment-v1-refs",
        "declarations-ordinary-comment-v2-list",
        "declarations-ordinary-comment-v2-check",
        "declarations-ordinary-comment-v2-refs",
        "declarations-ordinary-comment-v2-show",
        "declarations-ordinary-comment-v2-any-comment-list",
        "config-v2-languages-omitted-epoch",
        "config-v2-sources-accepted-list",
        "config-v2-sources-exclude-globs-list",
        "config-v2-sources-exclude-root-outruns-list",
        "config-v2-sources-config-show",
        "config-v2-sources-config-show-epoch",
        "config-v2-sources-unknown-language-refused",
        "config-v2-sources-unknown-definition-refused",
        "config-v2-sources-invalid-definition-refused",
        "config-scan-exclude-name-every-depth",
        "config-v2-sources-no-languages-list",
        "config-v2-sources-json-only-list",
        "config-v2-sources-exclude-scan-false-full",
        "config-v2-sources-values-mapped-suffix-list",
        "config-v2-sources-values-data-definition-list",
        "config-v2-sources-values-json-remapped-source-list",
        "declarations-off-grammar-plain-note-v2-doc-comments-check",
        "declarations-off-grammar-plain-note-v2-any-comment-check",
        "declarations-ineligible-line-in-body-v2-show",
        "declarations-stub-ineligible-target-v2-doc-comments-check",
        "declarations-stub-ineligible-target-v2-any-comment-check",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("v2 sources", &outcomes);
}

/// §FS-fields: a v2 kind's fields, `closed` and `form`, run alone. Every case is
/// also in the full pass; the `*-v1-*` cases are the v1 halves that must not move.
#[test]
fn v2_kind_fields_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "check-v2-field-terms-first",
        "check-v2-field-terms-first-json",
        "check-v2-field-only",
        "check-v2-field-after",
        "check-v2-field-status-must",
        "check-v2-field-status-warn",
        "check-v2-field-status-should",
        "check-v2-field-status-should-suggestions",
        "check-v2-field-status-may",
        "check-v2-field-closed-must",
        "check-v2-field-closed-warn",
        "check-v2-field-closed-should",
        "check-v2-field-closed-may",
        "check-v2-field-values-chapter",
        "check-chapter-value-invalid-shape",
        "check-v2-form-twin",
        "check-v1-form-twin",
        "check-v2-field-rule-pairing-missing",
        "check-v2-field-rule-pairing-two-titles",
        "check-v2-field-rule-pairing-other-name",
        "check-v2-field-rule-pairing-warn-field",
        "config-v2-field-position-and-after-refused",
        "config-v2-field-two-first-refused",
        "config-v2-field-after-unknown-refused",
        "config-v2-field-after-cycle-refused",
        "config-v2-field-one-of-title-refused",
        "config-v2-field-on-value-form-refused",
        "config-v1-fields-unknown-key",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("v2 kind fields", &outcomes);
}

/// §FS-show.3.5.2: the check breadcrumb belongs only to the existing-path
/// migration case. Port grund.33's clean-check and resolving-ID controls before
/// comparing the malformed coordinate and both non-path alias refusals.
#[test]
fn refused_queries_recommend_check_only_for_existing_paths() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let fixture = cases.join("cli-invalid-section-no-check-hint/repo");
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&fixture)
            .output()
            .expect("spawn this checkout's grund")
    };
    let assert_output = |args: &[&str], code, stdout: &str, stderr: &str| {
        let output = run(args);
        assert_eq!(output.status.code(), Some(code), "{args:?}: {output:?}");
        assert_eq!(String::from_utf8_lossy(&output.stdout), stdout, "{args:?}");
        assert_eq!(String::from_utf8_lossy(&output.stderr), stderr, "{args:?}");
    };
    // §FS-check.checks.config-version-1: the fixture is on version 1.
    let caution = VERSION_1_CAUTION;
    assert_output(&["check", "."], 0, "", caution);
    assert_output(
        &["FS-widget", "--brief"],
        0,
        "# FS-widget: The widget\n\nThe widget does a thing.\n",
        "",
    );

    // §FS-show.3.5.1: explicit show retains its format/list advice.
    assert_output(
        &["show", "FS-widget.1.nope"],
        1,
        "",
        concat!(
            "invalid ID `FS-widget.1.nope`\n",
            "hint: this repo's [id] format is `{kind}-{slug}` (run `grund config show`); ",
            "`grund list` shows the IDs that exist\n",
        ),
    );
    // §FS-show.3.5: an absent numeric section still offers a working map read.
    assert_output(
        &["FS-widget.99"],
        1,
        "",
        concat!(
            "section not found: FS-widget.99\n",
            "hint: run `grund FS-widget --toc` to print the lead with the section map\n",
        ),
    );
    assert_output(
        &["FS-widget", "--toc"],
        0,
        "The widget does a thing.\n\n## 1. It starts\n## 2. It stops\n",
        "",
    );

    // Follow the preserved breadcrumb's own command, using a real fixture path.
    let path_refusal = run(&["docs/functional-spec"]);
    assert_eq!(path_refusal.status.code(), Some(1));
    assert!(path_refusal.stdout.is_empty());
    let stderr = String::from_utf8(path_refusal.stderr).expect("UTF-8 stderr");
    let command = stderr
        .lines()
        .find_map(|line| {
            line.strip_prefix("hint: run `grund ")?
                .strip_suffix("` to validate a path")
        })
        .expect("existing-path migration breadcrumb");
    assert_eq!(command, "check docs/functional-spec");
    assert_output(
        &command.split_whitespace().collect::<Vec<_>>(),
        0,
        "",
        caution,
    );

    let outcomes = [
        "cli-invalid-section-no-check-hint",
        "cli-invalid-subcommand",
        "workspace-nested-include-root-false-root-alias",
        "workspace-nested-invalid-alias-segment",
        "cli-bare-path-requires-check",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("show failure hints", &outcomes);
}

/// §FS-rules.2.1: a chapter subject's NAME is the chapter's whole path, on every
/// surface that reads one — `list --selector`, `--size`, and `check` — and under
/// a `:` separator too. The dotted cases port grund.72's five; the
/// one-component cases hold `FS.terms` to the top-level chapter only.
#[test]
fn chapter_subject_name_is_its_whole_path() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "list-selector-chapter-dotted-name",
        "list-selector-chapter-dotted-name-sentence",
        "list-size-selector-chapter-dotted-name",
        "list-selector-chapter-dotted-name-colon-separator",
        "check-rules-chapter-dotted-name-reached",
        "check-rules-chapter-dotted-name-unreached",
        "check-rules-chapter-dotted-name-prohibition",
        "list-selector-chapter-one-component-name",
        "list-size-selector-chapter-one-component-name",
        "check-rules-chapter-one-component-name",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("chapter subject whole path", &outcomes);
}

/// §FS-languages.fence.comment-block: grund#641's tree, a heading inside a fence in a
/// `///`, `/** */` or docstring block is no section on any surface, while a citation in
/// the fence stays live, a value marker on a fenced heading is an illustration, and an
/// unclosed fence ends with its own comment block.
#[test]
fn a_fenced_heading_in_a_doc_comment_is_not_a_section() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "toc",
        "block-toc",
        "docstring-toc",
        "inner-tilde-toc",
        "go-toc",
        "full",
        "refs-json",
        "check-missing",
        "check-level",
        "check-duplicate",
        "fmt",
        "citation-live",
        "unclosed-ends-with-block",
        "value-marker",
    ]
    .iter()
    .map(|name| {
        let case = cases.join(format!("doc-comment-fenced-heading-{name}"));
        run_case(&root, &case, E2e)
    })
    .collect::<Vec<_>>();
    assert_every_case_passed("doc-comment fenced heading", &outcomes);
}

/// Completion scripts participate in the same two independent runs as every
/// immutable case, so their bytes are stable across invocations (§FS-completions.3).
#[test]
fn e2e_output_is_deterministic() {
    let started = Instant::now();
    let manifest_dir = repo_root();
    // §REQ-deterministic-output.1, §REQ-one-answer.proof.delivery.payment: the cases
    // overlap, each one's two runs stay one pair on one worker, outcomes in corpus order.
    let outcomes = ordered_pool::in_order(&discover_e2e_cases(&manifest_dir), |case| {
        assert_case_is_deterministic(&manifest_dir, case)
    });
    let compared = outcomes
        .iter()
        .filter(|outcome| matches!(outcome, CaseOutcome::Ran | CaseOutcome::Failed { .. }))
        .count();
    // §AR-ci.requirements.5.1: past libtest's capture, the seconds this sweep costs.
    let _ = writeln!(
        std::io::stderr(),
        "gate time: e2e determinism {compared} case(s) on {} worker(s) in {:.1}s",
        ordered_pool::workers(),
        started.elapsed().as_secs_f64()
    );
    assert_every_case_passed("e2e determinism", &outcomes);
}

/// Every maintained example runs through the ordinary case runner, goldens and
/// final tree included. §FS-examples.5.2 travels here: `examples/external-tickets`
/// fetches through its own repo-local stub integration, so the pass needs no
/// network, and its `expected.repo` is compared byte-for-byte — which is what
/// holds the snapshot declaration the fetcher printed in the final repository.
#[test]
fn examples_are_e2e_cases() {
    let manifest_dir = repo_root();
    let outcomes = discover_examples(&manifest_dir)
        .iter()
        .map(|case| run_case(&manifest_dir, case, Example))
        .collect::<Vec<_>>();
    assert_every_case_passed("examples", &outcomes);
}

#[test]
fn example_output_is_deterministic() {
    let manifest_dir = repo_root();
    let outcomes = discover_examples(&manifest_dir)
        .iter()
        .map(|case| assert_case_is_deterministic(&manifest_dir, case))
        .collect::<Vec<_>>();
    assert_every_case_passed("example determinism", &outcomes);
}

/// The goldens are themselves a contract, not just a comparison: every case's
/// are in the one on-disk form the harness writes, so refreshing the case a
/// change is about rewrites no other case's bytes (§AR-workspace.9.1.3). Judged as
/// bytes and reported all at once — the tree should be fixable from this failure
/// alone.
#[test]
fn goldens_are_in_canonical_form() {
    let manifest_dir = repo_root();
    let mut violations = golden_form_violations(&manifest_dir, &discover_e2e_cases(&manifest_dir));
    violations.extend(golden_form_violations(
        &manifest_dir,
        &discover_examples(&manifest_dir),
    ));
    assert!(
        violations.is_empty(),
        "{} golden file(s) are not in the canonical form of AR-workspace.9.1 — an output \
         golden is never zero bytes and holds no carriage return, an exit golden is the \
         decimal code and exactly one newline. UPDATE_EXPECTED=1 rewrites every one of \
         these, whatever case the change was about:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

/// §FS-examples.2.1: the first-class-values example is part of the maintained
/// suite rather than a stray fixture — `discover_examples` finds it, so the
/// passes above run it — and it still shows every ingredient §FS-examples.2.1 asks a
/// reader to see.
///
/// Asserted beside the runs rather than inside them because deleting the
/// directory is the failure this guards against: a missing example makes the
/// passes above smaller, not red.
#[test]
fn the_values_example_shows_what_it_is_the_example_of() {
    let root = repo_root();
    let example = root.join("examples/values");
    assert!(
        discover_examples(&root).contains(&example),
        "the values example is not in the maintained suite"
    );

    let read = |relative: &str| {
        std::fs::read_to_string(example.join(relative))
            .unwrap_or_else(|err| panic!("read examples/values/{relative}: {err}"))
    };

    // The opt-in itself, on the kind whose home holds the declarations.
    let config = read("repo/grund.toml");
    assert!(config.contains("kind = \"CONST\""), "{config}");
    assert!(config.contains("values = true"), "{config}");

    // Both declaration forms: Markdown sections, and a home JSON catalog.
    assert!(read("repo/values/time-step.md").contains("## 1. 0.001"));
    assert!(read("repo/values/runtime.json").contains("\"CONST-damping\""));

    // Both binding forms, and the runtime that reads the same JSON. The prose
    // binds the time step as the one quantity it is, amount and unit together
    // (§FS-values.3.1.2); the Python comment binds one component.
    let prose = read("repo/docs/solver.md");
    assert!(
        prose.contains("`1.0e-3 s` (\u{a7}CONST-time-step)"),
        "{prose}"
    );
    let code = read("repo/src/model.py");
    assert!(code.contains("`1e-3` (\u{a7}CONST-time-step.1)"), "{code}");
    assert!(code.contains("values/runtime.json"), "{code}");

    // The deliberate non-binding: an unbackticked literal beside a citation is
    // not a value claim, so the run below must not report it.
    assert!(
        prose.contains("not bound: 0.001 (\u{a7}CONST-time-step.1)"),
        "{prose}"
    );

    // And the caught mismatch beside the version 1 caution its config earns
    // (§FS-check.checks.config-version-1). The prose binds the root, so the line
    // names the root and the literal whole (§FS-values.5.2.2, §FS-errors.3.9.1).
    assert_eq!(read("expected.exit").trim(), "1");
    assert_eq!(read("expected.stderr"), VERSION_1_CAUTION);
    assert_eq!(
        read("expected.stdout"),
        "docs/solver.md:7: error: value mismatch for CONST-time-step (component 1): \
         bound `0.002 s`, declared `0.001 s` at values/time-step.md:2; \
         change the literal to the declared value, or the declaration to the bound one\n"
    );
}
