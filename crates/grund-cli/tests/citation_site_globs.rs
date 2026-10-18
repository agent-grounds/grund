//! Citation-site queries (§DF-citation-site-queries): `refs --glob` (§FS-refs.globs),
//! `refs --from` (§FS-refs.from) and `cover --glob` (§FS-cover.globs), pinned by the e2e cases
//! they own, by the walkthrough steps that teach them, and by agreements with `cover` and ID-mode
//! `refs` on this repository's own tree.
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

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

fn grund_output(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root())
        .stdin(Stdio::null())
        .output()
        .expect("run grund")
}

/// Stdout of a run that must succeed on this repository.
fn grund(args: &[&str]) -> String {
    let output = grund_output(args);
    assert!(
        output.status.success(),
        "grund {args:?} exited {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 stdout")
}

fn rows(stdout: &str) -> Vec<serde_json::Value> {
    stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|err| panic!("{line:?}: {err}")))
        .collect()
}

/// Every citation `cover --format json` reports on this tree, in its file and site order.
fn cover_citations() -> Vec<serde_json::Value> {
    rows(&grund(&["cover", "--format", "json"]))
        .iter()
        .flat_map(|entry| entry["citations"].as_array().expect("citations").clone())
        .collect()
}

/// §FS-refs.globs and §FS-coordinate-globs.matching.citations: a pattern selects the sites whose
/// cited coordinate it matches, declared or not, each once, in ID mode's order and shapes.
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn refs_glob_lists_who_cites_a_set() {
    run_cases(
        "refs_glob_lists_who_cites_a_set",
        &[
            "refs-glob-point-and-subtree",
            "refs-glob-bare-id-only",
            "refs-glob-sections-only",
            "refs-glob-dangling",
            "refs-glob-union-once-json",
            "refs-glob-local-numeric",
            "refs-glob-shorthand",
            "refs-glob-ambiguous-target-listed",
            "refs-glob-summary",
            "refs-glob-total",
            "refs-glob-total-json",
            "refs-glob-existing-path-is-scanned",
        ],
    );
}

/// §FS-refs.globs.unresolved: nothing a pattern matches is refused, so an empty set exits `0`,
/// and a pattern no kind can start prints its note (§FS-coordinate-globs.errors.note).
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn a_glob_that_matches_nothing_exits_0() {
    run_cases(
        "a_glob_that_matches_nothing_exits_0",
        &[
            "refs-glob-no-match",
            "refs-glob-no-match-total",
            "refs-glob-note",
        ],
    );
}

/// §FS-refs.1.2: every refusal of the new modes is one `error:` line and exit `2` before
/// discovery, with a pasteable glob only where pasting it back passes (§FS-errors.2.5).
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn refs_mode_refusals_are_pinned() {
    run_cases(
        "refs_mode_refusals_are_pinned",
        &[
            "refs-from-missing-value",
            "refs-from-empty-value",
            "refs-from-twice",
            "refs-glob-missing-value",
            "refs-glob-with-from",
            "refs-glob-with-section-offers-a-glob",
            "refs-glob-with-descendants-offers-a-glob",
            "refs-glob-with-section-escapes-the-offer",
            "refs-glob-with-section-two-globs-no-offer",
            "refs-glob-with-descendants-offer-withheld",
            "refs-from-with-section",
            "refs-from-with-descendants",
            "refs-glob-second-path",
            "refs-from-second-path",
            "refs-glob-bad-format-first",
            "refs-glob-combination-before-pattern-refusal",
            "refs-glob-refusal-before-broken-config",
            "refs-glob-refused-before-discovery",
        ],
    );
}

/// §FS-refs.1.3: a missing path that parses as an ID gets one hint naming the glob to use.
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn an_id_typed_as_the_path_gets_a_hint() {
    run_cases(
        "an_id_typed_as_the_path_gets_a_hint",
        &[
            "refs-glob-id-as-path-hint",
            "refs-from-id-as-path-hint",
            "refs-glob-path-no-hint-with-operator",
            "refs-glob-path-no-hint-invalid-pattern",
        ],
    );
}

