//! §FS-fmt.7.2 — the unreadable paths `fmt` reports are the ones `check`
//! reports over the same walk. `fmt` is the model `check` verifies plus a write
//! step, so the one command that edits files in place may not have a private
//! account of the files it never saw.
//!
//! Equality is of the `<path>: <reason>` pairs and their order, not of the bytes
//! of the line: a strict abort spells its lines `error: nothing was rewritten:
//! …` on purpose (§FS-fmt.3.4), because the two exit `2`s mean opposite things.
//! `scan_error_pairs` removes that one licensed prefix and nothing else.
//!
//! The comparison is per walk rather than per argv, which is the part a scoped
//! run makes visible. A `fmt` that needs the whole declaration set scans the
//! project even under a narrowed path (§FS-fmt.2.4.5), so `grund fmt --check docs`
//! on a tree whose unreadable file sits in `src/` names that file while `grund
//! check docs` does not — and is compared against `grund check .`, which walks
//! what it walked. A partial run scans only its scope and is compared against
//! `check` over that scope.
//!
//! Unix only: the corpus needs real broken symlinks.

#![cfg(unix)]

#[path = "support/fmt_shapes.rs"]
mod fmt_shapes;

use fmt_shapes::{
    Shape, UNREADABLE_MANIFEST, aborted_strictly, byte_snapshot, e2e_case_shapes, run_grund,
    scan_error_pairs, shapes, stderr,
};

/// Each `fmt` form §FS-fmt.7.2 names, over the default scope: the dry run, the
/// write, and the explicitly strict pass.
const DEFAULT_SCOPE_FORMS: &[&[&str]] = &[
    &["fmt", "--check"],
    &["fmt", "--write"],
    &["fmt", "--check", "--cross-refs"],
];

#[test]
fn fmt_and_check_give_the_same_account_of_the_default_scope() {
    for shape in shapes() {
        let checker = shape.materialize("reader-check");
        let expected = scan_error_pairs(&run_grund(&["check", "."], &checker));

        for form in DEFAULT_SCOPE_FORMS {
            let root = shape.materialize(&format!("reader-{}", form.join("-")));
            let output = run_grund(form, &root);
            assert_eq!(
                scan_error_pairs(&output),
                expected,
                "{} / `grund {}`: fmt's account of what it could not read differs \
                 from check's over the same tree\n  fmt stderr: {}",
                shape.name,
                form.join(" "),
                stderr(&output),
            );
        }
    }
}

#[test]
/// The scoped form. Which `check` run a narrowed `fmt` is comparable to is
/// decided by the walk `fmt` performed, and the run says which that was: a
/// strict abort scanned the project, an ordinary run scanned its scope.
fn a_scoped_fmt_gives_the_same_account_as_check_over_the_tree_it_walked() {
    for shape in shapes() {
        let root = shape.materialize("reader-scoped");
        let checker = shape.materialize("reader-scoped-check");

        let output = run_grund(&["fmt", "--check", shape.inner_scope], &root);
        let comparable = if aborted_strictly(&output) {
            "."
        } else {
            shape.inner_scope
        };
        let expected = scan_error_pairs(&run_grund(&["check", comparable], &checker));

        assert_eq!(
            scan_error_pairs(&output),
            expected,
            "{}: `grund fmt --check {}` disagrees with `grund check {comparable}`, \
             which walked the same tree\n  fmt stderr: {}",
            shape.name,
            shape.inner_scope,
            stderr(&output),
        );
    }
}

#[test]
/// The licensed difference is licensed in one direction only: the strict prefix
/// says the tree was not touched, so a run that carries it must have written
/// nothing, and a run that rewrote must not carry it (§FS-fmt.3.4).
fn the_strict_prefix_appears_exactly_where_nothing_was_rewritten() {
    for shape in shapes() {
        let root = shape.materialize("reader-prefix");
        let output = run_grund(&["fmt", "--write"], &root);
        let report = String::from_utf8_lossy(&output.stdout).to_string();

        if aborted_strictly(&output) {
            assert_eq!(
                report, "",
                "{}: a run claiming nothing was rewritten still reported rewrites",
                shape.name,
            );
        } else {
            assert!(
                !stderr(&output).contains("nothing was rewritten: "),
                "{}: a run that rewrote spelled its errors as a refusal: {}",
                shape.name,
                stderr(&output),
            );
        }
    }
}

#[test]
/// An equivalence between two runs that both do nothing is satisfied by
/// anything, so the corpus declares which half of it each shape exercises and
/// this test holds it to that. A fixture that stops meeting an unreadable path
/// fails here rather than quietly making three rules vacuous.
fn every_shape_exercises_the_half_of_the_corpus_it_claims() {
    for shape in shapes() {
        let root = shape.materialize("reader-coverage");
        let output = run_grund(&["fmt", "--check"], &root);
        assert_eq!(
            !scan_error_pairs(&output).is_empty(),
            shape.reports_unreadable,
            "{}: the shape's `reports_unreadable` no longer describes it: {}",
            shape.name,
            stderr(&output),
        );
    }
}

