//! §AR-lsp.5.1.1 — `grund_core::lsp_snapshot` anchored below a workspace root answers for an
//! unreadable file at or under the anchor by any of the file's names, as `grund check` of the
//! same path must (§FS-check.1.3.6.3, agent-grounds/grund#1299). The tree is the issue's: a
//! member that only has to exist, `real/s.md` at mode 000, and a directory link to `real` whose
//! name decides which name the walk keeps.
#![cfg(unix)]

use grund_core::{Finding, LspSnapshotOpts, lsp_snapshot};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const SCHEMA: &str = "[schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.GOAL]\nfolders = [\"docs\"]\nindex = false\n";

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str, link: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("grund-1299-lsp-{name}-{}", std::process::id()));
        let _ = fs::set_permissions(root.join("real/s.md"), fs::Permissions::from_mode(0o644));
        let _ = fs::remove_dir_all(&root);
        for dir in ["docs", "m/docs", "real", "other"] {
            fs::create_dir_all(root.join(dir)).expect("create a directory");
        }
        for (path, text) in [
            (
                "grund.toml",
                format!(
                    "grund_config_version = 2\nproject_name = \"root\"\n\n{SCHEMA}\n\
                     [workspace]\nmembers = [\"m\"]\n"
                ),
            ),
            (
                "m/grund.toml",
                format!("grund_config_version = 2\nproject_name = \"m\"\n\n{SCHEMA}"),
            ),
            (
                "docs/GOAL-a.md",
                "# GOAL-a: A\n\nSee \u{a7}GOAL-b.\n".into(),
            ),
            (
                "docs/GOAL-b.md",
                "# GOAL-b: B\n\nSee \u{a7}GOAL-a.\n".into(),
            ),
            (
                "m/docs/GOAL-c.md",
                "# GOAL-c: C\n\nSee \u{a7}GOAL-d.\n".into(),
            ),
            (
                "m/docs/GOAL-d.md",
                "# GOAL-d: D\n\nSee \u{a7}GOAL-c.\n".into(),
            ),
            ("real/s.md", "# A note\n".into()),
            ("other/o.md", "# Other\n".into()),
        ] {
            fs::write(root.join(path), text).expect("write a fixture file");
        }
        std::os::unix::fs::symlink("real", root.join(link)).expect("make the directory link");
        fs::set_permissions(root.join("real/s.md"), fs::Permissions::from_mode(0o000))
            .expect("chmod 000");
        let root = root.canonicalize().expect("canonical fixture root");
        Self { root }
    }

    /// Whether the mode bars this process: it does not bar root, which proves nothing here.
    fn barred(&self) -> bool {
        fs::read(self.root.join("real/s.md")).is_err()
    }

    /// The snapshot's io findings about s.md, as `(severity, message)`.
    fn unread(&self, anchor: &str) -> Vec<(&'static str, String)> {
        let snapshot = lsp_snapshot(LspSnapshotOpts {
            path: self.root.join(anchor),
            path_provided: true,
            ..Default::default()
        })
        .expect("lsp_snapshot");
        let about_s = |finding: &&Finding| {
            finding.code == "io"
                && finding
                    .path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("s.md"))
        };
        snapshot
            .report
            .errors
            .iter()
            .chain(&snapshot.report.warnings)
            .filter(about_s)
            .map(|finding| (finding.severity, finding.message.clone()))
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::set_permissions(
            self.root.join("real/s.md"),
            fs::Permissions::from_mode(0o644),
        );
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn is_error(unread: &[(&str, String)]) -> bool {
    !unread.is_empty()
        && unread.iter().all(|(severity, message)| {
            *severity == "error" && !message.contains("outside the report scope")
        })
}

fn is_caution(unread: &[(&str, String)]) -> bool {
    !unread.is_empty()
        && unread.iter().all(|(severity, message)| {
            *severity == "warning" && message.contains("outside the report scope")
        })
}

fn assert_answers_for_the_file(fixture: &Fixture) {
    for anchor in ["real", "real/s.md"] {
        let unread = fixture.unread(anchor);
        assert!(
            is_error(&unread),
            "lsp_snapshot({anchor}): s.md is under `{anchor}`, so its read failure is an error \
             and no outside-the-report-scope caution: {unread:#?}"
        );
    }
}

/// The control: a link sorting after `real` leaves `real/s.md` the surviving name.
#[test]
fn a_link_sorting_after_leaves_the_anchored_snapshot_an_error() {
    let fixture = Fixture::new("zz", "zz");
    if !fixture.barred() {
        eprintln!("skipped: chmod 000 does not stop this process reading a file (root?)");
        return;
    }
    assert_answers_for_the_file(&fixture);
}

/// The ticket: the link `aa` sorts first, so the walk keeps `aa/s.md`.
#[test]
fn a_link_sorting_before_leaves_the_anchored_snapshot_an_error() {
    let fixture = Fixture::new("aa", "aa");
    if !fixture.barred() {
        eprintln!("skipped: chmod 000 does not stop this process reading a file (root?)");
        return;
    }
    assert_answers_for_the_file(&fixture);
}

/// The guard: `other` holds the file under no name, so the snapshot keeps the caution.
#[test]
fn an_anchor_holding_the_file_under_no_name_keeps_the_caution() {
    for link in ["aa", "zz"] {
        let fixture = Fixture::new(&format!("other-{link}"), link);
        if !fixture.barred() {
            eprintln!("skipped: chmod 000 does not stop this process reading a file (root?)");
            return;
        }
        let unread = fixture.unread("other");
        assert!(
            is_caution(&unread),
            "lsp_snapshot(other) (link {link}): s.md is outside `other`, so it stays a caution: \
             {unread:#?}"
        );
    }
}
