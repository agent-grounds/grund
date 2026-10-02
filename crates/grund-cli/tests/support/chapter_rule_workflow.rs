//! §FS-init.2.3.5.10: init, fmt, and the chapter-rule drift checker agree on bytes.
//! Ports grund.34's isolated git reproducer into the CLI e2e harness. Unlike a
//! single-command golden, this workflow checks both gates on the same tree.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SENTENCE: &str = "Each FS must have exactly one Terms chapter.";
const LINK: &str = "[§RULE-terms](docs/rules/RULE-terms.md#rule-terms-each-fs-must-have-exactly-one-terms-chapter)";

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str, cross_refs: bool) -> Self {
        let root = std::env::temp_dir().join(format!("grund-376-{}-{name}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove old fixture");
        }
        for folder in ["docs/goals", "docs/fs", "docs/rules"] {
            fs::create_dir_all(root.join(folder)).expect("create fixture folder");
        }
        fs::write(
            root.join("grund.toml"),
            format!(
                r#"grund_config_version = 1
project_name = "grund376"
[reference]
strict = true
[id]
format = "{{kind}}-{{slug}}"
named_sections = true
[[kinds]]
kind = "GOAL"
folder = "docs/goals"
index = false
[[kinds]]
kind = "FS"
folder = "docs/fs"
index = false
[[kinds]]
kind = "RULE"
folder = "docs/rules"
index = false
rules = true
[fmt.cross_refs]
enabled = {cross_refs}
anchor_format = "github"
[scan]
include = ["docs", "AGENTS.md"]
"#
            ),
        )
        .expect("write fixture config");
        fs::write(
            root.join("docs/goals/GOAL-repro.md"),
            "# GOAL-repro: Every spec names its terms\n\nThe project wants its terms written down.\n",
        )
        .expect("write goal");
        fs::write(
            root.join("docs/fs/FS-one.md"),
            "# FS-one: A subject with its terms\n\n## terms: Terms\n\nThis chapter names the terms for \u{a7}GOAL-repro.\n",
        )
        .expect("write spec");
        fs::write(
            root.join("docs/rules/RULE-terms.md"),
            format!("# RULE-terms: {SENTENCE}\n\nThe rule protects \u{a7}GOAL-repro.\n"),
        )
        .expect("write rule");
        let git = Command::new("git")
            .args(["init", "-q"])
            .arg(&root)
            .output()
            .expect("spawn git init");
        assert!(git.status.success(), "git init: {}", report(&git));
        Self(root)
    }

    fn agents(&self) -> String {
        fs::read_to_string(self.0.join("AGENTS.md")).expect("read generated AGENTS.md")
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("spawn grund")
    }

    fn success(&self, args: &[&str]) {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "grund {args:?}: {}",
            report(&output)
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn report(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}stderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn bullet(agents: &str) -> &str {
    agents
        .lines()
        .find(|line| line.starts_with("- Each FS "))
        .expect("init must render the chapter-rule bullet")
}

fn both_gates(fixture: &Fixture, initial: &str) {
    let fmt = fixture.run(&["fmt", "--check"]);
    let init = fixture.run(&["init", "--check"]);
    let check = fixture.run(&["check", "--full"]);
    assert!(
        fmt.status.success() && init.status.success() && check.status.success(),
        "chapter-rule gates must accept the same tree\nafter init: {}\nafter fmt: {}\nfmt --check: {}\ninit --check: {}\ncheck --full: {}",
        bullet(initial),
        bullet(&fixture.agents()),
        report(&fmt),
        report(&init),
        report(&check)
    );
}

fn stable_cycle(fixture: &Fixture, cross_refs: bool) -> String {
    fixture.success(&["init"]);
    let initial = fixture.agents();
    assert!(
        !initial.contains("grund:fmt off"),
        "no suppression workaround"
    );
    fixture.success(&["fmt", "--write"]);
    let formatted = fixture.agents();
    let expected = if cross_refs {
        format!("- {SENTENCE} {LINK}")
    } else {
        format!("- {SENTENCE} §RULE-terms")
    };
    assert_eq!(bullet(&formatted), expected, "the rule citation stays live");
    fixture.success(&["fmt", "--write"]);
    assert_eq!(
        fixture.agents(),
        formatted,
        "second fmt pass changes no bytes"
    );
    both_gates(fixture, &initial);
    assert_eq!(
        bullet(&initial),
        expected,
        "init itself emits the canonical form"
    );

    fixture.success(&["init"]);
    assert_eq!(
        fixture.agents(),
        formatted,
        "repeated init changes no bytes"
    );
    fixture.success(&["fmt", "--write"]);
    assert_eq!(
        fixture.agents(),
        formatted,
        "repeated cycle changes no bytes"
    );
    both_gates(fixture, &initial);
    formatted
}

#[test]
fn chapter_rule_cross_refs_enabled_has_one_stable_init_fmt_tree() {
    let fixture = Fixture::new("enabled", true);
    stable_cycle(&fixture, true);
}

#[test]
fn chapter_rule_cross_refs_disabled_keeps_the_bare_citation_stable() {
    let fixture = Fixture::new("disabled", false);
    stable_cycle(&fixture, false);
}

/// §FS-check.3.5.4: agreement cannot be obtained by hiding real rule drift.
#[test]
fn chapter_rule_content_drift_is_detected_after_a_stable_linked_cycle() {
    let fixture = Fixture::new("drift", true);
    assert_content_drift(&fixture, true);
}

#[test]
fn chapter_rule_content_drift_with_cross_refs_disabled_is_still_detected() {
    let fixture = Fixture::new("drift-disabled", false);
    assert_content_drift(&fixture, false);
}

fn assert_content_drift(fixture: &Fixture, cross_refs: bool) {
    let before = stable_cycle(fixture, cross_refs);
    let rule = fixture.0.join("docs/rules/RULE-terms.md");
    replace(&rule, "must have exactly one", "should have exactly one");

    let init = fixture.run(&["init", "--check"]);
    assert_eq!(init.status.code(), Some(1), "{}", report(&init));
    assert!(String::from_utf8_lossy(&init.stderr).contains("would-update AGENTS.md"));
    let check = fixture.run(&["check", "--full", "--format", "json"]);
    assert_eq!(check.status.code(), Some(1), "{}", report(&check));
    let findings: Vec<serde_json::Value> = String::from_utf8_lossy(&check.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("check JSON diagnostic"))
        .collect();
    assert!(
        findings.iter().any(|error| {
            error["code"] == "agents-init"
                && error["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("chapter rules differ"))
        }),
        "real rule drift must carry agents-init: {}",
        report(&check)
    );
    assert_eq!(
        fixture.agents(),
        before,
        "checks never write the stale block"
    );
    fixture.success(&["init"]);
    let refreshed = fixture.agents();
    assert_ne!(refreshed, before, "refresh must expose the changed rule");
    let expected = if cross_refs {
        "- Each FS should have exactly one Terms chapter. [§RULE-terms](docs/rules/RULE-terms.md#rule-terms-each-fs-should-have-exactly-one-terms-chapter)"
    } else {
        "- Each FS should have exactly one Terms chapter. §RULE-terms"
    };
    assert_eq!(bullet(&refreshed), expected);
    fixture.success(&["fmt", "--write"]);
    assert_eq!(fixture.agents(), refreshed, "refreshed rule is fmt-stable");
    both_gates(fixture, &refreshed);
}

fn replace(path: &Path, from: &str, to: &str) {
    let before = fs::read_to_string(path).expect("read fixture file");
    assert!(before.contains(from), "fixture mutation must change bytes");
    fs::write(path, before.replace(from, to)).expect("write changed rule");
}
