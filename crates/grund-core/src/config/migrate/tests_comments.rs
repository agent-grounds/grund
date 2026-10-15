//! Test module: §FS-config-migrate.4, the comments a migration carries or
//! lists. No comment may be neither, since `--drop-comments` is the only way a
//! migration loses one (§REQ-no-data-loss.4).

use super::{Located, MigrateReport, Prepared, translate};
use crate::testing::{test_root, write};

/// Translate the v1 `toml` over a tree holding `docs/FS-a.md`.
fn report(name: &str, toml: &str) -> MigrateReport {
    let root = test_root(&format!("migrate-comments-{name}"));
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nText.\n");
    write(&root.join("grund.toml"), toml);
    let located = Located {
        file: root.join("grund.toml"),
        root: root.clone(),
        config: "grund.toml".into(),
    };
    let prepared = translate(&located, false, false).expect("translates");
    let _ = std::fs::remove_dir_all(&root);
    match prepared {
        Prepared::Translated { report, .. } => *report,
        Prepared::AlreadyV2 => panic!("read as version 2"),
    }
}

/// §FS-config-migrate.4.4: a table header and its first key both carry a
/// trailing comment and both become `[schema.kinds.FS]`; the line keeps the
/// first, and the second is listed rather than overwritten.
#[test]
fn a_second_trailing_comment_on_one_v2_line_is_listed() {
    let report = report(
        "shared-line",
        "grund_config_version = 1\n\
         [id]\nnamed_sections = true\n\
         [[kinds]] # the spec row\n\
         kind = \"FS\" # what we specify\n\
         folder = \"docs\"\n\
         [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
    );
    assert!(report.refusals.is_empty(), "{:?}", report.refusals);
    let candidate = report.candidate.expect("a candidate");
    assert!(
        candidate.contains("[schema.kinds.FS] # the spec row\n"),
        "{candidate}"
    );
    assert!(!candidate.contains("what we specify"), "{candidate}");
    let listed: Vec<_> = report
        .comments
        .iter()
        .map(|c| (c.first, c.last, c.place.as_str()))
        .collect();
    assert_eq!(listed, [(5, 5, "inside [[kinds]] kind")]);
}
