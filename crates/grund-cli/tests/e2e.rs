// §AR-bindings.3: the e2e tests exercise the dedicated CLI frontend crate.
use std::path::PathBuf;

#[path = "support/case_runner.rs"]
mod case_runner;

use case_runner::CaseKind::{E2e, Example};
use case_runner::{
    assert_case_is_deterministic, assert_every_case_passed, discover_e2e_cases, discover_examples,
    golden_form_violations, run_case,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// Every pass collects its per-case outcomes and hands them to
// `assert_every_case_passed`: a case that mismatched its goldens or that the
// platform could not build is counted and named there, never left to look
// like one of the passes libtest reports.

#[test]
fn e2e_cases_match_expected_reports() {
    let manifest_dir = repo_root();
    let outcomes = discover_e2e_cases(&manifest_dir)
        .iter()
        .map(|case| run_case(&manifest_dir, case, E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("e2e cases", &outcomes);
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
    assert_output(&["check", "."], 0, "success\n", "");
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
        "success\n",
        "",
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

/// Completion scripts participate in the same two independent runs as every
/// immutable case, so their bytes are stable across invocations (§FS-completions.3).
#[test]
fn e2e_output_is_deterministic() {
    let manifest_dir = repo_root();
    let outcomes = discover_e2e_cases(&manifest_dir)
        .iter()
        .map(|case| assert_case_is_deterministic(&manifest_dir, case))
        .collect::<Vec<_>>();
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
    assert!(read("repo/values/field-price.md").contains("## 1. 1200"));
    assert!(read("repo/values/runtime.json").contains("\"CONST-discount\""));

    // Both binding forms, and the runtime that reads the same JSON. The prose
    // binds the price as the one quantity it is, amount and unit together
    // (§FS-values.3.1.2); the Python comment binds one component.
    let prose = read("repo/docs/offer.md");
    assert!(
        prose.contains("`1200.0 USD` (\u{a7}CONST-field-price)"),
        "{prose}"
    );
    let code = read("repo/src/model.py");
    assert!(
        code.contains("`1.2e3` (\u{a7}CONST-field-price.1)"),
        "{code}"
    );
    assert!(code.contains("values/runtime.json"), "{code}");

    // The deliberate non-binding: an unbackticked literal beside a citation is
    // not a value claim, so the run below must not report it.
    assert!(
        prose.contains("not bound: 1200 (\u{a7}CONST-field-price.1)"),
        "{prose}"
    );

    // And the caught mismatch, which is the whole of the example's output.
    assert_eq!(read("expected.exit").trim(), "1");
    assert_eq!(read("expected.stderr"), "\n");
    assert_eq!(
        read("expected.stdout"),
        "docs/offer.md:7: error: value mismatch for CONST-discount.1: \
         bound `0.30`, declared `0.25` at values/runtime.json:2\n"
    );
}
