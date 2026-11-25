//! Black-box regression for #1492: correct scoped maintenance checks need help
//! that teaches their exceptions and recovery (§FS-cli.3.3).

#[path = "support/fixture_cleanup.rs"]
mod fixture_cleanup;

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture(PathBuf);

impl Fixture {
    fn new(version: u8) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1492-help-scope-{}-v{version}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create a fresh fixture root");
        let fixture = Self(root);
        fs::create_dir_all(fixture.0.join("docs/specs")).unwrap();
        fs::create_dir_all(fixture.0.join("docs/architecture")).unwrap();
        let config = if version == 2 {
            "grund_config_version = 2\nproject_name = \"demo\"\n\n\
             [schema.kinds.FS]\nfolders = [\"docs/specs\"]\nindex = false\n"
        } else {
            "project_name = \"demo\"\n\n[id]\nformat = \"{kind}-{slug}\"\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs/specs\"\nindex = false\n"
        };
        fs::write(fixture.0.join("grund.toml"), config).unwrap();
        fs::write(
            fixture.0.join("docs/specs/FS-login.md"),
            "# FS-login: Login\n\nUsers log in.\n",
        )
        .unwrap();
        fs::write(
            fixture.0.join("docs/architecture/ci.md"),
            "# CI\n\nThe gate holds \u{a7}FS-login.\n",
        )
        .unwrap();
        fixture.git(&["init", "-q"]);
        fixture
    }

    fn git(&self, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run git");
        assert!(output.status.success(), "git {args:?}: {output:?}");
    }

    fn grund(&self, args: &[&str], expected: i32) -> Output {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run the built CLI");
        eprintln!(
            "$ grund {} -> exit {}\n{}{}",
            args.join(" "),
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.status.code(), Some(expected), "grund {args:?}");
        output
    }

    fn stale_block(&self) {
        let path = self.0.join("AGENTS.md");
        let mut text = fs::read_to_string(&path).unwrap();
        let start = text.find("(block v").expect("init wrote a managed block");
        let end = start + text[start..].find(')').unwrap() + 1;
        assert_ne!(&text[start..end], "(block v39)", "fresh block is current");
        text.replace_range(start..end, "(block v39)");
        fs::write(path, text).unwrap();
        self.git(&["add", "-A"]);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fixture_cleanup::remove_fixture_tree(&self.0, "remove help-scope fixture");
    }
}

/// §FS-check.1.3.6.1: port the issue's fresh/stale/ignored controls for both
/// config versions, before judging the guidance rather than changing the verdict.
fn check_maintenance_controls(version: u8) {
    eprintln!("== V{version}: clean selected file, genuinely stale root block");
    let fixture = Fixture::new(version);
    fixture.grund(&["init"], 0);
    fixture.git(&["add", "-A"]);
    // A v1 config warning withholds the text-mode success line.
    let clean = if version == 1 { "" } else { "success\n" };
    assert_eq!(fixture.grund(&["check"], 0).stdout, clean.as_bytes());
    fixture.stale_block();
    for args in [vec!["check"], vec!["check", "docs/architecture/ci.md"]] {
        let output = fixture.grund(&args, 1);
        let report = String::from_utf8(output.stdout).unwrap();
        assert!(report.contains("AGENTS.md:"), "{report}");
        assert!(
            report.contains("repo maintenance: outdated grund init block v39"),
            "{report}"
        );
    }
    assert_eq!(
        fixture
            .grund(
                &[
                    "check",
                    "docs/architecture/ci.md",
                    "--ignore",
                    "agents-init"
                ],
                0,
            )
            .stdout,
        clean.as_bytes()
    );
}

/// §FS-cli.3.3: neither the code catalog nor a PATH-less example explains why
/// an explicit-PATH run reports the correctly retained root maintenance error.
#[test]
#[ignore = "red until #1492 check help teaches PATH exceptions and scoped ignore; implement removes this"]
fn check_help_explains_path_scope_exceptions_and_recovery() {
    for version in [1, 2] {
        check_maintenance_controls(version);
    }
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["check", "--help"])
        .output()
        .expect("read actual check help");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let help = String::from_utf8(output.stdout).unwrap();
    let (_, after_usage) = help.split_once("Usage:").expect("help usage");
    let (scope, _) = after_usage.split_once("\nOptions:").expect("scope prose");
    let scope = scope.split_whitespace().collect::<Vec<_>>().join(" ");
    let examples = help.split_once("Examples:").expect("help examples").1;
    let mut missing = Vec::new();
    if !(scope.contains("agents-init")
        && scope.contains("root")
        && (scope.contains("agent-entrypoint") || scope.contains("AGENTS.md")))
    {
        missing.push("root agent-entrypoint maintenance exception (agents-init)");
    }
    // §FS-check.1.3.6.2: teach a whole multi-site finding anchored outside PATH.
    if !((scope.contains("multi-site") || scope.contains("several sites"))
        && scope.contains("duplicate")
        && scope.contains("outside")
        && (scope.contains("twin") || scope.contains("anchor")))
    {
        missing.push("whole multi-site exception, including a duplicate anchored outside PATH");
    }
    if !scope.contains("--ignore agents-init") {
        missing.push("--ignore agents-init recovery beside PATH scope");
    }
    if !examples.lines().any(|line| {
        let command = line.split('#').next().unwrap_or("");
        command.contains("grund check ")
            && command.contains("--ignore agents-init")
            && command.split_whitespace().any(|word| {
                word.contains('/') || word.eq_ignore_ascii_case("<path>") || word == "PATH"
            })
    }) {
        missing.push("PATH-bearing --ignore agents-init example");
    }
    if scope.contains("reports exactly that file or directory") {
        missing
            .push("qualified PATH reporting promise (still says 'exactly that file or directory')");
    }
    assert!(
        missing.is_empty(),
        "check help scope omits:\n- {}\nActual scope:\n{scope}",
        missing.join("\n- ")
    );
}
