//! The configless-member readers agree with check (§FS-config.4.2,
//! §FS-config-explain.2, §FS-schema.2, §FS-scan.projects.member).

use crate::fixture::{BANNER, FORMAT, Tree, run};

fn reader_runs(tree: &Tree, command: &[&str]) -> Vec<crate::fixture::Run> {
    let mut runs = Vec::new();
    for scope in tree.scopes() {
        runs.push(run(&scope, command));
        let relative = scope.strip_prefix(&tree.0).unwrap().to_str().unwrap();
        let mut explicit = command.to_vec();
        // Explain's positional target comes first; --path also supports the index.
        if command.starts_with(&["config", "explain"]) {
            explicit.push("--path");
        }
        explicit.push(relative);
        runs.push(run(&tree.0, &explicit));
    }
    runs
}

#[test]
#[ignore = "red until #1446 show selects the configless member; implement removes this"]
fn configless_member_show_uses_canonical_defaults() {
    let tree = Tree::workspace();
    let mut failures = Vec::new();
    for out in reader_runs(&tree, &["config", "show"]) {
        out.expect(
            0,
            &[
                "grund_config_version = 1",
                "[id]",
                &format!("format = {FORMAT:?}"),
            ],
            &mut failures,
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
#[ignore = "red until #1446 explain stops at the configless member; implement removes this"]
fn configless_member_explain_answers_without_a_file() {
    let tree = Tree::workspace();
    let mut failures = Vec::new();
    for command in [vec!["config", "explain"], vec!["config", "explain", "id"]] {
        for out in reader_runs(&tree, &command) {
            out.expect(0, &[BANNER, "Version 2:", "Version 1:"], &mut failures);
            if let (Some(v2), Some(v1)) =
                (out.stdout.find("Version 2:"), out.stdout.find("Version 1:"))
            {
                assert!(v2 < v1, "{}", out.stdout);
            }
            if command.len() == 3 {
                out.expect(
                    0,
                    &["version 2 has no table `[id]`", "[id]", "format"],
                    &mut failures,
                );
                let v1 = out.stdout.split("Version 1:").nth(1).unwrap_or_default();
                if !v1.contains("[id]") || !v1.contains("format") {
                    failures.push(format!(
                        "id must be answered as a v1 table:\n{}",
                        out.stdout
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
#[ignore = "red until #1446 schema identifies the configless member; implement removes this"]
fn configless_member_schema_names_member_without_config() {
    let tree = Tree::workspace();
    let mut failures = Vec::new();
    for out in reader_runs(&tree, &["schema"]) {
        out.expect(
            0,
            &["nocfg · defaults, no grund.toml · v1\n"],
            &mut failures,
        );
        if out.stdout.lines().next() != Some("nocfg · defaults, no grund.toml · v1") {
            failures.push(format!(
                "member schema header was {:?}",
                out.stdout.lines().next()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn configless_member_check_keeps_v1_format_from_member_and_root() {
    let tree = Tree::workspace();
    for cwd in [&tree.0, &tree.scopes()[0]] {
        let out = run(cwd, &["check"]);
        assert_eq!(out.code, Some(1), "{}", out.stderr);
        let findings = format!("{}{}", out.stdout, out.stderr);
        assert!(
            findings.contains(&format!("does not match [id] format = {FORMAT:?}")),
            "{findings}"
        );
        let file = if cwd == &tree.0 {
            "apps/nocfg/docs/functional-spec/login.md:1: error:"
        } else {
            "docs/functional-spec/login.md:1: error:"
        };
        assert!(findings.contains(file), "{findings}");
    }
}

#[test]
fn reader_selection_preserves_root_configured_members_and_standalone() {
    let tree = Tree::workspace();
    // A sibling's unreadable version must not force readers to expand the workspace.
    tree.write("apps/broken/grund.toml", "grund_config_version = 999\n");
    let mut failures = Vec::new();
    run(&tree.0, &["config", "show"]).expect(0, &["project_name = \"root\""], &mut failures);
    run(&tree.0, &["config", "explain"]).expect(
        0,
        &["grund.toml is read as version 2."],
        &mut failures,
    );
    run(&tree.0, &["schema"]).expect(0, &["root · grund.toml · v2"], &mut failures);
    for form in ["grund.toml", ".agents/grund.toml"] {
        let name = if form.starts_with('.') {
            "hidden"
        } else {
            "bare"
        };
        tree.write(
            &format!("apps/{name}/{form}"),
            &format!("grund_config_version = 2\nproject_name = {name:?}\n"),
        );
        let cwd = tree.0.join(format!("apps/{name}"));
        run(&cwd, &["config", "show"]).expect(
            0,
            &[&format!("project_name = {name:?}")],
            &mut failures,
        );
        run(&cwd, &["config", "explain"]).expect(
            0,
            &[&format!("{form} is read as version 2.")],
            &mut failures,
        );
        run(&cwd, &["schema"]).expect(0, &[&format!("{name} · {form} · v2")], &mut failures);
    }
    let standalone = Tree::empty();
    run(&standalone.0, &["config", "show"]).expect(
        0,
        &["grund_config_version = 1", FORMAT],
        &mut failures,
    );
    run(&standalone.0, &["config", "explain"]).expect(
        0,
        &[BANNER, "Version 2:", "Version 1:"],
        &mut failures,
    );
    run(&standalone.0, &["schema"]).expect(0, &["defaults, no grund.toml · v1"], &mut failures);
    standalone.write(
        ".agents/grund.toml",
        "grund_config_version = 2\nproject_name = \"standalone\"\n",
    );
    run(&standalone.0, &["config", "show"]).expect(
        0,
        &["project_name = \"standalone\""],
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn explain_member_reads_only_version_even_when_settings_are_refused() {
    let tree = Tree::workspace();
    tree.write("apps/broken/grund.toml", "grund_config_version = 999\n");
    let mut failures = Vec::new();
    for (name, form, version) in [
        ("bare", "grund.toml", 1),
        ("hidden", ".agents/grund.toml", 2),
    ] {
        tree.write(
            &format!("apps/{name}/{form}"),
            &format!("grund_config_version = {version}\n[unknown_table]\nnope = true\n"),
        );
        let cwd = tree.0.join(format!("apps/{name}"));
        let refused = run(&cwd, &["config", "show"]);
        assert_eq!(refused.code, Some(2), "{}", refused.stderr);
        assert!(
            refused.stderr.contains("unknown_table"),
            "{}",
            refused.stderr
        );
        run(&cwd, &["config", "explain"]).expect(
            0,
            &[&format!("{form} is read as version {version}.")],
            &mut failures,
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
