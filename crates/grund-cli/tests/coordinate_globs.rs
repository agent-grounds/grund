//! Coordinate globs (§FS-coordinate-globs) and `list --total` (§FS-list.total), pinned by the
//! e2e cases they own, by agreements with the shipped surfaces on this repository's own tree,
//! and by a bound on matching time.
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

#[path = "support/case_runner.rs"]
#[allow(dead_code)]
mod case_runner;

use case_runner::{CaseKind, assert_every_case_passed, run_case};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_cases(label: &str, names: &[&str]) {
    run_in("tests/e2e/cases", CaseKind::E2e, label, names);
}

fn run_in(dir: &str, kind: CaseKind, label: &str, names: &[&str]) {
    let root = root();
    let outcomes: Vec<_> = names
        .iter()
        .map(|name| run_case(&root, &root.join(dir).join(name), kind))
        .collect();
    assert_every_case_passed(label, &outcomes);
}

fn grund_with(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn grund");
    let mut input = child.stdin.take().expect("piped stdin");
    input.write_all(stdin.as_bytes()).expect("write stdin");
    drop(input);
    child.wait_with_output().expect("wait for grund")
}

/// Stdout of a run that must succeed on this repository.
fn grund(args: &[&str]) -> String {
    stdout_of(args, grund_with(args, ""))
}

fn stdout_of(args: &[&str], output: Output) -> String {
    assert!(
        output.status.success(),
        "grund {args:?} exited {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 stdout")
}

fn row(line: &str) -> serde_json::Value {
    serde_json::from_str(line).unwrap_or_else(|err| panic!("JSON row {line:?}: {err}"))
}

/// §FS-coordinate-globs.grammar and §FS-coordinate-globs.matching: which rows a pattern selects, in list order.
#[test]
fn glob_rows_select_catalog_coordinates() {
    run_cases(
        "glob_rows_select_catalog_coordinates",
        &[
            "list-glob-children",
            "list-glob-descendants",
            "list-glob-point-and-subtree",
            "list-glob-point-and-subtree-json",
            "list-glob-id-atom-is-the-whole-id",
            "list-glob-equals-form",
            "list-glob-named-class",
            "list-glob-numbered-class",
            "list-glob-mixed-section",
            "list-glob-relative-numbered-title",
            "list-glob-numbered-then-named",
            "list-glob-middle-globstar",
            "list-glob-leading-globstar-json",
            "list-glob-union",
            "list-glob-negated-class-bang",
            "list-glob-negated-class-caret",
            "list-glob-stub-is-one-coordinate",
            "list-glob-no-match",
        ],
    );
}

/// §FS-coordinate-globs.scope.filters: path, `--kind`, `--unused`, `--selector`, `--summary` and `--size` read the glob's selection.
#[test]
fn glob_filters_intersect_and_never_widen() {
    run_cases(
        "glob_filters_intersect_and_never_widen",
        &[
            "list-glob-kind-filter",
            "list-glob-unused-filter",
            "list-glob-selector-intersects",
            "list-glob-path-filter",
            "list-glob-summary",
            "list-glob-size",
        ],
    );
}

/// §FS-list.total: the bare count, its JSON shape, and the launch errors.
#[test]
fn total_counts_the_rows_list_prints() {
    run_cases(
        "total_counts_the_rows_list_prints",
        &[
            "list-total-glob",
            "list-total-glob-json",
            "list-total-zero",
            "list-total-plain",
            "list-total-selector",
            "list-total-repeated",
            "list-total-with-summary",
            "list-total-with-size",
            "list-total-with-top",
            "list-total-before-discovery",
        ],
    );
}

/// §FS-coordinate-globs.errors.reasons, §FS-coordinate-globs.errors.marker and §FS-coordinate-globs.errors.flags.
#[test]
fn refused_globs_print_one_line_and_exit_2() {
    run_cases(
        "refused_globs_print_one_line_and_exit_2",
        &[
            "list-glob-refused-star-run-ends-the-pattern",
            "list-glob-refused-star-run-inside-a-component",
            "list-glob-refused-three-stars",
            "list-glob-refused-empty-component",
            "list-glob-refused-trailing-separator",
            "list-glob-refused-empty-pattern",
            "list-glob-refused-nested-braces",
            "list-glob-refused-unclosed-brace",
            "list-glob-refused-unclosed-class",
            "list-glob-refused-brace-range",
            "list-glob-refused-backwards-range",
            "list-glob-refused-dangling-escape",
            "list-glob-refused-qualifier-operator",
            "list-glob-refused-citation-marker",
            "list-glob-refused-first-pattern-wins",
            "list-glob-refused-before-discovery",
            "list-glob-missing-value",
            "show-glob-requires-batch",
        ],
    );
}

