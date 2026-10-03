//! §AR-lsp.5.1.1 — `grund_core::lsp_snapshot` anchored at a configured sub-folder is
//! a path-scoped run: it resolves against the whole project and narrows only its
//! report, so the report is the one `grund_core::check` of the same path returns
//! (§FS-check.1.3.6.1, §FS-rules.9.1.1). The fixture is agent-grounds/grund#414's:
//! the anchor `docs/fs` holds no rule declaration and cites a goal in `docs/goals`,
//! so a snapshot that walked only the anchor would render a bulletless
//! `### Chapter rules` section and leave the citation dangling.
//!
//! The managed block is written by the production `grund init`, the way a user's
//! tree gets it, so the test compares against bytes no test helper rendered.

#[path = "binaries.rs"]
mod binaries;

use grund_core::{Finding, LspSnapshot, LspSnapshotOpts, Report, check, lsp_snapshot};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG: &str = r#"grund_config_version = 1
project_name = "grund414"

[reference]
strict = true

[id]
format = "{kind}-{slug}"
named_sections = true

[[kinds]]
kind = "GOAL"
folder = "docs/goals"
index = false

[[kinds]]
kind = "FS"
folder = "docs/fs"
index = false

[[kinds]]
kind = "RULE"
folder = "docs/rules"
index = false
rules = true

[scan]
include = ["docs", "AGENTS.md"]
"#;

/// The chapter-rule bullet `grund init` renders for `RULE-terms`.
const BULLET: &str = "- Each FS must have exactly one Terms chapter.";

/// A fresh copy of the issue's fixture, with the `AGENTS.md` `grund init` wrote.
fn fixture(name: &str) -> PathBuf {
    let root = binaries::repo_root()
        .join("target/lsp-snapshot-path-scope-work")
        .join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture");
    }
    for (path, text) in [
        ("grund.toml", CONFIG),
        (
            "docs/goals/GOAL-repro.md",
            "# GOAL-repro: Every spec names its terms\n\nThe project wants its terms written down.\n",
        ),
        (
            "docs/fs/FS-one.md",
            "# FS-one: A subject with its terms\n\n## terms: Terms\n\nThis chapter names the terms for \u{a7}GOAL-repro.\n",
        ),
        (
            "docs/rules/RULE-terms.md",
            "# RULE-terms: Each FS must have exactly one Terms chapter.\n\nThe rule protects \u{a7}GOAL-repro.\n",
        ),
    ] {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("parent")).expect("create fixture dir");
        fs::write(path, text).expect("write fixture file");
    }
    let git = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&root)
        .status()
        .expect("spawn git init");
    assert!(git.success(), "git init failed in {}", root.display());
    for args in [&["init"][..], &["init", "--check"][..]] {
        let output = Command::new(binaries::grund())
            .args(args)
            .current_dir(&root)
            .output()
            .expect("spawn grund");
        assert!(
            output.status.success(),
            "grund {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(
        agents(&root).lines().any(|line| line.starts_with(BULLET)),
        "grund init must render the chapter-rule bullet, or the fixture no longer exercises the compare:\n{}",
        agents(&root)
    );
    root.canonicalize().expect("canonical fixture root")
}

fn agents(root: &Path) -> String {
    fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md")
}

fn delete_bullet(root: &Path) {
    let kept: String = agents(root)
        .split_inclusive('\n')
        .filter(|line| !line.starts_with(BULLET))
        .collect();
    fs::write(root.join("AGENTS.md"), kept).expect("write AGENTS.md");
}

fn snapshot(path: PathBuf) -> LspSnapshot {
    lsp_snapshot(LspSnapshotOpts {
        path,
        path_provided: true,
        ..Default::default()
    })
    .expect("lsp_snapshot")
}

