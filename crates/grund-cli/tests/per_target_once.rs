//! Black-box contracts for §FS-rules.3.3 and §FS-rule-refusals.sentences.
//! Ports the four spelling assertions recorded in #835's intake reproduction;
//! the shared physical-site fixture also pins the approved surface matrix.

#[path = "support/per_target_once.rs"]
mod support;

use support::{Repo, assert_clean, check, text};

fn sentence(modality: &str, count: &str) -> String {
    format!("SYS-overview.components {modality} cite each AR {count}.")
}

fn refusal(modality: &str, count: &str, reason: &str) -> String {
    format!("{reason}; accepted form: {}", sentence(modality, count))
}

/// §FS-rules.3.3, §FS-rules.7.3, §FS-rules.7.6: zero/one sites pass,
/// two sites to the SAME target fail; must errors and opt-in should suggestions
/// keep their exact output, exits, locations and authority on both surfaces.
#[test]
fn per_target_once_accepted_word_judges_physical_sites_on_both_surfaces() {
    let mut wrong = Vec::new();
    for configured in [false, true] {
        for modality in ["must", "should"] {
            for sites in 0..=2 {
                let repo = Repo::new(sites);
                let title = sentence(modality, "at most once");
                if configured {
                    repo.rule(&title);
                }
                // Configured rules are tested both with ordinary check and
                // beside a trial sentence; --only-rule drops only the former.
                for only_rule in [false, true] {
                    if configured && only_rule {
                        continue;
                    }
                    for suggestions in [false, true] {
                        for json in [false, true] {
                            let mut args = vec!["check"];
                            if !configured {
                                args.extend(["--rule", &title]);
                            }
                            if only_rule {
                                args.push("--only-rule");
                            }
                            if suggestions {
                                args.push("--suggestions");
                            }
                            if json {
                                args.extend(["--format", "json"]);
                            }
                            let shown = sites == 2 && (modality == "must" || suggestions);
                            let exit = i32::from(shown && modality == "must");
                            let authority = if configured { "RULE-once" } else { "--rule" };
                            let message = format!(
                                "SYS-overview.components cites AR-target 2 times; {authority} requires at most 1; add or remove those citations until the count fits"
                            );
                            let expected = if !shown {
                                if json {
                                    String::new()
                                } else {
                                    "success\n".into()
                                }
                            } else if json {
                                let mut row = serde_json::json!({
                                    "severity": if modality == "must" { "error" } else { "warning" },
                                    "path": "docs/sys.md", "line": 3,
                                    "code": "citation-cardinality", "message": message,
                                    "sites": null, "authority": [authority]
                                });
                                if modality == "should" {
                                    row.as_object_mut().unwrap().remove("severity");
                                    row["channel"] = "suggestion".into();
                                }
                                // JSON member order is not the contract.
                                let output = repo.run(&args);
                                let actual = text(&output.stdout);
                                let parsed =
                                    serde_json::from_str::<serde_json::Value>(&actual).ok();
                                if output.status.code() != Some(exit)
                                    || !output.stderr.is_empty()
                                    || parsed != Some(row.clone())
                                {
                                    wrong.push(format!("{configured}/{modality}/{sites}/{args:?}: {output:?}; expected exit {exit}, {row}"));
                                }
                                continue;
                            } else {
                                format!(
                                    "docs/sys.md:3: {}: {message}\n",
                                    if modality == "must" {
                                        "error"
                                    } else {
                                        "suggestion"
                                    }
                                )
                            };
                            if let Some(error) = check(
                                repo.run(&args),
                                exit,
                                &expected,
                                "",
                                &format!("{configured}/{modality}/{sites}/{args:?}"),
                            ) {
                                wrong.push(error);
                            }
                        }
                    }
                }
            }
        }
    }
    assert_clean(wrong);
}