/// §FS-coordinate-globs.errors.note and §FS-coordinate-globs.errors.hint.
#[test]
fn notes_and_the_path_hint_leave_stdout_alone() {
    run_cases(
        "notes_and_the_path_hint_leave_stdout_alone",
        &[
            "list-glob-note-suggests-the-kind",
            "list-glob-note-without-a-suggestion",
            "list-glob-note-silent-for-a-startable-prefix",
            "list-total-glob-note",
            "show-batch-glob-note",
            "list-glob-path-hint",
        ],
    );
}

/// §FS-coordinate-globs.matching.anchored, §FS-coordinate-globs.grammar.components and §FS-coordinate-globs.workspace.escapes; a check the separator decides waits for it (§FS-coordinate-globs.errors.shape).
#[test]
fn code_points_escapes_and_the_hash_separator() {
    run_cases(
        "code_points_escapes_and_the_hash_separator",
        &[
            "list-glob-hash-is-literal",
            "list-glob-question-is-one-code-point",
            "list-glob-class-range-by-code-point",
            "list-glob-escaped-star",
            "list-glob-hash-separator",
            "list-glob-hash-escaped-dots",
            "list-glob-hash-subtree",
            "list-glob-underscore-separator",
        ],
    );
}

/// §FS-show.1.9 and §FS-show.2.6.2: `show --batch --glob` reads the bodies `--all` would, filtered.
#[test]
fn batch_glob_filters_the_generated_batch() {
    run_cases(
        "batch_glob_filters_the_generated_batch",
        &[
            "show-batch-glob-json",
            "show-batch-glob-requires-json",
            "show-batch-glob-toc-json",
            "show-batch-glob-all-is-redundant",
            "show-batch-glob-ignores-stdin",
            "show-batch-glob-union",
            "show-batch-glob-refused",
            "show-batch-glob-ambiguous-section",
        ],
    );
}

/// §FS-coordinate-globs.scope and §FS-coordinate-globs.workspace.
#[test]
fn workspace_globs_split_per_project() {
    run_cases(
        "workspace_globs_split_per_project",
        &[
            "workspace-list-glob-unqualified-is-the-current-project",
            "workspace-list-glob-every-project-own-separator",
            "workspace-list-glob-member-separator-json",
            "workspace-list-glob-alias",
            "workspace-list-glob-project-never-widens",
            "workspace-list-glob-unknown-alias",
            "workspace-show-batch-glob-qualified",
            "workspace-list-glob-excluded-root-matches-nothing",
            "workspace-list-glob-excluded-root-every-project",
            "workspace-list-glob-nested-alias",
        ],
    );
}

/// §FS-citations.recognized.globs: `<§>FS-gate{.1,.2}` is a pattern, not a citation of `FS-gate`.
#[test]
fn a_brace_before_the_separator_opens_a_glob_citation() {
    run_cases(
        "a_brace_before_the_separator_opens_a_glob_citation",
        &[
            "glob-citation-check-brace-before-separator",
            "glob-citation-refs-brace-before-separator",
        ],
    );
}

/// §FS-coordinate-globs.errors.hint: a missing path without an operator, or one that is no
/// valid pattern, keeps today's error alone.
#[test]
fn the_path_hint_needs_a_valid_pattern() {
    run_cases(
        "the_path_hint_needs_a_valid_pattern",
        &[
            "list-glob-path-no-hint-without-operator",
            "list-glob-path-no-hint-invalid-pattern",
        ],
    );
}

/// §FS-examples.2: the review-checklist walkthrough counts its checklist and reads its bodies.
#[test]
fn review_checklist_example_counts_and_reads_the_checklist() {
    let runs = ["1-count", "2-bodies"];
    run_in(
        "examples/review-checklist",
        CaseKind::Example,
        "review-checklist",
        &runs,
    );
}

/// §FS-cover.6: its third step, the owners of a changed range, is shipped behaviour already.
#[test]
fn review_checklist_example_names_the_owners_of_a_change() {
    let runs = ["3-owners"];
    run_in(
        "examples/review-checklist",
        CaseKind::Example,
        "review-checklist",
        &runs,
    );
}

