//! Every command gives the same answer about one tree (§REQ-one-answer), proved
//! by comparison as §REQ-one-answer.proof says: each question is asked of every
//! surface that answers it, each through its own entrypoint, and each answer is
//! checked against the point that defines it as well, because two surfaces wrong
//! alike agree.
//!
//! Seeded thin under §AR-ci.requirements.1. The trees are the reproduced rows of
//! #576 (r1, r2 and r3), one tree per row of §REQ-one-answer.differences, and a
//! fixed sample of the end-to-end corpus: the first case of each family in
//! `SAMPLED_FAMILIES` whose fixture is a plain project. The surfaces are the
//! command line's; the editor and the bindings are compared by
//! `lsp_cli_parity.rs` and `stub_resolution_agreement.rs` until this test drives
//! them too.
//!
//! `LEDGER` is the divergence ledger: one row per known divergence, naming the
//! tree and the invariant. `DEFERRED` names each row of §REQ-one-answer.differences
//! only the editor, shell completion or an unsaved buffer shows, with the step of
//! #576 that builds its harness, and `WIDENING` what the seed leaves out. A divergence it does not list fails, and so does a
//! row that no longer reproduces, so the ledger only shrinks. A divergence found
//! here that §REQ-one-answer.differences does not list is added there by an
//! amendment naming its baseline, or kept as a ledger-only row with its reason
//! (the list rule of §REQ-one-answer); it is never kept in this file alone.

#[path = "support/agreement_identity.rs"]
mod agreement_identity;
#[path = "support/agreement_sections.rs"]
mod agreement_sections;
#[path = "support/agreement_stubs.rs"]
mod agreement_stubs;
#[path = "support/agreement_tree.rs"]
mod agreement_tree;

use agreement_identity as identity;
use agreement_sections as sections;
use agreement_stubs as stubs;
use agreement_tree::{Verdict, grund, json_lines, stdout};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// `(row of #576's ledger or of §REQ-one-answer.differences, tree, invariant)`.
/// G01 and P05 are defects every surface shares, kept here alone as #576's
/// ruling V-D04 says; every other row is listed in §REQ-one-answer.differences.
const LEDGER: &[(&str, &str, &str)] = &[
    ("G01", "r1", "citation-is-its-whole-token"),
    ("R01", "r2", "narrowed-resolves-as-check"),
    ("P05", "r3", "one-anchor"),
    ("G03", "g03", "a-target-kind-is-one-kind"),
    ("G04", "g04", "a-non-citable-kind-is-refused-once"),
    ("G05", "g05", "a-members-chapter-is-selected"),
    ("G06", "g06", "a-lowercase-subject-is-refused-once"),
    ("P01", "p01", "a-body-ends-where-refs-ends-it"),
    ("P02", "p02", "a-block-comment-section-is-one-section"),
    ("P03", "p03", "a-top-level-doc-heading-is-one-section"),
    ("P07", "p07", "a-plain-heading-is-in-one-section"),
    ("P10", "p10", "an-adjacent-note-is-in-one-body"),
    ("P12", "p12", "a-doc-comment-section-ends-once"),
    (
        "Placement",
        "placement",
        "a-stubbed-declaration-is-at-one-site",
    ),
    ("U01", "u01", "a-twice-declared-section-is-refused-once"),
    ("U03a", "u03a", "the-batch-returns-the-targets-section"),
    ("U03d", "u03d", "the-selector-selects-the-targets-chapter"),
    ("U04", "u04", "an-obligation-is-judged-on-the-target"),
    ("U05", "u05", "a-rule-targets-title-is-read-once"),
    ("U06", "u06", "an-unused-declaration-is-at-one-site"),
    ("R02", "r02", "a-narrowed-workspace-run-resolves-the-alias"),
];

