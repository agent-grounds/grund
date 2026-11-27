//! #1460's C1/C2 and controls: §FS-check.checks.closed-citation-grounding.
//! Run with TMPDIR outside any enclosing git tree or grund config.

#[path = "support/closed_citation_grounding.rs"]
mod support;
use support::{CODE, Tree, config, message, row, streams, warnings};

/// All four reported surfaces, their exits, null JSON sites and unchanged file verdicts.
#[test]
fn closed_citation_grounding_c1_c2_surfaces() {
    let mut missing = Vec::new();
    for version in [1, 2] {
        let tree = Tree::new(&format!("surfaces-{version}"), &row(version));
        let expected = message(version, "row", "must = \"file\"", false);
        for (args, exit) in [
            (vec!["config", "validate"], 0),
            (vec!["config", "show"], 0),
            (vec!["check"], 1),
            (vec!["check", "--format=json"], 1),
        ] {
            let output = tree.run(&args);
            assert_eq!(
                output.status.code(),
                Some(exit),
                "{args:?}: {}",
                streams(&output)
            );
            if args.contains(&"--format=json") {
                let objects: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stderr)
                    .lines()
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect();
                let found: Vec<_> = objects.iter().filter(|d| d["code"] == CODE).collect();
                if found.len() != 1 {
                    missing.push(format!(
                        "v{version} {args:?}: missing {CODE}: {}",
                        streams(&output)
                    ));
                    continue;
                }
                let d = found[0];
                assert_eq!(d["message"], expected);
                assert_eq!(d["severity"], "warning");
                for key in ["path", "line", "sites"] {
                    assert_eq!(d.get(key), Some(&serde_json::Value::Null));
                }
            } else {
                let want = vec![format!("warning: {expected}")];
                if warnings(&output) != want {
                    missing.push(format!(
                        "v{version} {args:?}: expected {want:?}: {}",
                        streams(&output)
                    ));
                }
            }
        }
        let uncited = tree.run(&["check"]);
        assert!(
            String::from_utf8_lossy(&uncited.stdout).contains("ungrounded file in kind home ce/")
        );
        tree.write("ce/A.java", "// \u{a7}FS-login\nclass A {}\n");
        let cited = tree.run(&["check"]);
        assert_eq!(cited.status.code(), Some(1));
        assert!(
            String::from_utf8_lossy(&cited.stdout)
                .contains("must not cite FS (citation direction)")
        );
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

/// All provenance spellings, inherited citation defaults and strongest rung, without suggestions.
#[test]
fn closed_citation_grounding_provenance_and_rungs() {
    let mut missing = Vec::new();
    for version in [1, 2] {
        for source in ["row", "project", "flag"] {
            for inherited in [false, true] {
                let mut text = match source {
                    "row" => row(version),
                    "project" => config(
                        version,
                        "",
                        if version == 1 {
                            "[reference]\nrequire_grounding = true\n"
                        } else {
                            "[rules.citations.grounding]\nmust = \"file\"\n"
                        },
                    ),
                    _ => config(version, "", ""),
                };
                if inherited {
                    let table = if version == 1 {
                        "citations"
                    } else {
                        "rules.citations"
                    };
                    text = text.replace(
                        &format!("[{table}.ce]\ndefault"),
                        &format!("[{table}]\ndefault"),
                    );
                }
                let tree = Tree::new(&format!("source-{version}-{source}-{inherited}"), &text);
                let args = if source == "flag" {
                    vec!["check", "--require-grounding"]
                } else {
                    vec!["config", "validate"]
                };
                let output = tree.run(&args);
                let expected = vec![format!(
                    "warning: {}",
                    message(version, source, "must = \"file\"", inherited)
                )];
                if warnings(&output) != expected {
                    missing.push(format!(
                        "v{version} {source} inherited={inherited}: expected {expected:?}: {}",
                        streams(&output)
                    ));
                }
                if source != "flag" {
                    assert_eq!(output.status.code(), Some(0), "{}", streams(&output));
                    assert_eq!(
                        warnings(&tree.run(&["check", "--require-grounding"])),
                        warnings(&output),
                        "the flag must retain config provenance"
                    );
                }
            }
        }
    }
    for (name, ladder, rung, exit) in [
        ("warn", "warn = \"file\"", "warn = \"file\"", 0),
        ("should", "should = \"h2\"", "should = \"h2\"", 0),
        (
            "strongest",
            "must = \"file\"\nwarn = \"h2\"\nshould = \"h3\"",
            "must = \"file\"",
            1,
        ),
    ] {
        let text = config(
            2,
            &format!("[rules.citations.ce.grounding]\n{ladder}\n"),
            if name == "should" {
                "[rules.citations.grounding]\nmust = \"file\"\n"
            } else {
                ""
            },
        );
        let tree = Tree::new(name, &text);
        let output = tree.run(&["check"]);
        assert_eq!(output.status.code(), Some(exit), "{}", streams(&output));
        let expected = vec![format!("warning: {}", message(2, "row", rung, false))];
        if warnings(&output) != expected {
            missing.push(format!(
                "{name}: expected {expected:?}: {}",
                streams(&output)
            ));
        }
        assert_eq!(
            warnings(&tree.run(&["check", "--require-grounding"])),
            warnings(&output),
            "a configured soft rung keeps config provenance under the flag"
        );
    }
    for strength in ["warn", "should"] {
        let rung = format!("{strength} = \"h2\"");
        let text = config(2, "", &format!("[rules.citations.grounding]\n{rung}\n"));
        let tree = Tree::new(&format!("project-{strength}"), &text);
        let expected = vec![format!("warning: {}", message(2, "project", &rung, false))];
        for args in [
            vec!["config", "validate"],
            vec!["check", "--require-grounding"],
        ] {
            let output = tree.run(&args);
            if warnings(&output) != expected {
                missing.push(format!(
                    "project {strength} {args:?}: expected {expected:?}: {}",
                    streams(&output)
                ));
            }
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

/// An empty home still warns; workspace selection names each selected config once.
#[test]
fn closed_citation_grounding_workspace_and_empty_home() {
    let tree = Tree::new(
        "workspace",
        &format!("{}\n[workspace]\nmembers = [\"a\", \"b\"]\n", row(2)),
    );
    std::fs::remove_file(tree.0.join("ce/A.java")).unwrap();
    tree.write(
        "specification/FS-login.md",
        "# FS-login: Login\n\nAs \u{a7}FS-login says.\n",
    );
    for member in ["a", "b"] {
        tree.write(
            &format!("{member}/grund.toml"),
            &row(2).replace(
                "project_name = \"p\"",
                &format!("project_name = \"{member}\""),
            ),
        );
        tree.write(
            &format!("{member}/specification/FS-login.md"),
            "# FS-login: Login\n\nAs \u{a7}FS-login says.\n",
        );
        std::fs::create_dir_all(tree.0.join(member).join("ce")).unwrap();
    }
    for member in ["a", "b"] {
        let init = tree.run(&["init", member]);
        assert_eq!(init.status.code(), Some(0), "{}", streams(&init));
    }
    let init = tree.run(&["init"]);
    assert_eq!(init.status.code(), Some(0), "{}", streams(&init));
    let mut missing = Vec::new();
    for (args, paths) in [
        (
            vec!["config", "validate"],
            vec!["a/grund.toml", "b/grund.toml", "grund.toml"],
        ),
        (
            vec!["check"],
            vec!["a/grund.toml", "b/grund.toml", "grund.toml"],
        ),
        (vec!["config", "show"], vec!["grund.toml"]),
        (vec!["check", "a"], vec!["grund.toml"]),
        (vec!["config", "validate", "a"], vec!["grund.toml"]),
    ] {
        let output = tree.run(&args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{args:?}: {}",
            streams(&output)
        );
        if args[0] == "check" {
            assert!(
                !String::from_utf8_lossy(&output.stdout).contains("success"),
                "warning replaces success: {}",
                streams(&output)
            );
        }
        let expected: Vec<_> = paths
            .iter()
            .map(|path| {
                format!(
                    "warning: {}",
                    message(2, "row", "must = \"file\"", false).replacen("grund.toml", path, 1)
                )
            })
            .collect();
        if warnings(&output) != expected {
            missing.push(format!(
                "{args:?}: expected {expected:?}: {}",
                streams(&output)
            ));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

#[path = "support/closed_citation_grounding_controls.rs"]
mod controls;
