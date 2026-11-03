//! Issue #1152's cases, run alone: a version-2 row holds every entry to what its form asks
//! of a home (§FS-fields.forms.places), and a later entry warns with `config-later-place`
//! through the window (§FS-fields.forms.places.window) or is refused where the scan already
//! fails on it (§FS-fields.forms.places.unreadable). Every one is also in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

/// What this change moves: each fails today, on the warning or the refusal it does not print.
const MOVES: [&str; 18] = [
    "config-v2-later-place-field-file-absent-warns",
    "config-v2-later-place-field-folder-absent-warns",
    "config-v2-later-place-field-folder-not-a-folder-warns",
    "config-v2-later-place-rule-and-field-both-warn",
    "config-v2-later-place-rule-file-not-markdown-warns",
    "config-v2-later-place-value-file-absent-ignored",
    "config-v2-later-place-value-file-absent-only",
    "config-v2-later-place-value-file-absent-show",
    "config-v2-later-place-value-file-absent-validate",
    "config-v2-later-place-value-file-absent-warns",
    "config-v2-later-place-value-file-not-a-file-warns",
    "config-v2-later-place-value-folder-absent-refused",
    "config-v2-later-place-value-folder-absent-schema-refused",
    "config-v2-later-place-value-folder-absent-show-refused",
    "config-v2-later-place-value-folder-absent-validate-refused",
    "config-v2-later-place-value-folder-not-a-folder-refused",
    "config-v2-later-place-value-json-absent-refused",
    "config-v2-later-place-value-two-files-absent-warns",
];

/// What must not move: each passes today and has to keep passing with the same bytes.
const STAYS: [&str; 13] = [
    "config-v2-later-place-field-file-and-folder-refused",
    "config-v2-later-place-field-first-file-absent-refused",
    "config-v2-later-place-rule-file-absent-quiet",
    "config-v2-later-place-rule-file-and-folder-loads",
    "config-v2-later-place-rule-first-file-not-markdown-refused",
    "config-v2-later-place-value-fetch-two-places-refused",
    "config-v2-later-place-value-file-and-folder-refused",
    "config-v2-later-place-value-first-file-absent-refused",
    "config-v2-later-place-value-first-folder-absent-refused",
    "config-v2-later-place-value-two-files-present",
    "config-v2-later-place-value-two-files-present-resolves",
    "config-v2-later-place-value-two-folders-present",
    "config-v2-later-place-value-two-folders-present-resolves",
];

fn assert_cases(label: &str, names: &[&str]) {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = names
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed(label, &outcomes);
}

#[test]
#[ignore = "red until #1152 asks every entry of a row what its form asks of a home; implement removes this"]
fn later_place_moves() {
    assert_cases("later place moves", &MOVES);
}

#[test]
fn later_place_stays() {
    assert_cases("later place stays", &STAYS);
}
