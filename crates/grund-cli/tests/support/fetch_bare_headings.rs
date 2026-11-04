//! Issue #1163's cases, run alone: `grund fetch` reads a heading with no text the way the next
//! scan reads it, and refuses one with exit 2 and the tree byte-identical (§FS-fetch.3.3,
//! §FS-declarations.checks.unmarked-heading.1). All of them are also in the full pass.

use std::path::Path;
use std::process::{Command, Output};

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case, run_case_inspecting};
use super::repo_root;

/// Each is a heading to the next scan, so each is refused with the tree byte-identical.
const REFUSED: [&str; 6] = [
    "fetch-bare-heading-h2-refused",
    "fetch-bare-heading-h1-refused",
    "fetch-bare-heading-h4-refused",
    "fetch-bare-heading-indented-refused",
    "fetch-bare-heading-folder-home-h1-refused",
    "fetch-bare-heading-folder-home-h3-refused",
];

/// Content to the next scan though they look like a bare heading, so each stays accepted and
/// the fetched text after it stays in the declaration.
const STAY_ACCEPTED: [&str; 6] = [
    "fetch-bare-heading-four-space-indent-accepted",
    "fetch-bare-heading-folder-home-four-space-indent-accepted",
    "fetch-bare-heading-tab-indent-accepted",
    "fetch-bare-heading-seven-hashes-accepted",
    "fetch-bare-heading-run-into-text-accepted",
    "fetch-bare-heading-fenced-accepted",
];

/// Refused today, and the fix must not lose one. The last three are why the fix adds to the
/// validator's heading test instead of replacing it with `markdown_heading_level`, which reads
/// none of them as a heading: a replacement would accept `    #### Foo` and `####### foo`, and
/// read `    ## AR-002-other: Other` as prose though the scanner's declaration pattern, which
/// opens with `^\s*`, still records it as a second declaration.
const STAY_REFUSED: [&str; 5] = [
    "fetch-bare-heading-trailing-space-refused",
    "fetch-bare-heading-first-line-refused",
    "fetch-heading-indented-declaration-refused",
    "fetch-heading-indented-deep-refused",
    "fetch-heading-seven-hashes-refused",
];

#[test]
fn fetch_bare_headings_refused_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = REFUSED
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("fetch bare headings refused", &outcomes);
}

/// The goldens hold the bytes; what the ticket is about is whether the declaration still holds
/// them, so each accepted tree is asked for the body and checked after the fetch.
#[test]
fn fetch_lines_the_next_scan_reads_as_content_stay_in_the_declaration() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = STAY_ACCEPTED
        .iter()
        .map(|name| {
            run_case_inspecting(&root, &cases.join(name), E2e, |repo| {
                let full = grund(repo, &["AR-001-thing", "--full"]);
                assert!(
                    full.status.success()
                        && String::from_utf8_lossy(&full.stdout)
                            .contains("prose after a bare closer"),
                    "{name}: the text after the line left the declaration: {full:?}"
                );
                let check = grund(repo, &["check"]);
                assert_eq!(check.status.code(), Some(0), "{name}: {check:?}");
            })
        })
        .collect::<Vec<_>>();
    assert_every_case_passed("fetch bare heading lookalikes", &outcomes);
}

/// §FS-fetch.4, §FS-fetch.5: a second fetch replaces the declaration and nothing else. The write
/// path has to read these lines as the next scan does (§FS-fetch.3.3): `    ##` ending the owned
/// declaration would leave a column-0 `##` and a second copy of the text after the replacement.
#[test]
fn fetching_a_line_the_next_scan_reads_as_content_twice_changes_nothing() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = STAY_ACCEPTED
        .iter()
        .map(|name| {
            run_case_inspecting(&root, &cases.join(name), E2e, |repo| {
                let home = ["docs/arch.md", "docs/arch/AR-001-thing.md"]
                    .map(|home| repo.join(home))
                    .into_iter()
                    .find(|home| home.exists())
                    .expect("the first fetch wrote a home");
                let first = std::fs::read_to_string(&home).expect("read the first fetch's home");
                let again = grund(repo, &["fetch", "AR-001-thing"]);
                assert_eq!(again.status.code(), Some(0), "{name}: {again:?}");
                let second = std::fs::read_to_string(&home).expect("read the second fetch's home");
                assert_eq!(second, first, "{name}: the second fetch changed the home");
            })
        })
        .collect::<Vec<_>>();
    assert_every_case_passed("fetch bare heading lookalikes fetched twice", &outcomes);
}

#[test]
fn fetch_refusals_the_bare_heading_fix_must_not_lose() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = STAY_REFUSED
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("fetch refusals kept", &outcomes);
}

fn grund(repo: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run grund")
}
