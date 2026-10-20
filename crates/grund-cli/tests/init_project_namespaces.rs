//! Workspace-only entrypoint guidance and its mechanical migration (§FS-init.2.3.4.16).
//! These are the plan diagnostic's standalone/root/member assertions through the CLI.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

#[path = "support/init_fixture.rs"]
mod init_fixture;
use init_fixture::{manifest_dir, run_grund, workdir};

// §FS-init.2.3.4.16: independent bytes captured at 4115eeab9a before extraction.
const BEFORE: &str =
    include_str!("../../grund-core/src/writers/fixtures/project-namespaces-v17.txt");

fn target(name: &str, config: Option<&str>) -> PathBuf {
    let root = workdir(name);
    if let Some(config) = config {
        fs::write(root.join("grund.toml"), config).unwrap();
    }
    root
}

fn init(root: &Path, flags: &[&str]) -> Output {
    let mut args = vec!["init", root.to_str().unwrap()];
    args.extend_from_slice(flags);
    run_grund(&args, manifest_dir())
}

fn success(output: Output) {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_sections(root: &Path, file: &str, namespaces: bool, members: bool, marker: &str) {
    let text = fs::read_to_string(root.join(file)).unwrap();
    assert_eq!(
        text.contains("### Workspace members"),
        members,
        "{file}: member list"
    );
    assert_eq!(
        text.contains("### Project namespaces"),
        namespaces,
        "{file}: standalone guidance still present or workspace guidance missing"
    );
    if namespaces {
        let start = text.find("### Project namespaces").unwrap();
        let end = start + text[start..].find("\n\n### ").unwrap();
        assert_eq!(
            &text[start..end],
            BEFORE.trim_end_matches('\n').replace("{MARKER}", marker)
        );
    }
}

/// §FS-init.2.3.4.16, §FS-init-fixtures.6.3: fresh standalone omission, then re-init/check.
#[test]
fn project_namespaces_fresh_standalone_omits_both_sections_and_stays_current() {
    let root = target("project_namespaces_fresh_standalone", None);
    success(init(&root, &[]));
    assert_sections(&root, "AGENTS.md", false, false, "§");
    let first = fs::read(root.join("AGENTS.md")).unwrap();
    success(init(&root, &[]));
    success(init(&root, &["--check"]));
    assert_eq!(fs::read(root.join("AGENTS.md")).unwrap(), first);
}

/// §FS-init.2.3.4.16: the workspace-root and claimed-member controls from the diagnostic.
#[test]
fn project_namespaces_workspace_root_and_claimed_member_preserve_bytes() {
    let root = target(
        "project_namespaces_workspace_controls",
        Some("project_name = \"root\"\n[workspace]\nmembers = [\"apps/api\"]\n"),
    );
    let api = root.join("apps/api");
    fs::create_dir_all(&api).unwrap();
    fs::write(api.join("grund.toml"), "project_name = \"api\"\n").unwrap();
    for path in [&api, &root] {
        success(init(path, &[]));
        assert_sections(path, "AGENTS.md", true, true, "§");
    }
    // Initialize both first, since the root/member link changes when AGENTS.md appears.
    for path in [&api, &root] {
        success(init(path, &[]));
        success(init(path, &["--check"]));
    }
}

/// §FS-init.2.3.4.16: expansion errors and duplicate aliases retain boundaries alone.
#[test]
fn project_namespaces_failed_expansion_and_duplicate_alias_controls() {
    for (name, members) in [
        ("missing", "[\"missing\"]"),
        ("duplicate", "[\"apps/api\", \"services/api\"]"),
    ] {
        let root = target(
            &format!("project_namespaces_suppressed_{name}"),
            Some(&format!(
                "project_name = \"root\"\n[workspace]\nmembers = {members}\n"
            )),
        );
        fs::create_dir_all(root.join("apps/api")).unwrap();
        fs::create_dir_all(root.join("services/api")).unwrap();
        success(init(&root, &[]));
        assert_sections(&root, "AGENTS.md", true, false, "§");
        success(init(&root, &["--check"]));
    }
}

fn marker_companions(workspace: bool) {
    let mut config = "project_name = \"root\"\n[reference]\nmarker = \"@\"\n".to_string();
    if workspace {
        config.push_str("[workspace]\nmembers = [\"api\"]\n");
    }
    let root = target(
        &format!("project_namespaces_companions_{workspace}"),
        Some(&config),
    );
    fs::create_dir_all(root.join("api")).unwrap();
    for file in ["AGENTS.md", "CLAUDE.md", "GEMINI.md"] {
        fs::write(root.join(file), format!("# {file} notes\n")).unwrap();
    }
    success(init(&root, &[]));
    for file in ["AGENTS.md", "CLAUDE.md", "GEMINI.md"] {
        assert_sections(&root, file, workspace, workspace, "@");
        assert!(
            fs::read_to_string(root.join(file))
                .unwrap()
                .starts_with(&format!("# {file} notes\n"))
        );
    }
    success(init(&root, &["--check"]));
    success(init(&root, &["--force"]));
    for file in ["AGENTS.md", "CLAUDE.md", "GEMINI.md"] {
        assert_sections(&root, file, workspace, workspace, "@");
    }
}

/// §FS-init.2.3.4.16: standalone companions omit the section with a configured marker.
#[test]
fn project_namespaces_standalone_marker_and_companions_share_omission() {
    marker_companions(false);
}

/// §FS-init.2.3.4.16: workspace companions preserve the section and configured marker.
#[test]
fn project_namespaces_workspace_marker_and_companions_preserve_bytes() {
    marker_companions(true);
}

/// §FS-init.2.3.4.16: explicit companion-only selection omits standalone guidance.
#[test]
fn project_namespaces_explicit_companion_only_standalone_omits_guidance() {
    let root = target("project_namespaces_companion_only", None);
    success(init(&root, &["--claude", "--template=none"]));
    assert!(!root.join("AGENTS.md").exists());
    assert_sections(&root, "CLAUDE.md", false, false, "§");
    success(init(&root, &["--claude", "--check"]));
}

/// §FS-init.2.3.4.16: preview measures the same omission, without writing.
#[test]
fn project_namespaces_standalone_preview_does_not_restore_guidance() {
    let root = target("project_namespaces_preview", None);
    success(init(&root, &[]));
    let path = root.join("AGENTS.md");
    let first = fs::read_to_string(&path).unwrap();
    // The baseline binary already emits the old section; install it explicitly
    // after the fix so the preview still has the same stale standalone input.
    let stale = if first.contains("### Project namespaces") {
        first.clone()
    } else {
        first.replacen(
            "### Declarations and citations",
            &format!("{}\n\n### Declarations and citations", BEFORE.trim_end()),
            1,
        )
    };
    fs::write(&path, &stale).unwrap();
    let preview = init(&root, &["--dry-run"]);
    success(preview.clone());
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        stale,
        "preview wrote the file"
    );
    assert!(
        String::from_utf8_lossy(&preview.stderr).contains("would-update AGENTS.md"),
        "preview still considers standalone Project namespaces current"
    );
}