/// §FS-rule-refusals.sentences: numeric one is refused before a trial scan;
/// a configured invalid-rule is an error even for should, located at its title.
/// §FS-rule-refusals.sentences.rewrite.by-refusal preserves the typed modality.
#[test]
fn per_target_once_numeric_refusal_has_exact_channels_exits_and_locations() {
    let mut wrong = Vec::new();
    for modality in ["must", "should"] {
        for suggestions in [false, true] {
            for configured in [false, true] {
                let repo = Repo::new(0);
                let title = sentence(modality, "at most 1 times");
                let message = refusal(
                    modality,
                    "at most once",
                    "numeric \"at most 1 times\" is not canonical",
                );
                if configured {
                    repo.rule(&title);
                }
                let mut args = vec!["check"];
                if !configured {
                    args.extend(["--rule", &title]);
                }
                if suggestions {
                    args.push("--suggestions");
                }
                let stdout = if configured {
                    format!(
                        "docs/rules/once.md:1: error: RULE-once is not a valid rule: {message}\n"
                    )
                } else {
                    String::new()
                };
                let stderr = if configured {
                    String::new()
                } else {
                    format!("error: {message}\n")
                };
                let exit = if configured { 1 } else { 2 };
                for only_rule in [false, true] {
                    if configured && only_rule {
                        continue;
                    }
                    let mut flags = args.clone();
                    if only_rule {
                        flags.push("--only-rule");
                    }
                    if let Some(error) = check(
                        repo.run(&flags),
                        exit,
                        &stdout,
                        &stderr,
                        &format!("{modality}/{configured}/{flags:?}"),
                    ) {
                        wrong.push(error);
                    }
                }
                args.extend(["--format", "json"]);
                let output = repo.run(&args);
                if configured {
                    let actual = text(&output.stdout);
                    let row = serde_json::from_str::<serde_json::Value>(&actual).ok();
                    let expected = serde_json::json!({
                        "severity": "error", "path": "docs/rules/once.md", "line": 1,
                        "code": "invalid-rule", "message": format!("RULE-once is not a valid rule: {message}"),
                        "sites": null, "authority": ["RULE-once"]
                    });
                    if output.status.code() != Some(1)
                        || !output.stderr.is_empty()
                        || row != Some(expected.clone())
                    {
                        wrong.push(format!(
                            "configured invalid-rule JSON: {output:?}; expected {expected}"
                        ));
                    }
                } else if let Some(error) = check(output, 2, "", &stderr, "trial invalid JSON") {
                    wrong.push(error);
                }
            }
        }
    }
    assert_clean(wrong);
}

/// §FS-rule-refusals.sentences.rewrite.by-refusal and
/// §FS-rule-refusals.sentences.rewrite.reparsed: malformed numeric one reaches
/// once after all repair passes, and the exact form works when pasted back.
#[test]
fn per_target_once_recovery_offers_typed_reparsable_forms() {
    let repo = Repo::new(0);
    let mut wrong = Vec::new();
    for modality in ["must", "should"] {
        for (count, reason) in [
            (
                "at most 1 times",
                "numeric \"at most 1 times\" is not canonical",
            ),
            ("at most 1", "per-target counts must end in \"times\""),
            ("at most 1 time", "per-target counts must end in \"times\""),
            (
                "at most 01 times",
                "count must be a canonical positive base-10 integer",
            ),
        ] {
            let title = sentence(modality, count);
            let expected = format!("error: {}\n", refusal(modality, "at most once", reason));
            if let Some(error) = check(
                repo.run(&["check", "--rule", &title, "--only-rule"]),
                2,
                "",
                &expected,
                &title,
            ) {
                wrong.push(error);
            }
            let form = sentence(modality, "at most once");
            if let Some(error) = check(
                repo.run(&["check", "--rule", &form, "--only-rule"]),
                0,
                "success\n",
                "",
                &form,
            ) {
                wrong.push(error);
            }
        }
    }
    assert_clean(wrong);
}

