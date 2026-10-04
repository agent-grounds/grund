//! Test module: the two axes of `check`'s presentation query — exact code, and
//! the rule authority `--only-rule` names (§FS-check.1.4, §FS-rules.8).
//!
//! The behaviour these units sit under is pinned black-box in
//! `crates/grund-cli/tests/rules_contract/authority.rs`. What is here is the
//! seam each of those cases rests on: which of the two axes a finding has to
//! pass, and which voice wins where they disagree.

use super::selection::CheckFindingSelection;

fn origins(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

/// §FS-check.1.4: with neither axis asked for, the complete report is retained.
#[test]
fn an_empty_selection_retains_every_finding() {
    let selection = CheckFindingSelection::default();
    assert!(selection.retains("dangling", &[]));
    assert!(selection.retains("citation-cardinality", &origins(&["RULE-count"])));
}

/// §FS-rules.8: the authority axis keeps what the trial sentence authored and
/// nothing else — not a declared rule's finding, and not one no rule authored.
#[test]
fn the_authority_axis_keeps_only_what_the_trial_sentence_authored() {
    let mut selection = CheckFindingSelection::default();
    selection.scope_to_trial_rule();
    assert!(selection.retains("citation-cardinality", &origins(&["--rule"])));
    assert!(!selection.retains("citation-cardinality", &origins(&["RULE-count"])));
    assert!(!selection.retains("missing-citation", &[]));
}

/// §FS-rules.6, §FS-rules.8: a finding the trial sentence and a declared rule
/// authored jointly is the sentence's too, so the axis keeps it.
#[test]
fn a_jointly_authored_finding_passes_the_authority_axis() {
    let mut selection = CheckFindingSelection::default();
    selection.scope_to_trial_rule();
    assert!(selection.retains(
        "missing-citation",
        &origins(&["--rule", "RULE-requirements"])
    ));
}

/// §FS-check.1.4: the axes intersect — a finding is retained when its code
/// passes *and* its authority does, which is what `--only` alone cannot do
/// because a trial sentence emits the codes the declared rules emit.
#[test]
fn the_two_axes_intersect_rather_than_union() {
    let mut selection = CheckFindingSelection::default();
    selection.add_only("citation-cardinality").expect("a code");
    selection.scope_to_trial_rule();
    assert!(selection.retains("citation-cardinality", &origins(&["--rule"])));
    // The code passes, the authority does not.
    assert!(!selection.retains("citation-cardinality", &origins(&["RULE-count"])));
    // The authority passes, the code does not.
    assert!(!selection.retains("missing-citation", &origins(&["--rule"])));
}

/// §FS-check.1.4: `--ignore` is the strongest voice, so it empties a scoped
/// report rather than losing to it.
#[test]
fn ignore_wins_over_the_authority_axis() {
    let mut selection = CheckFindingSelection::default();
    selection.scope_to_trial_rule();
    selection
        .add_ignore("citation-cardinality")
        .expect("a code");
    assert!(!selection.retains("citation-cardinality", &origins(&["--rule"])));
}

/// §FS-check.2.4: `io` marks an incomplete scan, so neither axis can hide it.
#[test]
fn io_survives_the_authority_axis() {
    let mut selection = CheckFindingSelection::default();
    selection.scope_to_trial_rule();
    assert!(selection.retains("io", &[]));
}

/// §FS-rules.8: the flag is a boolean the CLI reads back to refuse it without a
/// `--rule` sentence, and asking twice asks the same thing.
#[test]
fn scoping_is_a_boolean_the_cli_can_read_back() {
    let mut selection = CheckFindingSelection::default();
    assert!(!selection.scopes_to_trial_rule());
    selection.scope_to_trial_rule();
    selection.scope_to_trial_rule();
    assert!(selection.scopes_to_trial_rule());
    assert_eq!(selection, {
        let mut once = CheckFindingSelection::default();
        once.scope_to_trial_rule();
        once
    });
}

/// The eight codes a rule finding can carry, written out rather than read from
/// the selector, so these units pin the set itself (§FS-rules.7.6).
const RULE_PRODUCED_CODES: &[&str] = &[
    "chapter-cardinality",
    "citation-cardinality",
    "uncited-unit",
    "unreached-declaration",
    "missing-citation",
    "forbidden-citation",
    "discouraged-citation",
    "suggested-citation",
];

/// §FS-check.1.4, §FS-rules.7.6: selecting any rule-produced code also selects
/// `invalid-rule`, so a rule that could not run is never a narrowed pass.
#[test]
fn a_rule_produced_code_carries_every_invalid_rule() {
    // A parse failure has no family to match on, so every row is carried.
    let rows = [origins(&["RULE-terms"]), origins(&["RULE-other"])];
    let dropped: Vec<&str> = RULE_PRODUCED_CODES
        .iter()
        .copied()
        .filter(|code| {
            let mut selection = CheckFindingSelection::default();
            selection.add_only(code).expect("a code");
            !rows
                .iter()
                .all(|authority| selection.retains("invalid-rule", authority))
        })
        .collect();
    assert!(
        dropped.is_empty(),
        "--only drops invalid-rule for {dropped:?}"
    );
}

/// §FS-check.1.4: a selection that names no rule-produced code selects exactly
/// what it names.
#[test]
fn a_code_no_rule_produces_does_not_carry_invalid_rule() {
    let mut selection = CheckFindingSelection::default();
    selection.add_only("dangling").expect("a code");
    assert!(!selection.retains("invalid-rule", &origins(&["RULE-terms"])));
}

/// §FS-check.1.4: `--ignore invalid-rule` still wins over the carried rows.
#[test]
fn ignore_wins_over_the_carried_invalid_rule() {
    let mut selection = CheckFindingSelection::default();
    selection.add_only("chapter-cardinality").expect("a code");
    selection.add_ignore("invalid-rule").expect("a code");
    assert!(!selection.retains("invalid-rule", &origins(&["RULE-terms"])));
}

/// §FS-check.1.4, §FS-rules.8: the authority axis still applies to the carried
/// rows — a declared rule's `invalid-rule` drops, the trial sentence's stays.
#[test]
fn the_authority_axis_applies_to_the_carried_invalid_rule() {
    let mut selection = CheckFindingSelection::default();
    selection.add_only("chapter-cardinality").expect("a code");
    selection.scope_to_trial_rule();
    assert!(!selection.retains("invalid-rule", &origins(&["RULE-terms"])));
    assert!(selection.retains("invalid-rule", &origins(&["--rule"])));
}
