//! S4 acceptance against completed execution and independent source/golden facts
//! (§REQ-one-answer.proof.matrix, §REQ-one-answer.proof.evidence, §FS-list.3).

use super::agreement_tree::json_lines;
use super::{LEDGER, agreement_corpus, agreement_evidence, divergences, problems};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

/// Enabled in normal Cargo execution. Optional GRUND_AGREEMENT_EVIDENCE writes
/// the complete completed-call receipts without changing the assertions.
#[test]
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
    let mut errors = Vec::new();
    let calls = agreement_evidence::capture(|| {
        errors.extend(problems(&divergences(), LEDGER));
        errors.extend(agreement_corpus::execute());
    });
    if let Some(output) = std::env::var_os("GRUND_AGREEMENT_EVIDENCE") {
        fs::write(
            output,
            serde_json::to_vec_pretty(&calls).expect("serialize actual calls"),
        )
        .expect("write requested execution evidence");
    }
    positive_control(&cases, &calls);
    let mut cells = BTreeSet::new();
    let mut materializations = BTreeMap::new();
    let mut host_questions = BTreeSet::new();
    let mut incomplete_truth = Vec::new();
    for call in &calls {
        let Some(context) = &call.context else {
            continue;
        };
        let case = context["case"]
            .as_str()
            .expect("attributed original")
            .to_string();
        let variant = context["variant"]
            .as_str()
            .expect("attributed variant")
            .to_string();
        assert!(
            expected.contains(&case),
            "receipt names an undiscovered original {case}"
        );
        if call.surface == "materialization" {
            assert_eq!(
                call.expected["oracle"],
                "original-manifest-and-physical-bytes"
            );
            assert_eq!(call.result["case"], case);
            assert_eq!(call.result["variant"], variant);
            materializations.insert((case.clone(), variant.clone()), call);
            continue;
        }
        if call.surface == "inapplicable" {
            let materialization = materializations
                .get(&(case.clone(), variant.clone()))
                .expect("applicability follows verified preparation");
            assert!(!materialization.result["absent"].is_null());
            assert_eq!(call.result["reason"], materialization.expected["reason"]);
            assert_eq!(call.request["context"], materialization.result);
            cells.insert((
                case,
                variant,
                call.request["surface"].as_str().unwrap().to_string(),
            ));
            continue;
        }
        if agreement_corpus::SURFACES.contains(&call.surface.as_str()) {
            // CLI receipts contain a real exit and output. Host receipts contain
            // their exact framed request/result, including structured refusals.
            // A surface label alone is insufficient.
            match call.surface.as_str() {
                "python" | "node" => {
                    assert!(call.request["operation"].is_string());
                    assert!(
                        call.result.is_object()
                            && call.result.get("failure").is_some()
                            && call.result.get("result").is_some()
                    );
                    host_questions.insert((
                        case.clone(),
                        variant.clone(),
                        call.surface.clone(),
                        call.request["operation"].as_str().unwrap().to_string(),
                    ));
                }
                "editor" => {
                    assert!(call.result.is_object());
                    assert!(call.request["question"].is_string());
                }
                _ => {
                    assert!(call.exit.is_some(), "CLI receipt has no completed process");
                    assert!(!call.request.is_null());
                    if call.expected["oracle"] != "reviewed-manifest-golden"
                        && call.expected["oracle"] != "independent-mutation-input-bytes"
                    {
                        if variant == "narrowed" {
                            assert_eq!(
                                call.path,
                                PathBuf::from(context["cwd"].as_str().unwrap())
                                    .canonicalize()
                                    .unwrap()
                            );
                        }
                        if variant == "path-argument" && call.surface != "cover-lines" {
                            assert!(
                                call.args
                                    .iter()
                                    .any(|arg| Some(arg.as_str()) == context["path"].as_str())
                            );
                        }
                    }
                }
            }
            assert!(
                call.expected.is_object(),
                "{} / {} / {} has no independent fact",
                case,
                variant,
                call.surface
            );
            assert!(
                call.expected.get("oracleError").is_none(),
                "source oracle failed"
            );
            if call.expected["complete"] == false {
                incomplete_truth.push(format!("{case} / {variant} / {}", call.surface));
            }
            cells.insert((case, variant, call.surface.clone()));
        }
    }
    let mut missing = Vec::new();
    for case in &expected {
        for variant in agreement_corpus::VARIANTS {
            let materialization = materializations.get(&(case.clone(), variant.to_string()));
            assert!(
                materialization.is_some(),
                "{case} / {variant}: no verified original materialization"
            );
            for surface in agreement_corpus::SURFACES {
                if !cells.contains(&(case.clone(), variant.to_string(), surface.to_string())) {
                    missing.push(format!("{case} / {variant} / {surface}"));
                }
            }
            if materialization.unwrap().result["absent"].is_null() {
                for (surface, operations) in [
                    (
                        "python",
                        &[
                            "check",
                            "list_ids",
                            "list_sizes",
                            "show",
                            "show_batch",
                            "refs",
                            "cover",
                            "fmt",
                        ][..],
                    ),
                    (
                        "node",
                        &[
                            "check",
                            "list",
                            "listSizes",
                            "show",
                            "showBatch",
                            "refs",
                            "cover",
                            "fmt",
                        ][..],
                    ),
                ] {
                    for operation in operations {
                        assert!(
                            host_questions.contains(&(
                                case.clone(),
                                variant.to_string(),
                                surface.to_string(),
                                operation.to_string()
                            )),
                            "{case} / {variant} / {surface}: no actual {operation} call"
                        );
                    }
                }
            }
        }
    }
    let provenance = calls
        .iter()
        .find(|call| call.surface == "host-provenance")
        .expect("actual host setup");
    for field in [
        "sourceSha",
        "sourceRoot",
        "pythonPackage",
        "pythonNative",
        "pythonNativeSha256",
        "nodePackage",
        "nodeNative",
        "nodeNativeSha256",
    ] {
        assert!(
            provenance.result[field]
                .as_str()
                .is_some_and(|s| !s.is_empty()),
            "host provenance lacks {field}"
        );
    }
    eprintln!(
        "S4: {} originals, {} variant materializations, {} completed receipts, {} accounted surface cells",
        expected.len(),
        materializations.len(),
        calls.len(),
        cells.len()
    );
    assert!(
        missing.is_empty(),
        "S4 execution gap: {} missing cells, first {:?}",
        missing.len(),
        missing.first()
    );
    assert!(
        incomplete_truth.is_empty(),
        "S4 independent truth unfinished: {} corroboration-only receipts; first {:?}",
        incomplete_truth.len(),
        incomplete_truth.first()
    );
    assert!(
        errors.is_empty(),
        "S4 agreement/oracle failures ({}):\n{}",
        errors.len(),
        errors.join("\n")
    );
}

