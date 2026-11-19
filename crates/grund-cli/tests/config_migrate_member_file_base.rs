//! Issue #1464: the file a `member:` line names is a report path and follows
//! `--path-base` like the heading above it (§FS-config-migrate.8,
//! §FS-config-migrate.2.2), naming the member as `grund check` names it there
//! (§FS-check.checks.config-version-1). Ports the reproducer of tool report
//! grund.314. Only the file before ` is version 1` is judged: the directory the
//! line's command names is #1458's.

#[path = "support/temporary_root.rs"]
mod temporary_root;
use temporary_root::TemporaryRoot;

use std::fs;
use std::process::{Command, Output};

/// The workspace root of grund.314, listing the version-1 member `apps/api`.
const ROOT_V1: &str = "grund_config_version = 1

[id]
format = \"{kind}-{slug}\"
named_sections = true

[[kinds]]
kind = \"FS\"
folder = \"docs/fs\"
index = false

[scan]
include = [\"docs\"]

[workspace]
members = [\"apps/api\"]
";

/// The member of grund.314.
const MEMBER_V1: &str = "grund_config_version = 1

[id]
format = \"{kind}-{slug}\"
named_sections = true

[[kinds]]
kind = \"AR\"
folder = \"docs/ar\"
index = false

[scan]
include = [\"docs\"]
";

/// grund.314's tree in a committed git repository: `grund.toml` and
/// `apps/api/grund.toml`, both on version 1, and the root's `docs`.
fn workspace(name: &str) -> TemporaryRoot {
    let root = TemporaryRoot::new(&std::env::temp_dir(), name);
    for (relative, text) in [
        ("grund.toml", ROOT_V1),
        (
            "docs/fs/FS-root.md",
            "# FS-root: Root\n\nThe root project.\n",
        ),
        ("apps/api/grund.toml", MEMBER_V1),
        (
            "apps/api/docs/ar/AR-api.md",
            "# AR-api: Api\n\nThe member.\n",
        ),
    ] {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    for args in [
        &["init", "-q"][..],
        &["add", "-A"],
        &["commit", "-qm", "version 1"],
    ] {
        let out = Command::new("git")
            .args([
                "-c",
                "user.name=grund",
                "-c",
                "user.email=grund@example.invalid",
            ])
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .current_dir(&*root)
            .env("GIT_CONFIG_GLOBAL", root.join(".git/no-global-config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
    }
    root
}

fn grund(root: &TemporaryRoot, cwd: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root.join(cwd))
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The report's first line, and the file the `member:` line names before
/// ` is version 1`.
fn heading_and_member_file(out: &Output) -> (String, String) {
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "stderr:\n{stderr}");
    let heading = stderr.lines().next().unwrap_or_default().to_owned();
    let member = stderr
        .lines()
        .find_map(|line| line.strip_prefix("member: "))
        .and_then(|rest| rest.split_once(" is version 1"))
        .map(|(file, _)| file.to_owned())
        .unwrap_or_else(|| panic!("no member: line in:\n{stderr}"));
    (heading, member)
}

/// §FS-config-migrate.8, §FS-check.checks.config-version-1: the ticket's case.
/// From the root's `docs` under `--path-base invocation`, the heading is
/// `../grund.toml`, and the member's file is `../apps/api/grund.toml`, the
/// file `grund check` names with the same flag from the same place.
#[test]
fn member_file_follows_the_invocation_base() {
    let root = workspace("grund-1464-invocation");
    let (heading, member) = heading_and_member_file(&grund(
        &root,
        "docs",
        &[
            "config",
            "migrate",
            "--to",
            "2",
            "--path-base",
            "invocation",
        ],
    ));
    assert_eq!(
        heading,
        "../grund.toml: version 1 → 2, preview, nothing written"
    );
    let check = text(&grund(&root, "docs", &["check", "--path-base", "invocation"]).stderr);
    assert!(
        check.contains("warning: ../apps/api/grund.toml is version 1;"),
        "check names the member from the invocation base:\n{check}"
    );
    assert_eq!(
        member, "../apps/api/grund.toml",
        "the member: line names its file from the heading's base, as check does"
    );
}

/// §FS-config-migrate.2.2: the default project base, from the same place, names
/// the heading and the member's file from the workspace root.
#[test]
fn member_file_keeps_the_project_base() {
    let root = workspace("grund-1464-project");
    for args in [&["--path-base", "project"][..], &[]] {
        let (heading, member) = heading_and_member_file(&grund(
            &root,
            "docs",
            &[&["config", "migrate", "--to", "2"][..], args].concat(),
        ));
        assert_eq!(
            heading, "grund.toml: version 1 → 2, preview, nothing written",
            "{args:?}"
        );
        assert_eq!(member, "apps/api/grund.toml", "{args:?}");
    }
}
