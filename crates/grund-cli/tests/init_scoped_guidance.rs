//! §FS-init.2.3.5.14.3 — the companion symlinks `grund init` mirrors into a
//! scoped directory. The e2e goldens under `tests/e2e/cases/init-scoped-guidance-*`
//! pin the bytes and the report, but a tree golden reads through a symlink and
//! cannot tell one from a copy, so this suite pins what only the file system
//! shows: the mirrored companion is a symlink to its own directory's
//! `AGENTS.md`, a standalone companion stays a file, and a symlink goes with the
//! scoped file it points at (§FS-init.2.3.5.14.2, §FS-init.3.2).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

/// The config of the approved proposal's first section: FS in two folders under
/// `docs/spec/`, a non-citable test row in two folders under `tests/`.
const CONFIG: &str = r#"grund_config_version = 2
project_name = "scoped"
[schema.kinds.GOAL]
files = ["docs/goals.md"]
[schema.kinds.FS]
folders = ["docs/spec/core", "docs/spec/plugins"]
[schema.kinds.test]
citable = false
folders = ["tests/e2e", "tests/integration"]
[rules.citations.FS]
must = ["GOAL"]
[rules.citations.test]
must = ["FS"]
"#;

fn scoped_tree(name: &str) -> std::path::PathBuf {
    let target = workdir(name);
    for dir in [
        "docs/spec/core",
        "docs/spec/plugins",
        "tests/e2e",
        "tests/integration",
    ] {
        fs::create_dir_all(target.join(dir)).expect("create place");
    }
    fs::write(target.join("grund.toml"), CONFIG).expect("write grund.toml");
    fs::write(target.join("AGENTS.md"), "# scoped — agent instructions\n")
        .expect("write AGENTS.md");
    symlink("AGENTS.md", target.join("CLAUDE.md")).expect("root CLAUDE.md symlink");
    target
}

fn init(target: &Path) -> String {
    let output = run_grund(&["init", target.to_str().unwrap()], manifest_dir());
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "init failed: stderr={stderr}");
    stderr
}

fn link_target(path: &Path) -> Option<std::path::PathBuf> {
    let meta = fs::symlink_metadata(path).ok()?;
    meta.file_type()
        .is_symlink()
        .then(|| fs::read_link(path).expect("read link"))
}

#[test]
fn a_root_claude_symlink_is_mirrored_as_a_symlink_into_each_scoped_directory() {
    let target = scoped_tree("scoped_guidance_mirrors_claude_symlink");
    fs::write(target.join("tests/CLAUDE.md"), "# Test notes\n").expect("standalone CLAUDE.md");

    let stderr = init(&target);

    assert!(
        target.join("docs/spec/AGENTS.md").is_file(),
        "docs/spec/AGENTS.md should be written, got:\n{stderr}"
    );
    assert_eq!(
        link_target(&target.join("docs/spec/CLAUDE.md")),
        Some("AGENTS.md".into()),
        "docs/spec/CLAUDE.md should be a symlink to its own AGENTS.md, got:\n{stderr}"
    );
    assert!(
        stderr.contains("linked docs/spec/CLAUDE.md -> AGENTS.md\n"),
        "the report should name the symlink it created, got:\n{stderr}"
    );
    assert_eq!(
        link_target(&target.join("tests/CLAUDE.md")),
        None,
        "an existing standalone tests/CLAUDE.md must stay a file"
    );
    let notes = fs::read_to_string(target.join("tests/CLAUDE.md")).expect("read tests/CLAUDE.md");
    assert!(
        notes.starts_with("# Test notes\n\n<!-- BEGIN GRUND MANAGED BLOCK -->\n"),
        "the standalone companion keeps its text and gains the block:\n{notes}"
    );
}

#[test]
fn switching_to_rules_root_removes_the_scoped_file_and_its_symlink() {
    let target = scoped_tree("scoped_guidance_removes_mirrored_symlink");
    init(&target);
    assert!(
        link_target(&target.join("docs/spec/CLAUDE.md")).is_some(),
        "the first run should mirror the symlink"
    );

    fs::write(
        target.join("grund.toml"),
        format!("{CONFIG}[presentation]\nrules = \"root\"\n"),
    )
    .expect("switch to rules = root");
    let stderr = init(&target);

    for gone in [
        "docs/spec/AGENTS.md",
        "docs/spec/CLAUDE.md",
        "tests/AGENTS.md",
        "tests/CLAUDE.md",
    ] {
        assert!(
            fs::symlink_metadata(target.join(gone)).is_err(),
            "{gone} should be removed, got:\n{stderr}"
        );
        assert!(
            stderr.contains(&format!("removed {gone}\n")),
            "the report should name {gone}, got:\n{stderr}"
        );
    }
    assert_eq!(
        link_target(&target.join("CLAUDE.md")),
        Some("AGENTS.md".into()),
        "the root companion is not init's to remove"
    );
}
