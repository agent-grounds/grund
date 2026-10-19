//! Test module: `fmt` refused for a conflict block across the public API
//! (§FS-fmt.7.4, §FS-fmt.3.3, §FS-distribution.3.3.2), and `check`'s finding
//! beside it (§FS-declarations.checks.conflict-marker). Every marker line here is
//! inside an indented string literal, so no line of this file is itself one.

use std::path::Path;

use super::*;
use crate::testing::{test_root, write};
use crate::writers::{FmtConflictAbort, FmtScanAbort};

/// One conflict block, five lines long.
const BLOCK: &str = "<<<<<<< HEAD\nA.\n=======\nB.\n>>>>>>> other\n";
const WORKSPACE: &str = concat!(
    "grund_config_version = 1\n\n[workspace]\nmembers = [\"packages/sub\"]\n\n",
    "[scan]\ninclude = [\"docs\"]\n",
);
const MEMBER: &str = "grund_config_version = 1\n\n[scan]\ninclude = [\"docs\"]\n";

/// A declaration whose body holds `blocks` conflict blocks, the first at line 3.
fn conflicted(heading: &str, blocks: usize) -> String {
    format!("{heading}\n\n{}", vec![BLOCK; blocks].join("\n"))
}

fn read(root: &Path, path: &str) -> String {
    std::fs::read_to_string(root.join(path)).expect("read a fixture back")
}

fn fmt_write(root: &Path, cross_refs: bool) -> anyhow::Error {
    format_references(FmtOpts {
        path: root.to_path_buf(),
        path_provided: true,
        write: true,
        cross_refs,
        ..FmtOpts::default()
    })
    .expect_err("the run is refused")
}

/// §FS-fmt.3.3: a workspace-root run names every block, the root's first and then
/// each member's, files then lines within a project; nothing is written; and the
/// bindings' failure is located at the first block with one site per block
/// (§FS-distribution.3.3.2).
#[test]
fn a_workspace_refusal_names_every_block_root_first_files_then_lines() {
    let root = test_root("a_workspace_refusal_names_every_block_root_first_files_then_lines");
    write(&root.join("grund.toml"), WORKSPACE);
    write(&root.join("packages/sub/grund.toml"), MEMBER);
    let use_md = "# Use\n\nCites $$FS-001-alpha.\n";
    write(&root.join("docs/use.md"), use_md);
    let beta = conflicted("# FS-002-beta: Beta", 1);
    write(&root.join("docs/FS-002-beta.md"), &beta);
    write(
        &root.join("docs/FS-001-alpha.md"),
        &conflicted("# FS-001-alpha: Alpha", 2),
    );
    write(
        &root.join("packages/sub/docs/FS-003-gamma.md"),
        &conflicted("# FS-003-gamma: Gamma", 1),
    );

    let error = fmt_write(&root, true);
    let abort = error
        .downcast_ref::<FmtConflictAbort>()
        .expect("refused for its blocks");
    let located: Vec<(Option<&str>, Option<usize>)> = abort
        .conflicts
        .iter()
        .map(|conflict| (conflict.path.as_deref(), conflict.line))
        .collect();
    assert_eq!(
        located,
        vec![
            (Some("docs/FS-001-alpha.md"), Some(3)),
            (Some("docs/FS-001-alpha.md"), Some(9)),
            (Some("docs/FS-002-beta.md"), Some(3)),
            (Some("packages/sub/docs/FS-003-gamma.md"), Some(3)),
        ]
    );
    assert_eq!(
        read(&root, "docs/use.md"),
        use_md,
        "the `$$` is not rewritten"
    );
    assert_eq!(read(&root, "docs/FS-002-beta.md"), beta);

    let failure = super::embedding_failure::error_data(error, &[]);
    assert_eq!(failure["kind"], "operation");
    assert_eq!(failure["code"], "conflict-marker");
    assert_eq!(failure["path"], "docs/FS-001-alpha.md");
    assert_eq!(failure["line"], 3);
    let site_lines: Vec<_> = failure["sites"]
        .as_array()
        .expect("sites")
        .iter()
        .map(|site| site["line"].clone())
        .collect();
    assert_eq!(site_lines, vec![3, 9, 3, 3]);
    assert_eq!(
        failure["partial_output"],
        serde_json::json!({"changes": [], "scan_errors": [], "refused_writes": []})
    );
    assert_eq!(
        failure["details"]["conflicts"].as_array().map(Vec::len),
        Some(4)
    );
}