/// §FS-refs.from.owned and §FS-refs.from.stubs: a source owns its lead, its proper ancestors and
/// itself, a rejected heading's sites lie in the lead, and a doc-comment lead stops at the code.
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn refs_from_lists_what_one_point_cites() {
    run_cases(
        "refs_from_lists_what_one_point_cites",
        &[
            "refs-from-lead-ancestors-and-self",
            "refs-from-bare-source-is-its-lead",
            "refs-from-named-section",
            "refs-from-json",
            "refs-from-local-numeric",
            "refs-from-rejected-heading-is-the-lead",
            "refs-from-doc-comment-boundary",
            "refs-from-stub-unscanned-target",
            "refs-from-summary",
        ],
    );
}

/// §FS-refs.from.records: `--from --total` reads `cites at …` and `cites nothing`, and keeps
/// ID mode's JSON object.
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn refs_from_counts_in_its_own_sentence() {
    run_cases(
        "refs_from_counts_in_its_own_sentence",
        &[
            "refs-from-total",
            "refs-from-total-one-site",
            "refs-from-total-nothing",
            "refs-from-total-json",
        ],
    );
}

/// §FS-refs.4: a source that does not resolve fails as `grund <coordinate>` fails, exit `1`.
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn a_from_source_fails_as_show_fails() {
    run_cases(
        "a_from_source_fails_as_show_fails",
        &[
            "refs-from-not-found",
            "refs-from-not-found-json",
            "refs-from-missing-section",
            "refs-from-invalid-id",
            "refs-from-ambiguous-json",
            "refs-from-ambiguous-section",
            "refs-from-total-inherits-the-refusal",
        ],
    );
}

/// §FS-coordinate-globs.scope.targets and §FS-workspace.8.2: a citation matches by the project it
/// targets, and its record still names the project it sits in.
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn workspace_citations_match_the_project_they_target() {
    run_cases(
        "workspace_citations_match_the_project_they_target",
        &[
            "workspace-refs-glob-qualified-json",
            "workspace-refs-glob-unqualified-is-the-current-target",
            "workspace-refs-glob-every-project",
            "workspace-refs-glob-unknown-pattern-alias",
            "workspace-refs-glob-root-excluded",
            "workspace-refs-glob-root-excluded-unqualified",
            "workspace-refs-from-keeps-the-alias-json",
            "workspace-cover-glob-target",
            "workspace-cover-glob-every-project-json",
        ],
    );
}

/// §FS-cover.globs and §FS-cover.6.4: `cover --glob` keeps every file's record and only the
/// citations the patterns match, and refuses `--lines` before it reads the path.
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn cover_glob_keeps_every_file() {
    run_cases(
        "cover_glob_keeps_every_file",
        &[
            "cover-glob-keeps-every-file",
            "cover-glob-json",
            "cover-glob-no-match",
            "cover-glob-path",
            "cover-glob-note",
            "cover-glob-with-lines",
            "cover-glob-missing-value",
            "cover-glob-refused-before-discovery",
        ],
    );
}

/// §FS-examples.2, §FS-refs.globs and §FS-refs.from: the walkthrough asks who cites two points
/// of the checklist and what one of them cites.
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn review_checklist_example_asks_who_cites_and_what_is_cited() {
    let runs = ["4-citers", "5-cites"];
    run_in(
        "examples/review-checklist",
        CaseKind::Example,
        "review-checklist",
        &runs,
    );
}

/// §FS-refs.globs.descendants: on this tree each ID-mode spelling and its glob list the same
/// sites in every text mode, and the glob's JSON is ID mode's without `kind_title` (§FS-refs.3.2).
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn own_tree_glob_lists_what_id_mode_lists() {
    for (query, pattern) in [
        (&["FS-refs"][..], "FS-refs{,.**}"),
        (&["FS-cover", "--descendants"], "FS-cover{,.**}"),
        (&["FS-refs.globs", "--descendants"], "FS-refs.globs{,.**}"),
        (&["FS-refs", "--section", "globs"], "FS-refs.globs"),
        (
            &["FS-coordinate-globs.matching"],
            "FS-coordinate-globs.matching",
        ),
    ] {
        for mode in [&[][..], &["--summary"], &["--total"]] {
            let by_id = grund(&[&["refs"][..], query, mode].concat());
            let by_glob = grund(&[&["refs", "--glob", pattern][..], mode].concat());
            assert_eq!(
                by_glob, by_id,
                "refs {query:?} {mode:?} against --glob {pattern}"
            );
        }
        let mut by_id = rows(&grund(
            &[&["refs"][..], query, &["--format", "json"]].concat(),
        ));
        assert!(!by_id.is_empty(), "this tree cites {query:?}");
        for record in &mut by_id {
            record.as_object_mut().unwrap().remove("kind_title");
        }
        let by_glob = rows(&grund(&["refs", "--glob", pattern, "--format", "json"]));
        assert_eq!(
            by_glob, by_id,
            "refs {query:?} --format json against --glob {pattern}"
        );
    }
}