fn version(text: &str) -> u32 {
    let prefix = "## Grounding with grund (v";
    let line = text.lines().find(|line| line.starts_with(prefix)).unwrap();
    line.strip_prefix(prefix)
        .unwrap()
        .trim_end_matches(')')
        .parse()
        .unwrap()
}

fn migration(rules: bool) {
    let mut config = "project_name = \"demo\"\n".to_string();
    if rules {
        config.push_str("[[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\nrules = true\n");
    }
    let root = target(
        &format!("project_namespaces_migration_{rules}"),
        Some(&config),
    );
    success(init(&root, &[]));
    let path = root.join("AGENTS.md");
    let generated = fs::read_to_string(&path).unwrap();
    let prior = if rules { 18 } else { 17 };
    let old = generated.replace(
        &format!("(v{})", version(&generated)),
        &format!("(v{prior})"),
    );
    let old = if old.contains("### Project namespaces") {
        old
    } else {
        old.replacen(
            "### Declarations and citations",
            &format!("{}\n\n### Declarations and citations", BEFORE.trim_end()),
            1,
        )
    };
    let old = format!("# User notes before\n\n{old}\nUser notes after.\n");
    fs::write(&path, &old).unwrap();
    let check = run_grund(&["check", root.to_str().unwrap()], manifest_dir());
    let finding = String::from_utf8_lossy(&check.stdout);
    assert_eq!(
        check.status.code(),
        Some(1),
        "old block must be reported outdated"
    );
    assert!(
        finding.contains(&format!("outdated grund init block v{prior}")),
        "missing migration finding: {finding}"
    );
    assert_eq!(init(&root, &["--check"]).status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        old,
        "check changed legacy input"
    );
    success(init(&root, &[]));
    let new = fs::read_to_string(&path).unwrap();
    assert!(version(&new) > 18, "migration reused a prior version");
    assert!(new.starts_with("# User notes before\n"));
    assert!(new.ends_with("User notes after.\n"));
    assert_sections(&root, "AGENTS.md", false, false, "§");
    success(init(&root, &["--check"]));
    let checked = run_grund(&["check", root.to_str().unwrap()], manifest_dir());
    assert!(!String::from_utf8_lossy(&checked.stdout).contains("outdated grund init block"));
}

/// §FS-init.2.3.7.1: intentional legacy v17 input and one-command migration.
#[test]
fn project_namespaces_base_migration_keeps_unmanaged_content() {
    migration(false);
}

/// §FS-init.2.3.7.1: intentional legacy v18 rule input and one-command migration.
#[test]
fn project_namespaces_rule_migration_keeps_unmanaged_content() {
    migration(true);
}

/// §FS-init.5.3, §FS-init.5.4: the `members` section the CLI prints is the shared
/// skill's `references/members.md`, and it teaches the split.
#[test]
fn project_namespaces_cli_setup_instructions_teach_member_split() {
    let output = run_grund(
        &["agent-setup-instructions", "--section", "members"],
        manifest_dir(),
    );
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    for skill in ["skills", "crates/grund-core/assets/skills"] {
        let members = format!("{skill}/grund-init/references/members.md");
        assert_eq!(
            text,
            fs::read_to_string(manifest_dir().join(&members)).unwrap(),
            "{members}"
        );
    }
    assert!(
        text.contains("independently checked"),
        "setup guidance omits when to split off a member"
    );
    assert!(
        text.contains("alias/<ID>"),
        "setup guidance omits qualified citations"
    );
}
