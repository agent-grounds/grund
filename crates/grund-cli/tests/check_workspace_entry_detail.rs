//! Black-box regressions for §FS-check.3.5.3.1 and §FS-errors.3.6.1.
//! Port grund#932's intake join/mark reproducer: make blocks current before
//! changing them, and inspect only detail before the refresh command.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new(members: &[&str]) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "grund-932-entry-detail-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(git.status.success(), "git init: {git:?}");
        let fixture = Self(dir);
        fixture.config(members);
        fixture.write(
            "requirements.md",
            "# FS-receipt: Receipt\n\nA receipt is printed.\n",
        );
        for name in members {
            fixture.member(name);
        }
        // Settle independently of #931's init ordering. No red assertion runs
        // until the original reproducer's green baseline has been established.
        for _ in 0..3 {
            fixture.init(".");
            for name in members {
                fixture.init(&format!("packages/{name}"));
            }
            if fixture.run(&["check"]).status.success() {
                return fixture;
            }
        }
        panic!(
            "fixture never became current: {:?}",
            fixture.run(&["check"])
        );
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn config(&self, members: &[&str]) {
        let members = members
            .iter()
            .map(|name| format!("\"packages/{name}\""))
            .collect::<Vec<_>>()
            .join(", ");
        self.write(
            "grund.toml",
            &format!(
                "grund_config_version = 2\nproject_name = \"shop\"\n\n\
                 [schema.kinds.FS]\nfiles = [\"requirements.md\"]\n\n\
                 [workspace]\nmembers = [{members}]\n"
            ),
        );
    }

    fn member(&self, name: &str) {
        self.write(
            &format!("packages/{name}/grund.toml"),
            &format!(
                "grund_config_version = 2\nproject_name = \"{name}\"\n\n\
                 [schema.kinds.FS]\nfiles = [\"spec.md\"]\n"
            ),
        );
        self.write(
            &format!("packages/{name}/spec.md"),
            &format!("# FS-{name}: {name}\n\nThe package.\n"),
        );
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }

    fn init(&self, dir: &str) {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("init")
            .current_dir(self.0.join(dir))
            .output()
            .unwrap();
        assert!(out.status.success(), "init in {dir}: {out:?}");
    }

    fn replace(&self, from: &str, to: &str) {
        let text = fs::read_to_string(self.0.join("AGENTS.md")).unwrap();
        assert_eq!(text.matches(from).count(), 1, "fixture edit: {from}");
        self.write("AGENTS.md", &text.replacen(from, to, 1));
    }

    fn findings(&self, section: &str) -> Vec<String> {
        let out = self.run(&["check"]);
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert_eq!(
            out.status.code(),
            Some(1),
            "check must detect drift: {stdout}"
        );
        let findings: Vec<_> = stdout
            .lines()
            .filter(|line| line.contains("stale grund init block:"))
            .map(str::to_owned)
            .collect();
        assert!(!findings.is_empty(), "no stale finding: {stdout}");
        assert!(
            findings.iter().all(|line| line.contains(&format!(
                "error: repo maintenance: stale grund init block: {section} differ from the workspace"
            ))),
            "unexpected section or severity: {findings:?}"
        );
        findings
    }

    fn root_detail(&self, section: &str) -> String {
        let findings = self.findings(section);
        assert_eq!(
            findings.len(),
            1,
            "only the edited root block: {findings:?}"
        );
        assert!(findings[0].starts_with("AGENTS.md:3: error:"));
        assert!(
            findings[0]
                .ends_with("— run `grund init` to refresh (does not affect citation validity)")
        );
        detail(&findings[0], section).to_owned()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

// Exclude both the finding's path and its owner-directory command hint.
fn detail<'a>(finding: &'a str, section: &str) -> &'a str {
    finding
        .split_once(&format!("{section} differ from the workspace"))
        .unwrap()
        .1
        .split_once("— run")
        .unwrap()
        .0
        .trim()
}

fn expected_found(expected: &str, found: &str) -> String {
    format!("(expected {expected}; found {found})")
}

