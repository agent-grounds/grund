//! §FS-examples.5.6.2: external output goes through the real compare/refresh seam.
use super::super::{CaseOutcome, case_paths, prepare_case, run_prepared_case};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const TEST: &str =
    "case_runner::copy_output_tests::output_probe::comparison_and_refresh_preserve_foreign_paths";
const PROBE: &str = "GRUND_797_COPY_OUTPUT_PROBE";
const NAME: &str = "path-output";
const CHILD: &str = r#"
import pathlib, sys
sys.stdout.reconfigure(encoding='utf-8', newline='\n')
sys.stderr.reconfigure(encoding='utf-8', newline='\n')
text = pathlib.Path(sys.argv[2]).read_text().replace('{copy}', sys.argv[1])
text = text.replace('{absolute}', str((pathlib.Path.cwd() / sys.argv[1]).resolve()))
sys.stdout.write(text)
sys.stderr.write(text)
"#;
const TRAP: &str = "{copy}@other\n{copy}+other\n";
const OUTPUT: &str = "owned: {copy}\nabsolute: {absolute}\nquoted: `{copy}` \"{copy}/nested file\"\n\
    descendant: {copy}/file.md:12: diagnostic\n\
    sibling: {foreign}@other\nplus: {foreign}+other\n\
    enclosing: prefix@{foreign}\nenclosing-plus: prefix+{foreign}\n\
    unrelated: literal\\{foreign}\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .expect("scratch home");
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(home).join("ag/tmp").join(format!(
            "grund-797-copy-output-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("case/repo")).unwrap();
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Re-exec to select refresh without mutating any other test thread's environment.
#[test]
fn comparison_and_refresh_preserve_foreign_paths() {
    let Some(mode) = std::env::var_os(PROBE) else {
        for mode in ["compare", "refresh"] {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args(["--exact", TEST, "--nocapture"])
                .env(PROBE, mode);
            if mode == "refresh" {
                command.env("UPDATE_EXPECTED", "1");
            } else {
                command.env_remove("UPDATE_EXPECTED");
            }
            let output = command.output().expect("run isolated output probe");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                output.status.success()
                    && stdout.contains("running 1 test")
                    && stdout.contains(TEST),
                "{mode} probe failed:\n{stdout}\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    };
    let refresh = mode == "refresh";
    assert!(refresh || mode == "compare");
    let fixture = Fixture::new();
    let root = &fixture.0;
    let case = root.join("case");
    let template = root.join("output-template");
    fs::write(
        case.join("command.external"),
        serde_json::json!(["python3", "-c", CHILD, "{repo_copy}", template]).to_string(),
    )
    .unwrap();
    fs::write(case.join("expected.exit"), "0\n").unwrap();
    let execution = prepare_case(root, &case, NAME);
    // Read argv from this prepared real execution. The assertions below neither
    // guess its allocation name nor reuse the normalizer to build expectations.
    let actual = &execution.args[3];
    let logical = case_paths::logical_copy(NAME)
        .to_string_lossy()
        .into_owned();
    let write_goldens = |text: &str| {
        for stream in ["stdout", "stderr"] {
            fs::write(case.join(format!("expected.{stream}")), text).unwrap();
        }
    };
    if !refresh {
        fs::write(&template, TRAP).unwrap();
        write_goldens(&TRAP.replace("{copy}", &logical));
        let outcome = run_prepared_case(root, &case, NAME, &execution);
        let CaseOutcome::Failed { mismatches, .. } = outcome else {
            panic!("unrelated sibling mismatch must fail comparison");
        };
        assert_eq!(mismatches.len(), 2, "both streams must retain the mismatch");
        assert!(mismatches[0].contains("stdout mismatch"));
        assert!(mismatches[1].contains("stderr mismatch"));
    }
    // The same produced text must be compared and refreshed: owned fields use
    // logical paths, whereas foreign fields keep the actual invocation bytes.
    fs::write(&template, OUTPUT.replace("{foreign}", "{copy}")).unwrap();
    let expected = OUTPUT
        .replace("{copy}", &logical)
        .replace("{absolute}", &root.join(&logical).to_string_lossy())
        .replace("{foreign}", actual);
    write_goldens(if refresh { "stale\n" } else { &expected });
    let outcome = run_prepared_case(root, &case, NAME, &execution);
    if refresh {
        assert!(matches!(outcome, CaseOutcome::Refreshed { .. }));
    } else {
        assert!(matches!(outcome, CaseOutcome::Ran));
    }
    for stream in ["stdout", "stderr"] {
        assert_eq!(
            fs::read_to_string(case.join(format!("expected.{stream}"))).unwrap(),
            expected
        );
    }
}