/// Keep specify's reviewed list-basic truth, including the nonempty control.
fn positive_control(cases: &std::path::Path, calls: &[agreement_evidence::Call]) {
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
    for variant in agreement_corpus::VARIANTS {
        for surface in ["list", "list-size"] {
            let call = calls
                .iter()
                .find(|call| {
                    call.surface == surface
                        && call
                            .context
                            .as_ref()
                            .is_some_and(|c| c["case"] == "list-basic" && c["variant"] == *variant)
                        && call.args.iter().any(|arg| arg == "--format")
                })
                .expect("list-basic control executed");
            let context = call.context.as_ref().unwrap();
            let selected = std::path::Path::new(context["path"].as_str().unwrap())
                .strip_prefix(context["saved"].as_str().unwrap())
                .unwrap();
            let explicit = call
                .args
                .iter()
                .any(|arg| Some(arg.as_str()) == context["path"].as_str());
            let selected_sites = control_sites(&expected_sites, selected, explicit);
            assert!(
                !selected_sites.is_empty(),
                "list-basic {variant} independent control cannot be empty"
            );
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
                actual, selected_sites,
                "{surface} against reviewed list-basic golden"
            );
            for (host, operation) in [
                (
                    "python",
                    if surface == "list" {
                        "list_ids"
                    } else {
                        "list_sizes"
                    },
                ),
                (
                    "node",
                    if surface == "list" {
                        "list"
                    } else {
                        "listSizes"
                    },
                ),
            ] {
                let call = calls
                    .iter()
                    .find(|call| {
                        call.surface == host
                            && call.context.as_ref().is_some_and(|context| {
                                context["case"] == "list-basic" && context["variant"] == *variant
                            })
                            && call.request["operation"] == operation
                    })
                    .expect("actual host positive control");
                assert!(call.result["failure"].is_null());
                let host_explicit = if host == "python" {
                    call.request["explicit"] == true
                } else {
                    call.request["args"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|options| options.get("root").is_some())
                };
                let host_sites = control_sites(&expected_sites, selected, host_explicit);
                assert!(
                    !host_sites.is_empty(),
                    "nonempty actual host control domain"
                );
                let actual: BTreeSet<_> = call.result["result"]["entries"]
                    .as_array()
                    .unwrap()
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
                    actual, host_sites,
                    "{host} {operation} {variant}: reviewed independent control"
                );
            }
        }
    }
}

/// §FS-list.3: implicit cwd chooses a project; only an explicit subpath narrows
/// the control's declarations. Reviewed golden bytes remain the complete set.
fn control_sites(
    sites: &BTreeSet<(String, String)>,
    selected: &std::path::Path,
    explicit: bool,
) -> BTreeSet<(String, String)> {
    sites
        .iter()
        .filter(|(_, site)| {
            !explicit
                || selected == std::path::Path::new("")
                || selected == std::path::Path::new(".")
                || std::path::Path::new(site.rsplit_once(':').unwrap().0).starts_with(selected)
        })
        .cloned()
        .collect()
}

#[test]
fn list_basic_controls_keep_implicit_and_explicit_domains() {
    let golden = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/e2e/cases/list-basic/expected.stdout"),
    )
    .unwrap();
    let sites: BTreeSet<_> = golden
        .lines()
        .map(|line| {
            let words: Vec<_> = line.split_whitespace().collect();
            (words[0].to_string(), words[1].to_string())
        })
        .collect();
    assert_eq!(sites.len(), 3);
    assert_eq!(
        control_sites(&sites, std::path::Path::new("docs/architecture"), false),
        sites
    );
    let narrowed = control_sites(
        &sites,
        std::path::Path::new("docs/architecture/AR-001-auth.md"),
        true,
    );
    assert_eq!(
        narrowed,
        BTreeSet::from([(
            "AR-001-auth".into(),
            "docs/architecture/AR-001-auth.md:1".into()
        )])
    );
}
