//! Test module: `MigrateOpts::config_from`, the config a migrate run acts on
//! spelled from the directory a command is pasted in (§FS-config-migrate.6.5.1
//! item 1), found by the discovery the run itself runs (§FS-config-migrate.1.1).
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4.1).

use std::path::Path;

use crate::MigrateOpts;
use crate::model::format_path;
use crate::testing::{test_root, write};

const V1: &str = "grund_config_version = 1\n";

/// The options of a run on `path`; only the path matters to the spelling.
fn opts(path: &Path) -> MigrateOpts {
    MigrateOpts {
        path: path.to_path_buf(),
        write: false,
        drop_comments: false,
    }
}

/// §FS-config-migrate.6.5.1 item 1: a config in the directory keeps its bare
/// name, the deprecated location included (§FS-config.1).
#[test]
fn a_config_in_the_directory_is_named_as_itself() {
    let root = test_root("migrate-from-here");
    write(&root.join("grund.toml"), V1);
    assert_eq!(
        opts(&root).config_from(Some(&root)).as_deref(),
        Some("grund.toml")
    );

    let agents = test_root("migrate-from-agents");
    write(&agents.join(".agents/grund.toml"), V1);
    assert_eq!(
        opts(&agents).config_from(Some(&agents)).as_deref(),
        Some(".agents/grund.toml")
    );
}

/// §FS-config-migrate.6.5.1 item 1: a config below the directory is named
/// through it, one above or beside it is reached with `..`.
#[test]
fn a_config_elsewhere_is_named_from_the_directory() {
    let root = test_root("migrate-from-elsewhere");
    write(&root.join("apps/api/grund.toml"), V1);
    write(&root.join("apps/web/docs/x.md"), "# x\n");
    let member = opts(&root.join("apps/api"));
    assert_eq!(
        member.config_from(Some(&root)).as_deref(),
        Some("apps/api/grund.toml")
    );
    assert_eq!(
        member
            .config_from(Some(&root.join("apps/web/docs")))
            .as_deref(),
        Some("../../api/grund.toml")
    );

    let below = test_root("migrate-from-below");
    write(&below.join("grund.toml"), V1);
    write(&below.join("docs/x.md"), "# x\n");
    let docs = below.join("docs");
    assert_eq!(
        opts(&docs).config_from(Some(&docs)).as_deref(),
        Some("../grund.toml"),
        "discovery walks up from docs, and the undo climbs back"
    );
}

/// §FS-config-migrate.6.5.1 item 1: the directory is resolved before it is
/// compared, so a link to a directory below the config adds no `..` for the
/// path it was reached by.
#[cfg(unix)]
#[test]
fn a_linked_directory_is_compared_where_it_resolves() {
    let root = test_root("migrate-from-link");
    write(&root.join("project/grund.toml"), V1);
    write(&root.join("project/docs/x.md"), "# x\n");
    std::fs::create_dir_all(root.join("far/away")).unwrap();
    let link = root.join("far/away/docs");
    std::os::unix::fs::symlink(root.join("project/docs"), &link).unwrap();
    assert_eq!(
        opts(&link).config_from(Some(&link)).as_deref(),
        Some("../grund.toml")
    );
}

/// §FS-config-migrate.6.5.1 item 1: a `PATH` through a link inside the project's
/// tree is spelled to where the config is, the file git tracks, and not through
/// the link git refuses to look past.
#[cfg(unix)]
#[test]
fn a_path_through_a_link_names_the_config_where_it_is() {
    let root = test_root("migrate-from-path-link");
    write(&root.join("services/api/grund.toml"), V1);
    std::fs::create_dir_all(root.join("apps")).unwrap();
    std::os::unix::fs::symlink("../services/api", root.join("apps/api")).unwrap();
    assert_eq!(
        opts(&root.join("apps/api"))
            .config_from(Some(&root))
            .as_deref(),
        Some("services/api/grund.toml")
    );
}

/// §FS-config-migrate.6.5.1 item 1: a process with no working directory has no
/// relative spelling, so the config is named by its absolute path, never by the
/// report base's.
#[test]
fn no_working_directory_names_the_absolute_path() {
    let root = test_root("migrate-from-nowhere");
    write(&root.join("apps/api/grund.toml"), V1);
    let absolute = std::fs::canonicalize(root.join("apps/api"))
        .unwrap()
        .join("grund.toml");
    assert_eq!(
        opts(&root.join("apps/api")).config_from(None),
        Some(format_path(&absolute))
    );
}

/// §FS-config-migrate.1.1: where discovery finds no config there is nothing
/// to name.
#[test]
fn no_config_names_nothing() {
    let root = test_root("migrate-from-none");
    write(&root.join("docs/x.md"), "# x\n");
    assert_eq!(opts(&root).config_from(Some(&root)), None);
    assert_eq!(opts(&root).config_from(None), None);
}
