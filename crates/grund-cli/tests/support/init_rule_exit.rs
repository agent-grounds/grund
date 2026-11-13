//! §FS-init.4.1: rule findings and pending paths independently earn exit 1.
//! Reuse #277's fixture and goldens; #1368 corrects their caller-facing promise.

use super::case_runner::{CaseKind, assert_every_case_passed, copy_dir, run_case};
use super::repo_root;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "fixture_cleanup.rs"]
mod fixture_cleanup;

const CASE: &str = "init-workspace-pinned-rule-member";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "grund-1368-init-rule-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create isolated workspace");
        let fixture = Self(root);
        copy_dir(
            &repo_root().join("tests/e2e/cases").join(CASE).join("repo"),
            &fixture.0,
        );
        fs::create_dir(fixture.0.join(".git")).unwrap();
        // The issue's member is also a repository, as in a submodule checkout.
        fs::create_dir(fixture.0.join("member/.git")).unwrap();
        fixture
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fixture_cleanup::remove_fixture_tree(&self.0, "remove init rule fixture");
    }
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run grund")
}

fn assert_rule_report(output: &Output, events: &str) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let expected = fs::read_to_string(
        repo_root()
            .join("tests/e2e/cases")
            .join(CASE)
            .join("expected.stdout"),
    )
    .unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let paths = stderr.split("\nnext:\n").next().unwrap();
    assert_eq!(paths, events, "{stderr}");
    if events.lines().all(|line| line.starts_with("exists ")) {
        assert_eq!(
            stderr, events,
            "current paths have no pending event or next block"
        );
    }
}

/// §FS-init.4.1: help must name both causes and the existing stream/event split.
#[test]
fn help_explains_both_exit_one_causes_and_reports() {
    let output = run(&repo_root(), &["init", "--help"]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let help = String::from_utf8(output.stdout).unwrap();
    let required = [
        "1 rule findings or --check pending paths",
        "Rule findings: stdout",
        "path events: stderr",
        "would-write",
        "would-append",
        "would-update",
        "can coexist",
        "workspace root",
    ];
    let missing: Vec<_> = required
        .iter()
        .filter(|text| !help.contains(**text))
        .collect();
    assert!(
        missing.is_empty() && !help.contains("1 --check only"),
        "init help must explain both exit-1 causes and their reports; missing {missing:?}\nactual help:\n{help}"
    );
}

/// §FS-init.4.1, §FS-init.4.3, §FS-rules.4.1.2: writing/current and rule failure
/// coexist, including current and pending previews; no exit normalization.
#[test]
fn member_rule_findings_survive_written_current_and_pending_paths() {
    let fixture = Fixture::new();
    let member = fixture.0.join("member");
    let agents = member.join("AGENTS.md");
    fs::remove_file(&agents).unwrap();

    assert_rule_report(
        &run(&member, &["init"]),
        "wrote AGENTS.md\nexists grund.toml\n",
    );
    let current = fs::read(&agents).unwrap();
    assert!(
        String::from_utf8_lossy(&current)
            .contains("The operations chapter of each SEG must cite at least one workshop/OP.")
    );
    for args in [
        vec!["init"],
        vec!["init", "--check"],
        vec!["init", "--dry-run"],
    ] {
        assert_rule_report(
            &run(&member, &args),
            "exists AGENTS.md\nexists grund.toml\n",
        );
        assert_eq!(fs::read(&agents).unwrap(), current);
    }

    // The issue drifts the rendered member list through a sibling description.
    let sibling_config = fixture.0.join("workshop/grund.toml");
    let config = fs::read_to_string(&sibling_config).unwrap();
    fs::write(
        &sibling_config,
        format!("project_description = \"The welding workshop.\"\n{config}"),
    )
    .unwrap();
    for flag in ["--check", "--dry-run"] {
        assert_rule_report(
            &run(&member, &["init", flag]),
            "would-update AGENTS.md\nexists grund.toml\n",
        );
        assert_eq!(fs::read(&agents).unwrap(), current, "{flag} must not write");
    }
    assert_rule_report(
        &run(&member, &["init"]),
        "updated AGENTS.md\nexists grund.toml\n",
    );
    let updated = fs::read(&agents).unwrap();
    assert_ne!(updated, current);
    assert!(String::from_utf8_lossy(&updated).contains("The welding workshop."));
    assert_rule_report(
        &run(&member, &["init", "--check"]),
        "exists AGENTS.md\nexists grund.toml\n",
    );
    assert_eq!(fs::read(&agents).unwrap(), updated);

    // A sibling without this rule still succeeds, and its current gate is green.
    let sibling = fixture.0.join("workshop");
    for args in [vec!["init"], vec!["init", "--check"]] {
        let output = run(&sibling, &args);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
    }
}

/// §FS-rules.4, §FS-rules.4.1.2: retain the existing invalid/unverifiable mixed
/// refusal and workspace-root controls through the ordinary golden harness.
#[test]
fn existing_rule_write_and_scope_controls() {
    let root = repo_root();
    let outcomes = [
        "init-workspace-pinned-rule-member",
        "init-check-workspace-pinned-rule-member",
        "init-workspace-pinned-rule-unresolved-subject",
        "init-workspace-pinned-rule-invalid-sibling",
        "init-workspace-pinned-rule-root",
        "check-workspace-pinned-rule-block-current",
    ]
    .iter()
    .map(|name| {
        run_case(
            &root,
            &root.join("tests/e2e/cases").join(name),
            CaseKind::E2e,
        )
    })
    .collect::<Vec<_>>();
    assert_every_case_passed("init rule exit controls", &outcomes);
}
