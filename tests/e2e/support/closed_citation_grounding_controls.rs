//! Selection and quiet boundaries of §FS-check.checks.closed-citation-grounding.
use super::support::{CODE, Tree, config, row, streams, warnings};

/// Quiet controls run alongside a positive control, so the test cannot pass without the fix.
#[test]
#[ignore = "red until #1460 warns only for closed grounded homes; implement removes this"]
fn closed_citation_grounding_quiet_controls() {
    let mut cases = Vec::new();
    for version in [1, 2] {
        let closed = row(version);
        let table = if version == 1 {
            "citations"
        } else {
            "rules.citations"
        };
        cases.push((format!("off-{version}"), config(version, "", ""), false));
        cases.push((
            format!("citable-{version}"),
            closed.replace("citable = false", "citable = true"),
            false,
        ));
        let unscanned = config(
            version,
            "",
            if version == 1 {
                "[reference]\nrequire_grounding = true\n"
            } else {
                "[rules.citations.grounding]\nmust = \"file\"\n"
            },
        )
        .replace("citable = false", "citable = false\nscan = false")
        .replace(&format!("[{table}.ce]"), &format!("[{table}]"));
        cases.push((format!("unscanned-{version}"), unscanned, false));
        for exception in ["may", "should-not", "warn-not"] {
            if version == 1 && exception == "warn-not" {
                continue;
            }
            cases.push((
                format!("exception-{version}-{exception}"),
                closed.replace(
                    "default = \"must-not\"",
                    &format!("default = \"must-not\"\n{exception} = [\"FS\"]"),
                ),
                false,
            ));
        }
        for default in ["should-not", "warn-not"] {
            if version == 1 && default == "warn-not" {
                continue;
            }
            cases.push((
                format!("default-{version}-{default}"),
                closed.replace(
                    "default = \"must-not\"",
                    &format!("default = \"{default}\""),
                ),
                false,
            ));
        }
        let complement = if version == 1 {
            closed
                .replace("folder = \"ce\"\n", "")
                .replace("kind = \"ce\"", "kind = \"code\"")
                .replace(&format!("[{table}.ce]"), &format!("[{table}.code]"))
        } else {
            closed
                .replace(
                    "[schema.kinds.ce]\nfolders = [\"ce\"]",
                    "[schema.kinds.code]\nfolders = [\"src\"]",
                )
                .replace("rules.citations.ce", "rules.citations.code")
        };
        cases.push((format!("complement-{version}"), complement, false));
        cases.push((
            format!("grouping-{version}"),
            format!("{closed}\n[workspace]\ninclude_root = false\nmembers = [\"member\"]\n"),
            false,
        ));
    }
    cases.push((
        "v1-false".into(),
        config(
            1,
            "require_grounding = false\n",
            "[reference]\nrequire_grounding = true\n",
        ),
        true,
    ));
    for flag in [false, true] {
        cases.push((
            format!("v2-empty-{flag}"),
            config(
                2,
                "[rules.citations.ce.grounding]\n",
                "[rules.citations.grounding]\nmust = \"file\"\n",
            ),
            flag,
        ));
    }
    cases.push((
        "v2-empty-flag-alone".into(),
        config(2, "[rules.citations.ce.grounding]\n", ""),
        true,
    ));
    cases.push((
        "may-rung".into(),
        config(2, "[rules.citations.ce.grounding]\nmay = \"file\"\n", ""),
        false,
    ));
    for (name, text, flag) in cases {
        let tree = Tree::new(&name, &text);
        // Port D1/D2's controls: the complement is closed and grounded, yet inline
        // declaration remains an escape (§FS-check.3.6.2).
        if name.starts_with("complement-") {
            tree.write("src/A.java", "class A {}\n");
            let uncited = tree.run(&["check"]);
            assert!(
                String::from_utf8_lossy(&uncited.stdout)
                    .contains("src/A.java:1: error: ungrounded source file"),
                "{}",
                streams(&uncited)
            );
            tree.write("src/A.java", "// \u{a7}FS-login\nclass A {}\n");
            let cited = tree.run(&["check"]);
            assert!(
                String::from_utf8_lossy(&cited.stdout)
                    .contains("code must not cite FS (citation direction)"),
                "{}",
                streams(&cited)
            );
            tree.write("src/A.java", "class A {}\n");
            tree.write(
                "src/B.java",
                "/** FS-inline: An inline declaration. */\nclass B {}\n",
            );
            let inline = tree.run(&["check"]);
            assert!(
                !String::from_utf8_lossy(&inline.stdout)
                    .lines()
                    .any(|line| line.starts_with("src/B.java:") && line.contains(": error:")),
                "{}",
                streams(&inline)
            );
        }
        // E2: the exception is usable; F2: grounding is really off, while the
        // citation prohibition remains (§FS-check.checks.closed-citation-grounding).
        if name.starts_with("exception-") {
            let uncited = tree.run(&["check"]);
            assert!(
                String::from_utf8_lossy(&uncited.stdout)
                    .contains("ungrounded file in kind home ce/"),
                "{}",
                streams(&uncited)
            );
            tree.write("ce/A.java", "// \u{a7}FS-login\nclass A {}\n");
            let cited = tree.run(&["check"]);
            assert_eq!(cited.status.code(), Some(0), "{}", streams(&cited));
            tree.write("ce/A.java", "class A {}\n");
        }
        if name.starts_with("off-") {
            let uncited = tree.run(&["check"]);
            assert_eq!(uncited.status.code(), Some(0), "{}", streams(&uncited));
            tree.write("ce/A.java", "// \u{a7}FS-login\nclass A {}\n");
            let cited = tree.run(&["check"]);
            assert!(
                String::from_utf8_lossy(&cited.stdout)
                    .contains("must not cite FS (citation direction)"),
                "{}",
                streams(&cited)
            );
            tree.write("ce/A.java", "class A {}\n");
        }
        let args = if flag {
            vec!["check", "--require-grounding"]
        } else {
            vec!["check"]
        };
        for args in [vec!["config", "validate"], vec!["config", "show"], args] {
            let output = tree.run(&args);
            assert_ne!(
                output.status.code(),
                Some(2),
                "{name}: {}",
                streams(&output)
            );
            assert!(
                warnings(&output).is_empty(),
                "quiet {name} {args:?}: {}",
                streams(&output)
            );
        }
    }
    let positive = Tree::new("quiet-positive", &row(2)).run(&["config", "validate"]);
    assert_eq!(
        warnings(&positive).len(),
        1,
        "positive control missing {CODE}: {}",
        streams(&positive)
    );
}

