//! §AR-bindings.2: single show, batch show and the deprecated process adapter
//! expose the fenced-wrapper contract of §FS-show.2.5 and §FS-show.3.2 alike.

mod binaries;

use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SECTION_ONE: &str = concat!(
    "## 1. Both fence characters\n\n",
    "Before the fences, §FS-002-target.1 is ordinary prose.\n\n",
    "```markdown\n",
    "[§FS-002-target.1](FS-002-target.md#1-detail)\n",
    "[§FS-<foo>.3.1](<relative-path>#<anchor>)\n",
    "```\n\n",
    "Between the fences, §FS-002-target.1 is ordinary prose.\n\n",
    "~~~markdown\n",
    "[§FS-002-target.1](FS-002-target.md#1-detail)\n",
    "[§FS-<foo>.3.1](<relative-path>#<anchor>)\n",
    "~~~\n\n",
    "After the fences, §FS-002-target.1 is ordinary prose.\n",
);

const MARKDOWN_SECTION_ONE: &str = concat!(
    "## 1. Both fence characters\n\n",
    "Before the fences, [§FS-002-target.1](FS-002-target.md#1-detail) is ordinary prose.\n\n",
    "```markdown\n",
    "[§FS-002-target.1](FS-002-target.md#1-detail)\n",
    "[§FS-<foo>.3.1](<relative-path>#<anchor>)\n",
    "```\n\n",
    "Between the fences, [§FS-002-target.1](FS-002-target.md#1-detail) is ordinary prose.\n\n",
    "~~~markdown\n",
    "[§FS-002-target.1](FS-002-target.md#1-detail)\n",
    "[§FS-<foo>.3.1](<relative-path>#<anchor>)\n",
    "~~~\n\n",
    "After the fences, [§FS-002-target.1](FS-002-target.md#1-detail) is ordinary prose.\n",
);

const SECTION_TWO: &str = concat!(
    "## 2. Close and resume\n\n",
    "Before the fence, §FS-002-target.1 is ordinary prose.\n\n",
    "````markdown\n",
    "[§FS-002-target.1](FS-002-target.md#1-detail)\n",
    "~~~\n",
    "[§FS-002-target.1](FS-002-target.md#1-detail)\n",
    "```\n",
    "[§FS-002-target.1](FS-002-target.md#1-detail)\n",
    "`````\n\n",
    "After the valid longer closer, §FS-002-target.1 is ordinary prose.\n",
);

const SECTION_THREE: &str = concat!(
    "## 3. Unclosed fence\n\n",
    "Before the fence, §FS-002-target.1 is ordinary prose.\n\n",
    "~~~~markdown\n",
    "[§FS-002-target.1](FS-002-target.md#1-detail)\n",
    "~~~\n",
    "[§FS-002-target.1](FS-002-target.md#1-detail)\n",
);

fn full_body() -> String {
    format!(
        "Lead prose flattens §FS-002-target.1.\n\n{SECTION_ONE}\n{SECTION_TWO}\n{SECTION_THREE}"
    )
}

fn fixture() -> PathBuf {
    binaries::repo_root().join("tests/e2e/cases/show-fenced-cross-refs-preserved/repo")
}

fn run(binary: impl AsRef<Path>, args: &[&str], root: &Path) -> Output {
    Command::new(binary.as_ref())
        .args(args)
        .arg(root)
        .output()
        .expect("run show adapter")
}

fn run_batch(root: &Path) -> Output {
    let mut child = Command::new(binaries::grund())
        .args(["show", "--batch", "--format=json"])
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run batch show");
    child
        .stdin
        .take()
        .expect("batch stdin")
        .write_all(b"{\"id\":\"FS-001-wraps\",\"section\":\"1\"}\n")
        .expect("write batch query");
    child.wait_with_output().expect("read batch show")
}

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("UTF-8 stdout")
}

#[test]
fn reported_section_text_preserves_wrappers_inside_both_fence_characters() {
    let output = run(binaries::grund(), &["show", "FS-001-wraps.1"], &fixture());
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"");
    assert_eq!(stdout(&output), SECTION_ONE);
}

