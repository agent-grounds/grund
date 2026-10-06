//! §FS-examples.5.4: external argv uses the existing executor and golden surfaces.
#![allow(dead_code)]
#[path = "support/case_runner.rs"]
mod case_runner;

use case_runner::{CaseKind, assert_every_case_passed, run_case};
use std::fs;
use std::path::PathBuf;

struct Fixture {
    root: PathBuf,
    case: PathBuf,
}

impl Fixture {
    fn new(name: &str, external: &str) -> Self {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .expect("scratch home");
        let root = PathBuf::from(home)
            .join("ag/tmp")
            .join(format!("grund-external-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let case = root.join(name);
        fs::create_dir_all(case.join("repo")).unwrap();
        fs::write(
            case.join("repo/grund.toml"),
            concat!(
                "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n",
                "[reference]\nstrict = true\n[scan]\nrespect_gitignore = false\n",
                "[[kinds]]\nkind = \"FS\"\nfile = \"facts.md\"\n"
            ),
        )
        .unwrap();
        fs::create_dir_all(case.join("repo/src")).unwrap();
        fs::write(case.join("repo/src/empty.py"), "# \u{a7}FS-demo\n").unwrap();
        fs::write(case.join("command.external"), external).unwrap();
        fs::write(case.join("repo/facts.md"), "# FS-demo: Demo\n").unwrap();
        fs::write(case.join("expected.exit"), "0\n").unwrap();
        fs::write(case.join("expected.stdout"), "\n").unwrap();
        fs::write(case.join("expected.stderr"), "\n").unwrap();
        Self { root, case }
    }

    fn run(&self) {
        let outcome = run_case(&self.root, &self.case, CaseKind::Example);
        assert_every_case_passed("external manifest", &[outcome]);
    }

    fn refusal(&self, needles: &[&str]) {
        let failure = std::panic::catch_unwind(|| self.run()).expect_err("must refuse");
        let message = failure
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| failure.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        for needle in needles {
            assert!(
                message.contains(needle),
                "expected {needle:?} in {message:?}"
            );
        }
        assert!(
            !self.root.join("target/e2e-work").exists(),
            "refused manifest must not copy fixtures"
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn external_argv_preserves_literals_stdin_copy_and_final_tree() {
    let fixture = Fixture::new(
        "external-literals",
        r#"["python3","{repo_copy}/probe.py","a b","","$HOME",";","{other}"]"#,
    );
    let probe = "import pathlib, sys\nprint('|'.join(sys.argv[1:]))\nprint(sys.stdin.read().strip())\npathlib.Path(__file__).with_name('result.txt').write_text('done\\n', encoding='utf-8', newline='\\n')\n";
    fs::write(fixture.case.join("repo/probe.py"), probe).unwrap();
    fs::write(fixture.case.join("command.stdin"), "input\n").unwrap();
    fs::write(
        fixture.case.join("expected.stdout"),
        "a b||$HOME|;|{other}\ninput\n",
    )
    .unwrap();
    fs::create_dir_all(fixture.case.join("expected.repo/src")).unwrap();
    fs::copy(
        fixture.case.join("repo/src/empty.py"),
        fixture.case.join("expected.repo/src/empty.py"),
    )
    .unwrap();
    fs::create_dir_all(fixture.case.join("expected.repo")).unwrap();
    fs::write(fixture.case.join("expected.repo/probe.py"), probe).unwrap();
    fs::copy(
        fixture.case.join("repo/facts.md"),
        fixture.case.join("expected.repo/facts.md"),
    )
    .unwrap();
    fs::copy(
        fixture.case.join("repo/grund.toml"),
        fixture.case.join("expected.repo/grund.toml"),
    )
    .unwrap();
    fs::write(fixture.case.join("expected.repo/result.txt"), "done\n").unwrap();
    fixture.run();
    assert!(!fixture.case.join("repo/result.txt").exists());
}

#[test]
fn external_grund_placeholder_names_this_builds_binary() {
    let fixture = Fixture::new("external-grund", r#"["{grund}","--version"]"#);
    fs::write(
        fixture.case.join("expected.stdout"),
        format!("grund {}\n", env!("CARGO_PKG_VERSION")),
    )
    .unwrap();
    fixture.run();
    let outcome = case_runner::assert_case_is_deterministic(&fixture.root, &fixture.case);
    assert_every_case_passed("external determinism", &[outcome]);
}

#[test]
fn external_cwd_uses_copy_without_argument_placeholders() {
    let fixture = Fixture::new(
        "external-cwd",
        r#"["python3","-c","import pathlib, sys; sys.stdout.buffer.write(pathlib.Path('src/empty.py').read_bytes())"]"#,
    );
    fs::write(fixture.case.join("command.cwd"), "{repo_copy}\n").unwrap();
    fs::write(fixture.case.join("expected.stdout"), "# \u{a7}FS-demo\n").unwrap();
    fixture.run();
}

#[test]
fn external_refresh_accounts_for_actual_child_output() {
    // Select refresh in a child, without changing process-global environment
    // while the other runner tests execute in parallel (§FS-examples.5).
    if std::env::var_os("COCHANGE_EXTERNAL_REFRESH_PROBE").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "external_refresh_accounts_for_actual_child_output",
            ])
            .env("UPDATE_EXPECTED", "1")
            .env("COCHANGE_EXTERNAL_REFRESH_PROBE", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "refresh child failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let fixture = Fixture::new(
        "external-refresh",
        r#"["python3","-c","print('external output')"]"#,
    );
    let outcome = run_case(&fixture.root, &fixture.case, CaseKind::Example);
    match outcome {
        case_runner::CaseOutcome::Refreshed { changes, .. } => {
            assert!(!changes.is_empty());
            assert_eq!(
                fs::read_to_string(fixture.case.join("expected.stdout")).unwrap(),
                "external output\n"
            );
        }
        _ => panic!("external refresh must report refreshed surfaces"),
    }
}

#[test]
fn external_and_args_are_mutually_exclusive_before_copy() {
    let fixture = Fixture::new("external-conflict", r#"["{grund}","{repo_copy}"]"#);
    fs::write(fixture.case.join("command.args"), "--version\n").unwrap();
    fixture.refusal(&["external-conflict", "command.external", "command.args"]);
}

#[test]
fn malformed_external_argv_refuses_before_copy_or_spawn() {
    for (name, external) in [
        ("empty-array", "[]"),
        ("empty-executable", r#"[""]"#),
        ("non-string", r#"["{grund}",2]"#),
        ("not-array", r#"{"argv":[]}"#),
        ("truncated", r#"["{repo_copy}""#),
    ] {
        let fixture = Fixture::new(name, external);
        fixture.refusal(&[name, "command.external"]);
    }
}

#[test]
fn maintained_cochange_example_runs_through_shared_goldens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let example = root.join("examples/cochange");
    assert!(
        case_runner::discover_examples(&root).contains(&example),
        "examples/cochange must be a maintained shared-runner demonstration"
    );
    assert!(example.join("command.external").is_file());
    let outcome = run_case(&root, &example, CaseKind::Example);
    assert_every_case_passed("cochange example", &[outcome]);
}
