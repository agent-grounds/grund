//! §FS-check.checks.closed-citation-grounding: exact C1/C2 command goldens.
use super::case_runner::{CaseKind, assert_every_case_passed, run_case};

#[test]
#[ignore = "red until #1460 emits the closed grounding warning; implement removes this"]
fn closed_citation_grounding_goldens() {
    let root = super::repo_root();
    let mut cases: Vec<_> = std::fs::read_dir(root.join("tests/e2e/cases"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("closed-citation-grounding-")
        })
        .collect();
    cases.sort();
    assert_eq!(cases.len(), 8, "C1 and C2 on all four surfaces");
    let results: Vec<_> = cases
        .iter()
        .map(|case| run_case(&root, case, CaseKind::E2e))
        .collect();
    assert_every_case_passed("closed-citation-grounding", &results);
}