/// §FS-fmt.3.3: an unreadable path anywhere in a workspace-root run wins, so a
/// member that cannot be read refuses the run for that path alone and the
/// root's block goes unnamed.
#[test]
fn an_unreadable_member_path_wins_over_a_root_block() {
    let root = test_root("an_unreadable_member_path_wins_over_a_root_block");
    write(&root.join("grund.toml"), WORKSPACE);
    write(&root.join("packages/sub/grund.toml"), MEMBER);
    write(
        &root.join("docs/FS-001-alpha.md"),
        &conflicted("# FS-001-alpha: Alpha", 1),
    );
    write(
        &root.join("packages/sub/docs/FS-002-beta.md"),
        "# FS-002-beta: Beta\n\nCites $$FS-002-beta.\n",
    );
    std::os::unix::fs::symlink("nowhere.md", root.join("packages/sub/docs/FS-009-gone.md"))
        .expect("create a broken symlink");

    let error = fmt_write(&root, true);
    assert!(error.downcast_ref::<FmtConflictAbort>().is_none());
    let abort = error
        .downcast_ref::<FmtScanAbort>()
        .expect("refused for the unreadable path");
    let paths: Vec<&str> = abort.scan_errors.iter().map(|e| e.path.as_str()).collect();
    assert_eq!(paths, vec!["packages/sub/docs/FS-009-gone.md"]);
}

/// §FS-declarations.checks.conflict-marker: a member's owners are spelled as the
/// member spells them, by `check` and by `fmt`'s refusal, in a workspace-root run.
#[test]
fn a_member_block_names_its_owner_in_the_member_spelling() {
    let root = test_root("a_member_block_names_its_owner_in_the_member_spelling");
    write(&root.join("grund.toml"), WORKSPACE);
    write(
        &root.join("packages/sub/grund.toml"),
        "grund_config_version = 1\n\n[id]\nformat = \"{kind}_{slug}\"\n\n[scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("docs/FS-001-alpha.md"),
        "# FS-001-alpha: Alpha\n\nAlpha.\n",
    );
    write(
        &root.join("packages/sub/docs/beta.md"),
        &conflicted("# FS_beta: Beta", 1),
    );
    let message = "unresolved merge conflict in FS_beta, lines 3\u{2013}7: resolve it by hand";

    let report = check(&root).expect("check the workspace");
    let found: Vec<(&str, Option<&str>)> = report
        .errors
        .iter()
        .filter(|finding| finding.code == "conflict-marker")
        .map(|finding| (finding.message.as_str(), finding.path.as_deref()))
        .collect();
    assert_eq!(found, vec![(message, Some("packages/sub/docs/beta.md"))]);

    let error = fmt_write(&root, true);
    let abort = error
        .downcast_ref::<FmtConflictAbort>()
        .expect("refused for the member's block");
    assert_eq!(abort.conflicts.len(), 1);
    assert_eq!(abort.conflicts[0].message, message);
}

const LOCKED: &str = concat!(
    "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n",
    "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nindex = false\n\n",
    "[fmt.cross_refs]\nenabled = false\n",
);

/// §FS-fmt.7.4.3: with cross-references off the section-lock pass is where the
/// run first needs the declaration set; it refuses on the block after the walk
/// planned the `$$` rewrite, and neither that file nor the lock is written.
#[test]
fn the_lock_pass_refuses_a_block_with_nothing_written() {
    let root = test_root("the_lock_pass_refuses_a_block_with_nothing_written");
    write(&root.join("grund.toml"), LOCKED);
    let use_md = "# Use\n\nUsernames follow \u{a7}FS-login.format.1.\nSee also $$FS-login.\n";
    write(&root.join("docs/use.md"), use_md);
    let login = format!("# FS-login: Login\n\n## format: Format\n\n### .1: Username\n\n{BLOCK}");
    write(&root.join("docs/functional-spec/FS-login.md"), &login);
    let lock = "FS-login.format.1 0000000000000000\n";
    write(&root.join(".grund/sections/FS-login.lock"), lock);

    let error = fmt_write(&root, false);
    let abort = error
        .downcast_ref::<FmtConflictAbort>()
        .expect("refused for the block");
    assert_eq!(abort.conflicts[0].line, Some(7));
    assert_eq!(read(&root, "docs/use.md"), use_md);
    assert_eq!(read(&root, ".grund/sections/FS-login.lock"), lock);
}

/// §FS-fmt.7.4.3: a shorthand defers the declaration-set scan to the file that
/// holds it, after an earlier file's `$$` was planned; the block refuses there
/// and the earlier file is not written.
#[test]
fn the_deferred_shorthand_scan_refuses_a_block_before_any_write() {
    let root = test_root("the_deferred_shorthand_scan_refuses_a_block_before_any_write");
    write(
        &root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\nproject_name = \"sh\"\n\n",
            "[reference]\nstrict = true\nshorthand = \"canonical\"\n\n",
            "[id]\nformat = \"{kind}-{number}-{slug}\"\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nindex = false\n\n",
            "[fmt.cross_refs]\nenabled = false\n",
        ),
    );
    let first = "# Notes\n\nSee $$FS-042-user-login.\n";
    write(&root.join("docs/a.md"), first);
    write(&root.join("docs/z.md"), "# More\n\nSee \u{a7}FS-042.\n");
    write(
        &root.join("docs/functional-spec/FS-042-user-login.md"),
        &conflicted("# FS-042-user-login: Login", 1),
    );

    let error = fmt_write(&root, false);
    assert!(error.downcast_ref::<FmtConflictAbort>().is_some());
    assert_eq!(read(&root, "docs/a.md"), first);
}
