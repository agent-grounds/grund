//! Every command gives the same answer about one tree (§REQ-one-answer), proved
//! by comparison as §REQ-one-answer.proof says: each question is asked of every
//! surface that answers it, each through its own entrypoint, and each answer is
//! checked against the point that defines it as well, because two surfaces wrong
//! alike agree.
//!
//! Corpus execution belongs to `agreement_acceptance`: the seed and every case of
//! each family `COVERED` names, under every applicable invocation variant and
//! real CLI/editor/Python/Node entrypoint (§REQ-one-answer.proof.matrix.family).
//! The dedicated trees here keep each closed row and positive regression control
//! individually identifiable; each must agree (§REQ-one-answer.proof.evidence.closed).
//! `lsp_cli_parity.rs` retains the distinct real-server wire assertions
//! (§REQ-one-answer.proof.transport). Gate timing is measured separately at
//! fixed source SHAs (§AR-ci.requirements.1).
//!
//! `LEDGER` is the divergence ledger: one row per known divergence, naming the
//! tree and the invariant. `DEFERRED` names each row of §REQ-one-answer.differences
//! only the editor, shell completion or an unsaved buffer shows, with the step of
//! #576 that builds its harness, and `WIDENING` each family not yet asked whole, each step
//! listed live in `tests/integration/one_answer_steps.toml`. A divergence it does
//! not list fails, and so does a row that no longer reproduces, so the ledger only
//! shrinks. A divergence found here that §REQ-one-answer.differences does not list
//! is added there by an amendment naming its baseline, or kept as a ledger-only row
//! with its reason (the list rule of §REQ-one-answer); it is never kept in this file alone.

#[path = "support/agreement_acceptance.rs"]
mod agreement_acceptance;
#[path = "support/agreement_buffers.rs"]
mod agreement_buffers;
#[path = "support/agreement_closed_rows.rs"]
mod agreement_closed_rows;
#[path = "support/agreement_compare.rs"]
mod agreement_compare;
#[path = "support/agreement_context.rs"]
mod agreement_context;
#[path = "support/agreement_context_controls.rs"]
mod agreement_context_controls;
#[path = "support/agreement_controls.rs"]
mod agreement_controls;
#[path = "support/agreement_corpus.rs"]
mod agreement_corpus;
#[path = "support/agreement_editor.rs"]
mod agreement_editor;
#[path = "support/agreement_editor_worker.rs"]
mod agreement_editor_worker;
#[path = "support/agreement_evidence.rs"]
mod agreement_evidence;
#[path = "support/agreement_expectations.rs"]
mod agreement_expectations;
#[path = "support/agreement_expected_bytes.rs"]
mod agreement_expected_bytes;
#[path = "support/agreement_facts.rs"]
mod agreement_facts;
#[path = "support/agreement_families.rs"]
mod agreement_families;
#[path = "support/agreement_host.rs"]
mod agreement_host;
#[path = "support/agreement_host_projection.rs"]
mod agreement_host_projection;
#[path = "support/agreement_identity.rs"]
mod agreement_identity;
#[path = "support/agreement_manifest.rs"]
mod agreement_manifest;
#[path = "support/agreement_materialization.rs"]
mod agreement_materialization;
#[path = "support/agreement_mutations.rs"]
mod agreement_mutations;
#[path = "support/agreement_oracles.rs"]
mod agreement_oracles;
#[path = "support/agreement_paths.rs"]
mod agreement_paths;
#[path = "support/agreement_projection_controls.rs"]
mod agreement_projection_controls;
#[path = "support/agreement_query_controls.rs"]
mod agreement_query_controls;
#[path = "support/agreement_query_inventory.rs"]
mod agreement_query_inventory;
#[path = "support/agreement_questions.rs"]
mod agreement_questions;
#[path = "support/agreement_refs_fields.rs"]
mod agreement_refs_fields;
#[path = "support/agreement_refusals.rs"]
mod agreement_refusals;
#[path = "support/agreement_render.rs"]
mod agreement_render;
#[path = "support/agreement_render_answers.rs"]
mod agreement_render_answers;
#[path = "support/agreement_render_cli.rs"]
mod agreement_render_cli;
#[path = "support/agreement_render_editor.rs"]
mod agreement_render_editor;
#[path = "support/agreement_render_inventory.rs"]
mod agreement_render_inventory;
#[path = "support/agreement_render_model.rs"]
mod agreement_render_model;
#[path = "support/agreement_render_show.rs"]
mod agreement_render_show;
#[path = "support/agreement_render_tree.rs"]
mod agreement_render_tree;
#[path = "support/agreement_sections.rs"]
mod agreement_sections;
#[path = "support/agreement_sites.rs"]
mod agreement_sites;
#[path = "support/agreement_snapshot.rs"]
mod agreement_snapshot;
#[path = "support/agreement_stubs.rs"]
mod agreement_stubs;
#[path = "support/agreement_tranches.rs"]
mod agreement_tranches;
#[path = "support/agreement_tree.rs"]
mod agreement_tree;
#[path = "support/agreement_truth.rs"]
mod agreement_truth;
#[path = "support/agreement_truth_accounting.rs"]
mod agreement_truth_accounting;
#[path = "support/agreement_truth_controls.rs"]
mod agreement_truth_controls;

