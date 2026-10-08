//! Test module: a stub's Markdown target is read for its ID the way the scan
//! reads it, so a heading inside a fence of the target is an example and not
//! the declaration the stub needs (§FS-declarations.checks.broken-stub.2).
//!
//! Each case lays one target twice: outside `[scan] include`, where the stub's
//! own test is the only reader and `check` must report the stub exactly when the
//! scan would record no declaration; and inside it, where the scan's records say
//! what that reading is. The second half is the premise: a case whose fence the
//! scan reads differently is about the scanner, not about this rule.

use std::path::{Path, PathBuf};

use crate::testing::{check_run, codes, findings, scan_findings, test_root, write};

const STUB: &str = "# FS-second: [../target.md](../target.md)\n";

fn stub_repo(name: &str, target_name: &str, stub: &str, target: &str, scanned: bool) -> PathBuf {
    let root = test_root(name);
    let include = if scanned {
        format!("[\"docs\", \"{target_name}\"]")
    } else {
        "[\"docs\"]".to_string()
    };
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
             [id]\nformat = \"{{kind}}-{{slug}}\"\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n\
             [scan]\ninclude = {include}\n"
        ),
    );
    write(&root.join("docs/second.md"), stub);
    write(&root.join("docs/uses.md"), "Uses \u{a7}FS-second.\n");
    write(&root.join(target_name), target);
    root
}

/// Whether the scan records a real declaration of FS-second in `target_name`.
fn scan_declares(root: &Path, target_name: &str) -> bool {
    let run = check_run(root, false);
    scan_findings(&run.config, root)
        .declarations
        .values()
        .flatten()
        .any(|decl| !decl.is_stub && decl.file.ends_with(target_name))
}

/// The broken-stub findings `check` reports for the target outside the scan.
fn broken_stub(name: &str, target_name: &str, stub: &str, target: &str) -> Vec<String> {
    let scanned = stub_repo(&format!("{name}-scanned"), target_name, stub, target, true);
    let unscanned = stub_repo(name, target_name, stub, target, false);
    let run = check_run(&unscanned, false);
    let broken: Vec<String> = findings(&run)
        .into_iter()
        .filter(|line| line.contains("stub link target"))
        .collect();
    let declared = scan_declares(&scanned, target_name);
    assert!(
        broken.is_empty() == declared,
        "the stub's test and the scan disagree about {target_name}: the scan records {} \
         declaration of FS-second there, and check reports {broken:?} (codes {:?})",
        if declared { "a" } else { "no" },
        codes(&run)
    );
    broken
}

fn assert_broken(name: &str, target: &str) {
    assert_eq!(
        broken_stub(name, "target.md", STUB, target),
        vec!["docs/second.md:1: stub link target lacks FS-second: ../target.md".to_string()],
        "a heading of the ID only inside a fence leaves the stub broken"
    );
}

fn assert_healthy(name: &str, target_name: &str, stub: &str, target: &str) {
    assert_eq!(
        broken_stub(name, target_name, stub, target),
        Vec::<String>::new(),
        "the target declares the ID outside every fence"
    );
}

/// §FS-declarations.checks.broken-stub.2: the reported tree, a backtick fence.
#[test]
fn a_heading_only_inside_a_backtick_fence_leaves_the_stub_broken() {
    assert_broken(
        "a_heading_only_inside_a_backtick_fence_leaves_the_stub_broken",
        "# Notes\n\nAn example:\n\n```markdown\n# FS-second: Second\n\nFenced lead.\n```\n",
    );
}

/// §FS-declarations.checks.broken-stub.2, §FS-check.1.1.5: a tilde fence is a fence.
#[test]
fn a_heading_only_inside_a_tilde_fence_leaves_the_stub_broken() {
    assert_broken(
        "a_heading_only_inside_a_tilde_fence_leaves_the_stub_broken",
        "# Notes\n\n~~~markdown\n# FS-second: Second\n\nFenced lead.\n~~~\n",
    );
}

/// §FS-check.1.1.5: a fence closes only on a run of its own character at least
/// as long as the opener, so a shorter run and a tilde run leave it open.
#[test]
fn a_shorter_or_other_character_run_does_not_close_the_fence() {
    assert_broken(
        "a_shorter_or_other_character_run_does_not_close_the_fence",
        "# Notes\n\n````markdown\n```\n~~~~\n# FS-second: Second\n\nFenced lead.\n````\n",
    );
}

/// §FS-check.1.1.5: an unclosed fence runs to end of file.
#[test]
fn a_heading_after_an_unclosed_fence_leaves_the_stub_broken() {
    assert_broken(
        "a_heading_after_an_unclosed_fence_leaves_the_stub_broken",
        "# Notes\n\n```markdown\nAn example that never closes.\n\n# FS-second: Second\n",
    );
}

/// §FS-check.1.1.5: an opener indented by up to three spaces is still a fence.
#[test]
fn a_heading_inside_an_indented_fence_leaves_the_stub_broken() {
    assert_broken(
        "a_heading_inside_an_indented_fence_leaves_the_stub_broken",
        "# Notes\n\n   ```markdown\n# FS-second: Second\n   ```\n",
    );
}

/// §FS-declarations.checks.broken-stub.2: a fenced example before the real
/// declaration changes nothing.
#[test]
fn a_fenced_example_beside_the_real_declaration_keeps_the_stub_healthy() {
    assert_healthy(
        "a_fenced_example_beside_the_real_declaration_keeps_the_stub_healthy",
        "target.md",
        STUB,
        "# Notes\n\n~~~markdown\n# FS-second: Example\n~~~\n\n# FS-second: Second\n\nDeclared lead.\n",
    );
}

/// §AR-scanner.2.3.3: four spaces of indent make a code sample, not a fence, so
/// its delimiters cannot hide the declaration after it.
#[test]
fn an_indented_code_sample_does_not_open_a_fence() {
    assert_healthy(
        "an_indented_code_sample_does_not_open_a_fence",
        "target.md",
        STUB,
        "# Notes\n\nA sample:\n\n    ```\n\n# FS-second: Second\n\nDeclared lead.\n",
    );
}

/// §FS-declarations.checks.broken-stub.2, §FS-show.2.5: fences are Markdown's;
/// the scan tracks none inside a source doc-comment, and neither does the stub.
#[test]
fn a_fence_inside_a_source_doc_comment_is_not_tracked() {
    assert_healthy(
        "a_fence_inside_a_source_doc_comment_is_not_tracked",
        "source.rs",
        "# FS-second: [../source.rs](../source.rs)\n",
        "/// An example:\n///\n/// ```text\n/// FS-second: Second\n///\n/// Fenced lead.\n/// ```\n\
         pub fn second() {}\n",
    );
}
