//! Black-box regression for a listed value candidate whose metadata lookup fails
//! (§FS-values.5.3.1). Git cannot retain the directory modes, so these end-to-end
//! trees are built here, as in value_home_unlistable.rs. Skip where permissions
//! do not bar this process; a skip supplies no evidence for the regression.
#![cfg(unix)]

#[path = "support/fixture_cleanup.rs"]
mod fixture_cleanup;
use fixture_cleanup::remove_fixture_tree;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const VALUE_REMEDY: &str = "make the path readable, or keep it out of the scan by moving it out \
    of the value home, because exclude does not reach a value source";
const ROOT_REMEDY: &str = "make the path readable, or keep it out of the scan by removing it from \
    the [schema.kinds] row that lists it, because exclude does not prune scan roots";
const CONFIG: &str = "grund_config_version = 2\nproject_name = \"fixture\"\n\n\
    [schema.kinds.CONST]\nfolders = [\"values\", \"readable\"]\nindex = false\nform = \"value\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n";

struct Fixture {
    root: PathBuf,
    source: String,
}

impl Fixture {
    fn new(name: &str, extension: &str, cited: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1289-{name}-{extension}-{}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create isolated fixture");
        for dir in ["values", "readable", "docs"] {
            fs::create_dir(root.join(dir)).unwrap();
        }
        fs::write(root.join("grund.toml"), CONFIG).unwrap();
        let source = format!("values/prices.{extension}");
        let body = if extension == "json" {
            "{\n  \"CONST-a\": [\"a price\"]\n}\n"
        } else {
            "CONST-a: [a price]\n"
        };
        fs::write(root.join(&source), body).unwrap();
        fs::write(root.join("readable/other.json"), "{\"CONST-b\": [42]}\n").unwrap();
        fs::write(
            root.join("docs/FS-a.md"),
            if cited {
                "# FS-a: A\n\nSee §CONST-a.\n"
            } else {
                "# FS-a: A\n\nBody.\n"
            },
        )
        .unwrap();
        Self { root, source }
    }

    fn mode(&self, mode: u32) {
        fs::set_permissions(self.root.join("values"), fs::Permissions::from_mode(mode)).unwrap();
    }

    fn search_is_denied(&self) -> bool {
        self.mode(0o444);
        let names: Vec<_> = fs::read_dir(self.root.join("values"))
            .expect("mode 0444 still permits listing")
            .map(|entry| entry.unwrap().file_name())
            .collect();
        let source_path = self.root.join(&self.source);
        assert!(names.contains(&source_path.file_name().unwrap().to_os_string()));
        match fs::metadata(self.root.join(&self.source)) {
            Ok(_) => {
                eprintln!("skipped: mode 0444 does not deny this process metadata access (root?)");
                false
            }
            Err(error) => {
                assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
                true
            }
        }
    }

    fn run(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.root)
            .env("NO_COLOR", "1")
            .output()
            .expect("run compiled grund");
        Run {
            code: output.status.code().expect("exit status"),
            stdout: String::from_utf8(output.stdout).unwrap(),
            stderr: String::from_utf8(output.stderr).unwrap(),
        }
    }

    fn baseline(&self) {
        let list = self.run(&["list"]);
        assert_eq!(list.code, 0, "{list:?}");
        for id in ["CONST-a", "CONST-b", "FS-a"] {
            assert!(list.stdout.contains(id), "{list:?}");
        }
        let check = self.run(&["check"]);
        assert_eq!(check.code, 0, "{check:?}");
        assert!(check.stderr.is_empty(), "{check:?}");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // A reset that fails leaves the tree unremovable, which the removal then reports, so
        // neither raises while unwinding (§AR-ci-fixture-cleanup.1).
        let _ = fs::set_permissions(self.root.join("values"), fs::Permissions::from_mode(0o755));
        remove_fixture_tree(&self.root, "remove fixture");
    }
}

#[derive(Debug)]
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn is_failure(path: &str, message: &str, source: &str) -> bool {
    let remedy = if path == "values" {
        ROOT_REMEDY
    } else {
        VALUE_REMEDY
    };
    (path == "values" || path == source)
        && message.contains("permission denied")
        && message.ends_with(remedy)
}

/// Aggregate formats and commands so a red run retains evidence for every surface.
fn regression(name: &str, commands: &[&[&str]], cited: bool) {
    let mut failures = Vec::new();
    for extension in ["json", "yaml", "yml"] {
        let fixture = Fixture::new(name, extension, cited);
        fixture.baseline();
        if !fixture.search_is_denied() {
            return;
        }
        for args in commands {
            let run = fixture.run(args);
            let json = args.contains(&"json");
            let reported = if json {
                let records: Vec<serde_json::Value> = run
                    .stdout
                    .lines()
                    .chain(run.stderr.lines())
                    .filter_map(|line| serde_json::from_str(line).ok())
                    .filter(|record: &serde_json::Value| record["code"] == "io")
                    .collect();
                records.len() == 1
                    && records[0]["severity"] == "error"
                    && is_failure(
                        records[0]["path"].as_str().unwrap_or(""),
                        records[0]["message"].as_str().unwrap_or(""),
                        &fixture.source,
                    )
            } else {
                let lines: Vec<_> = run
                    .stderr
                    .lines()
                    .filter_map(|line| line.strip_prefix("error: "))
                    .collect();
                lines.len() == 1
                    && lines[0]
                        .split_once(": ")
                        .is_some_and(|(path, message)| is_failure(path, message, &fixture.source))
            };
            // Other readable values and ordinary findings must survive the failed lookup.
            let continued = run.stdout.contains("CONST-b") && run.stdout.contains("FS-a");
            if run.code != 2 || !reported || !continued {
                failures.push(format!(
                    "{extension}: grund {}: expected exit 2, permission failure at {} or values, \
                     and readable CONST-b/FS-a; got exit {} (reported={reported}, continued={continued})\n\
                     stdout:\n{}stderr:\n{}",
                    args.join(" "), fixture.source, run.code, run.stdout, run.stderr
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn list_reports_an_unsearchable_value_home() {
    regression("list", &[&["list"]], false);
}

#[test]
fn check_reports_an_unsearchable_value_home_without_a_citation() {
    regression(
        "uncited",
        &[&["check"], &["check", "--format", "json"]],
        false,
    );
}

#[test]
fn check_reports_an_unsearchable_value_home_with_a_citation() {
    regression("cited", &[&["check"]], true);
}

#[test]
fn confirmed_non_files_stay_silent_and_unlistable_homes_still_report() {
    let fixture = Fixture::new("controls", "json", false);
    for extension in ["json", "yaml", "yml"] {
        fs::create_dir(fixture.root.join(format!("values/sub.{extension}"))).unwrap();
    }
    fixture.baseline();
    fixture.mode(0o000);
    if fs::read_dir(fixture.root.join("values")).is_ok() {
        eprintln!("skipped: mode 000 does not bar this process from listing (root?)");
        return;
    }
    for args in [&["list"][..], &["check"]] {
        let run = fixture.run(args);
        assert_eq!(run.code, 2, "{run:?}");
        assert_eq!(
            run.stderr.trim(),
            format!("error: values: permission denied; {ROOT_REMEDY}"),
            "{run:?}"
        );
        assert!(run.stdout.contains("FS-a"), "{run:?}");
    }
}