/// One diagnostic as `severity code root-relative-path:line message`, so `check`'s
/// display paths and the snapshot's absolute paths compare as the same finding.
fn diagnostics(root: &Path, report: &Report) -> Vec<String> {
    let key = |finding: &Finding| {
        let path = finding.path.as_deref().map(|path| {
            let path = Path::new(path);
            let absolute = if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir().expect("cwd").join(path)
            };
            let absolute = absolute.canonicalize().unwrap_or(absolute);
            absolute
                .strip_prefix(root)
                .map(|relative| relative.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| path.to_string_lossy().into_owned())
        });
        format!(
            "{} {} {}:{} {}",
            finding.severity,
            finding.code,
            path.unwrap_or_default(),
            finding.line.unwrap_or_default(),
            finding.message
        )
    };
    let mut keys: Vec<String> = report
        .errors
        .iter()
        .chain(&report.warnings)
        .map(key)
        .collect();
    keys.sort();
    keys
}

fn has(diagnostics: &[String], needle: &str) -> bool {
    diagnostics.iter().any(|line| line.contains(needle))
}

const AGENTS_INIT: &str = "error agents-init AGENTS.md:";
const DANGLING: &str = "error dangling docs/fs/FS-one.md:5 ";

/// The block `grund init` wrote is correct under every scope, and the citation of a
/// goal outside the anchor resolves: the snapshot reports what `check(docs/fs)` does.
#[test]
fn a_sub_folder_snapshot_accepts_the_block_init_wrote() {
    let root = fixture("as-written");
    let whole = diagnostics(&root, &check(&root).expect("check(.)"));
    assert!(
        !has(&whole, AGENTS_INIT),
        "control: check(.) must accept init's block: {whole:#?}"
    );

    let scoped = diagnostics(
        &root,
        &check(&root.join("docs/fs")).expect("check(docs/fs)"),
    );
    let snapped = diagnostics(&root, &snapshot(root.join("docs/fs")).report);
    assert!(
        !has(&scoped, AGENTS_INIT),
        "control: check(docs/fs) must accept init's block: {scoped:#?}"
    );
    assert!(
        !has(&scoped, DANGLING),
        "control: check(docs/fs) resolves the citation: {scoped:#?}"
    );
    assert!(
        !has(&snapped, AGENTS_INIT),
        "lsp_snapshot(docs/fs) calls the block grund init wrote stale: {snapped:#?}"
    );
    assert!(
        !has(&snapped, DANGLING),
        "lsp_snapshot(docs/fs) reports the citation of GOAL-repro as dangling: {snapped:#?}"
    );
    assert_eq!(
        snapped, scoped,
        "lsp_snapshot(docs/fs) must report what check(docs/fs) reports"
    );
}

/// With a chapter-rule bullet deleted by hand the block is stale under every scope,
/// and the sub-folder snapshot says so with the finding `check(docs/fs)` carries.
#[test]
fn a_sub_folder_snapshot_rejects_a_block_missing_a_bullet() {
    let root = fixture("bullet-deleted");
    delete_bullet(&root);
    let whole = diagnostics(&root, &check(&root).expect("check(.)"));
    assert!(
        has(&whole, AGENTS_INIT),
        "control: check(.) must reject the edited block: {whole:#?}"
    );

    let scoped = diagnostics(
        &root,
        &check(&root.join("docs/fs")).expect("check(docs/fs)"),
    );
    let snapped = diagnostics(&root, &snapshot(root.join("docs/fs")).report);
    assert!(
        has(&scoped, AGENTS_INIT),
        "control: check(docs/fs) must reject the edited block: {scoped:#?}"
    );
    assert!(
        has(&snapped, AGENTS_INIT),
        "lsp_snapshot(docs/fs) accepts a block missing its chapter-rule bullet: {snapped:#?}"
    );
    assert_eq!(
        snapped, scoped,
        "lsp_snapshot(docs/fs) must report what check(docs/fs) reports"
    );
}

