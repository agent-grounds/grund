//! Missing-target repair guidance and its boundaries (§FS-declarations.stubs.repair).
//! These black-box tests invoke the built CLI on the issue #820 fixture.

#[path = "missing_stub_repair/support.rs"]
mod support;

#[path = "missing_stub_repair/own_file.rs"]
mod own_file;

use serde_json::Value;
use std::fs;
use support::*;

#[test]
fn missing_stub_check_text_connects_the_repair() {
    let repo = Repo::new();
    let output = repo.run(&["check"], 1);
    assert!(output.stderr.is_empty(), "{output:?}");
    let lines = utf8(&output.stdout).lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("docs/architecture/AR-cache.md:1: error: duplicate declaration of AR-cache (also declared at src/store/cache.rs:1)"));
    assert!(lines[1].starts_with(
        "docs/architecture/AR-cache.md:1: error: stub link target missing: src/cache.rs"
    ));
    located_repair(
        lines[0],
        &format!("{STUB}:1: error: {DUPLICATE}"),
        STUB,
        REAL,
    );
    site_free_connection(lines[1], &format!("{STUB}:1: error: {MISSING}"), &[REAL]);
}

#[test]
fn missing_stub_check_json_keeps_located_finding_contract() {
    let repo = Repo::new();
    let text = repo.run(&["check"], 1);
    let output = repo.run(&["check", "--format=json"], 1);
    assert!(output.stderr.is_empty(), "{output:?}");
    let findings = records(&output.stdout);
    assert_eq!(findings.len(), 2, "{findings:?}");
    for (finding, line) in findings.iter().zip(utf8(&text.stdout).lines()) {
        shape(finding);
        assert_eq!(finding["path"], STUB);
        assert_eq!(finding["line"], 1);
        assert_eq!(
            line,
            format!("{STUB}:1: error: {}", finding["message"].as_str().unwrap())
        );
    }
    assert_eq!(findings[0]["code"], "duplicate");
    assert_eq!(findings[0]["sites"], sites(&[STUB, REAL]));
    assert_eq!(findings[1]["code"], "broken-stub");
    assert_eq!(findings[1]["sites"], Value::Null);
    let message = findings[1]["message"].as_str().unwrap();
    site_free_connection(message, MISSING, &[REAL]);
    located_repair(
        findings[0]["message"].as_str().unwrap(),
        DUPLICATE,
        STUB,
        REAL,
    );
}

/// `refs` is not among them: it sets the broken stub aside and lists (§FS-refs.4.1.1).
#[test]
fn missing_stub_all_text_queries_explain_the_ambiguity() {
    let repo = Repo::new();
    let mut gaps = Vec::new();
    for args in [
        vec!["AR-cache"],
        vec!["AR-cache", "--toc"],
        vec!["AR-cache.1"],
    ] {
        let output = repo.run(&args, 1);
        assert!(output.stdout.is_empty(), "{args:?}: {output:?}");
        let message = utf8(&output.stderr);
        assert!(message.starts_with(QUERY_PREFIX), "{args:?}: {message}");
        let suffix = &message[QUERY_PREFIX.len()..];
        for required in [
            "broken stub",
            STUB,
            "src/cache.rs",
            REAL,
            "AR-cache is declared at",
            "point the stub",
        ] {
            if !suffix.contains(required) {
                gaps.push(format!(
                    "{args:?}: missing {required:?} after prefix: {message}"
                ));
            }
        }
    }
    assert!(gaps.is_empty(), "{}", gaps.join("\n"));
}

