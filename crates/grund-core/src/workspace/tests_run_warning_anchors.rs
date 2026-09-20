//! Test module: where each of the four run-level `[workspace]` warnings anchors
//! (§FS-check.4.7.7, §FS-check.4.8.15, §FS-check.4.10.11, §FS-workspace.6.1.7).
//!
//! The *text* of each is pinned end to end, in `tests/e2e/cases/`, because a byte
//! on stderr is what a repository greps. The anchor is not on stderr at all: it
//! is the location the engine hands back so an editor places the warning without
//! reading one back out of the message (§FS-lsp.1.1.3, §AR-bindings.2). No golden
//! can see it, and the CLI renders the same bytes whether it is right or wrong —
//! so it is held here, one case per section that names one.
//!
//! Each case reads the anchor off the surface that carries that warning: three
//! travel on `check`'s own output, and §FS-check.4.8.13 is one of `check`'s report
//! warnings with a deliberately null location (§FS-errors.5.2), so its anchor is
//! read where it exists — the editor snapshot the server publishes from.

use std::path::{Path, PathBuf};

use crate::config::ConfigLocation;
use crate::model::format_path;
use crate::testing::{test_root, write};
use crate::{CheckOpts, LspSnapshotOpts, check_with_opts, lsp_snapshot};

const ID_RULES: &str = "[reference]\nmarker = \"§\"\nstrict = true\n\n\
     [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n";

/// The run's warnings for a `check` over `root`, as (display path, line) pairs
/// in the order the run settled them.
fn check_anchors(root: &Path) -> Vec<(Option<String>, Option<usize>)> {
    check_with_opts(CheckOpts {
        path: root.to_path_buf(),
        path_provided: true,
        ..CheckOpts::default()
    })
    .expect("check the fixture")
    .warnings
    .into_iter()
    .map(|warning| (warning.path, warning.line))
    .collect()
}

/// §FS-check.4.7.7: "It **anchors at the block's `members` line**." The fixture is
/// the shape of the `workspace-member-absorbs-scan-check` case — a block whose
/// one scan root is also its one member — with the `members` key put on a line
/// no other key could be mistaken for.
#[test]
fn an_absorbed_scan_anchors_at_the_blocks_members_line() {
    let root = test_root("anchor-absorbed-scan");
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"root\"\n\n{ID_RULES}\n\
             [scan]\ninclude = [\"docs\"]\n\n[workspace]\nmembers = [\"docs\"]\n"
        ),
    );
    write(
        &root.join("docs/FS-001-root.md"),
        "# FS-001-root: Root\n\nCited as §FS-001-root\n",
    );
    assert_eq!(
        check_anchors(&root),
        vec![(Some("grund.toml".to_string()), Some(16))],
        "the absorbed scan must anchor at the `members` line, not at the block header"
    );
}

/// §FS-check.4.10.11: "It **anchors at the block's `include_root` line**" — the key
/// that took the block's files out of every scan is the line to open, which
/// neither the `members` line above it nor the `[workspace]` header is.
#[test]
fn an_unread_opted_out_block_anchors_at_its_include_root_line() {
    let root = test_root("anchor-unread-block");
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"root\"\n\n{ID_RULES}\n\
             [scan]\ninclude = [\"docs\"]\n\n[workspace]\nmembers = [\"group\"]\n"
        ),
    );
    write(
        &root.join("docs/FS-001-root.md"),
        "# FS-001-root: Root\n\nCited as §FS-001-root\n",
    );
    write(
        &root.join("group/grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"group\"\n\n{ID_RULES}\n\
             [scan]\ninclude = [\"docs\"]\n\n\
             [workspace]\nmembers = [\"alpha\"]\ninclude_root = false\n"
        ),
    );
    write(
        &root.join("group/alpha/grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"alpha\"\n\n{ID_RULES}\n\
             [scan]\ninclude = [\"docs\"]\n"
        ),
    );
    write(
        &root.join("group/docs/FS-001-g.md"),
        "# FS-001-g: Group\n\nRead by nobody, and cited as §FS-001-g\n",
    );
    write(
        &root.join("group/alpha/docs/FS-001-a.md"),
        "# FS-001-a: Alpha\n\nCited as §FS-001-a\n",
    );
    assert_eq!(
        check_anchors(&root),
        vec![(Some("group/grund.toml".to_string()), Some(17))],
        "the unread block must anchor at its `include_root` line"
    );
}

