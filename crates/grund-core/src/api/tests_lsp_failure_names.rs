//! Test module: the editor's report asks a scan failure's names as `grund check <path>` does, so
//! a failure under the anchor stays an error under any of them, in the tree where the name the
//! whole run keeps is under the anchor neither as written nor resolved (§FS-check.1.3.6.3,
//! §FS-lsp.1.1, §AR-scanner.1.entry). Unix-only: the tree is links and a mode.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::check::{CheckOpts, check_with_opts};
use super::lsp_snapshot::lsp_snapshot;
use crate::queries::LspSnapshotOpts;
use crate::testing::{physical_test_root, symlink, write};

/// A workspace root with `aa -> zz`, `realE/lnk -> ../zz` and `zz/x` barred, so its whole walk
/// keeps `aa/x`; `None` where the mode does not bar this user.
fn third_name_tree(name: &str) -> Option<PathBuf> {
    let root = physical_test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"q\"\n\n[schema]\n\
         slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.doc]\ncitable = false\n\
         folders = [\"docs\"]\n\n[workspace]\nmembers = [\"m\"]\n",
    );
    write(
        &root.join("m/grund.toml"),
        "grund_config_version = 2\nproject_name = \"m\"\n\n[schema]\n\
         slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.FS]\nfolders = [\"spec\"]\n",
    );
    fs::create_dir_all(root.join("m/spec")).expect("create the member's home");
    write(&root.join("docs/notes.md"), "# Notes\n");
    fs::create_dir_all(root.join("realE")).expect("create the directory");
    fs::create_dir_all(root.join("zz/x")).expect("create the barred directory");
    symlink("zz", &root.join("aa"));
    symlink("../zz", &root.join("realE/lnk"));
    fs::set_permissions(root.join("zz/x"), fs::Permissions::from_mode(0o000)).expect("bar it");
    if fs::read_dir(root.join("zz/x")).is_ok() {
        fs::set_permissions(root.join("zz/x"), fs::Permissions::from_mode(0o755)).expect("free it");
        return None;
    }
    Some(root)
}

/// The paths of the `io` errors a snapshot anchored at `rel` reports.
fn io_errors(root: &Path, rel: &str) -> Vec<String> {
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: root.join(rel),
        path_provided: true,
        ..LspSnapshotOpts::default()
    })
    .expect("build the snapshot");
    snapshot
        .report
        .errors
        .iter()
        .filter(|finding| finding.code == "io")
        .map(|finding| finding.path.clone().expect("a located failure"))
        .collect()
}

#[test]
fn a_failure_under_the_anchor_stays_an_error_under_any_of_its_names() {
    let Some(root) = third_name_tree("a_failure_under_the_anchor_stays_an_error") else {
        return;
    };

    let under_the_anchor = io_errors(&root, "realE");
    let under_another_name = io_errors(&root, "zz");
    let outside = io_errors(&root, "docs");
    fs::set_permissions(root.join("zz/x"), fs::Permissions::from_mode(0o755)).expect("free it");

    // Membership considers every alias (§FS-check.1.3.6.3); naming is asserted separately below.
    assert_eq!(under_the_anchor.len(), 1, "{under_the_anchor:?}");
    assert_eq!(under_another_name.len(), 1, "{under_another_name:?}");
    assert!(outside.is_empty(), "{outside:?}");
}

