//! A dry-run row names every kind of rewrite its line receives, the `--write`
//! summary totals them, and a lock file a write creates or deletes is said to be
//! created or deleted (§FS-fmt.3.5.1, §FS-fmt.3.7, §FS-section-lock.write.report,
//! agent-grounds/grund#1021). Every line an earlier release printed stays a
//! verbatim prefix of the line printed now, so each expectation below is today's
//! line with text appended.
//!
//! The `marker` fixture is the report's first case: a version-1 project whose
//! `--marker` run also links what it marks. The `rename` fixture is its second:
//! a project whose lock was written for `FS-login`, after the declaration was
//! renamed `FS-sign-in` and its authored citations followed, so one link is
//! stale and the lock's entries move to a file of the new name.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use grund_core::{FmtOpts, format_references};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fmt_report_kinds")
        .join(name)
        .join("repo")
}

fn e2e_case(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases")
        .join(name)
        .join("repo")
}

/// A fresh copy of `from` under Cargo's scratch directory, for a run that
/// writes: the committed fixture is never touched.
fn copy_of(from: &Path, copy: &str) -> PathBuf {
    let to = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("fmt_report_kinds")
        .join(copy)
        .join("repo");
    let _ = fs::remove_dir_all(&to);
    copy_dir(from, &to);
    to
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|err| panic!("create {}: {err}", to.display()));
    for entry in fs::read_dir(from).unwrap_or_else(|err| panic!("read {}: {err}", from.display())) {
        let source = entry.unwrap().path();
        let target = to.join(source.file_name().unwrap());
        if source.is_dir() {
            copy_dir(&source, &target);
        } else {
            fs::copy(&source, &target).unwrap_or_else(|err| {
                panic!("copy {} to {}: {err}", source.display(), target.display())
            });
        }
    }
}

fn grund(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn grund")
}