/// §FS-check.4.10.11: the breadcrumb falls back to the block's `[workspace]` line
/// when there is no `include_root` key to point at — a shape the default `true`
/// makes unreachable through a config file, so it is held against the builder
/// rather than against a fixture no `grund.toml` can produce.
#[test]
fn an_unread_block_with_no_include_root_key_falls_back_to_the_workspace_line() {
    let root = test_root("anchor-unread-block-fallback");
    write(&root.join("grund.toml"), "grund_config_version = 1\n");
    let mut config = crate::config::Config::default_for(root.clone());
    config.workspace_declared = true;
    config.workspace_include_root = false;
    config.workspace_section_source = Some(ConfigLocation {
        path: PathBuf::from("grund.toml"),
        line: 9,
    });
    let diagnostic = super::findings::unread_block_diagnostic(&config, "docs");
    assert_eq!(diagnostic.line, Some(9));
    assert_eq!(diagnostic.path, Some(root.join("grund.toml")));
    assert!(
        diagnostic
            .message
            .starts_with("grund.toml:9: no project scans `docs`"),
        "the breadcrumb falls back to the same line the anchor does: {}",
        diagnostic.message
    );
}

/// §FS-check.4.8.15, §FS-lsp.1.1.3: the unlisted block anchors at its `[workspace]`
/// line — "the reader has two files to open and this is the one that is wrong".
/// `check`'s own report keeps the location null on purpose (§FS-errors.5.2), so
/// the anchor is read off the snapshot the editor publishes from.
#[test]
fn an_unlisted_block_anchors_at_its_workspace_line() {
    let root = test_root("anchor-unlisted-block");
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"root\"\n\n{ID_RULES}\n\
             [scan]\ninclude = [\"docs\", \"b\"]\n"
        ),
    );
    write(
        &root.join("docs/FS-001-root.md"),
        "# FS-001-root: Root\n\nCited as §FS-001-root\n",
    );
    write(
        &root.join("b/grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"b\"\n\n{ID_RULES}\n\
             [workspace]\nmembers = [\"c\"]\n"
        ),
    );
    write(
        &root.join("b/c/grund.toml"),
        "grund_config_version = 1\nproject_name = \"c\"\n",
    );
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: root.clone(),
        path_provided: true,
        ..LspSnapshotOpts::default()
    })
    .expect("snapshot the fixture");
    let anchors: Vec<(Option<&str>, Option<usize>)> = snapshot
        .run_warnings
        .iter()
        .map(|warning| (warning.path.as_deref(), warning.line))
        .collect();
    assert_eq!(
        anchors.len(),
        1,
        "one unlisted block, one warning: {anchors:?}"
    );
    let (path, line) = anchors[0];
    assert_eq!(line, Some(12), "the block's `[workspace]` line");
    assert!(
        path.is_some_and(|path| Path::new(path).ends_with("b/grund.toml")),
        "the block's own config, not the enclosing project's: {path:?}"
    );
}

/// §FS-workspace.6.1.7 / §FS-workspace.6.1.7.6: the undecidable ancestor claim
/// travels as one of the run's warnings and "anchors at the config it
/// could not read — that file and no line, because the `members` value whose
/// line would be the anchor is exactly what could not be obtained".
#[test]
fn an_undecidable_ancestor_claim_anchors_at_the_config_with_no_line() {
    let outer = test_root("anchor-undecidable-claim");
    write(
        &outer.join("grund.toml"),
        "grund_config_version = 1\nproject_name = \"stray\"\n\n\
         [workspace]\nmembers = \"not-a-list\"\n",
    );
    let root = outer.join("ws");
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"ws\"\n\n{ID_RULES}\n\
             [scan]\ninclude = [\"docs\"]\n\n[workspace]\nmembers = [\"a\"]\n"
        ),
    );
    write(
        &root.join("docs/FS-001-root.md"),
        "# FS-001-root: Root\n\nCited as §FS-001-root\n",
    );
    write(
        &root.join("a/grund.toml"),
        &format!(
            "grund_config_version = 1\nproject_name = \"a\"\n\n{ID_RULES}\n\
             [scan]\ninclude = [\"docs\"]\n"
        ),
    );
    write(
        &root.join("a/docs/FS-001-a.md"),
        "# FS-001-a: A\n\nCited as §FS-001-a\n",
    );
    let anchors = check_anchors(&root);
    assert_eq!(
        anchors.len(),
        1,
        "one unreadable ancestor, one warning: {anchors:?}"
    );
    let (path, line) = &anchors[0];
    assert_eq!(*line, None, "the line is what the run could not read");
    // §FS-errors.4: an anchor above the run's own root renders absolute, the way
    // every other out-of-root reported path does, while the message keeps the
    // `../grund.toml` breadcrumb a reader resolves from where they stand.

    // The climb starts from the canonical run root, so the ancestor comes out in
    // the same canonical, forward-slash form `format_path` renders every reported
    // path in: `/private/var/…` on macOS, `//?/C:/…` on Windows, the raw temp
    // path only where the two coincide. Compare in that form, not in the raw one.
    let expected = format_path(
        &std::fs::canonicalize(outer.join("grund.toml")).expect("canonical ancestor config"),
    );
    assert_eq!(
        path.as_deref(),
        Some(expected.as_str()),
        "the ancestor config, not the run's own"
    );
}
