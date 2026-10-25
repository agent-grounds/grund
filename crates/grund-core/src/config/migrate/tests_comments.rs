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
    assert_eq!(
        listed,
        [(
            5,
            5,
            concat!(
                "inside [[kinds]] kind: trailing comments collide at the shared destination ",
                "[schema.kinds.FS]; retain the prose and choose its placement manually; ",
                "authorize the listed drops with --write --drop-comments, then re-add it at ",
                "that chosen place after a successful write (subject to all other refusals and gates)"
            )
        )]
    );
}

/// §FS-config-migrate.4.1.1: values' boolean-to-form translation carries a
/// comment and warns before the later guidance gate refuses the migration.
/// A false opt-in emits no form key, so its comment stays in the existing list.
#[test]
fn value_form_carried_comment_reports_translated_vocabulary() {
    for enabled in [false, true] {
        let report = report(
            if enabled {
                "value-comment"
            } else {
                "no-value-key"
            },
            &format!(
                "grund_config_version = 1\n\
                 [id]\nnamed_sections = true\n\
                 [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\
                 values = {enabled} # true | false\n\
                 [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n"
            ),
        );
        assert!(report.refusals.is_empty(), "{:?}", report.refusals);
        let candidate = report
            .candidate
            .as_ref()
            .expect("candidate before guidance gate");
        if !enabled {
            assert!(!candidate.contains("true | false"), "{candidate}");
            assert_eq!(report.comments.len(), 1);
            assert_eq!(
                report.comments[0].place,
                concat!(
                    "inside [[kinds]] values: the destination key [schema.kinds.FS] form ",
                    "is omitted from the candidate; retain the prose and choose its placement ",
                    "manually; authorize the listed drops with --write --drop-comments, then ",
                    "re-add it at that chosen place after a successful write ",
                    "(subject to all other refusals and gates)"
                )
            );
            assert!(
                !report
                    .notices
                    .iter()
                    .any(|notice| notice.why.contains("comment"))
            );
            continue;
        }
        assert!(
            candidate.contains("form = \"value\" # true | false"),
            "{candidate}"
        );
        assert!(report.comments.is_empty(), "{:?}", report.comments);
        assert!(
            report.notices.iter().any(|notice| {
                let why = notice.why.to_lowercase();
                let shown = format!("{} {}", notice.key, notice.why);
                !notice.dropped
                    && notice.line == 7
                    && shown.contains("[[kinds]] values")
                    && shown.contains("[schema.kinds.FS] form")
                    && why.contains("comment")
                    && (why.contains("check") || why.contains("review"))
                    && (why.contains("translat") || why.contains("vocab"))
            }),
            "missing carried-comment warning for [[kinds]] values -> [schema.kinds.FS] form: {:?}",
            report.notices
        );
    }
}