/// The E2E case pass is part of the scan both commands read (§FS-fmt.7.2,
/// §FS-declarations.e2e-case.failure): a case manifest `check` cannot decode is an
/// unreadable path of `fmt`'s scope wherever it is one of `check`'s, whether or
/// not the scope holds a file `fmt` rewrites. Each row is a shape, the scope
/// both commands are narrowed to (`None` for the default scope) and the `fmt`
/// mode; `check` over the same scope is the control.
const E2E_CASE_ROWS: &[(&str, Option<&str>, &str)] = &[
    ("e2e-cases-bare", None, "--check"),
    ("e2e-cases-bare", None, "--write"),
    ("e2e-cases-docs", Some("e2e/cases"), "--check"),
    ("e2e-cases-docs", Some("e2e/cases"), "--write"),
    ("e2e-cases-docs", Some("e2e/cases/b"), "--check"),
    ("e2e-cases-docs", Some("e2e/cases/b"), "--write"),
    ("e2e-cases-docs", Some(UNREADABLE_MANIFEST), "--check"),
    ("e2e-cases-docs", Some(UNREADABLE_MANIFEST), "--write"),
];

/// The rows that agree today: the docs shape's whole-project run takes the strict
/// path (§FS-fmt.3.2), whose complete scan already meets the manifest. Pinned so
/// the case pass the fix adds neither breaks them nor names the manifest twice
/// (§FS-check.2.4.2).
const E2E_CASE_STRICT_ROWS: &[(&str, Option<&str>, &str)] = &[
    ("e2e-cases-docs", None, "--check"),
    ("e2e-cases-docs", None, "--write"),
];

fn e2e_case_shape(name: &str) -> Shape {
    e2e_case_shapes()
        .into_iter()
        .find(|shape| shape.name == name)
        .expect("a row names a shape of the E2E case corpus")
}

/// One row: `check` over the scope names the manifest and exits 2, `fmt` over the
/// same scope gives the same `(path, reason)` pairs, once each, and exits 2, and a
/// `--write` leaves every byte of the tree where it was (§FS-fmt.3.2,
/// §FS-fmt.7.4.3).
fn assert_e2e_case_row(shape_name: &str, scope: Option<&str>, mode: &str) {
    let shape = e2e_case_shape(shape_name);
    let slot = format!(
        "reader-e2e-case{}{mode}",
        scope
            .map(|s| format!("-{}-", s.replace('/', "-")))
            .unwrap_or("-".into())
    );
    let checker = shape.materialize(&format!("{slot}-check"));
    let root = shape.materialize(&slot);

    let mut check_args = vec!["check"];
    let mut fmt_args = vec!["fmt", mode];
    check_args.extend(scope);
    fmt_args.extend(scope);
    let label = format!("{shape_name} / `grund {}`", fmt_args.join(" "));

    let check = run_grund(&check_args, &checker);
    let expected = scan_error_pairs(&check);
    assert!(
        check.status.code() == Some(2)
            && expected
                .iter()
                .any(|pair| pair.starts_with(UNREADABLE_MANIFEST)),
        "{label}: the control `grund {}` no longer names {UNREADABLE_MANIFEST} with exit 2, \
         so the fixture is not the one this row pins\n  check stderr: {}",
        check_args.join(" "),
        stderr(&check),
    );

    let before = byte_snapshot(&root);
    let output = run_grund(&fmt_args, &root);
    let pairs = scan_error_pairs(&output);
    assert_eq!(
        (output.status.code(), &pairs),
        (Some(2), &expected),
        "{label}: fmt's account of what it could not read differs from \
         `grund {}` over the same scope\n  fmt stdout: {}\n  fmt stderr: {}",
        check_args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        stderr(&output),
    );
    if mode == "--write" {
        assert_eq!(
            byte_snapshot(&root),
            before,
            "{label}: a run with an unreadable path in scope wrote to the tree"
        );
    }
}

#[test]
#[ignore = "red until #1329 fmt runs the E2E case pass on every scope; the fix removes this"]
fn fmt_names_an_unreadable_case_manifest_wherever_check_does_on_the_same_scope() {
    // Every row runs before the verdict, so a failure names each row that fails.
    let failed: Vec<String> = E2E_CASE_ROWS
        .iter()
        .filter_map(|(shape, scope, mode)| {
            std::panic::catch_unwind(|| assert_e2e_case_row(shape, *scope, mode))
                .err()
                .map(|_| {
                    format!(
                        "{shape} / fmt {mode} {}",
                        scope.unwrap_or("(default scope)")
                    )
                })
        })
        .collect();
    assert!(
        failed.is_empty(),
        "rows that disagree with check: {failed:#?}"
    );
}

#[test]
fn a_strict_fmt_names_an_unreadable_case_manifest_once_as_check_does() {
    for (shape, scope, mode) in E2E_CASE_STRICT_ROWS {
        assert_e2e_case_row(shape, *scope, mode);
    }
}