/// `(row, tree, invariant, the question asked of the tree)`: each tree this test
/// builds, the reproduced rows of #576 first, then one per row of
/// §REQ-one-answer.differences the command line can show.
const TREES: &[(&str, &str, &str, fn() -> Verdict)] = &[
    (
        "G01",
        "r1",
        "citation-is-its-whole-token",
        identity::citation_is_its_whole_token,
    ),
    (
        "R01",
        "r2",
        "narrowed-resolves-as-check",
        identity::a_narrowed_run_resolves_what_check_resolves,
    ),
    (
        "P05",
        "r3",
        "one-anchor",
        sections::the_anchor_is_one_and_the_headings,
    ),
    (
        "G03",
        "g03",
        "a-target-kind-is-one-kind",
        identity::a_target_kind_is_one_kind,
    ),
    (
        "G04",
        "g04",
        "a-non-citable-kind-is-refused-once",
        identity::a_non_citable_kind_is_refused_once,
    ),
    (
        "G05",
        "g05",
        "a-members-chapter-is-selected",
        identity::a_members_chapter_is_selected_where_it_answers,
    ),
    (
        "G06",
        "g06",
        "a-lowercase-subject-is-refused-once",
        identity::a_lowercase_subject_is_refused_once,
    ),
    (
        "P01",
        "p01",
        "a-body-ends-where-refs-ends-it",
        sections::a_body_ends_where_refs_ends_it,
    ),
    (
        "P02",
        "p02",
        "a-block-comment-section-is-one-section",
        sections::a_block_comment_section_is_one_section,
    ),
    (
        "P03",
        "p03",
        "a-top-level-doc-heading-is-one-section",
        sections::a_top_level_doc_heading_is_one_section,
    ),
    (
        "P07",
        "p07",
        "a-plain-heading-is-in-one-section",
        sections::a_plain_heading_is_in_one_section,
    ),
    (
        "P10",
        "p10",
        "an-adjacent-note-is-in-one-body",
        sections::an_adjacent_note_is_in_one_body,
    ),
    (
        "P12",
        "p12",
        "a-doc-comment-section-ends-once",
        sections::a_doc_comment_section_ends_once,
    ),
    (
        "Placement",
        "placement",
        "a-stubbed-declaration-is-at-one-site",
        stubs::a_stubbed_declaration_is_at_one_site,
    ),
    (
        "U01",
        "u01",
        "a-twice-declared-section-is-refused-once",
        stubs::a_twice_declared_section_is_refused_once,
    ),
    (
        "U03a",
        "u03a",
        "the-batch-returns-the-targets-section",
        stubs::the_batch_returns_the_targets_section,
    ),
    (
        "U03d",
        "u03d",
        "the-selector-selects-the-targets-chapter",
        stubs::the_selector_selects_the_targets_chapter,
    ),
    (
        "U04",
        "u04",
        "an-obligation-is-judged-on-the-target",
        stubs::an_obligation_is_judged_on_the_target,
    ),
    (
        "U05",
        "u05",
        "a-rule-targets-title-is-read-once",
        stubs::a_rule_targets_title_is_read_once,
    ),
    (
        "U06",
        "u06",
        "an-unused-declaration-is-at-one-site",
        stubs::an_unused_declaration_is_at_one_site,
    ),
    (
        "R02",
        "r02",
        "a-narrowed-workspace-run-resolves-the-alias",
        identity::a_narrowed_workspace_run_resolves_the_alias,
    ),
];

/// `(row, step, what it needs)`: each row of §REQ-one-answer.differences no
/// command-line tree shows, and the step of #576 that gives this test its harness.
const DEFERRED: &[(&str, &str, &str)] = &[
    ("P16", "#598", "the editor snapshot's section span"),
    ("U03b", "#597", "shell completion, a row read from code"),
    ("U03c", "#597", "the editor snapshot"),
    (
        "R04",
        "#595",
        "an editor buffer under the target's physical path",
    ),
    ("R05", "#595", "an editor buffer with no file behind it"),
    ("R06a", "#595", "an editor buffer of an index"),
    ("R06b", "#595", "an editor buffer with a dangling citation"),
    ("R06c", "#595", "an editor buffer of an AGENTS.md block"),
    ("R06d", "#595", "an editor buffer of an end-to-end manifest"),
];

/// What the seed leaves out under §AR-ci.requirements.1, and the step that
/// widens it before #576 closes.
const WIDENING: &[(&str, &str)] = &[(
    "every end-to-end tree, its narrowed, path-argument and editor-overlay variants, \
     and `cover`, the editor snapshot and the bindings as surfaces",
    "#600",
)];