/// Runs `grund fmt <args>` in `cwd` and requires exit `code`; returns stdout.
fn fmt(args: &[&str], cwd: &Path, code: i32) -> String {
    let output = grund(&[&["fmt"], args].concat(), cwd);
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 stdout");
    assert_eq!(
        output.status.code(),
        Some(code),
        "grund fmt {} exited {:?}\n--- stdout\n{stdout}--- stderr\n{}",
        args.join(" "),
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
}

/// A tree fmt has written is a fixed point (§FS-fmt.requirements.fixed-point):
/// the totals add nothing to `rewrote 0 lines`.
fn assert_second_write_is_empty(args: &[&str], repo: &Path) {
    let again = fmt(&[args, &["--write"]].concat(), repo, 0);
    assert_eq!(
        again, "rewrote 0 lines\n",
        "a second write found more to do"
    );
    fmt(&[args, &["--check"]].concat(), repo, 0);
}

const MARKER_DRY_RUN: &str = "\
docs/architecture/AR-ledger.md:3: bare \u{2192} marker, markdown link
docs/architecture/README.md:5: markdown link
docs/runbook.md:3: bare \u{2192} marker, markdown link
requirements.md:5: bare \u{2192} marker, markdown link
requirements.md:17: bare \u{2192} marker, markdown link
src/inventory/reserve.py:3: bare \u{2192} marker
src/inventory/reserve.py:7: bare \u{2192} marker
";

#[test]
#[ignore = "red until #1021 names every rewrite kind on a row; implement removes this"]
fn a_marker_line_the_link_pass_also_wraps_names_both_kinds() {
    // §FS-fmt.3.5.1: the list follows a detail-free leading label with `, `.
    assert_eq!(fmt(&["--marker"], &fixture("marker"), 1), MARKER_DRY_RUN);
}

#[test]
#[ignore = "red until #1021 totals the write summary by kind; implement removes this"]
fn the_marker_write_totals_what_it_marked_and_what_it_linked() {
    let repo = copy_of(&fixture("marker"), "marker-write");
    // §FS-fmt.3.7: six bare citations marked (AR-ledger 1, runbook 1,
    // requirements 2, reserve.py 2) and six links new or re-derived (AR-ledger 2,
    // README 1, runbook 1, requirements 2), on seven changed lines.
    assert_eq!(
        fmt(&["--marker", "--write"], &repo, 0),
        "\
rewrote 7 lines: 6 citations marked, 6 links written
  docs/architecture/AR-ledger.md (1)
  docs/architecture/README.md (1)
  docs/runbook.md (1)
  requirements.md (2)
  src/inventory/reserve.py (2)
"
    );
    assert_second_write_is_empty(&["--marker"], &repo);
}

#[test]
#[ignore = "red until #1021 says which lock file a rename deletes and creates; implement removes this"]
fn a_rename_says_which_lock_file_it_deletes_and_which_it_creates() {
    // §FS-section-lock.write.report: the suffix goes on the file's first row,
    // after the lock rows are sorted and after every source row.
    assert_eq!(
        fmt(&[], &fixture("rename"), 1),
        "\
docs/notes.md:1: markdown link
.grund/sections/FS-login.lock:1: section lock: drop FS-login.code.1; deletes the file
.grund/sections/FS-sign-in.lock:1: section lock: add FS-sign-in.code.1; creates the file
"
    );
    let repo = copy_of(&fixture("rename"), "rename-write");
    assert_eq!(
        fmt(&["--write"], &repo, 0),
        "\
rewrote 3 lines: 1 link written, 2 section-lock entries
  .grund/sections/FS-login.lock (1) deleted
  .grund/sections/FS-sign-in.lock (1) created
  docs/notes.md (1)
"
    );
    // What the report says is what the disk holds.
    assert!(
        !repo.join(".grund/sections/FS-login.lock").exists(),
        "the old lock file is still there"
    );
    assert_eq!(
        fs::read_to_string(repo.join(".grund/sections/FS-sign-in.lock"))
            .expect("the new lock file"),
        "FS-sign-in.code.1 78a809b83fd8753e\n"
    );
    assert_second_write_is_empty(&[], &repo);
}

#[test]
#[ignore = "red until #1021 totals section-lock entries and names a created lock file; implement removes this"]
fn a_lock_only_write_totals_its_entries_and_names_the_file_it_creates() {
    let repo = copy_of(&e2e_case("fmt-section-lock-login-write"), "login-write");
    assert_eq!(
        fmt(&["--write"], &repo, 0),
        "rewrote 1 line: 1 section-lock entry\n  .grund/sections/FS-login.lock (1) created\n"
    );
    assert_second_write_is_empty(&[], &repo);
}

#[test]
#[ignore = "red until #1021 totals the write summary by kind; implement removes this"]
fn a_lock_file_that_exists_before_and_after_carries_no_suffix() {
    // §FS-section-lock.write.report: one entry dropped and another added in the
    // same file is neither a creation nor a deletion.
    assert_eq!(
        fmt(
            &["--check"],
            &e2e_case("fmt-section-lock-move-citations-check"),
            1
        ),
        "\
.grund/sections/FS-login.lock:1: section lock: add FS-login.parse.2
.grund/sections/FS-login.lock:1: section lock: drop FS-login.format.1
"
    );
    let moved = copy_of(
        &e2e_case("fmt-section-lock-move-citations-write"),
        "move-write",
    );
    assert_eq!(
        fmt(&["--write"], &moved, 0),
        "rewrote 2 lines: 2 section-lock entries\n  .grund/sections/FS-login.lock (2)\n"
    );
    // A refreshed entry, beside a trigger the same run marked.
    let refreshed = copy_of(
        &e2e_case("fmt-section-lock-trigger-refresh-write"),
        "refresh-write",
    );
    assert_eq!(
        fmt(&["--write"], &refreshed, 0),
        "\
rewrote 2 lines: 1 trigger marked, 1 section-lock entry
  .grund/sections/FS-login.lock (1)
  docs/functional-spec/FS-login.md (1)
"
    );
}

const SEVERAL: &str = "docs/functional-spec/FS-004-quick-actions.md";

#[test]
#[ignore = "red until #1021 names every rewrite kind on a row; implement removes this"]
fn the_kinds_after_an_expansion_follow_its_detail_after_a_semicolon() {
    let rows = |flag: &str| -> Vec<String> {
        fmt(&["--check", flag], &fixture("several"), 1)
            .lines()
            .map(|row| {
                row.strip_prefix(SEVERAL)
                    .expect("one citing file")
                    .to_string()
            })
            .collect()
    };
    // §FS-fmt.3.5.1, §FS-fmt.3.5.2, §FS-fmt.3.6.1: the detail stays on the
    // leading label; a withheld token (line 11's <§>9) is never named.
    assert_eq!(
        rows("--cross-refs"),
        [
            ":3: trigger \u{2192} marker: \u{a7}FS-042 \u{2192} \u{a7}FS-042-user-login; shorthand \u{2192} canonical, markdown link",
            ":5: shorthand \u{2192} canonical: \u{a7}FS-042 \u{2192} \u{a7}FS-042-user-login; markdown link",
            ":7: local section \u{2192} canonical: \u{a7}1 \u{2192} \u{a7}FS-004-quick-actions.1, \u{a7}2 \u{2192} \u{a7}FS-004-quick-actions.2; markdown link",
            ":9: trigger \u{2192} marker: \u{a7}1 \u{2192} \u{a7}FS-004-quick-actions.1; local section \u{2192} canonical, markdown link",
            ":11: local section \u{2192} canonical: \u{a7}1 \u{2192} \u{a7}FS-004-quick-actions.1; markdown link",
            ":13: shorthand \u{2192} canonical: \u{a7}FS-042 \u{2192} \u{a7}FS-042-user-login; markdown link",
        ]
    );
    assert_eq!(
        rows("--marker")[5],
        ":13: bare \u{2192} marker: \u{a7}FS-042 \u{2192} \u{a7}FS-042-user-login; shorthand \u{2192} canonical, markdown link"
    );
    // §FS-fmt.3.7: each occurrence counts, in the label order.
    let repo = copy_of(&fixture("several"), "several-write");
    let summary = fmt(&["--cross-refs", "--write"], &repo, 0);
    assert_eq!(
        summary.lines().next(),
        Some(
            "rewrote 6 lines: 2 triggers marked, 4 local sections expanded, 3 shorthands expanded, 8 links written"
        )
    );
    let repo = copy_of(&fixture("several"), "several-marker-write");
    let summary = fmt(&["--marker", "--write"], &repo, 0);
    assert_eq!(
        summary.lines().next(),
        Some(
            "rewrote 6 lines: 2 triggers marked, 1 citation marked, 4 local sections expanded, 3 shorthands expanded, 9 links written"
        )
    );
}

#[test]
#[ignore = "red until #1021 keeps a trailing link's repeated-anchor detail; implement removes this"]
fn a_trailing_markdown_link_keeps_its_repeated_anchor_detail() {
    // §FS-fmt.3.6, §DF-repeated-heading-anchor.decision.row: the detail ends the
    // row. The first link is already right, so it is no link written.
    assert_eq!(
        fmt(&["--check"], &fixture("anchor"), 1),
        "docs/use.md:3: trigger \u{2192} marker, markdown link: repeated heading anchor \
         #1-overview \u{2192} #1-overview-1, numbered since grund 0.18\n"
    );
    let repo = copy_of(&fixture("anchor"), "anchor-write");
    assert_eq!(
        fmt(&["--write"], &repo, 0),
        "rewrote 1 line: 1 trigger marked, 2 links written\n  docs/use.md (1)\n"
    );
}

#[test]
#[ignore = "red until #1021 appends the same kinds to FmtChange.label; implement removes this"]
fn the_api_label_is_the_cli_row_after_its_location() {
    // §FS-distribution.3.2.2.2: `label` carries what the CLI row appends.
    let mut opts = FmtOpts::default();
    opts.path = fixture("marker");
    opts.add_marker = true;
    let output = format_references(opts).expect("format the marker fixture");
    let rows = output
        .changes
        .iter()
        .map(|change| format!("{}:{}: {}\n", change.path, change.line, change.label))
        .collect::<String>();
    assert_eq!(rows, MARKER_DRY_RUN);
}