/// The navigation records are the resolution scope's, not the anchor's: the citation
/// in the anchor names a target the snapshot also holds as a declaration.
#[test]
fn a_sub_folder_snapshot_carries_the_resolution_scope_records() {
    let root = fixture("records");
    let snapshot = snapshot(root.join("docs/fs"));
    let mut ids: Vec<&str> = snapshot
        .declarations
        .iter()
        .map(|declaration| declaration.query_id.as_str())
        .collect();
    ids.sort();
    assert_eq!(
        ids,
        ["FS-one", "GOAL-repro", "RULE-terms"],
        "declarations in the snapshot"
    );

    let goal = root.join("docs/goals/GOAL-repro.md");
    let citation = snapshot
        .citations
        .iter()
        .find(|citation| citation.path.ends_with("docs/fs/FS-one.md") && citation.line == 5)
        .expect("the snapshot carries the citation at docs/fs/FS-one.md:5");
    assert_eq!(citation.query_id, "GOAL-repro");
    let target = citation
        .target_path
        .as_deref()
        .map(|path| path.canonicalize().expect("target"));
    assert_eq!(target, Some(goal.clone()), "the citation's target");
    assert_eq!(citation.target_line, Some(1), "the citation's target line");
    assert!(
        snapshot
            .scanned_files
            .iter()
            .any(|file| file.ends_with("docs/goals/GOAL-repro.md"))
    );
}

/// §FS-check.1.3.6.3: a file outside the anchor that the wider walk cannot read is
/// not dropped with the rest of the out-of-path report — the snapshot cautions about
/// it, as `check(docs/fs)` does.
#[cfg(unix)]
#[test]
fn a_sub_folder_snapshot_cautions_about_an_unreadable_file_outside_it() {
    use std::os::unix::fs::PermissionsExt;
    let root = fixture("unreadable-outside");
    let unreadable = root.join("docs/goals/GOAL-bad.md");
    fs::write(&unreadable, "# GOAL-bad: Unreadable\n").expect("write GOAL-bad");
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).expect("chmod 000");
    if fs::read(&unreadable).is_ok() {
        eprintln!("skipped: chmod 000 does not stop this process reading a file (root?)");
        return;
    }
    let scoped = diagnostics(
        &root,
        &check(&root.join("docs/fs")).expect("check(docs/fs)"),
    );
    let snapped = diagnostics(&root, &snapshot(root.join("docs/fs")).report);
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o644)).expect("chmod back");
    let caution = "warning io docs/goals/GOAL-bad.md:0 ";
    assert!(
        has(&scoped, caution),
        "control: check(docs/fs) cautions about the unread file: {scoped:#?}"
    );
    assert!(
        has(&snapped, caution),
        "lsp_snapshot(docs/fs) drops the unread file outside the anchor: {snapped:#?}"
    );
    assert_eq!(
        snapped, scoped,
        "lsp_snapshot(docs/fs) must report what check(docs/fs) reports"
    );
}

/// §FS-check.2.2: an anchor holding no scannable file earns the empty-scan caution
/// about the anchor, however much the wider walk read.
#[test]
fn a_sub_folder_snapshot_cautions_about_an_empty_anchor() {
    let root = fixture("empty-anchor");
    fs::create_dir_all(root.join("docs/empty")).expect("create docs/empty");
    let scoped = diagnostics(
        &root,
        &check(&root.join("docs/empty")).expect("check(docs/empty)"),
    );
    let snapped = diagnostics(&root, &snapshot(root.join("docs/empty")).report);
    assert!(
        has(&scoped, "warning empty-scan "),
        "control: check(docs/empty) cautions about the empty scan: {scoped:#?}"
    );
    assert!(
        has(&snapped, "warning empty-scan "),
        "lsp_snapshot(docs/empty) loses the empty-scan caution: {snapped:#?}"
    );
    assert_eq!(
        snapped, scoped,
        "lsp_snapshot(docs/empty) must report what check(docs/empty) reports"
    );
}