#[test]
fn missing_stub_query_json_preserves_original_sites_and_prefix() {
    let repo = Repo::new();
    // A second lexical order prevents a repair helper from assuming the first site is a stub.
    repo.reverse_sites();
    let prefix = "ambiguous ID: AR-cache (declared at asrc/cache.rs:1, zdocs/cache.md:1)";
    // §FS-refs.4.1.1: `refs` sets the broken stub aside, so only `show` refuses here.
    for args in [vec!["AR-cache.1"]] {
        let text = repo.run(&args, 1);
        let mut args = args;
        args.push("--format=json");
        let output = repo.run(&args, 1);
        assert!(output.stdout.is_empty(), "{output:?}");
        let findings = records(&output.stderr);
        assert_eq!(findings.len(), 1);
        let finding = &findings[0];
        shape(finding);
        assert_eq!(finding["code"], "ambiguous");
        assert_eq!(finding["path"], Value::Null);
        assert_eq!(finding["line"], Value::Null);
        assert_eq!(
            finding["sites"],
            sites(&["asrc/cache.rs", "zdocs/cache.md"])
        );
        let message = finding["message"].as_str().unwrap();
        assert_eq!(utf8(&text.stderr), format!("{message}\n"));
        located_repair(message, prefix, "zdocs/cache.md", "asrc/cache.rs");
    }
}

#[test]
fn missing_stub_several_real_homes_are_facts_without_a_chosen_repair() {
    let repo = Repo::new();
    repo.write("src/other/cache.rs", DECLARATION);
    let check = repo.run(&["check", "--format=json"], 1);
    let findings = records(&check.stdout);
    let broken = findings
        .iter()
        .find(|f| f["code"] == "broken-stub")
        .unwrap();
    assert_eq!(broken["sites"], Value::Null);
    site_free_connection(
        broken["message"].as_str().unwrap(),
        MISSING,
        &["src/other/cache.rs", REAL],
    );
    let duplicate = findings.iter().find(|f| f["code"] == "duplicate").unwrap();
    assert_eq!(
        duplicate["sites"],
        sites(&[STUB, "src/other/cache.rs", REAL])
    );
    let prefix = "duplicate declaration of AR-cache (also declared at src/other/cache.rs:1, src/store/cache.rs:1)";
    let message = duplicate["message"]
        .as_str()
        .unwrap()
        .strip_prefix(prefix)
        .expect("complete duplicate prefix");
    assert!(
        message.contains("src/other/cache.rs:1") && message.contains("src/store/cache.rs:1"),
        "missing available homes: {message}"
    );
    assert!(message.find("src/other/cache.rs:1") < message.find("src/store/cache.rs:1"));
    assert!(
        message.contains("broken stub")
            && message.contains(STUB)
            && message.contains("src/cache.rs"),
        "{message}"
    );
    assert!(
        !message.contains("point the stub"),
        "must not choose: {message}"
    );
    let query = repo.run(&["refs", "AR-cache", "--format=json"], 1);
    let query = records(&query.stderr);
    assert_eq!(
        query[0]["sites"],
        sites(&[STUB, "src/other/cache.rs", REAL])
    );
    let message = query[0]["message"].as_str().unwrap();
    let message = message.strip_prefix("ambiguous ID: AR-cache (declared at docs/architecture/AR-cache.md:1, src/other/cache.rs:1, src/store/cache.rs:1)").expect("complete query prefix");
    assert!(
        message.contains("broken stub") && message.contains("src/cache.rs"),
        "{message}"
    );
    assert!(
        !message.contains("point the stub"),
        "must not choose: {message}"
    );
    assert!(
        message.contains("src/other/cache.rs:1") && message.contains("src/store/cache.rs:1"),
        "missing available homes: {message}"
    );
    assert!(message.find("src/other/cache.rs:1") < message.find("src/store/cache.rs:1"));
}

