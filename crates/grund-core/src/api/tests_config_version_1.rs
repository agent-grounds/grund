//! Test module: the caution a config file read as version 1 earns
//! (§FS-check.checks.config-version-1, §FS-config.5.3) — for an explicit `1` and
//! an omitted version, never for version 2 or a project with no config file,
//! and in the same bytes `config migrate` prints for a member still on version 1.

use std::path::{Path, PathBuf};

use super::check::{CheckOpts, check_with_projects};
use super::config_findings::config_diagnostics;
use crate::config::load_config;
use crate::model::Diagnostic;
use crate::testing::{test_root, write};
use crate::{MigrateOpts, MigrateOutcome, config_migrate};

/// A project whose config is `config` at `file` (none when `None`), under a
/// fresh root.
fn project(name: &str, file: &str, config: Option<&str>) -> PathBuf {
    let root = test_root(&format!("config-version-1-{name}"));
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nText.\n");
    if let Some(config) = config {
        write(&root.join(file), config);
    }
    root
}

/// The `config-version-1` findings the config at `root` carries.
fn cautions(root: &Path) -> Vec<Diagnostic> {
    let config = load_config(root).expect("the config loads");
    config_diagnostics(&config, config.display())
        .filter(|d| d.code == "config-version-1")
        .collect()
}

/// §FS-check.checks.config-version-1: an explicit `1` is cautioned, sited
/// nowhere, and a single project's line ends at `--to 2`.
#[test]
fn an_explicit_version_1_is_cautioned() {
    let root = project("explicit", "grund.toml", Some("grund_config_version = 1\n"));
    let found = cautions(&root);
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].message,
        "grund.toml is version 1; migrate it with grund config migrate --to 2"
    );
    assert_eq!((found[0].path.as_deref(), found[0].line), (None, None));
}

/// §FS-config.5.3: a file that writes no version is read as version 1, and is
/// cautioned the same.
#[test]
fn an_omitted_version_is_cautioned() {
    let root = project(
        "omitted",
        "grund.toml",
        Some("[scan]\ninclude = [\"docs\"]\n"),
    );
    assert_eq!(cautions(&root).len(), 1);
}

/// §FS-check.checks.config-version-1: never for a version 2 file.
#[test]
fn version_2_is_not_cautioned() {
    let root = project("v2", "grund.toml", Some("grund_config_version = 2\n"));
    assert!(cautions(&root).is_empty());
}

/// §FS-config.5.3, §GOAL-zero-config: no file, no caution, though the defaults
/// a zero-config project loads are version 1's.
#[test]
fn no_config_file_is_not_cautioned() {
    let root = project("none", "grund.toml", None);
    assert!(cautions(&root).is_empty());
}

/// §FS-check.checks.config-version-1.1: `.agents/grund.toml` names the project
/// root, not `.agents/`, after the location warning.
#[test]
fn an_agents_config_names_the_project_root_after_the_location_warning() {
    let root = project(
        "agents",
        ".agents/grund.toml",
        Some("grund_config_version = 1\n"),
    );
    let config = load_config(&root).expect("the config loads");
    let found: Vec<_> = config_diagnostics(&config, config.display())
        .map(|d| (d.code, d.message))
        .collect();
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].0, "deprecated-config-location");
    assert_eq!(
        found[1],
        (
            "config-version-1",
            ".agents/grund.toml is version 1; migrate it with grund config migrate --to 2"
                .to_string()
        )
    );
}

/// §FS-check.checks.config-version-1, §FS-config-migrate.8: the caution `check`
/// prints for a v1 member and the member line `config migrate` prints for it
/// are the same bytes.
#[test]
fn the_member_caution_and_the_migrate_member_line_are_the_same_bytes() {
    let root = test_root("config-version-1-member-line");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[scan]\ninclude = [\"docs\"]\n\n\
         [id]\nnamed_sections = true\n\n[workspace]\nmembers = [\"packages/beta\"]\n",
    );
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nText.\n");
    write(
        &root.join("packages/beta/grund.toml"),
        "grund_config_version = 1\n\n[scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("packages/beta/docs/FS-b.md"),
        "# FS-b: B\n\nText.\n",
    );

    let (warnings, output) = check_with_projects(CheckOpts {
        path: root.clone(),
        path_provided: true,
        ..CheckOpts::default()
    });
    assert!(warnings.is_empty());
    let (output, _) = output.expect("the workspace checks");
    let cautioned: Vec<_> = output
        .report
        .warnings
        .iter()
        .filter(|f| f.code == "config-version-1")
        .map(|f| f.message.clone())
        .collect();
    let member = "packages/beta/grund.toml is version 1; \
                  migrate it with grund config migrate --to 2 packages/beta";
    assert_eq!(
        cautioned,
        [
            "grund.toml is version 1; migrate it with grund config migrate --to 2",
            member
        ]
    );

    let outcome = config_migrate(&MigrateOpts {
        path: root.clone(),
        write: false,
        drop_comments: false,
    })
    .expect("the preview runs");
    let MigrateOutcome::Migrated(report) = outcome else {
        panic!("the root is on version 1");
    };
    assert_eq!(report.members, [member]);
}
