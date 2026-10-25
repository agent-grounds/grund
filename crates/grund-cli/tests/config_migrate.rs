//! `grund config migrate --to 2` where a golden case cannot reach (§FS-config-migrate):
//! a tree outside this repository, file modes and times, the directory's other
//! entries, and grund's own config. The byte-exact streams are the
//! `config-migrate-*` e2e cases.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const V1: &str = "grund_config_version = 1
project_name = \"example\"

[id]
format = \"{kind}-{slug}\"
named_sections = true
slug_pattern = \"[a-z][a-z0-9-]*\"

[[kinds]]
kind = \"FS\"
file = \"requirements.md\"

[scan]
include = [\"requirements.md\"]
extensions = [\"md\"]
";

const REQUIREMENTS: &str =
    "# Requirements\n\n## FS-login: a user signs in\n\nA session lasts a day.\n";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn grund(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(repository_root())
        .output()
        .expect("spawn grund")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// A fresh directory named after the case. `outside` puts it under the system
/// temp directory, where no config above it can be discovered; otherwise it is
/// under this repository's `target/`, where its own config stops discovery.
fn fixture(name: &str, outside: bool) -> PathBuf {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
    let base = if outside {
        std::env::temp_dir().join(format!("grund-config-migrate-{}", std::process::id()))
    } else {
        repository_root().join("target/config-migrate-tests")
    };
    let dir = base.join(format!("{name}-{serial}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create fixture");
    dir
}

fn v1_tree(name: &str) -> PathBuf {
    let dir = fixture(name, false);
    fs::write(dir.join("grund.toml"), V1).expect("write config");
    fs::write(dir.join("requirements.md"), REQUIREMENTS).expect("write requirements");
    dir
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names = fs::read_dir(dir)
        .expect("read fixture")
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    names.sort();
    names
}

/// §FS-config-migrate.7: with no config anywhere up the walk there is nothing to
/// migrate, and creating one is `init`'s, so the run exits 2 and writes nothing.
#[test]
fn no_config_is_exit_two_and_names_init() {
    let dir = fixture("no-config", true);
    let out = grund(&["config", "migrate", "--to", "2", dir.to_str().unwrap()]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "stderr: {stderr}");
    assert!(out.stdout.is_empty(), "stdout: {}", text(&out.stdout));
    assert!(
        stderr.starts_with("error: no grund.toml at ")
            && stderr.ends_with(" or above it; grund init writes one\n"),
        "stderr: {stderr}"
    );
    assert!(
        entries(&dir).is_empty(),
        "the run wrote {:?}",
        entries(&dir)
    );
}

/// §FS-config-migrate.2.1: a preview prints the candidate and touches nothing,
/// which is the read-only half of §REQ-no-data-loss.1.
#[test]
fn preview_writes_nothing() {
    let dir = v1_tree("preview");
    let out = grund(&["config", "migrate", "--to", "2", dir.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    assert!(text(&out.stdout).starts_with("grund_config_version = 2\n"));
    assert_eq!(fs::read_to_string(dir.join("grund.toml")).unwrap(), V1);
    assert_eq!(entries(&dir), ["grund.toml", "requirements.md"]);
}

/// §FS-config-migrate.6.2: the write goes through a temporary file in the same
/// directory and a rename, keeping the original's permissions and leaving no
/// temporary file; the bytes written are the preview's stdout.
#[cfg(unix)]
#[test]
fn write_keeps_permissions_and_leaves_no_temporary_file() {
    use std::os::unix::fs::PermissionsExt;
    let dir = v1_tree("write");
    let config = dir.join("grund.toml");
    fs::set_permissions(&config, fs::Permissions::from_mode(0o640)).unwrap();
    let preview = grund(&["config", "migrate", "--to", "2", dir.to_str().unwrap()]);
    assert_eq!(
        preview.status.code(),
        Some(0),
        "stderr: {}",
        text(&preview.stderr)
    );

    let out = grund(&[
        "config",
        "migrate",
        "--to",
        "2",
        "--write",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", text(&out.stderr));
    assert_eq!(fs::read(&config).unwrap(), preview.stdout);
    let mode = fs::metadata(&config).unwrap().permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o640,
        "the write must keep the original's permissions"
    );
    assert_eq!(entries(&dir), ["grund.toml", "requirements.md"]);
}

/// §FS-config-migrate.6.6: a second run finds version 2, says so, exits 0 with
/// nothing on stdout, and leaves the file and its modification time alone.
#[test]
fn second_write_is_a_no_op() {
    let dir = v1_tree("twice");
    let config = dir.join("grund.toml");
    let first = grund(&[
        "config",
        "migrate",
        "--to",
        "2",
        "--write",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(
        first.status.code(),
        Some(0),
        "stderr: {}",
        text(&first.stderr)
    );
    let bytes = fs::read(&config).unwrap();
    let modified = fs::metadata(&config).unwrap().modified().unwrap();

    let second = grund(&[
        "config",
        "migrate",
        "--to",
        "2",
        "--write",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(
        second.status.code(),
        Some(0),
        "stderr: {}",
        text(&second.stderr)
    );
    assert!(second.stdout.is_empty(), "stdout: {}", text(&second.stdout));
    assert_eq!(
        text(&second.stderr),
        "grund.toml is already version 2; nothing to migrate\n"
    );
    assert_eq!(fs::read(&config).unwrap(), bytes);
    assert_eq!(fs::metadata(&config).unwrap().modified().unwrap(), modified);
}

/// §FS-config-migrate.7: a config that cannot be read is exit 2, with the
/// original untouched; read permission is the one failure a test can arrange.
#[cfg(unix)]
#[test]
fn unreadable_config_is_exit_two_and_untouched() {
    use std::os::unix::fs::PermissionsExt;
    let dir = v1_tree("unreadable");
    let config = dir.join("grund.toml");
    fs::set_permissions(&config, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&config).is_ok() {
        // Running as root: permissions do not stop the read, so nothing is shown.
        fs::set_permissions(&config, fs::Permissions::from_mode(0o644)).unwrap();
        return;
    }
    let out = grund(&[
        "config",
        "migrate",
        "--to",
        "2",
        "--write",
        dir.to_str().unwrap(),
    ]);
    fs::set_permissions(&config, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(out.status.code(), Some(2), "stderr: {}", text(&out.stderr));
    assert!(out.stdout.is_empty());
    // The error `config validate` gives for the same file (§FS-config.4.3).
    assert_eq!(
        text(&out.stderr),
        "error: read grund.toml: Permission denied (os error 13)\n"
    );
    assert_eq!(fs::read_to_string(&config).unwrap(), V1);
    assert_eq!(entries(&dir), ["grund.toml", "requirements.md"]);
}

/// §FS-config-migrate.5: grund's own config migrates with both gates passing.
/// Without `--drop-comments` only the two comment blocks above `[scan] include`
/// and `extensions` refuse; with it the preview exits 0. Nothing is written.
/// agent-grounds/grund#464 committed the migration, so the v1 file is frozen as
/// it stood before it, in `tests/fixtures/config_migrate/repo/`, and the gates
/// run on a tree holding that config and the four places its `[scan] include`
/// names, which version 2 must find to tell a folder from a file.
#[test]
fn grunds_own_config_passes_both_gates() {
    let root = fixture("grunds-own", false);
    for folder in ["docs", "crates", "tests/bindings"] {
        fs::create_dir_all(root.join(folder)).expect("create included folder");
    }
    fs::write(root.join("AGENTS.md"), "").expect("write included file");
    let config = root.join("grund.toml");
    let frozen = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/config_migrate/repo/grund.toml");
    fs::copy(&frozen, &config).expect("copy grund's v1 config");
    let before = fs::read(&config).unwrap();

    // §FS-config-migrate.2.1: only comments refuse, so --full shows the candidate.
    let out = grund(&[
        "config",
        "migrate",
        "--to",
        "2",
        "--full",
        root.to_str().unwrap(),
    ]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "stderr: {stderr}");
    assert!(text(&out.stdout).contains("grund_config_version = 2\n"));
    assert!(!stderr.contains("refused:"), "stderr: {stderr}");
    assert!(!stderr.contains("differs:"), "stderr: {stderr}");
    let listed = stderr
        .lines()
        .filter(|line| line.starts_with("  grund.toml:"))
        .collect::<Vec<_>>();
    assert_eq!(listed.len(), 2, "listed comments: {listed:?}");
    assert_eq!(
        listed[0],
        concat!(
            "  grund.toml:172-176  above [scan] include: setting splits or merges into ",
            "version-2 settings: [schema.kinds.code] folders, docs, crates and tests/bindings; ",
            "files, AGENTS.md; README.md is a place of [schema.kinds.readme] already; ",
            "retain the prose, authorize the listed drops with --write --drop-comments, ",
            "then re-add it at one of these named destinations after a successful write ",
            "(subject to all other refusals and gates)"
        )
    );
    assert_eq!(
        listed[1],
        concat!(
            "  grund.toml:184-186  above [scan] extensions: setting splits or merges into ",
            "version-2 settings: [schema.sources] languages and extensions, definitions ",
            "[schema.sources.definitions.v1_markdown], [schema.sources.definitions.v1_c_family], ",
            "[schema.sources.definitions.v1_go], [schema.sources.definitions.v1_python], ",
            "[schema.sources.definitions.v1_ruby], [schema.sources.definitions.v1_lisp], ",
            "[schema.sources.definitions.v1_sql], [schema.sources.definitions.v1_haddock], ",
            "[schema.sources.definitions.v1_ldoc], [schema.sources.definitions.v1_plain] and ",
            "[schema.sources.definitions.v1_shell]; retain the prose, authorize the listed ",
            "drops with --write --drop-comments, then re-add it at one of these named ",
            "destinations after a successful write (subject to all other refusals and gates)"
        )
    );
    assert!(
        stderr.contains("\ncompared: the records are equal"),
        "stderr: {stderr}"
    );

    let dropped = grund(&[
        "config",
        "migrate",
        "--to",
        "2",
        "--drop-comments",
        root.to_str().unwrap(),
    ]);
    assert_eq!(
        dropped.status.code(),
        Some(0),
        "stderr: {}",
        text(&dropped.stderr)
    );
    assert_eq!(
        fs::read(&config).unwrap(),
        before,
        "a preview must not write"
    );
}