#[test]
fn reported_section_decoded_json_preserves_the_exact_text_body() {
    let output = run(
        binaries::grund(),
        &["show", "FS-001-wraps.1", "--format=json"],
        &fixture(),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"");
    let value: Value = serde_json::from_slice(&output.stdout).expect("show JSON");
    assert_eq!(value["body"], SECTION_ONE);
}

#[test]
fn markdown_output_remains_the_verbatim_control() {
    let output = run(
        binaries::grund(),
        &["show", "FS-001-wraps.1", "--format=md"],
        &fixture(),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"");
    assert_eq!(
        stdout(&output),
        format!("# FS-001-wraps: Wrapped citation examples\n{MARKDOWN_SECTION_ONE}")
    );
}

#[test]
fn default_brief_and_toc_keep_their_existing_slice_boundaries() {
    let root = fixture();
    let default = run(binaries::grund(), &["show", "FS-001-wraps"], &root);
    let brief = run(
        binaries::grund(),
        &["show", "FS-001-wraps", "--brief"],
        &root,
    );
    let toc = run(binaries::grund(), &["show", "FS-001-wraps", "--toc"], &root);
    assert_eq!(stdout(&default), "Lead prose flattens §FS-002-target.1.\n");
    assert_eq!(
        stdout(&brief),
        "# FS-001-wraps: Wrapped citation examples\n\nLead prose flattens §FS-002-target.1.\n"
    );
    assert_eq!(
        stdout(&toc),
        concat!(
            "Lead prose flattens §FS-002-target.1.\n\n",
            "## 1. Both fence characters\n",
            "## 2. Close and resume\n",
            "## 3. Unclosed fence\n",
        )
    );
}

#[test]
fn full_slice_preserves_fenced_wrappers_and_flattens_lead_prose() {
    let output = run(
        binaries::grund(),
        &["show", "FS-001-wraps", "--full"],
        &fixture(),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), full_body());
}

#[test]
fn shared_fence_grammar_ignores_short_and_mismatched_closers_then_resumes() {
    let output = run(binaries::grund(), &["show", "FS-001-wraps.2"], &fixture());
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), SECTION_TWO);
}

#[test]
fn shared_fence_grammar_runs_an_unclosed_fence_to_body_end() {
    let output = run(binaries::grund(), &["show", "FS-001-wraps.3"], &fixture());
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), SECTION_THREE);
}

#[test]
fn batch_show_uses_the_same_fenced_body_as_single_show() {
    let output = run_batch(&fixture());
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"");
    let envelope: Value = serde_json::from_slice(&output.stdout).expect("batch envelope");
    assert_eq!(envelope["result"]["body"], SECTION_ONE);
}

#[test]
fn deprecated_adapter_uses_the_same_fenced_body_as_single_show() {
    let output = run(
        binaries::grund_core_compat(),
        &["show", "FS-001-wraps.1"],
        &fixture(),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"");
    assert_eq!(stdout(&output), SECTION_ONE);
}

#[test]
fn measured_point_body_counts_the_preserved_section_bytes() {
    let output = run(
        binaries::grund(),
        &["list", "--size=lines,words,bytes", "--format=json"],
        &fixture(),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"");
    let row = stdout(&output)
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("size row JSON"))
        .find(|row| row["id"] == "FS-001-wraps" && row["section"] == "1")
        .expect("section size row");
    assert_eq!(
        row["lead_lines"],
        SECTION_ONE
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
    );
    assert_eq!(
        row["lead_words"],
        SECTION_ONE.split_ascii_whitespace().count()
    );
    assert_eq!(row["lead_bytes"], SECTION_ONE.len());
}

#[test]
fn fence_looking_source_text_does_not_suppress_existing_flattening() {
    let output = run(
        binaries::grund(),
        &["show", "AR-003-source", "--full"],
        &fixture(),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"");
    let body = stdout(&output);
    assert!(
        body.contains("```markdown\n§FS-002-target.1\n```"),
        "{body}"
    );
    assert!(!body.contains("[§FS-002-target.1]"), "{body}");
}
