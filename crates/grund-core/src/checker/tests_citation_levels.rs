//! Enforcement of all five citation levels and Boolean obligations (§FS-config.3.9.1).

use std::path::PathBuf;

use super::check_findings;
use crate::config::load_config;
use crate::model::CheckReport;
use crate::scanner::scan_tree;
use crate::testing::{test_root, write};

fn check_rules(name: &str, rules: &str, body: &str) -> (PathBuf, CheckReport) {
    let root = test_root(name)
        .canonicalize()
        .expect("canonical rule fixture");
    let mut config = String::from("grund_config_version = 1\n[reference]\nstrict = true\n");
    for kind in ["FS", "GOAL", "GRUND", "AR", "DF"] {
        let file = format!("docs/{}.md", kind.to_ascii_lowercase());
        config.push_str(&format!(
            "\n[[kinds]]\nkind = \"{kind}\"\nfile = \"{file}\"\n"
        ));
        let contents = if kind == "FS" {
            format!("# FS-001-subject: Subject\n\n{body}")
        } else {
            format!("# {kind}-001-target: Target\n")
        };
        write(&root.join(file), &contents);
    }
    config.push_str(&format!("\n[scan]\ninclude = [\"docs\"]\n{rules}\n"));
    write(&root.join("grund.toml"), &config);
    let config = load_config(&root).expect("load rules");
    let (findings, errors) = scan_tree(&config, Some(&root), true).expect("scan rules");
    assert!(errors.is_empty(), "{errors:?}");
    let report = check_findings(&findings, &config);
    (root, report)
}

/// §FS-config.3.9.1.3: the level→surface mapping is fixed — `must` and
/// `must-not` gate as `grund check` errors while `should` and `should-not` stay
/// off the gating channels and surface only on demand. §FS-config.6.1: that is
/// a separate advisory channel rather than a third severity, so the frozen
/// `{error, warning}` set stays exactly two.
#[test]
fn citation_levels_keep_both_suggestions_out_of_gating_channels() {
    let (root, report) = check_rules(
        "citation_levels_all_five",
        r#"[citations]
default = "must-not"
[citations.FS]
must = ["GOAL"]
should = ["GRUND"]
may = ["FS"]
should-not = ["DF"]
must-not = ["AR"]"#,
        "See §FS-001-subject.\nSee §AR-001-target.\nSee §DF-001-target.\n",
    );
    let signature = |diagnostics: &[crate::model::Diagnostic]| {
        diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.code,
                    diagnostic.path.clone(),
                    diagnostic.line,
                    diagnostic.message.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    let path = Some(root.join("docs/fs.md"));
    assert_eq!(
        signature(&report.errors),
        [
            (
                "missing-citation",
                path.clone(),
                Some(1),
                "FS-001-subject must cite GOAL (citation direction)".into()
            ),
            (
                "forbidden-citation",
                path.clone(),
                Some(4),
                "FS must not cite AR (citation direction) — re-point the citation or downgrade it to a plain Markdown link".into()
            ),
        ]
    );
    assert_eq!(
        signature(&report.suggestions),
        [
            (
                "suggested-citation",
                path.clone(),
                Some(1),
                "FS-001-subject should cite GRUND (citation direction)".into()
            ),
            (
                "discouraged-citation",
                path,
                Some(5),
                "FS should not cite DF (citation direction)".into()
            ),
        ]
    );
    assert!(
        report
            .errors
            .iter()
            .chain(&report.warnings)
            .all(|diagnostic| {
                !matches!(
                    diagnostic.code,
                    "suggested-citation" | "discouraged-citation"
                )
            })
    );
}

#[test]
fn explicit_may_overrides_a_restrictive_default() {
    for (suffix, permission, expected) in [
        ("closed", "", vec!["forbidden-citation"]),
        ("permitted", "[citations.FS]\nmay = [\"GOAL\"]", vec![]),
    ] {
        let (_, report) = check_rules(
            &format!("citation_may_override_{suffix}"),
            &format!("[citations]\ndefault = \"must-not\"\n{permission}"),
            "See §GOAL-001-target.\n",
        );
        assert_eq!(
            report
                .errors
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            expected
        );
        assert!(report.suggestions.is_empty());
    }
}

/// §FS-config.3.9.1.1: an obligation asks whether the citing declaration
/// carries a citation to the target kind anywhere in its body, and multiple
/// array entries are conjunctive while a `|` inside one entry is satisfied by
/// any one alternative.
#[test]
fn citation_obligations_require_each_entry_but_allow_either_alternative() {
    for level in ["must", "should"] {
        for (name, targets, misses) in [
            ("conjunction", "[\"GOAL\", \"GRUND\"]", [2, 1, 1, 0]),
            ("disjunction", "[\"GOAL|GRUND\"]", [1, 0, 0, 0]),
        ] {
            for (index, body) in [
                "No citations.\n",
                "See §GOAL-001-target.\n",
                "See §GRUND-001-target.\n",
                "See §GOAL-001-target and §GRUND-001-target.\n",
            ]
            .iter()
            .enumerate()
            {
                let (_, report) = check_rules(
                    &format!("citation_{level}_{name}_{index}"),
                    &format!("[citations]\ndefault = \"may\"\n[citations.FS]\n{level} = {targets}"),
                    body,
                );
                let (expected_errors, expected_suggestions) = if level == "must" {
                    (misses[index], 0)
                } else {
                    (0, misses[index])
                };
                assert_eq!(
                    report.errors.len(),
                    expected_errors,
                    "{level} {name} {index}"
                );
                assert!(
                    report
                        .errors
                        .iter()
                        .all(|diagnostic| diagnostic.code == "missing-citation")
                );
                assert_eq!(
                    report.suggestions.len(),
                    expected_suggestions,
                    "{level} {name} {index}"
                );
                assert!(
                    report
                        .suggestions
                        .iter()
                        .all(|diagnostic| diagnostic.code == "suggested-citation")
                );
            }
        }
    }
}