/// §FS-rules.8: `--glob 'FS-*.terms'` selects exactly what `--selector FS.terms` does on this
/// tree, so every output mode prints the same bytes for both.
#[test]
fn own_tree_glob_lists_what_the_selector_lists() {
    for mode in [
        &[][..],
        &["--format", "json"],
        &["--summary"],
        &["--size=lines"],
    ] {
        let selector = grund(&[&["list", "--selector", "FS.terms"], mode].concat());
        assert!(!selector.is_empty(), "this tree has FS Terms chapters");
        let glob = grund(&[&["list", "--glob", "FS-*.terms"], mode].concat());
        assert_eq!(
            glob, selector,
            "list {mode:?}: --glob and --selector disagree"
        );
    }
}

/// §FS-show.1.9: the batch a glob generates is the one the selector's rows would feed
/// `show --batch` in generation order, so the `jq` pipe of §FS-init.2.3.4.3.1 has a one-flag form.
#[test]
fn own_tree_glob_batch_reads_what_the_selector_pipe_reads() {
    let rows = grund(&["list", "--selector", "FS.terms", "--format", "json"]);
    let mut queries: Vec<(String, String)> = rows
        .lines()
        .map(|line| {
            let row = row(line);
            (
                row["id"].as_str().unwrap().to_owned(),
                row["section"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    queries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()).then(a.1.cmp(&b.1)));
    let stdin: String = queries
        .iter()
        .map(|(id, section)| format!("{}\n", serde_json::json!({"id": id, "section": section})))
        .collect();
    let toc = ["--toc", "--format", "json"];
    let piped = grund_with(&[&["show", "--batch"][..], &toc].concat(), &stdin);
    let piped = stdout_of(&["show", "--batch"], piped);
    let glob = grund(&[&["show", "--batch", "--glob", "FS-*.terms"][..], &toc].concat());
    assert_eq!(glob, piped);
}

/// §FS-list.total: `FS-config.requirements{,.**}` counts the chapter and every heading beneath
/// it, counted here from the file's own headings.
#[test]
fn own_tree_total_counts_a_chapter_and_its_points() {
    let text = fs::read_to_string(root().join("docs/functional-spec/FS-config.md"))
        .expect("read FS-config.md");
    let mut fenced = false;
    let mut count = 0;
    for line in text.lines() {
        if line.starts_with("```") {
            fenced = !fenced;
        }
        let title = line.trim_start_matches('#');
        if fenced || title.len() == line.len() || !title.starts_with(' ') {
            continue;
        }
        if title[1..].starts_with("requirements:") || title[1..].starts_with("requirements.") {
            count += 1;
        }
    }
    assert!(
        count > 1,
        "FS-config has a requirements chapter with points beneath it"
    );
    let pattern = "FS-config.requirements{,.**}";
    assert_eq!(
        grund(&["list", "--glob", pattern, "--total"]),
        format!("{count}\n")
    );
    assert_eq!(grund(&["list", "--glob", pattern]).lines().count(), count);
}

/// §FS-coordinate-globs.grammar.components: an ID atom is matched against the whole ID, so
/// `FS-c*` lists the FS declarations whose ID starts `FS-c` and none of their sections.
#[test]
fn own_tree_id_glob_lists_the_declarations_with_its_prefix() {
    let expected: String = grund(&["list", "--kind", "FS", "--format", "json"])
        .lines()
        .filter(|line| row(line)["id"].as_str().unwrap().starts_with("FS-c"))
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(!expected.is_empty(), "this tree has FS-c declarations");
    assert_eq!(
        grund(&["list", "--glob", "FS-c*", "--format", "json"]),
        expected
    );
}

/// §FS-list.total: the count is the number of rows the same filters list, compared with `wc -l`.
#[test]
fn own_tree_total_is_the_row_count() {
    let rows = grund(&["list"]).lines().count();
    assert_eq!(grund(&["list", "--total"]), format!("{rows}\n"));
    let fs_rows = grund(&["list", "--kind", "FS"]).lines().count();
    assert_eq!(
        grund(&["list", "--kind", "FS", "--total", "--format", "json"]),
        format!("{{\"count\":{fs_rows}}}\n")
    );
}

/// §FS-coordinate-globs.matching.bounded: 24 two-way groups stand for 2^24 alternatives, and
/// the pattern is still answered in seconds, as an empty result.
#[test]
fn a_brace_product_is_matched_without_expanding_it() {
    let pattern = format!("FS-{}*", "{a,b}".repeat(24));
    let repo = root().join("tests/e2e/cases/list-glob-children/repo");
    let mut child = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["list", "--glob", &pattern])
        .current_dir(&repo)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn grund");
    let start = Instant::now();
    while child.try_wait().expect("poll grund").is_none() {
        if start.elapsed() > Duration::from_secs(10) {
            let _ = child.kill();
            panic!("list --glob over 2^24 alternatives ran past 10s");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().expect("wait for grund");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}
