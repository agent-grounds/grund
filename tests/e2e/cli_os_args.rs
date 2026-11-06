//! §FS-cli.5.1: reject invalid OS arguments before dispatch; version still wins.
//! The byte arguments cannot be represented by the text command.args fixtures.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str, extensions: Option<&str>) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1259-{name}-{}", std::process::id()));
        fs::create_dir(&root).expect("create isolated fixture");
        if let Some(extensions) = extensions {
            fs::create_dir(root.join("docs")).unwrap();
            fs::write(root.join("docs/fs.md"), "# FS-001-a: Alpha\n\nBody.\n").unwrap();
            fs::write(
                root.join("grund.toml"),
                format!(
                    "project_name = \"r\"\n\n[scan]\nextensions = [{extensions}]\n\
                     include = [\"docs\"]\n\n[[kinds]]\nkind = \"FS\"\nfile = \"docs/fs.md\"\n"
                ),
            )
            .unwrap();
        }
        Self(root)
    }

    fn run(&self, args: &[&OsStr]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env_remove("RUST_BACKTRACE")
            .output()
            .expect("spawn the compiled grund binary")
    }

    fn write(&self, path: &OsStr, body: &str) {
        fs::write(self.0.join(path), body).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn bad(bytes: &[u8]) -> OsString {
    OsString::from_vec(bytes.to_vec())
}

fn status(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn refused(output: &Output, value_fragment: &str) {
    status(output, 2);
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = std::str::from_utf8(&output.stderr).expect("diagnostic is valid UTF-8");
    assert!(stderr.starts_with("error: "), "{stderr:?}");
    assert_eq!(stderr.lines().count(), 1, "{stderr:?}");
    assert!(stderr.ends_with('\n'), "{stderr:?}");
    assert!(
        !stderr.trim_end_matches('\n').chars().any(char::is_control),
        "escape control characters: {stderr:?}"
    );
    assert!(stderr.to_ascii_lowercase().contains("utf-8"), "{stderr:?}");
    assert!(
        stderr.contains(value_fragment),
        "name the argument: {stderr:?}"
    );
    assert!(stderr.contains('\\'), "escape the invalid byte: {stderr:?}");
    assert!(!stderr.contains('\u{fffd}'), "no lossy value: {stderr:?}");
    assert!(!stderr.contains("path does not exist"), "{stderr:?}");
    assert!(!stderr.contains("panicked"), "{stderr:?}");
}

#[test]
#[ignore = "red until #1259 OS argv is checked before dispatch; implement removes this"]
fn existing_non_utf8_path_is_refused() {
    let fixture = Fixture::new("existing", Some("\"md\", \"\""));
    let path = bad(b"docs/FACTS.\xff");
    fixture.write(&path, "x\n");
    let control = fixture.run(&["check".as_ref(), "docs/FACTS.X".as_ref()]);
    status(&control, 2);
    assert!(String::from_utf8_lossy(&control.stderr).contains("path does not exist: docs/FACTS.X"));
    refused(&fixture.run(&["check".as_ref(), &path]), "docs/FACTS.");
}

#[test]
#[ignore = "red until #1259 OS argv is checked before dispatch; implement removes this"]
fn missing_non_utf8_path_is_refused() {
    let fixture = Fixture::new("missing", Some("\"md\""));
    let control = fixture.run(&["check".as_ref(), "nope.X".as_ref()]);
    status(&control, 2);
    assert!(String::from_utf8_lossy(&control.stderr).contains("path does not exist: nope.X"));
    refused(
        &fixture.run(&["check".as_ref(), &bad(b"nope.\xff")]),
        "nope.",
    );
}

#[test]
#[ignore = "red until #1259 OS argv is checked before dispatch; implement removes this"]
fn non_utf8_default_id_is_refused_without_config() {
    let fixture = Fixture::new("id", None);
    status(&fixture.run(&["FS-X".as_ref()]), 1);
    refused(&fixture.run(&[&bad(b"FS-\xff")]), "FS-");
}

#[test]
#[ignore = "red until #1259 OS argv is checked before dispatch; implement removes this"]
fn non_utf8_flag_values_are_refused() {
    let fixture = Fixture::new("flag", Some("\"md\""));
    for separate in [false, true] {
        let (control, invalid) = if separate {
            (
                fixture.run(&["check".as_ref(), "--format".as_ref(), "X".as_ref()]),
                fixture.run(&["check".as_ref(), "--format".as_ref(), &bad(b"value\xff")]),
            )
        } else {
            (
                fixture.run(&["check".as_ref(), "--format=X".as_ref()]),
                fixture.run(&["check".as_ref(), &bad(b"--format=value\xff")]),
            )
        };
        status(&control, 2);
        assert!(String::from_utf8_lossy(&control.stderr).contains("format"));
        refused(&invalid, "value");
    }
}

fn version_wins(name: &str, alias: &str, first: bool) {
    let fixture = Fixture::new(name, None);
    let normal = fixture.run(&["--version".as_ref()]);
    status(&normal, 0);
    assert!(normal.stderr.is_empty());
    let line = std::str::from_utf8(&normal.stdout).unwrap();
    assert!(line.starts_with("grund ") && line.ends_with('\n'));
    assert_eq!(line.lines().count(), 1);
    for extra in [OsString::from("X"), bad(b"\xff")] {
        let args = if first {
            vec![OsStr::new(alias), &extra]
        } else {
            vec![&extra, OsStr::new(alias)]
        };
        let output = fixture.run(&args);
        status(&output, 0);
        assert_eq!(output.stdout, normal.stdout);
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

macro_rules! version_case {
    ($name:ident, $alias:literal, $first:literal) => {
        #[test]
        #[ignore = "red until #1259 version is checked in OS argv; implement removes this"]
        fn $name() {
            version_wins(stringify!($name), $alias, $first);
        }
    };
}

version_case!(long_version_before_non_utf8, "--version", true);
version_case!(long_version_after_non_utf8, "--version", false);
version_case!(short_version_before_non_utf8, "-V", true);
version_case!(short_version_after_non_utf8, "-V", false);

#[test]
#[ignore = "red until #1259 OS argv is checked before dispatch; implement removes this"]
fn non_utf8_refusal_precedes_help() {
    let fixture = Fixture::new("help", None);
    let control = fixture.run(&["--help".as_ref(), "X".as_ref()]);
    status(&control, 0);
    assert!(!control.stdout.is_empty());
    refused(
        &fixture.run(&["--help".as_ref(), &bad(b"extra\xff")]),
        "extra",
    );
}

#[test]
#[ignore = "red until #1259 OS argv is checked before dispatch; implement removes this"]
fn non_utf8_completion_argument_is_not_a_quiet_scan_failure() {
    let fixture = Fixture::new("completion", None);
    let control = fixture.run(&["complete".as_ref(), "ids".as_ref(), "missingX".as_ref()]);
    status(&control, 0);
    assert!(control.stdout.is_empty() && control.stderr.is_empty());
    refused(
        &fixture.run(&["complete".as_ref(), "ids".as_ref(), &bad(b"missing\xff")]),
        "missing",
    );
}

#[test]
#[ignore = "red until #1259 OS argv is checked before dispatch; implement removes this"]
fn non_utf8_argument_diagnostic_escapes_controls() {
    let fixture = Fixture::new("escaping", None);
    status(
        &fixture.run(&["check".as_ref(), "badX\n\r\t\x1b.md".as_ref()]),
        2,
    );
    refused(
        &fixture.run(&["check".as_ref(), &bad(b"bad\xff\n\r\t\x1b.md")]),
        "bad",
    );
}

#[test]
#[ignore = "red until #1259 OS argv is checked before dispatch; implement removes this"]
fn non_utf8_path_never_reads_replacement_character_file() {
    let fixture = Fixture::new("replacement", Some("\"md\""));
    let invalid = bad(b"docs/notes\xff.md");
    fixture.write(&invalid, "See \u{a7}FS-001-zz.\n");
    fixture.write("docs/notes\u{fffd}.md".as_ref(), "See \u{a7}FS-001-a.\n");
    status(
        &fixture.run(&["check".as_ref(), "docs/notes\u{fffd}.md".as_ref()]),
        0,
    );
    let scan = fixture.run(&["check".as_ref(), "docs".as_ref()]);
    status(&scan, 1);
    assert!(String::from_utf8_lossy(&scan.stdout).contains("unknown reference FS-001-zz"));
    refused(&fixture.run(&["check".as_ref(), &invalid]), "docs/notes");
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            result.insert(path.clone(), None);
            result.extend(snapshot(&path));
        } else {
            result.insert(path.clone(), Some(fs::read(path).unwrap()));
        }
    }
    result
}

#[test]
#[ignore = "red until #1259 OS argv is checked before dispatch; implement removes this"]
fn non_utf8_mutating_invocation_changes_no_files() {
    let fixture = Fixture::new("mutation", Some("\"md\""));
    let invalid = bad(b"docs/notes\xff.md");
    let body = "See $$FS-001-a.\n";
    fixture.write(&invalid, body);
    fixture.write("docs/notes\u{fffd}.md".as_ref(), body);
    fixture.write("docs/notesX.md".as_ref(), body);
    let control = fixture.run(&[
        "fmt".as_ref(),
        "--write".as_ref(),
        "docs/notesX.md".as_ref(),
    ]);
    status(&control, 0);
    assert_ne!(
        fs::read(fixture.0.join("docs/notesX.md")).unwrap(),
        body.as_bytes()
    );
    fixture.write("docs/notesX.md".as_ref(), body);
    let before = snapshot(&fixture.0);
    let output = fixture.run(&["fmt".as_ref(), "--write".as_ref(), &invalid]);
    assert_eq!(
        snapshot(&fixture.0),
        before,
        "refusal must precede every write"
    );
    refused(&output, "docs/notes");
}
