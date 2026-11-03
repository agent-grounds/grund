//! Test module: the one policy that says whether a version-1 block with
//! `include_root = false` migrates compactly (§FS-config-migrate.8.1.2,
//! §AR-config.8.4), driven through `config_migrate` so item 1 is answered by the
//! unread-block probe itself. A compact block's report carries the one notice
//! about no key, version 1's defaults left unwritten (§FS-config-migrate.8.1.3);
//! a conservative one never does.

use std::path::PathBuf;

use crate::config::migrate::{MigrateOutcome, MigrateReport};
use crate::testing::{test_root, write};
use crate::{MigrateOpts, config_migrate};

const WORKSPACE: &str = "[workspace]\ninclude_root = false\nmembers = [\"a\"]\n";

/// §FS-config-migrate.8.1.3: the version line and the `[workspace]` table.
const ENVELOPE: &str =
    "grund_config_version = 2\n\n[workspace]\nmembers = [\"a\"]\ninclude_root = false\n";

/// A block at a fresh root whose `grund.toml` is `head` then `[workspace]`
/// (`members = ["a"]`, member `a` on version 2), with the files of `files`.
fn block(name: &str, head: &str, files: &[&str]) -> PathBuf {
    let root = test_root(&format!("migrate-opted-out-{name}"));
    write(&root.join("a/grund.toml"), "grund_config_version = 2\n");
    for path in files {
        write(&root.join(path), "# x\n");
    }
    write(&root.join("grund.toml"), &format!("{head}{WORKSPACE}"));
    root
}

/// The preview of `root`'s config.
fn preview(root: PathBuf) -> MigrateReport {
    let opts = MigrateOpts {
        path: root,
        write: false,
        drop_comments: false,
    };
    match config_migrate(&opts).expect("the block migrates") {
        MigrateOutcome::Migrated(report) => *report,
        MigrateOutcome::AlreadyV2 { .. } => panic!("read as version 2"),
    }
}

/// Whether the report took the compact path (§FS-config-migrate.8.1.3).
fn compact(report: &MigrateReport) -> bool {
    report.notices.iter().any(|notice| notice.line.is_none())
}

/// A compact preview: unrefused, and its candidate is the envelope.
fn assert_compact(name: &str, head: &str, files: &[&str]) -> MigrateReport {
    let report = preview(block(name, head, files));
    assert!(report.refusals.is_empty(), "{name}: {:?}", report.refusals);
    assert!(compact(&report), "{name}: {:?}", report.notices);
    assert_eq!(report.candidate.as_deref(), Some(ENVELOPE), "{name}");
    report
}

/// A conservative preview: no notice about version 1's defaults, and no key
/// dropped as unread on a block that keeps every key.
fn assert_conservative(name: &str, head: &str, files: &[&str]) -> MigrateReport {
    let report = preview(block(name, head, files));
    assert!(!compact(&report), "{name}: {:?}", report.notices);
    assert!(
        report
            .notices
            .iter()
            .all(|notice| !notice.why.contains("include_root = false")),
        "{name}: {:?}",
        report.notices
    );
    report
}

/// §FS-config-migrate.8.1.2: `[id] named_sections` omitted, true or false, has
/// no reader on the block, so each of the three migrates compactly.
#[test]
fn named_sections_omitted_true_or_false_stays_compact() {
    assert_compact("named-omitted", "", &[]);
    for value in ["true", "false"] {
        let head = format!("[id]\nnamed_sections = {value}\n\n");
        let report = assert_compact(&format!("named-{value}"), &head, &[]);
        assert!(
            report
                .notices
                .iter()
                .any(|notice| notice.dropped && notice.key.contains("named_sections")),
            "{value}: {:?}",
            report.notices
        );
    }
}

/// §FS-config-migrate.8.1.2 item 2: an include that is `[]`, or exactly version
/// 1's default list, counts as omitted.
#[test]
fn an_empty_or_default_include_counts_as_omitted() {
    assert_compact("include-empty", "[scan]\ninclude = []\n\n", &[]);
    let default = "[scan]\ninclude = [\"requirements.md\", \"docs\", \"e2e\", \"src\"]\n\n";
    assert_compact("include-default", default, &[]);
}

/// §FS-config-migrate.8.1.2 item 2: a partial include is a setting someone
/// chose, so the block keeps the conservative path, and the missing entry is
/// refused there (§FS-config-migrate.3.1.3).
#[test]
fn a_partial_include_goes_conservative() {
    let report = assert_conservative("include-partial", "[scan]\ninclude = [\"docs\"]\n\n", &[]);
    assert!(!report.refusals.is_empty());
}

/// §FS-config-migrate.8.1.2 item 2: `[output]` keys leave the block compact and
/// are dropped as on any config (§FS-config-migrate.3.2).
#[test]
fn output_keys_stay_compact() {
    let report = assert_compact("output", "[output]\nformat = \"json\"\n\n", &[]);
    assert!(
        report
            .notices
            .iter()
            .any(|notice| notice.dropped && notice.key.contains("format"))
    );
}

/// §FS-config-migrate.8.1.3: an authored description is kept, as
/// `[presentation] description`, beside the kept `project_name`.
#[test]
fn a_project_description_is_kept_on_the_compact_candidate() {
    let head = "project_name = \"grp\"\nproject_description = \"a block\"\n\n";
    let report = preview(block("description", head, &[]));
    assert!(report.refusals.is_empty() && compact(&report), "{report:?}");
    let candidate = "grund_config_version = 2\nproject_name = \"grp\"\n\n[workspace]\n\
members = [\"a\"]\ninclude_root = false\n\n[presentation]\ndescription = \"a block\"\n";
    assert_eq!(report.candidate.as_deref(), Some(candidate));
}

/// §FS-config-migrate.8.1.2 item 1: a version-1 default kind home on disk is in
/// the block's scope even under `include = []`, so the block is conservative.
#[test]
fn a_kind_home_on_disk_goes_conservative() {
    let files = ["docs/functional-spec/FS-a.md"];
    assert_conservative("kind-home", "[scan]\ninclude = []\n\n", &files);
}

/// §FS-config-migrate.8.1.2 item 1: a default root inside a member is the
/// member's, outside the block's scope (§FS-check.4.10.2).
#[test]
fn a_default_root_inside_a_member_stays_compact() {
    assert_compact("member-docs", "", &["a/docs/x.md", "a/src/x.rs"]);
}

/// §FS-config-migrate.8.1.2 item 1: a block whose members do not expand keeps
/// the conservative path.
#[test]
fn a_missing_member_goes_conservative() {
    let root = test_root("migrate-opted-out-missing-member");
    write(&root.join("a/grund.toml"), "grund_config_version = 2\n");
    let text = "[workspace]\ninclude_root = false\nmembers = [\"a\", \"b\"]\n";
    write(&root.join("grund.toml"), text);
    let report = preview(root);
    assert!(!compact(&report), "{:?}", report.notices);
}

/// §FS-config-migrate.8.1.2 item 2: `[reference] strict = false` is not version
/// 1's default, though the records gate's reduction does not see it, so the
/// block is conservative and the key is refused rather than dropped as unread.
#[test]
fn strict_false_goes_conservative() {
    let report = assert_conservative("strict", "[reference]\nstrict = false\n\n", &[]);
    assert!(
        report
            .refusals
            .iter()
            .any(|refusal| refusal.what.contains("strict")),
        "{:?}",
        report.refusals
    );
    assert_compact("strict-true", "[reference]\nstrict = true\n\n", &[]);
}
