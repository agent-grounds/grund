//! Black-box: every surface that reads a selected section gets the one first line the slicer
//! gives it, its own heading at the canonical path (§FS-declarations.section.relative.shown.selected):
//! the numeric form, a name under a number, a stub's target, a fenced heading left as written,
//! `show --batch` by query and by glob, and the bytes `list --size` measures (§FS-list.3.4.1).

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

const CONFIG: &str = "grund_config_version = 1

[reference]
strict = true
require_grounding = false

[id]
format = \"{kind}-{slug}\"
named_sections = true

[[kinds]]
kind = \"FS\"
folder = \"docs/spec\"
index = false

[fmt.cross_refs]
enabled = false
";

/// Relative headings in each form, and a fenced heading inside a selected slice.
const FORMS: &str = "# FS-forms: Relative forms

## retry: Retry schedule

### .2: Ceiling

Never more than five seconds.

### .3: Example

```markdown
### .9: Not a heading
```

## 3. Third

### .1. Scope

Scope text.

## 1. Overview

### .goals: Goals

Goals text.
";

/// The same `retry.2` written with its complete path: what the relative one now measures as.
const COMPLETE: &str = "# FS-complete: Complete form

## retry: Retry schedule

### retry.2: Ceiling

Never more than five seconds.
";

/// A source doc-comment declaration, reached through its stub in the kind home.
const SOURCE: &str = "/// FS-thing: A thing
///
/// ## format: Formatting
///
/// ### .1: Username
///
/// Username text.
pub struct Thing;
";

const STUB: &str = "# FS-thing: [src/lib.rs](../../src/lib.rs)\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "grund-1009-surfaces-{}-{serial}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        for (path, contents) in [
            ("grund.toml", CONFIG),
            ("docs/spec/FS-forms.md", FORMS),
            ("docs/spec/FS-complete.md", COMPLETE),
            ("docs/spec/FS-thing.md", STUB),
            ("src/lib.rs", SOURCE),
        ] {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    /// Run `grund args…` in the fixture with `stdin`, require exit 0, and return stdout.
    fn run(&self, args: &[&str], stdin: &str) -> String {
        let mut child = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run grund");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        let out = child.wait_with_output().expect("wait for grund");
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        assert!(
            out.status.success(),
            "grund {} exited {:?}: stdout:\n{stdout}\nstderr:\n{}",
            args.join(" "),
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
        stdout
    }

    fn show(&self, args: &[&str]) -> String {
        self.run(args, "")
    }

    fn json_lines(&self, args: &[&str], stdin: &str) -> Vec<serde_json::Value> {
        self.run(args, stdin)
            .lines()
            .map(|line| serde_json::from_str(line).expect("one JSON object per line"))
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The numeric form and a name under a number print their canonical paths, punctuation kept.
#[test]
fn numeric_and_name_under_number_forms_name_their_path() {
    let fixture = Fixture::new();
    for mode in [&[][..], &["--brief"], &["--full"]] {
        let args = |id| [&[id][..], mode].concat();
        assert_eq!(
            fixture.show(&args("FS-forms.3.1")),
            "### 3.1. Scope\n\nScope text.\n"
        );
        assert_eq!(
            fixture.show(&args("FS-forms.1.goals")),
            "### 1.goals: Goals\n\nGoals text.\n"
        );
    }
}

/// A heading inside a fence is no heading: below the selected first line it prints as written.
#[test]
fn a_fenced_heading_in_the_slice_stays_as_written() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture.show(&["FS-forms.retry.3", "--full"]),
        "### retry.3: Example\n\n```markdown\n### .9: Not a heading\n```\n"
    );
}

/// A section read through a stub is sliced from its target, so it names its path the same way.
#[test]
fn a_stub_target_section_names_its_path() {
    let fixture = Fixture::new();
    assert_eq!(
        fixture.show(&["FS-thing.format.1", "--full"]),
        "### format.1: Username\n\nUsername text.\n"
    );
}

/// `show --batch`, by query and by glob, carries the body the direct read prints.
#[test]
fn batch_reads_agree_with_the_direct_read() {
    let fixture = Fixture::new();
    for mode in ["--brief", "--full"] {
        let direct: serde_json::Value =
            serde_json::from_str(&fixture.show(&["FS-forms.retry.2", mode, "--format", "json"]))
                .unwrap();
        let body = direct["body"].as_str().unwrap().to_string();
        assert!(body.starts_with("### retry.2: Ceiling\n"), "{body:?}");
        let queried = fixture.json_lines(
            &["show", "--batch", mode, "--format", "json"],
            "{\"id\":\"FS-forms\",\"section\":\"retry.2\"}\n",
        );
        assert_eq!(queried[0]["result"]["body"].as_str(), Some(body.as_str()));
        let globbed = fixture.json_lines(
            &[
                "show",
                "--batch",
                "--glob",
                "FS-forms.retry.2",
                mode,
                "--format",
                "json",
            ],
            "",
        );
        assert_eq!(globbed[0]["result"]["body"].as_str(), Some(body.as_str()));
    }
}

/// `list --size` measures the printed first line: the relative `retry.2` now measures exactly
/// as the complete one does, so its bytes grow by the parent path and its words and lines do not.
#[test]
fn list_size_measures_the_selected_heading_as_printed() {
    let fixture = Fixture::new();
    let rows = fixture.json_lines(&["list", "--size", "--format", "json"], "");
    let row = |id: &str| {
        rows.iter()
            .find(|row| row["id"] == id && row["section"] == "retry.2")
            .unwrap_or_else(|| panic!("no size row for {id}.retry.2 in {rows:?}"))
    };
    let (relative, complete) = (row("FS-forms"), row("FS-complete"));
    for field in [
        "lead_bytes",
        "full_bytes",
        "lead_words",
        "full_words",
        "lead_lines",
        "full_lines",
    ] {
        assert_eq!(relative[field], complete[field], "{field}");
    }
}
