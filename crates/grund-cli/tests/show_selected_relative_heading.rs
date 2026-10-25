//! Black-box: a read of a selected section opens with its own heading's canonical path, so the
//! output names the section it read, while every other heading prints as written
//! (§FS-declarations.section.relative.shown.selected, §FS-show.2.2). Ports the #1009 billing case.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
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

/// The #1009 spec, with a nested relative heading under `.1` to show interior headings stay.
const BILLING: &str = "# FS-billing: Billing retries a failed charge

## retry: Retry schedule

The first retry waits one second, and each wait doubles the one before it.

### .1: Attempts

A charge is attempted at most six times, the first attempt included.

#### .1: First attempt

The first attempt is not a retry.

### .2: Ceiling

The wait between two attempts never exceeds 4817 milliseconds, however many
attempts came before it.
";

/// The twin whose section shape is the same, so only the body tells the two reads apart.
const PAYOUTS: &str = "# FS-payouts: Payouts retry a failed transfer

## retry: Retry schedule

Each wait doubles the one before it.

### .2: Ceiling

The wait between two attempts never exceeds 90000 milliseconds.
";

/// The control: the same section written with its complete path.
const CONTROL: &str = "# FS-control: Written in complete form

## retry: Retry schedule

### retry.2: Ceiling

Never more than five seconds.
";

/// A source doc-comment body with a relative heading.
const SOURCE: &str = "/// FS-thing: A thing
///
/// ## format: Formatting
///
/// ### .1: Username
///
/// Username text.
pub struct Thing;
";

const CEILING_BODY: &str =
    "\nThe wait between two attempts never exceeds 4817 milliseconds, however many
attempts came before it.
";

