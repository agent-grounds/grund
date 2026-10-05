//! Chapter diagnostic regressions for §FS-rules.7.2.1 and §FS-rules.3.5.1.
//! The unchanged-selector controls prove the goal chapter is actually reached.

use super::case_runner::{CaseKind::E2e, assert_every_case_passed, run_case};
use super::repo_root;

fn case(name: &str) {
    let root = repo_root();
    let path = root.join("tests/e2e/cases").join(name);
    assert_every_case_passed(name, &[run_case(&root, &path, E2e)]);
}

/// §FS-rules.7.2.1, §FS-errors.3: old bytes plus display-name context in text.
#[test]
fn chapter_diagnostic_display_name_text() {
    case("check-chapter-diagnostic-display-name-text");
}

/// §FS-rules.7.2.1: preserve every JSON field while explaining the zero count.
#[test]
fn chapter_diagnostic_display_name_json() {
    case("check-chapter-diagnostic-display-name-json");
}

/// §FS-rules.3.5.1: keep the full refusal prefix and explain internal whitespace.
#[test]
fn chapter_diagnostic_whitespace_refusal() {
    case("check-chapter-diagnostic-whitespace-refusal");
}

/// §FS-rules.2: a chapter subject selects the handle despite its display title.
#[test]
fn chapter_diagnostic_handle_subject_reaches_citation() {
    case("check-chapter-diagnostic-handle-subject");
    case("check-chapter-diagnostic-handle-subject-missing-citation");
}
