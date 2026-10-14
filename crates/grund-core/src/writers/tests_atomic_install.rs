//! Test module: the shared atomic writer (§AR-config.8.3). A failure injected at
//! each step — the temporary write, the sync, the rename — and an edit made to
//! the target just before the rename each leave the original byte for byte and
//! no temporary file behind (§FS-config-migrate.6.2, §FS-config-migrate.6.3); the
//! install that succeeds keeps the original's permissions.

use std::path::{Path, PathBuf};

use super::atomic_install::{FAULT, Install, InstallFailure, Stage, atomic_install};
use crate::testing::{test_root, write};

const ORIGINAL: &str = "grund_config_version = 1\n";
const REPLACEMENT: &str = "grund_config_version = 2\n";

fn target(name: &str) -> (PathBuf, PathBuf) {
    let root = test_root(name);
    let path = root.join("grund.toml");
    write(&path, ORIGINAL);
    (root, path)
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("read fixture")
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Install `REPLACEMENT` over the target as migration does, expecting `ORIGINAL`,
/// with `stage` failing and `edit` written to the target before the rename check.
fn install_with(
    path: &Path,
    stage: Option<Stage>,
    edit: Option<&str>,
) -> Result<(), InstallFailure> {
    FAULT.with(|fault| *fault.borrow_mut() = (stage, edit.map(|edit| edit.as_bytes().to_vec())));
    let install = Install {
        bytes: REPLACEMENT.as_bytes(),
        tag: "migrate",
        expected: Some(ORIGINAL.as_bytes()),
    };
    let result = atomic_install(path, install);
    FAULT.with(|fault| *fault.borrow_mut() = (None, None));
    result
}

/// §FS-config-migrate.6.2: a failure at the temporary write, the sync or the
/// rename leaves the original byte for byte, and no temporary file.
#[test]
fn a_failure_at_any_step_leaves_the_original_and_no_temporary_file() {
    for stage in [Stage::Write, Stage::Sync, Stage::Rename] {
        let (root, path) = target(&format!("atomic-install-{stage:?}"));
        let result = install_with(&path, Some(stage), None);
        assert!(
            matches!(result, Err(InstallFailure::Io { .. })),
            "{stage:?}: {result:?}"
        );
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            ORIGINAL,
            "{stage:?}"
        );
        assert_eq!(entries(&root), ["grund.toml"], "{stage:?}");
    }
}

/// §FS-config-migrate.6.3: the config is read again just before the rename, and
/// an edit made since the run read it stops the write with the edit kept and the
/// temporary file removed.
#[test]
fn a_concurrent_edit_stops_the_rename_and_keeps_the_edit() {
    let (root, path) = target("atomic-install-concurrent");
    let edit = "grund_config_version = 1\nproject_name = \"edited\"\n";
    let result = install_with(&path, None, Some(edit));
    assert!(matches!(result, Err(InstallFailure::Changed)), "{result:?}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), edit);
    assert_eq!(entries(&root), ["grund.toml"]);
}

/// §FS-config-migrate.6.3: a target that no longer holds the bytes the run read
/// is refused before any temporary file exists.
#[test]
fn a_target_changed_before_the_install_is_refused() {
    let (root, path) = target("atomic-install-changed-before");
    write(&path, "edited\n");
    let result = install_with(&path, None, None);
    assert!(matches!(result, Err(InstallFailure::Changed)), "{result:?}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "edited\n");
    assert_eq!(entries(&root), ["grund.toml"]);
}

/// §FS-config-migrate.6.2: the install that succeeds replaces the bytes, keeps
/// the original's permissions, and leaves no temporary file.
#[cfg(unix)]
#[test]
fn a_successful_install_keeps_the_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let (root, path) = target("atomic-install-success");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
    install_with(&path, None, None).expect("install");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), REPLACEMENT);
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o640);
    assert_eq!(entries(&root), ["grund.toml"]);
}