#[test]
fn missing_stub_another_broken_stub_is_not_a_repair_destination() {
    let repo = Repo::new();
    repo.write(
        "docs/architecture/second.md",
        "# AR-cache: [src/also-missing.rs](src/also-missing.rs)\n",
    );
    let output = repo.run(&["check", "--format=json"], 1);
    let findings = records(&output.stdout);
    let broken = findings
        .iter()
        .filter(|f| f["code"] == "broken-stub")
        .collect::<Vec<_>>();
    assert_eq!(broken.len(), 2);
    for finding in broken {
        let message = finding["message"].as_str().unwrap();
        assert_eq!(finding["sites"], Value::Null);
        let target = if finding["path"] == STUB {
            "src/cache.rs"
        } else {
            "src/also-missing.rs"
        };
        site_free_connection(
            message,
            &format!("stub link target missing: {target}"),
            &[REAL],
        );
        assert!(!message.contains("second.md"), "{message}");
    }
    let duplicate = findings.iter().find(|f| f["code"] == "duplicate").unwrap();
    assert_eq!(
        duplicate["sites"],
        sites(&[STUB, "docs/architecture/second.md", REAL])
    );
    let suffix = duplicate["message"].as_str().unwrap().strip_prefix("duplicate declaration of AR-cache (also declared at docs/architecture/second.md:1, src/store/cache.rs:1)").unwrap();
    unique_repair(suffix, "src/store/cache.rs:1");
    assert!(
        suffix.contains("src/also-missing.rs")
            && suffix.contains("docs/architecture/second.md:1")
            && suffix.contains(STUB),
        "{suffix}"
    );
    assert!(
        !suffix.contains("AR-cache is declared at docs/architecture/second.md"),
        "{suffix}"
    );
}

#[test]
fn missing_stub_a_healthy_stub_can_represent_the_only_real_home() {
    let repo = Repo::new();
    fs::remove_file(repo.0.join(REAL)).unwrap();
    // The real home is deliberately outside the configured scan places.
    repo.write("unwalked/cache.rs", DECLARATION);
    repo.write(
        "docs/architecture/healthy.md",
        "# AR-cache: [unwalked/cache.rs](unwalked/cache.rs)\n",
    );
    let output = repo.run(&["check", "--format=json"], 1);
    let findings = records(&output.stdout);
    let duplicate = findings.iter().find(|f| f["code"] == "duplicate").unwrap();
    assert_eq!(duplicate["sites"], sites(&[STUB, "unwalked/cache.rs"]));
    let broken = findings
        .iter()
        .find(|f| f["code"] == "broken-stub")
        .unwrap();
    assert_eq!(broken["sites"], Value::Null);
    site_free_connection(
        broken["message"].as_str().unwrap(),
        MISSING,
        &["unwalked/cache.rs"],
    );
    located_repair(
        duplicate["message"].as_str().unwrap(),
        "duplicate declaration of AR-cache (also declared at unwalked/cache.rs:1)",
        STUB,
        "unwalked/cache.rs",
    );
}

#[test]
fn missing_stub_check_selection_preserves_original_structured_contract() {
    for reversed in [false, true] {
        let repo = Repo::new();
        let (stub, primary, other) = if reversed {
            repo.reverse_sites();
            ("zdocs/cache.md", "asrc/cache.rs", "zdocs/cache.md")
        } else {
            (STUB, STUB, REAL)
        };
        let all = repo.run(&["check", "--format=json"], 1);
        assert!(all.stderr.is_empty());
        let findings = records(&all.stdout);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0]["code"], "duplicate");
        assert_eq!(findings[1]["code"], "broken-stub");
        for (code, path, expected_sites, prefix) in [
            (
                "duplicate",
                primary,
                sites(&[primary, other]),
                format!("duplicate declaration of AR-cache (also declared at {other}:1)"),
            ),
            ("broken-stub", stub, Value::Null, MISSING.to_owned()),
        ] {
            let finding = findings.iter().find(|f| f["code"] == code).unwrap();
            shape(finding);
            assert_eq!(finding["path"], path);
            assert_eq!(finding["line"], 1);
            assert_eq!(finding["sites"], expected_sites);
            assert!(finding["message"].as_str().unwrap().starts_with(&prefix));
            let selected = repo.run(&["check", "--only", code, "--format=json"], 1);
            assert!(selected.stderr.is_empty());
            assert_eq!(records(&selected.stdout), vec![finding.clone()]);
            let text = repo.run(&["check", "--only", code], 1);
            assert!(text.stderr.is_empty());
            assert_eq!(
                utf8(&text.stdout),
                format!(
                    "{path}:1: error: {}\n",
                    finding["message"].as_str().unwrap()
                )
            );
        }
    }
}