use agreement_buffers as buffers;
use agreement_closed_rows::problems;
use agreement_identity as identity;
use agreement_sections as sections;
use agreement_stubs as stubs;
use agreement_tree::{Verdict, listed_steps};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// `(row of #576's ledger or of §REQ-one-answer.differences, tree, invariant)`.
/// G01 is a defect every surface shares, kept here alone as #576's ruling V-D04
/// says; every other row is listed in §REQ-one-answer.differences.
const LEDGER: &[(&str, &str, &str)] = &[("G01", "r1", "citation-is-its-whole-token")];

/// `(row, tree, invariant, the question asked of the tree)`: each tree this test
/// builds, the reproduced rows of #576 first, then one per row of
/// §REQ-one-answer.differences the command line, shell completion or the editor
/// snapshot can show. A tree with no row holds a row that closed, U01 and U05 with
/// #597, U04, U07 and U08 with #651, G06 with #671, G09 with #680, Placement,
/// U03a to U03d and U06 with #650, P01, P02, P03, P07, P10 and P12 with #639, P16
/// with #655, and R01 and R02 with #621, so its two answers must agree.
const TREES: &[(Option<&str>, &str, &str, fn() -> Verdict)] = &[
    (
        Some("G01"),
        "r1",
        "citation-is-its-whole-token",
        identity::citation_is_its_whole_token,
    ),
    (
        None,
        "r2",
        "narrowed-resolves-as-check",
        identity::a_narrowed_run_resolves_what_check_resolves,
    ),
    (
        None,
        "g06",
        "a-lowercase-subject-is-refused-once",
        identity::a_lowercase_subject_is_refused_once,
    ),
    (
        None,
        "g09",
        "the-homeless-kind-is-one-verdict",
        identity::the_homeless_kind_is_one_verdict,
    ),
    (
        None,
        "p01",
        "a-body-ends-where-refs-ends-it",
        sections::a_body_ends_where_refs_ends_it,
    ),
    (
        None,
        "p02",
        "a-block-comment-section-is-one-section",
        sections::a_block_comment_section_is_one_section,
    ),
    (
        None,
        "p03",
        "a-top-level-doc-heading-is-one-section",
        sections::a_top_level_doc_heading_is_one_section,
    ),
    (
        None,
        "p07",
        "a-plain-heading-is-in-one-section",
        sections::a_plain_heading_is_in_one_section,
    ),
    (
        None,
        "p10",
        "an-adjacent-note-is-in-one-body",
        sections::an_adjacent_note_is_in_one_body,
    ),
    (
        None,
        "p12",
        "a-doc-comment-section-ends-once",
        sections::a_doc_comment_section_ends_once,
    ),
    (
        None,
        "p12-duplicate",
        "a-doc-comment-section-ends-at-a-duplicate",
        sections::a_doc_comment_section_ends_at_a_duplicate,
    ),
    (
        None,
        "p16",
        "a-section-title-span-is-the-toc-title",
        sections::a_section_title_span_is_the_toc_title,
    ),
    (
        None,
        "placement",
        "a-stubbed-declaration-is-at-one-site",
        stubs::a_stubbed_declaration_is_at_one_site,
    ),
    (
        None,
        "u01",
        "a-twice-declared-section-is-refused-once",
        stubs::a_twice_declared_section_is_refused_once,
    ),
    (
        None,
        "u03a",
        "the-batch-returns-the-targets-section",
        stubs::the_batch_returns_the_targets_section,
    ),
    (
        None,
        "u03b",
        "completion-offers-the-targets-section",
        stubs::completion_offers_the_targets_section,
    ),
    (
        None,
        "u03c",
        "the-editor-enumerates-the-targets-section",
        stubs::the_editor_enumerates_the_targets_section,
    ),
    (
        None,
        "u03d",
        "the-selector-selects-the-targets-chapter",
        stubs::the_selector_selects_the_targets_chapter,
    ),
    (
        None,
        "u04-prohibition",
        "a-prohibition-is-judged-on-the-target",
        stubs::a_prohibition_is_judged_on_the_target,
    ),
    (
        None,
        "u04-count",
        "a-count-is-judged-on-the-target",
        stubs::a_count_is_judged_on_the_target,
    ),
    (
        None,
        "u05",
        "a-rule-targets-title-is-read-once",
        stubs::a_rule_targets_title_is_read_once,
    ),
    (
        None,
        "u06",
        "an-unused-declaration-is-at-one-site",
        stubs::an_unused_declaration_is_at_one_site,
    ),
    (
        None,
        "u07",
        "a-chapter-rule-counts-the-targets-chapters",
        stubs::a_chapter_rule_counts_the_targets_chapters,
    ),
    (
        None,
        "u08",
        "refs-lists-the-targets-citation",
        stubs::refs_lists_the_targets_citation,
    ),
    (
        None,
        "r02",
        "a-narrowed-workspace-run-resolves-the-alias",
        identity::a_narrowed_workspace_run_resolves_the_alias,
    ),
];

