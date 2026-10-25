//! Refused-write cause summaries and combined causes (§FS-config-migrate.2.2).

use super::*;

fn setting_refusal(anchor: RefusalAnchor) -> MigrateRefusal {
    MigrateRefusal {
        anchor,
        what: "cannot preserve this setting or file".into(),
        details: Vec::new(),
        repair: "repair this cause before migrating".into(),
    }
}

fn refused_report() -> MigrateReport {
    let mut report = successful_report();
    report.wrote = false;
    report
}

fn assert_closing(report: &MigrateReport, expected: &str) {
    assert!(report.refused());
    for full in [false, true] {
        let actual = render(report, full);
        assert_eq!(actual.lines().last(), Some(expected), "{actual}");
        assert!(!actual.lines().any(|l| l.starts_with("written:")), "{actual}");
        assert!(!actual.contains(PROJECT_MAP), "{actual}");
        assert!(!actual.contains(ADVICE), "{actual}");
    }
}

#[test]
fn refused_setting_and_file_summaries_count_blocks_and_exclude_authorized_comments() {
    for anchor in [RefusalAnchor::Line(2), RefusalAnchor::Unset, RefusalAnchor::File] {
        let mut report = refused_report();
        report.refusals = vec![setting_refusal(anchor)];
        report.gates = None;
        assert!(report.drop_comments && !report.comments.is_empty());
        assert_closing(
            &report,
            "not written: 1 refusal (follow the repairs above); grund.toml is unchanged",
        );
    }
}

#[test]
fn refused_combined_settings_and_comments_keep_each_remedy() {
    let mut report = refused_report();
    report.drop_comments = false;
    report.refusals = vec![
        setting_refusal(RefusalAnchor::Line(2)),
        setting_refusal(RefusalAnchor::File),
    ];
    report.gates = None;
    for (count, expected) in [
        (3, "not written: 2 refusals (follow the repairs above); 3 comments have no single key to follow (follow the comment guidance above); grund.toml is unchanged"),
        (1, "not written: 2 refusals (follow the repairs above); 1 comment has no single key to follow (follow the comment guidance above); grund.toml is unchanged"),
    ] {
        report.comments.truncate(count);
        assert_closing(&report, expected);
    }
}

#[test]
fn refused_comment_and_gate_summaries_retain_differing_lines() {
    for (gate, expected) in [
        (Gate::Records, "not written: 3 comments have no single key to follow (follow the comment guidance above); the records differ (inspect the comparison above); grund.toml is unchanged"),
        (Gate::Report, "not written: 3 comments have no single key to follow (follow the comment guidance above); grund check --suggestions differs (inspect the comparison above); grund.toml is unchanged"),
    ] {
        let mut report = refused_report();
        report.drop_comments = false;
        report.gates = Some(GateResult::Differs {
            gate,
            lines: vec!["- before".into(), "+ after".into()],
        });
        for full in [false, true] {
            let actual = render(&report, full);
            assert!(actual.contains("- before\n+ after\n"), "{actual}");
        }
        assert_closing(&report, expected);
    }
}

#[test]
fn refused_summary_uses_the_report_path_base() {
    let mut report = refused_report();
    report.config = "../project/grund.toml".into();
    report.drop_comments = false;
    report.comments.truncate(1);
    assert_closing(
        &report,
        "not written: 1 comment has no single key to follow (follow the comment guidance above); ../project/grund.toml is unchanged",
    );
}
