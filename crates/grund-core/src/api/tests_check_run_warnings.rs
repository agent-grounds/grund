//! Check warning-channel API contracts (§FS-distribution.3.1).

use anyhow::Result;

use super::{CheckOpts, CheckOutput, check_with_opts, check_with_run_warnings};
use crate::Finding;
use crate::testing::{test_root, write};

const ID_RULES: &str = "[reference]\nmarker = \"§\"\nstrict = true\n\n\
     [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n";

fn opts(path: std::path::PathBuf) -> CheckOpts {
    CheckOpts {
        path,
        path_provided: true,
        ..CheckOpts::default()
    }
}

fn warning(code: &'static str, line: usize, message: &str) -> Finding {
    Finding {
        severity: "warning",
        code,
        path: Some("grund.toml".to_string()),
        line: Some(line),
        column: None,
        message: message.to_string(),
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// §FS-check.3.30.1: the absorbed scan's config error, as `{err:#}` renders it.
const ABSORBED_ERROR: &str = "grund.toml:16: [workspace] members swallows this project's whole scan — \
     every scan root is inside a member: `docs` in `docs` — so its declarations \
     are unreachable and its citations are never checked. Point [scan] include \
     at a directory that is not a member, or set include_root = false; this \
     became an error in grund 0.16.0";

fn unread_root_warning() -> Finding {
    warning(
        "unread-workspace-block",
        17,
        "grund.toml:17: no project scans `docs`, so its citations are never checked. \
         Set include_root = true, or point another project's [scan] include at it.",
    )
}

fn absorbed_root(name: &str, broken_member: bool) -> std::path::PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"root\"\n\n{ID_RULES}\n\
             [scan]\ninclude = [\"docs\"]\n\n[workspace]\nmembers = [\"docs\"]\n"
        ),
    );
    write(
        &root.join("docs/FS-root.md"),
        "# FS-root: Root concern\n\nThe root scan cannot reach this declaration.\n",
    );
    let member_config = if broken_member {
        "grund_config_version = 1\nproject_name = \"docs\"\n\n[workspace]\nmembers = [\"/broken\"]\n"
    } else {
        "grund_config_version = 1\nproject_name = \"docs\"\n"
    };
    write(&root.join("docs/grund.toml"), member_config);
    root
}

fn unread_root(name: &str, broken_member: bool) -> std::path::PathBuf {
    let root = test_root(name);
    let members = if broken_member {
        "[\"alpha\", \"broken\"]"
    } else {
        "[\"alpha\"]"
    };
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"root\"\n\n{ID_RULES}\n\
             [scan]\ninclude = [\"docs\"]\n\n[workspace]\nmembers = {members}\n\
             include_root = false\n"
        ),
    );
    write(
        &root.join("docs/FS-root.md"),
        "# FS-root: Unread root declaration\n\nNothing scans this file.\n",
    );
    write(
        &root.join("alpha/grund.toml"),
        "grund_config_version = 1\nproject_name = \"alpha\"\n",
    );
    write(
        &root.join("alpha/docs/FS-alpha.md"),
        "# FS-alpha: Alpha\n\nAlpha.\n",
    );
    if broken_member {
        write(
            &root.join("broken/grund.toml"),
            "grund_config_version = 1\nproject_name = \"broken\"\n\n[workspace]\n\
             members = [\"/broken\"]\n",
        );
    }
    root
}

#[test]
fn check_with_opts_keeps_its_result_output_contract() {
    let root = test_root("check_with_opts_keeps_its_result_output_contract");
    write(&root.join("grund.toml"), "grund_config_version = 1\n");

    let output: Result<CheckOutput> = check_with_opts(CheckOpts {
        path: root,
        path_provided: true,
        ..CheckOpts::default()
    });

    assert!(
        output.is_ok(),
        "the established check API still returns output"
    );
}

/// §FS-check.3.30.2: the absorbed scan is asked of the run's root block before
/// its nested members expand, so when a later expansion refusal would also apply
/// the absorbed scan is the one error reported, and no warning is left behind.
#[test]
fn absorbed_root_error_wins_over_a_later_expansion_refusal() {
    let root = absorbed_root("check-warning-absorbed-refusal", true);

    let (warnings, result) = check_with_run_warnings(opts(root));

    assert_eq!(warnings, Vec::<Finding>::new());
    assert_eq!(
        format!(
            "{:#}",
            result.expect_err("the absorbed scan must be refused")
        ),
        ABSORBED_ERROR
    );
}

/// §FS-check.4.10.8: the run root's unread-block question is answerable from its
/// populated boundary and survives a later nested expansion refusal.
#[test]
fn answerable_unread_root_warning_survives_a_later_expansion_refusal() {
    let root = unread_root("check-warning-unread-root-refusal", true);

    let (warnings, result) = check_with_run_warnings(opts(root));

    assert_eq!(warnings, vec![unread_root_warning()]);
    assert_eq!(
        format!(
            "{:#}",
            result.expect_err("the nested member must be refused")
        ),
        "broken/grund.toml:5: invalid [workspace] member `/broken` \
         (expected relative path or trailing /* glob)"
    );
}

/// §FS-check.3.30, §FS-distribution.3.1: with every member valid the absorbed
/// scan still refuses the run, on the error path and on neither warning channel.
#[test]
fn absorbed_root_success_is_refused_with_no_warning() {
    let root = absorbed_root("check-warning-absorbed-success", false);

    let (warnings, result) = check_with_run_warnings(opts(root));

    assert_eq!(warnings, Vec::<Finding>::new());
    assert_eq!(
        format!(
            "{:#}",
            result.expect_err("the absorbed scan must be refused")
        ),
        ABSORBED_ERROR
    );
}

/// §FS-check.4.10.8, §FS-distribution.3.1: a successful root unread-block answer
/// has identical bytes, location, order, and cardinality on both channels.
#[test]
fn unread_root_success_returns_the_same_warning_on_both_channels() {
    let root = unread_root("check-warning-unread-root-success", false);

    let (warnings, result) = check_with_run_warnings(opts(root));
    let output = result.expect("check the valid workspace");

    assert_eq!(warnings, vec![unread_root_warning()]);
    assert_eq!(warnings, output.warnings);
}
