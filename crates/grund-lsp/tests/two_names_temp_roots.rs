//! Run the existing real-server cases across temporary-root spellings (§AR-ci.10.3.1).
#![cfg(unix)]

#[path = "two_names.rs"]
mod cases;
mod support;

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;
use support::{fixture_base, uncovered_base};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove temporary-root regression fixtures");
    }
}

#[test]
#[ignore = "red until #1307 resolves the shared LSP fixture base; implement removes this"]
fn two_names_survive_an_ancestor_alias_in_tmpdir() {
    let scratch =
        Scratch(fixture_base().join(format!("grund-1307-two-names-temp-{}", std::process::id())));
    fs::create_dir(&scratch.0).expect("create regression scratch");
    let physical = scratch.0.join("physical");
    fs::create_dir(&physical).expect("create physical temporary base");
    let physical = fs::canonicalize(physical).expect("resolve physical temporary base");
    let alias = scratch.0.join("alias");
    symlink(&physical, &alias).expect("create temporary ancestor alias");
    let alias = fs::canonicalize(&scratch.0)
        .expect("resolve scratch parent")
        .join("alias");
    assert_ne!(physical, alias);
    assert_eq!(fs::canonicalize(&alias).expect("resolve alias"), physical);

    let executable = std::env::current_exe().expect("test executable");
    let cases = [
        "a_content_finding_reaches_the_second_name_once_it_is_open",
        "a_place_finding_stays_at_its_own_spelling",
        "a_duplicate_between_two_files_stays_where_it_was",
        "a_refused_file_takes_no_on_type_edit_under_any_name",
    ];
    let mut failures = Vec::new();
    for (spelling, base) in [("physical", physical), ("ancestor alias", alias)] {
        // A covered candidate would fall back and could hide the alias scenario.
        assert_eq!(
            uncovered_base(std::slice::from_ref(&base)),
            Ok(base.clone())
        );
        for case in cases {
            // Exact selection runs one existing case, never this driver or a nested suite.
            let output = Command::new(&executable)
                .args([
                    "--exact",
                    &format!("cases::{case}"),
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env("TMPDIR", &base)
                .output()
                .expect("run existing two-name case in an isolated process");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{spelling}: {case}\n{stdout}{stderr}");
            assert!(stdout.contains("running 1 test"), "case was not selected");
            if !output.status.success() {
                failures.push(format!("{spelling}: {case}: {}", output.status));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "all four real-server cases must pass under both temporary bases:\n{}",
        failures.join("\n")
    );
}