/// `(row, step, what it needs)`: each row of §REQ-one-answer.differences no
/// command-line tree shows, and the step of #576 that gives this test its harness.
const DEFERRED: &[(&str, &str, &str)] = &[];

/// Each end-to-end family `COVERED` does not yet name, and the step that widens
/// the comparison over it (§REQ-one-answer.proof.delivery.widening). The seed
/// asks only a sampled project of some of them (§REQ-one-answer.proof.matrix.seed).
const WIDENING: &[(&str, &str)] = &[
    ("agent", "S4"),
    ("bare", "S4"),
    ("block", "S4"),
    ("check", "S4"),
    ("cli", "S4"),
    ("complete", "S4"),
    ("completions", "S4"),
    ("config", "S4"),
    ("configured", "S4"),
    ("cross", "S4"),
    ("declarations", "S4"),
    ("declare", "S4"),
    ("deterministic", "S4"),
    ("doc", "S4"),
    ("doccomment", "S4"),
    ("e2e", "S4"),
    ("external", "S4"),
    ("fetch", "S4"),
    ("fmt", "S4"),
    ("glob", "S4"),
    ("guide", "S4"),
    ("haskell", "S4"),
    ("id", "S4"),
    ("init", "S4"),
    ("inline", "S4"),
    ("integrations", "S4"),
    ("json", "S4"),
    ("line", "S4"),
    ("list", "S4"),
    ("markdown", "S4"),
    ("marked", "S4"),
    ("marker", "S4"),
    ("named", "S4"),
    ("non", "S4"),
    ("path", "S4"),
    ("refs", "S4"),
    ("relative", "S4"),
    ("remote", "S4"),
    ("report", "S4"),
    ("rust", "S4"),
    ("schema", "S4"),
    ("section", "S4"),
    ("show", "S4"),
    ("symlink", "S4"),
    ("word", "S4"),
    ("workspace", "S4"),
];