/// The steps of #576 a deferred row or a widening may name.
const STEPS: &[&str] = &["#595", "#596", "#597", "#598", "#599", "#600"];

/// The families of `tests/e2e/cases/` whose first plain project is sampled.
const SAMPLED_FAMILIES: &[&str] = &[
    "check",
    "show",
    "refs",
    "list",
    "fmt",
    "stub",
    "workspace",
    "symlink",
    "doccomment",
    "scan",
    "glob",
    "cover",
];

/// Every ID `list` prints is the ID `list --size` measures, at the same
/// `path:line` (§REQ-one-answer.identity.declarations).
fn one_declared_set(root: &Path) -> Verdict {
    let sites = |args: &[&str]| -> BTreeSet<(String, String, u64)> {
        json_lines(&stdout(&grund(root, args)))
            .iter()
            .filter(|row| row["section"].is_null())
            .map(|row| {
                (
                    row["id"].as_str().unwrap_or("").to_string(),
                    row["path"].as_str().unwrap_or("").to_string(),
                    row["line"].as_u64().unwrap_or(0),
                )
            })
            .collect()
    };
    let listed = sites(&["list", "--format", "json"]);
    let measured = sites(&["list", "--size", "--format", "json"]);
    if listed != measured {
        let only_list: Vec<_> = listed.difference(&measured).collect();
        let only_size: Vec<_> = measured.difference(&listed).collect();
        return Err(format!(
            "list only {only_list:?}; list --size only {only_size:?}"
        ));
    }
    Ok(())
}

