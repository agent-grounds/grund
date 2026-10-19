//! Numeric subject identity stays independent of authored report spelling
//! (§FS-config.3.2.8, §FS-rules.2, §FS-rules.8).

#[path = "support/numeric_spelling.rs"]
#[allow(dead_code)]
mod support;

use support::Repo;

fn fixture(version: u8, per_kind: bool, cited: bool) -> Repo {
    let repo = Repo::new(version, per_kind);
    if version == 1 {
        repo.write(
            "grund.toml",
            &repo
                .read("grund.toml")
                .replace("[id]\n", "[id]\nnamed_sections = true\n"),
        );
    }
    let body = if cited {
        "Cites §ADR-0002."
    } else {
        "No citations."
    };
    repo.write(
        "docs/adr/ADR-0001.md",
        &format!(
            "# ADR-0001: Decision\n\nLead.\n\n## terms: Terms\n\n{body}\n\n\
         ### terms.detail: Detail\n\nNested body.\n\n## other: Other\n\n\
         ### other.terms: Other terms\n\nDifferent chapter.\n"
        ),
    );
    repo.write("docs/notes.md", "");
    repo
}

#[test]
fn equivalent_numeric_subjects_select_declarations_and_whole_chapter_paths() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = fixture(version, per_kind, false);
            for id in ["ADR-1", "ADR-001", "ADR-0001"] {
                for section in [
                    None,
                    Some("terms"),
                    Some("terms.detail"),
                    Some("other.terms"),
                ] {
                    let selector = section.map_or_else(|| id.into(), |s| format!("{id}.{s}"));
                    let show = repo.json(&["show", &selector, "--format=json"], 0);
                    for size in [false, true] {
                        let mut args = vec!["list", "--selector", &selector, "--format=json"];
                        if size {
                            args.push("--size=words");
                        }
                        let rows = repo.json(&args, 0);
                        assert_eq!(rows.len(), 1, "v{version}/{per_kind} {args:?}: {rows:?}");
                        assert_eq!(rows[0]["id"], "ADR-0001");
                        assert_eq!(rows[0]["section"].as_str(), section);
                        assert_eq!(rows[0]["line"], show[0]["line"]);
                        if size {
                            assert!(rows[0]["lead_words"].as_u64().is_some());
                            assert!(rows[0]["full_words"].as_u64().is_some());
                        }
                        let text_args = args
                            .iter()
                            .copied()
                            .filter(|arg| *arg != "--format=json")
                            .collect::<Vec<_>>();
                        let text = repo.run(&text_args, 0);
                        let reported =
                            section.map_or_else(|| "ADR-0001".into(), |s| format!("ADR-0001.{s}"));
                        assert!(text.contains(&reported), "{args:?}: {text}");
                    }
                }
            }
        }
    }
}

#[test]
fn equivalent_numeric_chapter_rules_keep_success_and_missing_citation_verdicts() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            for cited in [false, true] {
                let repo = fixture(version, per_kind, cited);
                for id in ["ADR-1", "ADR-001", "ADR-0001"] {
                    let rule = format!("{id}.terms must cite at least one ADR.");
                    let rows = repo.json(
                        &[
                            "check",
                            "--rule",
                            &rule,
                            "--only",
                            "invalid-rule",
                            "--only",
                            "missing-citation",
                            "--format=json",
                        ],
                        if cited { 0 } else { 1 },
                    );
                    assert!(
                        !rows.iter().any(|row| row["code"] == "invalid-rule"),
                        "{rows:?}"
                    );
                    if cited {
                        assert!(rows.is_empty(), "{rows:?}");
                        assert_eq!(
                            repo.run(
                                &[
                                    "check",
                                    "--rule",
                                    &rule,
                                    "--only",
                                    "invalid-rule",
                                    "--only",
                                    "missing-citation"
                                ],
                                0
                            ),
                            "success\n"
                        );
                    } else {
                        let missing = rows
                            .iter()
                            .filter(|row| row["code"] == "missing-citation")
                            .collect::<Vec<_>>();
                        assert_eq!(missing.len(), 1, "{rows:?}");
                        assert_eq!(missing[0]["line"], 5);
                        assert!(
                            missing[0]["message"]
                                .as_str()
                                .unwrap()
                                .contains("ADR-0001.terms")
                        );
                    }
                }
            }
        }
    }
}
