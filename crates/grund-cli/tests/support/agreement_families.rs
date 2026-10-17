//! Which end-to-end cases the comparison asks: every case of each family
//! `COVERED` names (§REQ-one-answer.proof.matrix.family) and the seed's sampled
//! first projects (§REQ-one-answer.proof.matrix.seed). Every other family is a
//! row of `WIDENING`, never both and never neither
//! (§REQ-one-answer.proof.delivery.widening).

use super::{COVERED, WIDENING};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// The families of `tests/e2e/cases/` whose first plain project the seed asks.
pub const SAMPLED_FAMILIES: &[&str] = &[
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

/// The corpus the comparison discovers, never a list kept beside it.
pub fn cases_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/e2e/cases")
}

/// Every case directory of the live corpus, by name.
pub fn discovered() -> BTreeSet<String> {
    fs::read_dir(cases_root())
        .expect("discover every case")
        .map(|entry| entry.expect("case").path())
        .filter(|path| path.is_dir())
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect()
}

/// A case's family: its directory name up to the first hyphen.
pub fn family(case: &str) -> &str {
    case.split('-').next().unwrap_or(case)
}

/// The seed's end-to-end cases: the first case of each sampled family whose
/// fixture is a plain project.
pub fn sampled(cases: &BTreeSet<String>) -> Vec<String> {
    let root = cases_root();
    SAMPLED_FAMILIES
        .iter()
        .filter_map(|wanted| {
            cases
                .iter()
                .find(|case| {
                    family(case) == *wanted && root.join(case).join("repo/grund.toml").is_file()
                })
                .cloned()
        })
        .collect()
}

/// The cases the acceptance asks: every case of a covered family, and the seed.
pub fn scope(cases: &BTreeSet<String>) -> BTreeSet<String> {
    cases
        .iter()
        .filter(|case| COVERED.contains(&family(case)))
        .cloned()
        .chain(sampled(cases))
        .collect()
}

/// What is wrong between the corpus's `families`, `covered` and `widening`.
fn problems(
    families: &BTreeSet<String>,
    covered: &[&str],
    widening: &[(&str, &str)],
) -> Vec<String> {
    let mut wrong = Vec::new();
    let mut widened = BTreeSet::new();
    for (row, step) in widening {
        if !families.contains(*row) {
            wrong.push(format!(
                "WIDENING row {row:?} names no family of the corpus"
            ));
        }
        if *step != "S4" {
            wrong.push(format!("WIDENING row {row:?} names {step}, not S4"));
        }
        if !widened.insert(*row) {
            wrong.push(format!("{row} is widened twice"));
        }
    }
    for name in covered {
        if !families.contains(*name) {
            wrong.push(format!("covered family {name} has no case"));
        }
        if widened.contains(name) {
            wrong.push(format!("{name} is covered and widened at once"));
        }
    }
    for name in families {
        if !covered.contains(&name.as_str()) && !widened.contains(name.as_str()) {
            wrong.push(format!("{name} is neither covered nor widened"));
        }
    }
    wrong
}

#[test]
fn every_family_is_covered_or_widened_never_both() {
    let families: BTreeSet<_> = discovered()
        .iter()
        .map(|case| family(case).to_string())
        .collect();
    let wrong = problems(&families, COVERED, WIDENING);
    assert!(
        wrong.is_empty(),
        "{} problem(s) between the corpus's families, COVERED and WIDENING \
         (\u{a7}REQ-one-answer.proof.delivery.widening):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn the_family_accounting_refuses_a_gap_an_overlap_and_a_stranger() {
    let families = BTreeSet::from(["a".to_string(), "b".to_string()]);
    assert!(problems(&families, &["a"], &[("b", "S4")]).is_empty());
    assert_eq!(
        problems(&families, &["a"], &[]),
        ["b is neither covered nor widened"]
    );
    assert_eq!(
        problems(&families, &["a", "b"], &[("b", "S4")]),
        ["b is covered and widened at once"]
    );
    assert_eq!(
        problems(&families, &["a"], &[("b", "S4"), ("c", "S4")]),
        ["WIDENING row \"c\" names no family of the corpus"]
    );
    assert_eq!(
        problems(&families, &["a", "c"], &[("b", "S4")]),
        ["covered family c has no case"]
    );
    assert_eq!(
        problems(&families, &["a"], &[("b", "S5")]),
        ["WIDENING row \"b\" names S5, not S4"]
    );
}

#[test]
fn the_sample_names_one_case_per_family() {
    let sampled = sampled(&discovered());
    assert_eq!(
        sampled.len(),
        SAMPLED_FAMILIES.len(),
        "a family has no plain project to sample: {sampled:?}"
    );
}

/// A covered family is asked whole: the scope holds every one of its cases,
/// however many the corpus has when the comparison runs.
#[test]
fn the_scope_takes_every_case_of_a_covered_family() {
    let cases = discovered();
    let scope = scope(&cases);
    for case in &cases {
        if COVERED.contains(&family(case)) {
            assert!(scope.contains(case), "{case} is left out of its family");
        }
    }
    for case in sampled(&cases) {
        assert!(scope.contains(&case), "seed case {case} is left out");
    }
    assert_eq!(family("cover-lines-text"), "cover");
}
