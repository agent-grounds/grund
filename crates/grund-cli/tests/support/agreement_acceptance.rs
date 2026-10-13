//! S4 acceptance against execution, not the harness's declared coverage
//! (§REQ-one-answer.proof.matrix, §REQ-one-answer.proof.evidence).

use super::{agreement_evidence, divergences, json_lines};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// Force with `cargo test -p grund --test command_agreement
/// agreement_acceptance::s4_executes_the_complete_corpus_matrix
/// -- --ignored --exact --nocapture`.
/// Remove the ignore when the implementation supplies the full observed matrix.
/// Applicability/materialized-copy attribution must be added at the actual call
/// boundary, with validated reasons; absent calls cannot become empty successes.
#[test]
#[ignore = "implementation pending: #674 complete S4 execution matrix"]
fn s4_executes_the_complete_corpus_matrix() {
    let cases = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases")
        .canonicalize()
        .expect("corpus");
    let expected: BTreeSet<String> = fs::read_dir(&cases)
        .expect("discover every case")
        .map(|entry| entry.expect("case").path())
        .filter(|path| path.is_dir())
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    let calls = agreement_evidence::capture(|| {
        let _ = divergences();
    });
    if let Some(output) = std::env::var_os("GRUND_AGREEMENT_EVIDENCE") {
        fs::write(
            output,
            serde_json::to_vec_pretty(&calls).expect("serialize actual calls"),
        )
        .expect("write requested execution evidence");
    }
    let mut cells = BTreeSet::new();
    let mut synthetic = 0;
    for call in &calls {
        let path = &call.path;
        let Ok(relative) = path.strip_prefix(&cases) else {
            synthetic += 1;
            continue;
        };
        let case = relative
            .components()
            .next()
            .unwrap()
            .as_os_str()
            .to_string_lossy();
        let has_path_argument = call.args.iter().any(|arg| {
            arg == "--path" || arg.starts_with("--path=") || call.path.join(arg).exists()
        });
        let variant = if call.overlay {
            "editor-overlay"
        } else if has_path_argument {
            "path-argument"
        } else if relative.components().count() > 2 {
            "narrowed"
        } else {
            "whole-tree"
        };
        cells.insert((case.into_owned(), variant.to_string(), call.surface.clone()));
    }
    let observed: BTreeSet<_> = cells.iter().map(|(case, ..)| case.clone()).collect();
    let surfaces: BTreeSet<_> = calls.iter().map(|call| call.surface.as_str()).collect();
    eprintln!(
        "observed {} completed CLI/editor calls: {} corpus cases / {} discovered; \
               {} synthetic calls; surfaces {surfaces:?}",
        calls.len(),
        observed.len(),
        expected.len(),
        synthetic
    );

    // Independent positive control: these reviewed fixture facts are not derived
    // from a second engine call. Two empty list answers must not pass this check.
    // §REQ-one-answer.identity.declarations: IDs and physical sites, including lines.
    let golden = fs::read_to_string(cases.join("list-basic/expected.stdout")).unwrap();
    let expected_sites: BTreeSet<_> = golden
        .lines()
        .map(|line| {
            let words: Vec<_> = line.split_whitespace().collect();
            (words[0].to_string(), words[1].to_string())
        })
        .collect();
    assert!(
        !expected_sites.is_empty(),
        "independent fixture must contain facts"
    );
    for surface in ["list", "list-size"] {
        let call = calls
            .iter()
            .find(|call| call.surface == surface && call.path.ends_with("list-basic/repo"))
            .expect("list-basic control executed");
        assert_eq!(call.exit, Some(0), "{surface}: {:?}", call.stderr);
        let actual: BTreeSet<_> = json_lines(&String::from_utf8_lossy(&call.stdout))
            .iter()
            .filter(|row| row["section"].is_null())
            .map(|row| {
                (
                    row["id"].as_str().unwrap().to_string(),
                    format!("{}:{}", row["path"].as_str().unwrap(), row["line"]),
                )
            })
            .collect();
        assert_eq!(
            actual, expected_sites,
            "{surface} against reviewed list-basic golden"
        );
    }

    let variants = ["whole-tree", "narrowed", "path-argument", "editor-overlay"];
    let required = [
        "check",
        "show",
        "show-batch",
        "refs",
        "list",
        "list-size",
        "cover",
        "cover-lines",
        "fmt",
        "editor",
        "python",
        "node",
    ];
    let missing_cases: Vec<_> = expected.difference(&observed).collect();
    let mut missing_cells = 0;
    for case in &expected {
        for variant in variants {
            for surface in required {
                if !cells.contains(&(case.clone(), variant.to_string(), surface.to_string())) {
                    missing_cells += 1;
                }
            }
        }
    }
    assert!(
        missing_cells == 0,
        "S4 execution gap: {} cases have no corpus execution (first: {:?}); \
         {missing_cells} case/variant/surface cells have no execution or validated \
         applicability evidence; cover, editor and actual hosts must enter themselves",
        missing_cases.len(),
        missing_cases.first()
    );
}