/// The end-to-end families the comparison asks whole, every case under every
/// applicable variant and surface (§REQ-one-answer.proof.matrix.family);
/// `WIDENING` names every other family (§REQ-one-answer.proof.delivery.widening).
const COVERED: &[&str] = &["basic", "cover", "scan", "stub"];

/// Every divergence found, as `(tree, invariant, the answers that differ)`.
fn divergences() -> Vec<(String, &'static str, String)> {
    let mut found = Vec::new();
    for (_, tree, invariant, verdict) in TREES {
        if let Err(answers) = verdict() {
            found.push((tree.to_string(), *invariant, answers));
        }
    }
    for (_, tree, verdict) in buffers::TREES {
        if let Err(answers) = verdict() {
            found.push((tree.to_string(), buffers::INVARIANT, answers));
        }
    }
    found
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
/// shares, so §REQ-one-answer.differences does not list them. P05 left it when
/// §DF-closing-hashes-are-not-the-heading corrected the anchor on every surface.
const LEDGER_ONLY: &[&str] = &["G01"];

/// Row P05, closed: `show` reports and `fmt --cross-refs` writes one anchor, the
/// rendered heading's with its closing `#` run taken off, `fs.md#1-steps`
/// (§FS-declarations.heading.rendered-text, §REQ-one-answer.writes.anchors).
#[test]
fn the_anchor_is_one_and_the_headings() {
    if let Err(divergence) = sections::the_anchor_is_one_and_the_headings() {
        panic!("{divergence}");
    }
}

/// What is wrong between this file's trees and deferrals and the rows
/// §REQ-one-answer.differences lists, both ways round: a listed row with no tree
/// and no deferral, a row both built and deferred, and a built or deferred row
/// the requirement no longer lists, which would otherwise live here alone.
fn listing_problems(rows: &[String]) -> Vec<String> {
    let built: BTreeSet<&str> = TREES
        .iter()
        .filter_map(|(row, ..)| *row)
        .chain(buffers::TREES.iter().filter_map(|(row, ..)| *row))
        .collect();
    listing_problems_against(rows, &built)
}

/// [`listing_problems`] against the rows `built` names rather than the rows the
/// trees here carry.
fn listing_problems_against(rows: &[String], built: &BTreeSet<&str>) -> Vec<String> {
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
/// ledger-only rows stay here unlisted (§REQ-one-answer.proof). No listed row is
/// open since R06a to R06d closed with #624, so R06a stands in as still built.
#[test]
fn a_row_the_requirement_drops_is_refused() {
    let built = BTreeSet::from(["G01", "R06a"]);
    let wrong = listing_problems_against(&difference_rows(), &built);
    assert_eq!(
        wrong,
        vec!["R06a is no longer listed: take it out of this test".to_string()]
    );
}

#[test]
fn every_deferral_and_widening_names_its_step() {
    let steps = listed_steps();
    for (what, step) in DEFERRED
        .iter()
        .map(|(row, step, _)| (*row, *step))
        .chain(WIDENING.iter().copied())
    {
        assert!(
            steps.contains(step),
            "{what} names no step listed in one_answer_steps.toml: {step}"
        );
    }
}

/// Each tree is one ledger row, under the same tree and invariant.
#[test]
fn every_tree_is_its_ledger_row_or_agrees() {
    for (row, tree, invariant, _) in TREES {
        let listed = LEDGER
            .iter()
            .find(|(_, t, i)| t == tree && i == invariant)
            .map(|(listed, ..)| *listed);
        assert_eq!(
            listed, *row,
            "{tree} / {invariant} is ledgered under another row, or a closed row's tree is"
        );
    }
    for (row, tree, _) in buffers::TREES {
        let listed = LEDGER
            .iter()
            .find(|(_, t, i)| t == tree && *i == buffers::INVARIANT)
            .map(|(listed, ..)| *listed);
        assert_eq!(
            listed, *row,
            "{tree} is ledgered under another row, or a closed row's tree is"
        );
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