/// §FS-refs.globs.descendants: `--glob 'X'` is only the citations of `X` with no section, which
/// `cover` counts on this tree, and fewer than `refs X` lists.
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn own_tree_bare_glob_selects_only_the_bare_citations() {
    let expected: Vec<_> = cover_citations()
        .into_iter()
        .filter(|site| site["id"] == "FS-refs" && site["section"].is_null())
        .collect();
    assert!(!expected.is_empty(), "this tree cites FS-refs bare");
    let by_glob = rows(&grund(&["refs", "--glob", "FS-refs", "--format", "json"]));
    assert_eq!(by_glob, expected);
    let by_id = grund(&["refs", "FS-refs"]).lines().count();
    assert!(
        by_glob.len() < by_id,
        "this tree also cites sections of FS-refs"
    );
}

/// §FS-refs.from.owned: `--from FS-refs.3.2.1` lists exactly the sites `cover` gives `FS-refs`
/// under no section, `3`, `3.2` or `3.2.1`, in `cover`'s records (§FS-refs.from.records).
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn own_tree_from_lists_the_sites_of_its_lead_ancestors_and_self() {
    let owned = [None, Some("3"), Some("3.2"), Some("3.2.1")];
    let expected: Vec<_> = cover_citations()
        .into_iter()
        .filter(|site| {
            site["path"] == "docs/functional-spec/FS-refs.md"
                && site["enclosing_declaration"] == "FS-refs"
                && owned.contains(&site["enclosing_section"].as_str())
        })
        .collect();
    assert!(
        expected
            .iter()
            .any(|site| site["enclosing_section"].is_null()),
        "FS-refs's lead cites something"
    );
    let source = "FS-refs.3.2.1";
    let listed = rows(&grund(&["refs", "--from", source, "--format", "json"]));
    assert_eq!(listed, expected);
    let sites = expected.len();
    assert_eq!(
        grund(&["refs", "--from", source, "--total", "--format", "json"]),
        format!("{{\"sites\":{sites},\"files\":1}}\n")
    );
}

/// §FS-cover.globs and §AR-resolver.3.2: with `--glob` every file `cover` lists keeps its record,
/// holding exactly the citations of that file the pattern matches.
#[test]
#[ignore = "red until #810 ships refs --glob, refs --from and cover --glob; implement removes this"]
fn own_tree_cover_glob_keeps_every_record() {
    let all = rows(&grund(&["cover", "--format", "json"]));
    let filtered = rows(&grund(&[
        "cover",
        "--glob",
        "FS-cover{,.**}",
        "--format",
        "json",
    ]));
    assert_eq!(
        filtered.len(),
        all.len(),
        "every scanned file keeps its record"
    );
    for (kept, entry) in filtered.iter().zip(&all) {
        assert_eq!(kept["path"], entry["path"]);
        let expected: Vec<_> = entry["citations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|site| site["id"] == "FS-cover")
            .cloned()
            .collect();
        assert_eq!(
            kept["citations"].as_array().unwrap(),
            &expected,
            "{}",
            entry["path"]
        );
    }
}

/// §FS-refs.1.2: ID mode keeps its refusals byte for byte beside the new modes.
#[test]
fn id_mode_refusals_keep_their_bytes() {
    for (args, stderr) in [
        (&["refs"][..], "error: refs requires an ID\n"),
        (
            &["refs", "FS-refs", "docs", "crates"],
            "error: refs takes an ID and at most one path argument\n",
        ),
        (
            &["refs", "FS-refs", "--section"],
            "error: --section requires a value\n",
        ),
        (&["refs", "--section", "3"], "error: refs requires an ID\n"),
    ] {
        let output = grund_output(args);
        assert_eq!(output.status.code(), Some(2), "grund {args:?}");
        assert!(output.stdout.is_empty(), "grund {args:?}");
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            stderr,
            "grund {args:?}"
        );
    }
}