/// §FS-lsp.1.1 and §FS-check.2.4.2: every editor anchor uses the whole run's first name,
/// `aa/x`, even though the anchor `realE` reaches the entry through a third name and resolving
/// either gives `zz/x`. CLI `check <path>` alone uses the anchor's own spelling.
#[test]
fn issue_1337_an_anchored_editor_failure_keeps_the_whole_runs_name() {
    let Some(root) = third_name_tree("anchored_editor_failure_whole_run_name") else {
        return;
    };
    let reports = [".", "realE", "realE/lnk", "zz", "aa", "docs"].map(|anchor| {
        let report = lsp_snapshot(LspSnapshotOpts {
            path: root.join(anchor),
            path_provided: true,
            ..LspSnapshotOpts::default()
        })
        .expect("build the snapshot")
        .report;
        (anchor, report)
    });
    fs::set_permissions(root.join("zz/x"), fs::Permissions::from_mode(0o755)).expect("free it");

    let expected = root.join("aa/x").to_string_lossy().into_owned();
    let mut failures = Vec::new();
    for (anchor, report) in reports {
        let (own, other) = if anchor == "docs" {
            (&report.warnings, &report.errors)
        } else {
            (&report.errors, &report.warnings)
        };
        let paths: Vec<_> = own
            .iter()
            .filter(|finding| finding.code == "io")
            .map(|finding| finding.path.as_deref().expect("located failure"))
            .collect();
        if paths != [expected.as_str()] {
            failures.push(format!("{anchor}: expected {expected:?}, got {paths:?}"));
        }
        assert!(
            other.iter().all(|finding| finding.code != "io"),
            "{report:#?}"
        );
        if anchor == "docs" {
            assert!(
                own.iter().any(|finding| {
                    finding.code == "io" && finding.message.contains("outside the report scope")
                }),
                "{report:#?}"
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Assert the public failure list as well as the located report, with independent expected
/// names. A healthy sibling holds no alias and must caution once (§FS-lsp.1.1).
fn assert_whole_run_failure(root: &Path, anchors: &[&str], expected: &str) {
    let absolute = root.join(expected).to_string_lossy().into_owned();
    for anchor in anchors {
        let snapshot = lsp_snapshot(LspSnapshotOpts {
            path: root.join(anchor),
            path_provided: true,
            ..Default::default()
        })
        .expect("build the snapshot");
        let scan_paths: Vec<_> = snapshot
            .scan_errors
            .iter()
            .map(|e| e.path.as_str())
            .collect();
        assert_eq!(scan_paths, [expected], "{anchor}: {snapshot:#?}");
        let (own, other) = if *anchor == "docs" {
            (&snapshot.report.warnings, &snapshot.report.errors)
        } else {
            (&snapshot.report.errors, &snapshot.report.warnings)
        };
        let failures: Vec<_> = own.iter().filter(|f| f.code == "io").collect();
        assert_eq!(failures.len(), 1, "{anchor}: {snapshot:#?}");
        assert_eq!(
            failures[0].path.as_deref(),
            Some(absolute.as_str()),
            "{anchor}"
        );
        assert_eq!(
            failures[0].message.contains("outside the report scope"),
            *anchor == "docs",
            "{anchor}: {snapshot:#?}"
        );
        assert!(
            other.iter().all(|f| f.code != "io"),
            "{anchor}: {snapshot:#?}"
        );
    }
}

/// CLI reporting alone selects the alias under its anchor (§FS-check.2.4.2).
fn assert_cli_failure_names(root: &Path, anchors: &[(&str, &str)]) {
    for (anchor, expected) in anchors {
        let output = check_with_opts(CheckOpts {
            path: root.join(anchor),
            path_provided: true,
            ..Default::default()
        })
        .expect("check the path");
        assert!(output.had_scan_errors, "{anchor}: {output:#?}");
        let paths: Vec<_> = output
            .report
            .errors
            .iter()
            .filter(|f| f.code == "io")
            .map(|f| f.path.as_deref().expect("located failure"))
            .collect();
        assert_eq!(paths, [*expected], "{anchor}: {output:#?}");
    }
}

/// No workspace and no E2E pass: the explicit root precedes ordinary roots in the walk, but
/// must not select the editor's failure name (§FS-lsp.1.1, §FS-check.2.4.2).
#[test]
fn issue_1337_standalone_walk_failures_keep_the_whole_runs_name() {
    let root = physical_test_root("standalone_editor_failure_whole_run_name");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"p\"\n\
         [schema.kinds.doc]\ncitable = false\nfolders = [\"docs\"]\n",
    );
    write(&root.join("docs/notes.md"), "# Notes\n");
    fs::create_dir_all(root.join("realE")).expect("create the directory");
    fs::create_dir_all(root.join("zz/x")).expect("create the barred directory");
    symlink("zz", &root.join("aa"));
    symlink("../zz", &root.join("realE/lnk"));

    struct Unbar(PathBuf);
    impl Drop for Unbar {
        fn drop(&mut self) {
            let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
        }
    }
    let barred = root.join("zz/x");
    let _unbar = Unbar(barred.clone());
    fs::set_permissions(&barred, fs::Permissions::from_mode(0o000)).expect("bar it");
    if fs::read_dir(&barred).is_ok() {
        return; // This user is not barred, so this run proves nothing about a walk failure.
    }
    assert_whole_run_failure(
        &root,
        &[".", "realE", "realE/lnk", "zz", "aa", "docs"],
        "aa/x",
    );
    assert_cli_failure_names(
        &root,
        &[
            (".", "aa/x"),
            ("realE", "realE/lnk/x"),
            ("zz", "zz/x"),
            ("aa", "aa/x"),
        ],
    );
}

/// A case introduces an alias absent from the ordinary file walk. Retain it for membership,
/// without replacing the file pass's whole-run decode-failure name (§FS-check.2.4.2).
#[test]
fn issue_1337_e2e_decode_failures_keep_the_whole_runs_name_and_case_alias() {
    let root = physical_test_root("e2e_editor_decode_failure_whole_run_name");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"p\"\n\
         [schema.kinds.doc]\ncitable = false\nfolders = [\"docs\"]\n\
         [schema.kinds.E2E]\nfolders = [\"realE/cases\"]\nindex = false\n",
    );
    write(&root.join("docs/notes.md"), "# Notes\n");
    write(&root.join("realE/cases/b/expected.exit"), "0\n");
    fs::write(root.join("payload.md"), b"\xff\xfe").expect("write invalid UTF-8");
    symlink("../../../payload.md", &root.join("realE/cases/b/spec.refs"));
    assert_whole_run_failure(
        &root,
        &[
            ".",
            "realE",
            "realE/cases/b",
            "realE/cases/b/spec.refs",
            "payload.md",
            "docs",
        ],
        "payload.md",
    );
    assert_cli_failure_names(
        &root,
        &[
            (".", "payload.md"),
            ("realE", "realE/cases/b/spec.refs"),
            ("realE/cases/b", "realE/cases/b/spec.refs"),
            ("realE/cases/b/spec.refs", "realE/cases/b/spec.refs"),
        ],
    );
}

/// §FS-check.2.4 in the editor: one report, the same limit as `grund check`. Over the root the
/// unread value source is the snapshot's own error, so the citation earns nothing; anchored at
/// `docs` it is outside the report and the finding stands (§FS-check.1.3.6.3).
#[test]
fn an_unread_home_withholds_the_finding_only_where_the_failure_is_the_reports_own() {
    let root = physical_test_root("an_unread_home_withholds_in_the_editor");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"q\"\n\n[schema.kinds.CONST]\n\
         folders = [\"values\"]\nform = \"value\"\nfetch = \"scripts/fetch\"\n\n\
         [schema.kinds.page]\ncitable = false\nfiles = [\"docs/offer.md\"]\n",
    );
    write(&root.join("values/limits.json"), "{\"CONST-a\": [40\n");
    write(
        &root.join("docs/offer.md"),
        "# Offer\n\nSee \u{a7}CONST-a.\n",
    );
    let cites_missing = |rel: &str| {
        let snapshot = lsp_snapshot(LspSnapshotOpts {
            path: root.join(rel),
            path_provided: true,
            ..LspSnapshotOpts::default()
        })
        .expect("build the snapshot");
        snapshot
            .report
            .errors
            .iter()
            .any(|finding| finding.message.contains("unknown reference"))
    };

    assert!(!cites_missing("."), "the root's report holds the failure");
    assert!(cites_missing("docs"), "the anchor leaves it a caution");
}
