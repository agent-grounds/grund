//! Black-box: a dangling citation of a kind none of whose v2 places exists names those
//! places and their `grund.toml` lines (§FS-check.3.1.4), so a place written as a glob,
//! which loads as a literal path that names nothing (grund#1069), is pointed at from the
//! citer instead of leaving only `unknown reference` behind.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The report's Spec Kit-style tree: one folder per feature, and a source file citing both.
struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str, places: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("grund-1069-{name}-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create fixture");
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .output()
            .expect("initialize fixture Git repository");
        assert!(git.status.success(), "git init: {}", text(&git.stderr));
        let fixture = Self(dir);
        fixture.write(
            "grund.toml",
            &format!(
                "grund_config_version = 2\n\n[schema.kinds.SPEC]\n{places}\n\n\
                 [schema.kinds.code]\nfolders = [\"src\"]\n"
            ),
        );
        fixture.write("specs/001-a/spec.md", "# SPEC-upload: Upload\n\nUploads.\n");
        fixture.write("specs/002-b/spec.md", "# SPEC-share: Share\n\nShares.\n");
        fixture.write(
            "src/main.rs",
            "// Implements §SPEC-upload and §SPEC-share.\nfn main() {}\n",
        );
        fixture
    }

    fn write(&self, path: &str, contents: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn check(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn assert_report(out: &Output, stdout: &str) {
    assert_eq!(text(&out.stdout), stdout, "stderr: {}", text(&out.stderr));
    assert_eq!(text(&out.stderr), "");
    assert_eq!(out.status.code(), Some(1));
}

const GLOB_TAIL: &str =
    "; a place is a path from the config root, not a pattern, so list each file or name the folder";

#[test]
fn a_files_glob_is_named_at_the_citer() {
    let fixture = Fixture::new("files-glob", "files = [\"specs/*/spec.md\"]");
    let tail = format!("no place of kind SPEC exists: specs/*/spec.md (grund.toml:4){GLOB_TAIL}");
    let expected = format!(
        "src/main.rs:1: error: unknown reference SPEC-share; {tail}\n\
         src/main.rs:1: error: unknown reference SPEC-upload; {tail}\n"
    );
    assert_report(&fixture.check(&[]), &expected);
    assert_report(&fixture.check(&["--full"]), &expected);
}

#[test]
fn a_folders_glob_is_named_at_the_citer() {
    let fixture = Fixture::new("folders-glob", "folders = [\"specs/*\"]");
    let tail = format!("no place of kind SPEC exists: specs/* (grund.toml:4){GLOB_TAIL}");
    assert_report(
        &fixture.check(&[]),
        &format!(
            "src/main.rs:1: error: unknown reference SPEC-share; {tail}\n\
             src/main.rs:1: error: unknown reference SPEC-upload; {tail}\n"
        ),
    );
}

#[test]
fn every_missing_place_is_named_in_row_order_without_the_pattern_clause() {
    let fixture = Fixture::new(
        "literal-missing",
        "folders = [\"docs/specs\"]\nfiles = [\"docs/spec.md\"]",
    );
    let tail =
        "no place of kind SPEC exists: docs/spec.md (grund.toml:5), docs/specs (grund.toml:4)";
    assert_report(
        &fixture.check(&[]),
        &format!(
            "src/main.rs:1: error: unknown reference SPEC-share; {tail}\n\
             src/main.rs:1: error: unknown reference SPEC-upload; {tail}\n"
        ),
    );
}

#[test]
fn the_json_finding_stays_dangling_with_the_hint_in_its_message() {
    let fixture = Fixture::new("json", "files = [\"specs/*/spec.md\"]");
    let out = fixture.check(&["--format", "json"]);
    assert_eq!(out.status.code(), Some(1), "stderr: {}", text(&out.stderr));
    let tail = format!("no place of kind SPEC exists: specs/*/spec.md (grund.toml:4){GLOB_TAIL}");
    let lines: Vec<serde_json::Value> = text(&out.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("one JSON finding per line"))
        .collect();
    assert_eq!(lines.len(), 2);
    for (finding, id) in lines.iter().zip(["SPEC-share", "SPEC-upload"]) {
        assert_eq!(finding["code"], "dangling");
        assert_eq!(finding["severity"], "error");
        assert_eq!(
            finding["message"],
            format!("unknown reference {id}; {tail}")
        );
    }
}

/// Guard: with literal places that exist, a genuinely missing ID keeps the bare message.
#[test]
fn literal_places_keep_the_bare_message() {
    let fixture = Fixture::new(
        "literal-control",
        "files = [\"specs/001-a/spec.md\", \"specs/002-b/spec.md\"]",
    );
    fixture.write("src/lib.rs", "// Implements §SPEC-zzzzzzzzzz.\n");
    assert_report(
        &fixture.check(&[]),
        "src/lib.rs:1: error: unknown reference SPEC-zzzzzzzzzz\n",
    );
}

/// Guard: the condition is every place, so one existing place withholds the hint.
#[test]
fn one_existing_place_withholds_the_hint() {
    let fixture = Fixture::new(
        "partly-missing",
        "files = [\"specs/001-a/spec.md\", \"specs/*/spec.md\"]",
    );
    assert_report(
        &fixture.check(&[]),
        "src/main.rs:1: error: unknown reference SPEC-share\n",
    );
}
