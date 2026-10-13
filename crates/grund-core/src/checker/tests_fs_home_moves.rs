//! Test module: the fs-home-moves warning (§FS-check.checks.fs-home-moves) — when
//! it fires, that it fires once per project, and that the release it names is
//! still ahead (§FS-check.checks.fs-home-moves.2).

use super::fs_home_moves::FS_HOME_FLIP;
use crate::testing::{codes, only, test_root, write, zero_config_check_run};

fn warnings(run: &crate::api::CheckRun) -> usize {
    codes(run)
        .iter()
        .filter(|code| *code == "fs-home-moves")
        .count()
}

/// §FS-check.checks.fs-home-moves: no config file, and an `FS` declaration in
/// the built-in `requirements.md` home — the one case the 0.19.0 move breaks.
#[test]
fn a_zero_config_fs_declaration_in_requirements_md_is_warned() {
    let root = test_root("fs_home_moves_trigger");
    write(
        &root.join("requirements.md"),
        "# Requirements\n\n## FS-001-login: Login\n\nBody.\n",
    );
    let run = zero_config_check_run(&root);
    let warning = only(&run, "fs-home-moves");
    assert_eq!(warning.line, Some(3));
    assert!(
        warning
            .message
            .starts_with("FS-001-login is in requirements.md, the FS home only by default"),
        "{}",
        warning.message
    );
    assert!(run.report.errors.is_empty(), "{:?}", codes(&run));
}

/// §FS-check.checks.fs-home-moves.1: a config file of any shape names its own
/// `FS` home or keeps the older folder, so the same declarations earn nothing.
#[test]
fn a_config_file_silences_it() {
    let root = test_root("fs_home_moves_config");
    write(
        &root.join("requirements.md"),
        "# Requirements\n\n## FS-001-login: Login\n\nBody.\n",
    );
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[[kinds]]\nkind = \"FS\"\nfile = \"requirements.md\"\n",
    );
    assert_eq!(warnings(&zero_config_check_run(&root)), 0);
}

/// §FS-check.checks.fs-home-moves: one edit clears every declaration, so the
/// project earns one warning, at the lowest line, whatever the IDs sort as.
#[test]
fn one_warning_per_project_at_the_lowest_line() {
    let root = test_root("fs_home_moves_once");
    write(
        &root.join("requirements.md"),
        "# Requirements\n\n## FS-009-first: First\n\nBefore §FS-001-second.\n\n\
         ## FS-001-second: Second\n\nAfter §FS-009-first.\n\n\
         ## FS-005-third: Third\n\nAfter §FS-001-second.\n",
    );
    let run = zero_config_check_run(&root);
    assert_eq!(warnings(&run), 1, "{:?}", codes(&run));
    let warning = only(&run, "fs-home-moves");
    assert_eq!(warning.line, Some(3));
    assert!(
        warning.message.starts_with("FS-009-first is in"),
        "{}",
        warning.message
    );
}

/// §FS-check.checks.fs-home-moves.2: the message names a release still to come;
/// once this crate reaches it the home has moved, and the check must go.
#[test]
fn the_fs_home_flip_is_still_ahead() {
    let parse = |version: &str| -> Vec<u64> {
        version
            .split(['.', '-'])
            .take(3)
            .map(|part| part.parse().expect("numeric version"))
            .collect()
    };
    assert!(
        parse(env!("CARGO_PKG_VERSION")) < parse(FS_HOME_FLIP),
        "grund {} has reached {FS_HOME_FLIP}: remove fs-home-moves and move the FS home",
        env!("CARGO_PKG_VERSION")
    );
}
