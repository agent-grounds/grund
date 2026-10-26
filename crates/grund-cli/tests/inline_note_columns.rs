//! CLI regression contract for §FS-inline-citation-style.4.1.1.1:
//! append every over-limit line without changing the diagnostic or its prefix.

use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const SOURCE: &str = "\
\"\"\"Parsing stage of the ingest pipeline.\"\"\"


def parse_batch(body: str):
    # Parse every line before accepting any of them, so that one malformed line rejects the whole batch
    # instead of half of it landing in the log, which is what the old ingester did and why the dashboards
    # showed gaps after every bad deploy of the collector agents (\u{a7}AR-ingest.1.1).
    return [line.split(\" \", 2) for line in body.splitlines() if line]
";

struct Fixture(PathBuf);

impl Fixture {
    fn new(budgets: &str, source: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1024-columns-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("docs/architecture")).unwrap();
        fs::create_dir_all(root.join("src/ingest")).unwrap();
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(git.status.success(), "{git:?}");
        fs::write(
            root.join("grund.toml"),
            format!(
                "grund_config_version = 2\nproject_name = \"metrics\"\n\
                 [schema.kinds.AR]\nfolders = [\"docs/architecture\"]\nindex = false\n{budgets}"
            ),
        )
        .unwrap();
        fs::write(
            root.join("docs/architecture/AR-ingest.md"),
            "# AR-ingest: Ingestion is a three-stage pipeline\n\n\
             ## 1. Pipeline\n\nThe stages run in one worker per connection.\n\n\
             ### 1.1 Parsing\n\nA malformed line rejects the whole batch.\n",
        )
        .unwrap();
        let fixture = Self(root);
        fixture.source(source);
        fixture
    }

    fn source(&self, source: &str) {
        fs::write(self.0.join("src/ingest/parse.py"), source).unwrap();
    }

    fn check(&self, flags: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .args(flags)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn stdout(out: &Output, exit: i32) -> String {
    assert_eq!(out.status.code(), Some(exit), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn prefix(max: usize, limit: usize) -> String {
    format!(
        "inline note is {max} columns, over the {limit}-column maximum: \
         lines 5-7 cite \u{a7}AR-ingest.1.1"
    )
}

fn message(max: usize, limit: usize, widths: &str) -> String {
    format!("{}; over-limit lines: {widths}", prefix(max, limit))
}

/// Check legacy identity before checking the new detail, so fixture failures
/// cannot masquerade as this regression (§FS-errors.3).
fn assert_text(out: &Output, exit: i32, severity: &str, old: &str, expected: &str) {
    let actual = stdout(out, exit);
    let location = format!("src/ingest/parse.py:5: {severity}: ");
    assert_eq!(actual.lines().count(), 1, "{actual}");
    assert!(actual.starts_with(&format!("{location}{old}")), "{actual}");
    assert_eq!(
        actual,
        format!("{location}{expected}\n"),
        "missing complete over-limit line detail"
    );
}

fn assert_json(out: &Output, exit: i32, severity: &str, old: &str, expected: &str) {
    let actual = stdout(out, exit);
    let records: Vec<Value> = actual
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records.len(), 1, "{actual}");
    let mut record = records[0].clone();
    let actual_message = record["message"].as_str().unwrap().to_owned();
    assert!(actual_message.starts_with(old), "{actual}");
    record["message"] = json!(old);
    let mut identity = json!({
        "severity": severity, "path": "src/ingest/parse.py", "line": 5,
        "code": "inline-citation-style", "message": old,
        "sites": null, "authority": null
    });
    if severity == "suggestion" {
        identity.as_object_mut().unwrap().remove("severity");
        identity["channel"] = json!("suggestion");
    }
    assert_eq!(
        record, identity,
        "legacy diagnostic identity and JSON fields"
    );
    assert_eq!(
        actual_message, expected,
        "missing complete over-limit line detail"
    );
}

fn source_with_widths(widths: [usize; 3]) -> String {
    let mut lines: Vec<String> = SOURCE.lines().map(str::to_owned).collect();
    for (offset, width) in widths.into_iter().enumerate() {
        let start = if offset == 2 {
            "    # \u{a7}AR-ingest.1.1 ".to_owned()
        } else {
            // §FS-inline-citation-style.2.3.2: scalar values, including a tab,
            // a double-width glyph, and a combining mark; not bytes or cells.
            "    # é界e\u{301}\t".to_owned()
        };
        lines[4 + offset] = format!("{start}{}", "x".repeat(width - start.chars().count()));
        assert_eq!(lines[4 + offset].chars().count(), width);
    }
    format!("{}\n", lines.join("\n"))
}

#[test]
#[ignore = "red until #1024 reports all over-limit lines; implement removes this"]
fn issue_1024_hard_text_lists_both_lines_and_suppresses_soft_findings() {
    let fixture = Fixture::new(
        "[schema.notes.columns]\nmust = 100\nwarn = 90\nshould = 80\n",
        SOURCE,
    );
    assert_eq!(
        SOURCE
            .lines()
            .skip(4)
            .take(3)
            .map(|line| line.chars().count())
            .collect::<Vec<_>>(),
        vec![103, 105, 82]
    );
    assert_text(
        &fixture.check(&["--suggestions"]),
        1,
        "error",
        &prefix(105, 100),
        &message(105, 100, "line 5: 103 columns, line 6: 105 columns"),
    );
}

#[test]
#[ignore = "red until #1024 reports all over-limit lines; implement removes this"]
fn issue_1024_hard_json_keeps_identity_and_complete_message() {
    let fixture = Fixture::new("", SOURCE);
    assert_json(
        &fixture.check(&["--format", "json"]),
        1,
        "error",
        &prefix(105, 100),
        &message(105, 100, "line 5: 103 columns, line 6: 105 columns"),
    );
}

#[test]
#[ignore = "red until #1024 reports all over-limit lines; implement removes this"]
fn issue_1024_edit_cycle_keeps_the_other_line_identified() {
    let fixture = Fixture::new("", SOURCE);
    let lines: Vec<&str> = SOURCE.lines().collect();
    let only_widest = SOURCE.replace(lines[5], "    # Shortened line.");
    fixture.source(&only_widest);
    let remaining_text = fixture.check(&[]);
    let remaining_json = fixture.check(&["--format", "json"]);
    fixture.source(&only_widest.replace(lines[4], "    # Parse before accepting."));
    assert_eq!(stdout(&fixture.check(&[]), 0), "success\n");
    assert_eq!(stdout(&fixture.check(&["--format", "json"]), 0), "");
    let expected = message(103, 100, "line 5: 103 columns");
    assert_text(&remaining_text, 1, "error", &prefix(103, 100), &expected);
    assert_json(&remaining_json, 1, "error", &prefix(103, 100), &expected);
}

#[test]
#[ignore = "red until #1024 reports all over-limit lines; implement removes this"]
fn issue_1024_soft_warning_text_uses_selected_threshold() {
    let fixture = Fixture::new(
        "[schema.notes.columns]\nmust = 120\nwarn = 100\nshould = 90\n",
        &source_with_widths([103, 105, 100]),
    );
    assert_text(
        &fixture.check(&["--suggestions"]),
        0,
        "warning",
        &prefix(105, 100),
        &message(105, 100, "line 5: 103 columns, line 6: 105 columns"),
    );
}

#[test]
#[ignore = "red until #1024 reports all over-limit lines; implement removes this"]
fn issue_1024_soft_warning_json_omits_lines_below_selected_threshold() {
    let fixture = Fixture::new(
        "[schema.notes.columns]\nmust = 120\nwarn = 104\nshould = 100\n",
        SOURCE,
    );
    assert_json(
        &fixture.check(&["--format", "json", "--suggestions"]),
        0,
        "warning",
        &prefix(105, 104),
        &message(105, 104, "line 6: 105 columns"),
    );
}

#[test]
#[ignore = "red until #1024 reports all over-limit lines; implement removes this"]
fn issue_1024_soft_suggestion_keeps_gating_and_uses_its_own_limit() {
    let fixture = Fixture::new(
        "[schema.notes.columns]\nmust = 120\nwarn = 104\nshould = 100\n",
        &source_with_widths([103, 104, 100]),
    );
    assert_eq!(stdout(&fixture.check(&[]), 0), "success\n");
    assert_eq!(stdout(&fixture.check(&["--format", "json"]), 0), "");
    let expected = message(104, 100, "line 5: 103 columns, line 6: 104 columns");
    // Suggestions retain their distinct channel in text and JSON.
    let text = fixture.check(&["--suggestions"]);
    let json = fixture.check(&["--suggestions", "--format", "json"]);
    assert_json(&json, 0, "suggestion", &prefix(104, 100), &expected);
    assert_text(&text, 0, "suggestion", &prefix(104, 100), &expected);
}

#[test]
#[ignore = "red until #1024 reports all over-limit lines; implement removes this"]
fn issue_1024_single_over_limit_line_counts_unicode_and_excludes_boundary() {
    let fixture = Fixture::new("", &source_with_widths([100, 101, 100]));
    let expected = message(101, 100, "line 6: 101 columns");
    assert_text(
        &fixture.check(&[]),
        1,
        "error",
        &prefix(101, 100),
        &expected,
    );
    assert_json(
        &fixture.check(&["--format", "json"]),
        1,
        "error",
        &prefix(101, 100),
        &expected,
    );
}