const MODES: &[&[&str]] = &[&[], &["--brief"], &["--full"], &["--toc"]];

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1009-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, contents) in [
            ("grund.toml", CONFIG),
            ("docs/spec/FS-billing.md", BILLING),
            ("docs/spec/FS-payouts.md", PAYOUTS),
            ("docs/spec/FS-control.md", CONTROL),
            ("src/lib.rs", SOURCE),
        ] {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    /// Run `grund args…` in the fixture, require exit 0, and return stdout.
    fn show(&self, args: &[&str]) -> String {
        let out: Output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund");
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

    fn json(&self, args: &[&str]) -> serde_json::Value {
        serde_json::from_str(&self.show(args)).expect("one JSON object")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn with<'a>(head: &[&'a str], mode: &[&'a str]) -> Vec<&'a str> {
    head.iter().chain(mode).copied().collect()
}

/// Every text mode of `FS-billing.retry.2` opens with `### retry.2: Ceiling`, and so does the
/// twin's, whose body then tells the two apart (§FS-declarations.section.relative.shown.selected).
#[test]
fn every_text_mode_names_the_selected_section() {
    let fx = Fixture::new();
    let expected = format!("### retry.2: Ceiling\n{CEILING_BODY}");
    for mode in MODES {
        let args = with(&["FS-billing.retry.2"], mode);
        assert_eq!(fx.show(&args), expected, "grund {}", args.join(" "));
        let twin = fx.show(&with(&["FS-payouts.retry.2"], mode));
        assert!(
            twin.starts_with("### retry.2: Ceiling\n"),
            "FS-payouts {mode:?}:\n{twin}"
        );
    }
}

/// `--section` selects the same slice the direct coordinate does, first line included.
#[test]
fn section_flag_agrees_with_the_direct_coordinate() {
    let fx = Fixture::new();
    for mode in MODES {
        for format in ["text", "md", "json"] {
            let flags = with(&["--format", format], mode);
            let direct = fx.show(&with(&["FS-billing.retry.2"], &flags));
            let flag = fx.show(&with(&["FS-billing", "--section", "retry.2"], &flags));
            assert_eq!(flag, direct, "--section retry.2 {flags:?}");
        }
    }
    let flag = fx.show(&["FS-billing", "--section", "retry.2", "--full"]);
    assert!(
        flag.starts_with("### retry.2: Ceiling\n"),
        "--section --full:\n{flag}"
    );
}

/// `md` prefixes the declaration heading, except in brief mode, and then prints `text`'s slice.
#[test]
fn md_adds_the_declaration_heading_before_the_same_slice() {
    let fx = Fixture::new();
    let slice = format!("### retry.2: Ceiling\n{CEILING_BODY}");
    for mode in MODES {
        let md = fx.show(&with(&["FS-billing.retry.2", "--format", "md"], mode));
        let expected = if *mode == ["--brief"] {
            slice.clone()
        } else {
            format!("# FS-billing: Billing retries a failed charge\n{slice}")
        };
        assert_eq!(md, expected, "md {mode:?}");
    }
}

/// The JSON `body` is the text body; `anchor` still follows the heading as written, and the
/// coordinate fields do not move (§FS-show.3.1.3, §FS-fmt.6.2.2).
#[test]
fn json_body_names_the_section_and_the_anchor_stays_as_written() {
    let fx = Fixture::new();
    for mode in MODES {
        let json = fx.json(&with(&["FS-billing.retry.2", "--format", "json"], mode));
        let body = json["body"].as_str().expect("body");
        assert!(
            body.starts_with("### retry.2: Ceiling\n"),
            "json {mode:?} body:\n{body}"
        );
        assert_eq!(json["id"], "FS-billing");
        assert_eq!(json["section"], "retry.2");
        assert_eq!(json["anchor"], "2-ceiling");
        assert_eq!(json["path"], "docs/spec/FS-billing.md");
        assert_eq!(json["line"], 15);
    }
}

/// A source doc-comment section opens the same way, and its anchor stays null.
#[test]
fn a_doc_comment_section_names_its_path_too() {
    let fx = Fixture::new();
    for mode in MODES {
        let text = fx.show(&with(&["FS-thing.format.1"], mode));
        assert_eq!(
            text, "### format.1: Username\n\nUsername text.\n",
            "text {mode:?}"
        );
        let json = fx.json(&with(&["FS-thing.format.1", "--format", "json"], mode));
        assert_eq!(json["body"], "### format.1: Username\n\nUsername text.\n");
        assert_eq!(json["anchor"], serde_json::Value::Null);
    }
}

/// Only the selected section's own first line changes: a relative heading inside the slice
/// prints as written, and the default read stops before it.
#[test]
fn an_interior_relative_heading_stays_as_written() {
    let fx = Fixture::new();
    assert_eq!(
        fx.show(&["FS-billing.retry.1", "--full"]),
        "### retry.1: Attempts\n\nA charge is attempted at most six times, the first attempt \
         included.\n\n#### .1: First attempt\n\nThe first attempt is not a retry.\n"
    );
    assert_eq!(
        fx.show(&["FS-billing.retry.1"]),
        "### retry.1: Attempts\n\nA charge is attempted at most six times, the first attempt \
         included.\n"
    );
}

/// The parent's full read keeps every relative child heading as written: the parent's own
/// heading is complete, so nothing in this slice is expanded.
#[test]
fn a_selected_parent_prints_its_relative_children_as_written() {
    let fx = Fixture::new();
    let full = fx.show(&["FS-billing.retry", "--full"]);
    assert!(full.starts_with("## retry: Retry schedule\n"), "{full}");
    for written in [
        "\n### .1: Attempts\n",
        "\n#### .1: First attempt\n",
        "\n### .2: Ceiling\n",
    ] {
        assert!(full.contains(written), "{written:?} as written in:\n{full}");
    }
    assert!(
        !full.contains("retry.2: Ceiling"),
        "interior heading relabelled:\n{full}"
    );
}

/// The section map already prints the canonical path, for the selected line and its children.
#[test]
fn the_section_map_is_unchanged() {
    let fx = Fixture::new();
    assert_eq!(
        fx.show(&["FS-billing.retry.1", "--toc"]),
        "### retry.1: Attempts\n\nA charge is attempted at most six times, the first attempt \
         included.\n\n#### retry.1.1: First attempt\n"
    );
}

/// A heading written with its complete path, and a whole-declaration read, are unchanged.
#[test]
fn complete_headings_and_whole_declaration_reads_are_unchanged() {
    let fx = Fixture::new();
    for mode in MODES {
        let text = fx.show(&with(&["FS-control.retry.2"], mode));
        assert_eq!(
            text, "### retry.2: Ceiling\n\nNever more than five seconds.\n",
            "{mode:?}"
        );
    }
    let whole = fx.show(&["FS-billing", "--full"]);
    assert_eq!(
        whole,
        BILLING.split_once("\n\n").unwrap().1,
        "whole declaration"
    );
}