const BILLING: &str = "- [`billing`](packages/billing/AGENTS.md)";
const BILLING_MARK: &str = "- [`billing`](packages/billing/) *(not yet initialized)*";
const NOTIFY: &str = "- [`notify`](packages/notify/AGENTS.md)";

#[test]
fn joined_member_detail_names_notify_in_root_and_member_findings() {
    let fixture = Fixture::new(&["billing"]);
    fixture.member("notify");
    fixture.config(&["billing", "notify"]);
    let findings = fixture.findings("workspace members");
    assert_eq!(findings.len(), 2, "root and billing only: {findings:?}");
    for (path, owner, entry) in [
        (
            "AGENTS.md",
            "",
            "- [`notify`](packages/notify/) *(not yet initialized)*",
        ),
        (
            "packages/billing/AGENTS.md",
            " in packages/billing",
            "- [`notify`](../notify/) *(not yet initialized)*",
        ),
    ] {
        let finding = findings
            .iter()
            .find(|line| line.starts_with(&format!("{path}:3: error:")))
            .unwrap();
        assert!(finding.ends_with(&format!(
            "— run `grund init`{owner} to refresh (does not affect citation validity)"
        )));
        assert_eq!(
            detail(finding, "workspace members"),
            expected_found(entry, "<missing>"),
            "joined notify must be identified in message detail: {finding}"
        );
    }
}

#[test]
fn billing_mark_detail_shows_initialized_link_and_stale_mark() {
    let fixture = Fixture::new(&["billing"]);
    assert!(fixture.0.join("packages/billing/AGENTS.md").is_file());
    fixture.replace(BILLING, BILLING_MARK);
    assert_eq!(
        fixture.root_detail("workspace members"),
        expected_found(BILLING, BILLING_MARK)
    );
}

#[test]
fn project_namespaces_detail_shows_first_changed_body_line() {
    let fixture = Fixture::new(&["billing"]);
    let original = fs::read_to_string(fixture.0.join("AGENTS.md")).unwrap();
    let expected = original
        .lines()
        .find(|line| line.starts_with("A namespace is "))
        .unwrap();
    let found = expected.replace("A namespace is", "A stale namespace was");
    fixture.replace(expected, &found);
    assert_eq!(
        fixture.root_detail("project namespaces"),
        expected_found(expected, &found)
    );
}

#[test]
fn missing_member_detail_has_missing_found_side() {
    let fixture = Fixture::new(&["billing"]);
    fixture.replace(&format!("{BILLING}\n"), "");
    assert_eq!(
        fixture.root_detail("workspace members"),
        expected_found(BILLING, "<missing>")
    );
}

#[test]
fn unexpected_member_detail_has_missing_expected_side() {
    let fixture = Fixture::new(&["billing"]);
    let ghost = "- [`ghost`](packages/ghost/) *(not yet initialized)*";
    fixture.replace(BILLING, &format!("{BILLING}\n{ghost}"));
    assert_eq!(
        fixture.root_detail("workspace members"),
        expected_found("<missing>", ghost)
    );
}

#[test]
fn multiple_changed_entries_select_first_in_rendered_order() {
    let fixture = Fixture::new(&["billing", "notify"]);
    fixture.replace(BILLING, BILLING_MARK);
    fixture.replace(
        NOTIFY,
        "- [`notify`](packages/notify/) *(not yet initialized)*",
    );
    let expected = expected_found(BILLING, BILLING_MARK);
    assert_eq!(fixture.root_detail("workspace members"), expected);
    assert_eq!(fixture.root_detail("workspace members"), expected);
}

#[test]
fn reordered_members_compare_first_positional_pair() {
    let fixture = Fixture::new(&["billing", "notify"]);
    fixture.replace(
        &format!("{BILLING}\n{NOTIFY}"),
        &format!("{NOTIFY}\n{BILLING}"),
    );
    assert_eq!(
        fixture.root_detail("workspace members"),
        expected_found(BILLING, NOTIFY)
    );
}

#[test]
fn blank_line_drift_keeps_finding_with_section_fallback() {
    let fixture = Fixture::new(&["billing"]);
    fixture.replace("### Workspace members\n", "### Workspace members\n\n");
    assert_eq!(
        fixture.root_detail("workspace members"),
        "(section text or order differs)"
    );
}