#[test]
#[ignore = "red until #1460 is selectable and queries remain quiet; implement removes this"]
fn closed_citation_grounding_selection_and_silent_commands() {
    let tree = Tree::new("selection", &row(2));
    for args in [
        vec!["list"],
        vec!["refs", "FS-login"],
        vec!["cover"],
        vec!["fmt", "--check"],
        vec!["FS-login"],
        vec!["init"],
    ] {
        let output = tree.run(&args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{args:?}: {}",
            streams(&output)
        );
        let expected_stderr = if args[0] == "refs" {
            "note: FS-login is declared but nothing cites it; `grund list --unused` lists every such declaration\n"
        } else if args[0] == "init" {
            "exists AGENTS.md\nexists ce/AGENTS.md\nexists grund.toml\n"
        } else {
            ""
        };
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            expected_stderr,
            "{args:?}: {}",
            streams(&output)
        );
    }
    let baseline = include_str!("../cases/closed-citation-grounding-v2-check/expected.stdout");
    for args in [
        vec!["check", "--only", "ungrounded"],
        vec![
            "check",
            "--rule",
            "Each FS must have exactly one Terms chapter.",
            "--only-rule",
        ],
        vec!["check", "--ignore", CODE],
    ] {
        let output = tree.run(&args);
        // An unknown selector or other launch refusal is not silence.
        assert_eq!(
            output.status.code(),
            Some(1),
            "{args:?}: {}",
            streams(&output)
        );
        assert!(
            warnings(&output).is_empty(),
            "{args:?}: {}",
            streams(&output)
        );
        assert!(output.stderr.is_empty(), "{args:?}: {}", streams(&output));
        let stdout = String::from_utf8_lossy(&output.stdout);
        if args.contains(&"--ignore") {
            assert_eq!(stdout, baseline, "ignore restores today's bytes");
        } else if args.contains(&"--only") {
            assert_eq!(stdout, format!("{}\n", baseline.lines().next().unwrap()));
        } else {
            assert!(
                !stdout.contains("ungrounded"),
                "only-rule keeps rule findings: {stdout}"
            );
        }
    }
    let selected = tree.run(&["check", "--only", CODE, "--format=json"]);
    assert_eq!(selected.status.code(), Some(0), "{}", streams(&selected));
    let stderr = String::from_utf8_lossy(&selected.stderr);
    assert!(!stderr.contains("selected-check-off"), "{stderr}");
    assert_eq!(
        warnings(&selected).len(),
        1,
        "missing {CODE}: {}",
        streams(&selected)
    );
}

#[test]
#[ignore = "red until #1460 adds the catalog explain answer; implement removes this"]
fn closed_citation_grounding_explain_both_versions() {
    let mut missing = Vec::new();
    for version in [1, 2] {
        let tree = Tree::new(&format!("explain-{version}"), &row(version));
        let output = tree.run(&["config", "explain", CODE]);
        if output.status.code() != Some(0) {
            missing.push(format!(
                "v{version}: missing explain answer: {}",
                streams(&output)
            ));
            continue;
        }
        assert!(output.stderr.is_empty(), "{}", streams(&output));
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains(CODE) && text.contains("warning"), "{text}");
        if version == 1 {
            assert!(text.contains("require_grounding = false"), "{text}");
        } else {
            assert!(
                text.contains("empty [rules.citations.") && text.contains("write an empty"),
                "{text}"
            );
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}
