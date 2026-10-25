//! Applied version-specific repairs and quiet claim uncertainty for #1000.
//! §FS-declarations.checks.misplaced-declaration.unlisted-project;
//! §FS-config.3.5.16; §FS-workspace.6.1.7.5; §FS-check.3.29.2.

use super::*;

const V1: &str = "grund_config_version = 1\nproject_name = \"shop\"\n\
    [id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfile = \"requirements.md\"\n\
    [scan]\ninclude = [\".\"]\n";
const ORIGINAL: &str = "FS-email must be declared in requirements.md (single-file kind)";

fn v1_check(repo: &Fixture, json: bool, exit: i32) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_grund"));
    command.arg("check").current_dir(&repo.0);
    if json {
        command.args(["--format", "json"]);
    }
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(exit), "{output:?}");
    let caution = "grund.toml is version 1; migrate it with grund config migrate --to 2";
    if json {
        let warning: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(
            warning,
            json!({"severity": "warning", "path": null, "line": null,
            "code": "config-version-1", "message": caution, "sites": null, "authority": null})
        );
    } else {
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            format!("warning: {caution}\n")
        );
    }
    output
}

fn v1_rows(repo: &Fixture, exit: i32) -> Vec<Value> {
    String::from_utf8(v1_check(repo, true, exit).stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn v1_exclusion_repair_applies_in_text_and_json() {
    let repo = Fixture::issue();
    fs::remove_dir_all(repo.0.join("packages/billing")).unwrap();
    repo.write("grund.toml", V1);
    repo.write("other/notify/requirements.md", "# FS-other: Other notify\n");
    let text = String::from_utf8(v1_check(&repo, false, 1).stdout).unwrap();
    let rows = v1_rows(&repo, 1);
    assert_eq!(errors(&rows).len(), 2, "{rows:?}");
    let finding = rows
        .iter()
        .find(|row| {
            row["path"] == "packages/notify/requirements.md"
                && row["code"] == "misplaced-declaration"
        })
        .unwrap();
    let message = finding["message"].as_str().unwrap();
    assert!(message.starts_with(ORIGINAL), "{message}");
    assert!(message.contains("add \"packages/notify\" to [workspace] members in grund.toml"));
    let value = message
        .split("or exclude \"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    assert_eq!(value, "notify");
    assert!(
        message.contains("with [scan] exclude in grund.toml"),
        "{message}"
    );
    assert!(
        message.contains("all directories named \"notify\" at every depth"),
        "{message}"
    );
    assert_eq!(
        *finding,
        json!({"severity": "error", "path": "packages/notify/requirements.md",
        "line": 3, "code": "misplaced-declaration", "message": message,
        "sites": null, "authority": null})
    );
    assert!(
        text.lines()
            .any(|line| line == format!("packages/notify/requirements.md:3: error: {message}"))
    );

    // Apply the value actually offered, including its documented name-wide scope.
    repo.write("grund.toml", &format!("{V1}exclude = [\"{value}\"]\n"));
    v1_check(&repo, false, 0);
    let repaired = v1_rows(&repo, 0);
    assert!(errors(&repaired).is_empty(), "{repaired:?}");
    assert!(
        repaired
            .iter()
            .all(|row| !row["path"].as_str().unwrap().contains("notify/"))
    );
}

#[test]
fn v1_configured_root_requires_removal_as_well_as_exclusion() {
    let repo = Fixture::issue();
    fs::remove_dir_all(repo.0.join("packages/billing")).unwrap();
    let config = V1.replace("include = [\".\"]", "include = [\"packages/notify\"]");
    repo.write("grund.toml", &config);
    let rows = v1_rows(&repo, 1);
    let message = errors(&rows)[0]["message"].as_str().unwrap();
    assert!(
        message.contains("exclude \"notify\" with [scan] exclude"),
        "{message}"
    );
    assert!(
        message.contains("also remove configured scan roots at or beneath \"packages/notify\""),
        "{message}"
    );
    repo.write("grund.toml", &format!("{config}exclude = [\"notify\"]\n"));
    assert_eq!(errors(&v1_rows(&repo, 1)).len(), 1);
    repo.write("grund.toml", &format!("{V1}exclude = [\"notify\"]\n"));
    assert!(errors(&v1_rows(&repo, 0)).is_empty());
}

fn under_ancestor(workspace: &str) -> (Fixture, Fixture) {
    let outer = Fixture::issue();
    let project = Fixture(outer.0.join("project"));
    fs::create_dir(&project.0).unwrap();
    for path in [".git", "grund.toml", "requirements.md", "packages"] {
        fs::rename(outer.0.join(path), project.0.join(path)).unwrap();
    }
    fs::remove_dir_all(project.0.join("packages/billing")).unwrap();
    project.write(
        "grund.toml",
        &format!("grund_config_version = 2\nproject_name = \"shop\"\n{SINGLE_FILE}"),
    );
    outer.write(
        "grund.toml",
        &format!("grund_config_version = 2\nproject_name = \"outer\"\n[workspace]\n{workspace}\n"),
    );
    (outer, project)
}

fn assert_quiet_placement_context(workspace: &str) {
    let (_outer, repo) = under_ancestor(workspace);
    // §FS-check.3.29.2: an ancestor claim left undecidable, by member text that
    // cannot be obtained or a named claim that cannot expand, keeps both consumers quiet.
    for nested_workspace in [false, true] {
        repo.write("packages/notify/requirements.md", DECLARATION);
        if nested_workspace {
            repo.write(
                "packages/notify/grund.toml",
                &format!(
                    "grund_config_version = 2\nproject_name = \"notify\"\n{SINGLE_FILE}\n\
                 [workspace]\nmembers = []\n"
                ),
            );
        }
        let rows = repo.rows(1);
        assert_eq!(
            rows.len(),
            3,
            "only the placement error and two unused warnings: {rows:?}"
        );
        assert_eq!(
            errors(&rows),
            [&json!({"severity": "error",
            "path": "packages/notify/requirements.md", "line": 3,
            "code": "misplaced-declaration", "message": ORIGINAL,
            "sites": null, "authority": null})]
        );
        let text = String::from_utf8(repo.check(false, 1).stdout).unwrap();
        let text_errors: Vec<_> = text
            .lines()
            .filter(|line| line.contains(": error:"))
            .collect();
        assert_eq!(
            text_errors,
            [format!(
                "packages/notify/requirements.md:3: error: {ORIGINAL}"
            )]
        );
        assert!(!text.contains("is not read:"), "{text}");

        // The scanning surface carries neither the standalone finding nor an
        // alias-path warning.
        let listed = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("list")
            .current_dir(&repo.0)
            .output()
            .unwrap();
        assert_eq!(listed.status.code(), Some(0), "{listed:?}");
        assert_eq!(String::from_utf8(listed.stderr).unwrap(), "");

        // With the misplaced declaration removed, nothing is left to fail check.
        fs::remove_file(repo.0.join("packages/notify/requirements.md")).unwrap();
        let rows = repo.rows(0);
        assert!(errors(&rows).is_empty(), "{rows:?}");
        let text = String::from_utf8(repo.check(false, 0).stdout).unwrap();
        assert!(!text.contains(": error:"), "{text}");
    }
}

#[test]
fn unreadable_ancestor_members_keeps_placement_context_quiet() {
    assert_quiet_placement_context("members = [\"project/packages/notify\"");
}

#[test]
fn unreadable_ancestor_optional_members_keeps_placement_context_quiet() {
    assert_quiet_placement_context("members = []\noptional_members = [\"project/packages/notify\"");
}

#[test]
fn non_list_ancestor_members_keeps_placement_context_quiet() {
    assert_quiet_placement_context("members = \"not-a-list\"");
}

#[test]
fn non_list_ancestor_optional_members_keeps_placement_context_quiet() {
    assert_quiet_placement_context("members = []\noptional_members = \"not-a-list\"");
}

#[test]
fn readable_but_unexpandable_ancestor_claim_keeps_placement_context_quiet() {
    assert_quiet_placement_context("members = [\"project/packages/notify\", \"missing\"]");
}