#[test]
fn missing_stub_filtered_duplicate_explains_repair_with_real_home_first() {
    let repo = Repo::new();
    repo.reverse_sites();
    let output = repo.run(&["check", "--only", "duplicate", "--format=json"], 1);
    let findings = records(&output.stdout);
    assert_eq!(findings.len(), 1);
    located_repair(
        findings[0]["message"].as_str().unwrap(),
        "duplicate declaration of AR-cache (also declared at zdocs/cache.md:1)",
        "zdocs/cache.md",
        "asrc/cache.rs",
    );
}

#[test]
fn missing_stub_filtered_broken_stub_connects_companion_with_real_home_first() {
    let repo = Repo::new();
    repo.reverse_sites();
    let output = repo.run(&["check", "--only", "broken-stub", "--format=json"], 1);
    let findings = records(&output.stdout);
    assert_eq!(findings.len(), 1);
    site_free_connection(
        findings[0]["message"].as_str().unwrap(),
        MISSING,
        &["asrc/cache.rs"],
    );
}

#[test]
fn missing_stub_without_real_homes_keeps_the_ordinary_check_message() {
    let repo = Repo::new();
    fs::remove_file(repo.0.join(REAL)).unwrap();
    let output = repo.run(&["check", "--format=json"], 1);
    let findings = records(&output.stdout);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0]["message"], MISSING);
    assert_eq!(findings[0]["sites"], Value::Null);
    repo.write(
        "docs/architecture/second.md",
        "# AR-cache: [src/also-missing.rs](src/also-missing.rs)\n",
    );
    let output = repo.run(&["check", "--format=json"], 1);
    for finding in records(&output.stdout)
        .iter()
        .filter(|f| f["code"] == "broken-stub")
    {
        assert!(
            !finding["message"]
                .as_str()
                .unwrap()
                .contains("point the stub")
        );
        assert_eq!(finding["sites"], Value::Null);
    }
}

#[test]
fn missing_stub_guidance_does_not_leak_to_other_verdicts() {
    for (target, contents, source_config, expected) in [
        ("src/cache.rs", DECLARATION, "", None),
        (
            "src/.cache.rs",
            DECLARATION,
            "",
            Some("stub link target lacks AR-cache: src/.cache.rs"),
        ),
        (
            "src/cache.zz",
            DECLARATION,
            "",
            Some("is not scanned: extension zz is not in [scan] extensions"),
        ),
        (
            "src/cache.rs",
            "pub struct Other;\n",
            "",
            Some("stub link target lacks AR-cache: src/cache.rs"),
        ),
        (
            "src/old/cache.rs",
            DECLARATION,
            "\n[schema.sources]\nexclude = [\"src/old/**\"]\n",
            Some("is not scanned:"),
        ),
    ] {
        let repo = Repo::new();
        repo.write("grund.toml", &format!("{CONFIG}{source_config}"));
        repo.write(STUB, &format!("# AR-cache: [{target}]({target})\n"));
        repo.write(target, contents);
        let output = repo.run(&["check", "--format=json"], 1);
        let findings = records(&output.stdout);
        for finding in &findings {
            let message = finding["message"].as_str().unwrap();
            assert!(
                !message.contains("point the stub") && !message.contains("broken stub"),
                "{finding}"
            );
        }
        let broken = findings.iter().find(|f| f["code"] == "broken-stub");
        if let Some(expected) = expected {
            let finding = broken.expect("non-missing broken verdict");
            assert!(
                finding["message"].as_str().unwrap().contains(expected),
                "{finding}"
            );
            assert_eq!(finding["sites"], Value::Null);
        } else {
            assert!(broken.is_none(), "healthy pair: {findings:?}");
        }
        let query = repo.run(&["AR-cache"], 1);
        let text = utf8(&query.stderr);
        assert!(
            text.starts_with("ambiguous ID: AR-cache"),
            "{target}: {text}"
        );
        assert!(
            !text.contains("point the stub") && !text.contains("broken stub"),
            "{target}: {text}"
        );
    }
}