/// The sampled end-to-end trees: the first plain project of each family.
fn sampled_trees() -> Vec<(String, PathBuf)> {
    let cases = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/e2e/cases");
    let mut names: Vec<String> = fs::read_dir(&cases)
        .expect("read e2e cases")
        .map(|entry| {
            entry
                .expect("case")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    SAMPLED_FAMILIES
        .iter()
        .filter_map(|family| {
            names
                .iter()
                .find(|name| {
                    name.starts_with(&format!("{family}-"))
                        && cases.join(name).join("repo/grund.toml").is_file()
                })
                .map(|name| (name.clone(), cases.join(name).join("repo")))
        })
        .collect()
}

/// Every divergence found, as `(tree, invariant, the answers that differ)`.
fn divergences() -> Vec<(String, &'static str, String)> {
    let mut found = Vec::new();
    for (_, tree, invariant, verdict) in TREES {
        if let Err(answers) = verdict() {
            found.push((tree.to_string(), *invariant, answers));
        }
    }
    for (name, root) in sampled_trees() {
        if let Err(answers) = one_declared_set(&root) {
            found.push((name, "one-declared-set", answers));
        }
    }
    found
}

/// What is wrong with `ledger` against what was `found`, both ways round.
fn problems(found: &[(String, &str, String)], ledger: &[(&str, &str, &str)]) -> Vec<String> {
    let mut wrong = Vec::new();
    for (tree, invariant, answers) in found {
        if !ledger.iter().any(|(_, t, i)| t == tree && i == invariant) {
            wrong.push(format!("{tree} / {invariant}: {answers}"));
        }
    }
    for (row, tree, invariant) in ledger {
        if !found.iter().any(|(t, i, _)| t == tree && i == invariant) {
            wrong.push(format!(
                "ledger row {row} ({tree} / {invariant}) no longer reproduces: take it out"
            ));
        }
    }
    wrong
}

#[test]
fn every_command_gives_one_answer_but_for_the_ledger() {
    let wrong = problems(&divergences(), LEDGER);
    assert!(
        wrong.is_empty(),
        "{} divergence(s) not as the ledger says (\u{a7}REQ-one-answer):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// Each row of §REQ-one-answer.differences, as `differences.<group>` lists it.
fn difference_rows() -> Vec<String> {
    let requirement =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/requirements/REQ-one-answer.md");
    let text = fs::read_to_string(requirement).expect("read REQ-one-answer");
    let mut inside = false;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.starts_with("### differences.") {
            inside = true;
        } else if line.starts_with("## ") || line.starts_with("### ") {
            inside = false;
        } else if inside && let Some(rest) = line.strip_prefix("- **") {
            rows.push(rest.split(".**").next().unwrap_or(rest).to_string());
        }
    }
    rows
}

/// The rows #576's ruling V-D04 keeps in this file alone: defects every surface
/// shares, so §REQ-one-answer.differences does not list them.
const LEDGER_ONLY: &[&str] = &["G01", "P05"];

/// What is wrong between this file's trees and deferrals and the rows
/// §REQ-one-answer.differences lists, both ways round: a listed row with no tree
/// and no deferral, a row both built and deferred, and a built or deferred row
/// the requirement no longer lists, which would otherwise live here alone.
fn listing_problems(rows: &[String]) -> Vec<String> {
    let built: BTreeSet<&str> = TREES.iter().map(|(row, ..)| *row).collect();
    let deferred: BTreeSet<&str> = DEFERRED.iter().map(|(row, ..)| *row).collect();
    let listed = |row: &str| rows.iter().any(|r| r == row);
    let mut wrong = Vec::new();
    for row in rows {
        if !built.contains(row.as_str()) && !deferred.contains(row.as_str()) {
            wrong.push(format!("{row} is listed but has no tree in this test"));
        }
    }
    for row in built.intersection(&deferred) {
        wrong.push(format!("{row} is built and deferred at once"));
    }
    for row in built.union(&deferred) {
        if !listed(row) && !LEDGER_ONLY.contains(row) {
            wrong.push(format!(
                "{row} is no longer listed: take it out of this test"
            ));
        }
    }
    wrong
}

/// Every listed difference has its tree here, or is deferred by name with the
/// step that builds its harness, so its row closes only when this test sees the
/// two answers agree (§REQ-one-answer.proof).
#[test]
fn every_listed_difference_has_its_tree() {
    let wrong = listing_problems(&difference_rows());
    assert!(
        wrong.is_empty(),
        "{} row(s) not as \u{a7}REQ-one-answer.differences lists them:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// A built row the requirement drops is refused like a deferred one, and only the
/// ledger-only rows stay here unlisted (§REQ-one-answer.proof).
#[test]
fn a_row_the_requirement_drops_is_refused() {
    let rows: Vec<String> = difference_rows()
        .into_iter()
        .filter(|r| r != "G03")
        .collect();
    let wrong = listing_problems(&rows);
    assert_eq!(
        wrong,
        vec!["G03 is no longer listed: take it out of this test".to_string()]
    );
}

#[test]
fn every_deferral_and_widening_names_its_step() {
    for (what, step) in DEFERRED
        .iter()
        .map(|(row, step, _)| (*row, *step))
        .chain(WIDENING.iter().copied())
    {
        assert!(
            STEPS.contains(&step),
            "{what} names no step of #576: {step}"
        );
    }
}

/// Each tree is one ledger row, under the same tree and invariant.
#[test]
fn every_tree_is_its_ledger_row_or_agrees() {
    for (row, tree, invariant, _) in TREES {
        if let Some((listed, ..)) = LEDGER.iter().find(|(_, t, i)| t == tree && i == invariant) {
            assert_eq!(
                listed, row,
                "{tree} / {invariant} is ledgered under another row"
            );
        }
    }
}

#[test]
fn the_ledger_fails_on_a_new_divergence_and_on_a_stale_row() {
    let found = vec![("t".to_string(), "i", "a != b".to_string())];
    assert!(problems(&found, &[("X01", "t", "i")]).is_empty());
    assert_eq!(problems(&found, &[]), ["t / i: a != b"]);
    assert_eq!(
        problems(&[], &[("X01", "t", "i")]),
        ["ledger row X01 (t / i) no longer reproduces: take it out"]
    );
}

#[test]
fn the_sample_names_one_tree_per_family() {
    let sampled = sampled_trees();
    assert_eq!(
        sampled.len(),
        SAMPLED_FAMILIES.len(),
        "a family has no plain project to sample: {sampled:?}"
    );
}
