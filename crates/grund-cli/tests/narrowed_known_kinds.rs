//! §FS-list.1.2 — under `--project`, a refused kind ends in a `known kinds:`
//! line naming the selected projects' citable kinds only, on every `list`
//! surface that prints it. The fixture is the `workspace-list-project-unknown-kind`
//! e2e case's repository: `root` configures `FS` and `AR`, its member `api`
//! configures `API`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/workspace-list-project-unknown-kind/repo")
}

fn run_grund(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn grund")
}

/// The mismatch, if any, between one run and a refusal with `expected_stderr`
/// on stderr, nothing on stdout, and exit `2`.
fn refusal_mismatch(args: &[&str], expected_stderr: &str) -> Option<String> {
    let output = run_grund(args, &fixture_root());
    let actual = (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    );
    let expected = (Some(2), String::new(), expected_stderr.to_owned());
    (actual != expected).then(|| {
        format!(
            "grund {}\n  expected (exit, stdout, stderr): {expected:?}\n  actual: {actual:?}",
            args.join(" ")
        )
    })
}

/// Every case is run before any is judged, so one surface's mismatch never
/// hides another's.
fn assert_refusals(cases: &[(&[&str], &str)]) {
    let mismatches = cases
        .iter()
        .filter_map(|(args, stderr)| refusal_mismatch(args, stderr))
        .collect::<Vec<_>>();
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// §FS-list.1.2: `--kind`, `--size --kind`, `--selector` and `--size --selector`
/// each refuse `API` under `--project root` and list only `root`'s kinds; the
/// member narrowed alone lists only its own.
#[test]
fn a_narrowed_refusal_lists_only_the_selected_projects_kinds() {
    let kind = "error: unknown kind `API`\nknown kinds: FS, AR\n";
    let selector = "error: unknown kind \"API\"\nknown kinds: FS, AR\n";
    assert_refusals(&[
        (&["list", "--project", "root", "--kind", "API"], kind),
        (
            &["list", "--project", "root", "--size", "--kind", "API"],
            kind,
        ),
        (
            &["list", "--project", "root", "--selector", "Each API"],
            selector,
        ),
        (
            &[
                "list",
                "--project",
                "root",
                "--size",
                "--selector",
                "Each API",
            ],
            selector,
        ),
        (
            &["list", "--project", "api", "--kind", "FS"],
            "error: unknown kind `FS`\nknown kinds: API\n",
        ),
    ]);
}

/// §FS-list.1.2: without `--project` every loaded project is selected, so the
/// same fixture still lists every project's kinds, once each, in configuration
/// order.
#[test]
fn an_unnarrowed_refusal_still_lists_every_projects_kinds() {
    assert_refusals(&[
        (
            &["list", "--kind", "ZZ"],
            "error: unknown kind `ZZ`\nknown kinds: FS, AR, API\n",
        ),
        (
            &["list", "--size", "--selector", "Each ZZ"],
            "error: unknown kind \"ZZ\"\nknown kinds: FS, AR, API\n",
        ),
    ]);
}