/// §FS-rules.7.6: --only-rule suppresses configured enforcement while
/// --suggestions controls valid recommendations, independently of that switch.
#[test]
fn per_target_once_only_rule_keeps_suggestion_selection_and_skips_configured_rules() {
    let repo = Repo::new(2);
    repo.rule(&sentence("must", "at most once"));
    let trial = sentence("should", "at most once");
    let both = repo.run(&[
        "check",
        "--rule",
        &trial,
        "--format",
        "json",
        "--suggestions",
    ]);
    assert_eq!(both.status.code(), Some(1), "{both:?}");
    assert!(both.stderr.is_empty(), "{both:?}");
    let rows: Vec<serde_json::Value> = text(&both.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(rows.iter().any(|r| r["authority"] == serde_json::json!(["RULE-once"]) && r["severity"] == "error"));
    assert!(
        rows.iter().any(
            |r| r["authority"] == serde_json::json!(["--rule"]) && r["channel"] == "suggestion"
        )
    );
    for suggestions in [false, true] {
        let mut args = vec!["check", "--rule", &trial, "--only-rule", "--format", "json"];
        if suggestions {
            args.push("--suggestions");
        }
        let output = repo.run(&args);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let rows: Vec<serde_json::Value> = text(&output.stdout)
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(rows.len(), usize::from(suggestions), "{rows:?}");
        if suggestions {
            assert_eq!(rows[0]["channel"], "suggestion");
            assert_eq!(rows[0]["authority"], serde_json::json!(["--rule"]));
            assert_eq!(rows[0]["code"], "citation-cardinality");
        }
    }
}

/// §FS-rules.3, §FS-rule-refusals.sentences.count-range: the intake's
/// at-least pair, exact once, counts above one and other upper-one families
/// retain their existing grammar and evaluation. These controls pass today.
#[test]
fn per_target_once_existing_count_controls_keep_their_verdicts() {
    let repo = Repo::new(0);
    for bound in ["at least", "exactly"] {
        let valid = sentence("must", &format!("{bound} once"));
        let output = repo.run(&["check", "--rule", &valid, "--only-rule"]);
        assert_eq!(output.status.code(), Some(1), "{valid}: {output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        let required = if bound == "exactly" {
            "exactly once"
        } else {
            "at least one"
        };
        assert_eq!(
            text(&output.stdout),
            format!(
                "docs/sys.md:3: error: SYS-overview.components cites AR-target 0 times; --rule requires {required}; add or remove those citations until the count fits\n"
            )
        );
        let invalid = sentence("must", &format!("{bound} 1 times"));
        let expected = format!(
            "error: numeric \"{bound} 1 times\" is not canonical; accepted form: {valid}\n"
        );
        assert_clean(
            check(
                repo.run(&["check", "--rule", &invalid, "--only-rule"]),
                2,
                "",
                &expected,
                &invalid,
            )
            .into_iter()
            .collect(),
        );
    }
    for (title, exit) in [
        (sentence("must", "at most 2 times"), 0),
        (sentence("must", "at least 2 times"), 1),
        (sentence("must", "exactly 2 times"), 1),
        ("Each SYS must have at most 1 Components chapter.".into(), 0),
        ("Each SYS must cite at most 1 AR.".into(), 0),
        ("Each AR must be cited by at most 1 SYS.".into(), 0),
        (
            sentence("must", &format!("at most {} times", usize::MAX)),
            0,
        ),
    ] {
        let output = repo.run(&["check", "--rule", &title, "--only-rule"]);
        assert_eq!(output.status.code(), Some(exit), "{title}: {output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
    for (sites, bound, exit) in [
        (2, "at most", 0),
        (3, "at most", 1),
        (2, "at least", 0),
        (2, "exactly", 0),
    ] {
        let repo = Repo::new(sites);
        let title = sentence("must", &format!("{bound} 2 times"));
        let output = repo.run(&["check", "--rule", &title, "--only-rule"]);
        assert_eq!(output.status.code(), Some(exit), "{title}: {output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
    let title = sentence("must", &format!("at most {} times", usize::MAX as u128 + 1));
    let expected = format!(
        "error: count must be a canonical positive base-10 integer; the count is above the largest supported count, {}\n",
        usize::MAX
    );
    assert_clean(
        check(
            repo.run(&["check", "--rule", &title]),
            2,
            "",
            &expected,
            &title,
        )
        .into_iter()
        .collect(),
    );
}
