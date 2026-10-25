//! Migration presentation contracts: §FS-config-migrate.6.5 compact success,
//! §FS-config-migrate.2.2 complete reports and §FS-config-migrate.5.4 differences.

use super::*;
use grund_core::{MappedKey, MigrateNotice, MigrateRefusal};

#[path = "tests_config_migrate_refused.rs"]
mod refused;

const HEADING: &str = "grund.toml: version 1 → 2, --write\n";
const PREVIEW: &str = "grund.toml: version 1 → 2, preview, nothing written\n";
const COMMENTS: &str = "comments with no single key to follow (--write refuses; --drop-comments drops exactly these):\n  grund.toml:6-7  standing alone\n  grund.toml:12  inside [scan] include\n  grund.toml:22-23  above [scan] include\n";
const NOTICES: &str = "dropped: grund.toml:3: [output] color: use --color instead\nchanged: grund.toml:4: [reference] inline_style: check the carried comment: vocabulary translated to [schema.notes.text] must\n";
const MAPPINGS: &str = "mapped: grund.toml:1: grund_config_version → grund_config_version = 2\nmapped: grund.toml:4: [reference] inline_style → [schema.notes.text] must\n";
const DEFAULTS: &str = "written: [schema.notes.lines] may = 1: version 1's default, written out\nwritten: [presentation] rules = \"root\": version 1's default, written out\n";
const MEMBERS: &str = "member: api/grund.toml is version 1; migrate it with grund config migrate --to 2 api\nmember: web/grund.toml is version 1; migrate it with grund config migrate --to 2 web\n";
const EQUAL: &str = "compared: the records are equal, and grund check --suggestions on this tree gives the same findings and the same exit before and after\n";
const PROJECT_MAP: &str = "note: the next grund init adds a `code` row to the AGENTS.md Project map, because the complement's places are now listed; grund check reports the block stale until then\n";
const OUTCOME: &str = "wrote grund.toml: version 1 (24 lines) replaced by version 2\n";
const DROPS: &str = "dropped the comments at grund.toml:6-7, grund.toml:12 and grund.toml:22-23 (version 1 lines)\n";
const ADVICE: &str = "to get version 1 back: it is in version control, e.g. git restore grund.toml\nto see this candidate again: restore version 1 and run grund config migrate --to 2\nnote: existing AGENTS.md managed instructions may still describe version 1; run grund init in the migrated project to refresh them\n";
const REFUSALS: &str = "3 refusals (follow the repairs above)";
const COMMENT_REFUSAL: &str =
    "3 comments have no single key to follow (follow the comment guidance above)";

fn successful_report() -> MigrateReport {
    MigrateReport {
        config: "grund.toml".into(),
        write: true,
        drop_comments: true,
        comments: vec![
            ListedComment {
                first: 6,
                last: 7,
                place: "standing alone".into(),
            },
            ListedComment {
                first: 12,
                last: 12,
                place: "inside [scan] include".into(),
            },
            ListedComment {
                first: 22,
                last: 23,
                place: "above [scan] include".into(),
            },
        ],
        notices: vec![
            MigrateNotice {
                line: 3,
                dropped: true,
                key: "[output] color".into(),
                why: "use --color instead".into(),
            },
            MigrateNotice {
                line: 4,
                dropped: false,
                key: "[reference] inline_style".into(),
                why: "check the carried comment: vocabulary translated to [schema.notes.text] must"
                    .into(),
            },
        ],
        mapped: vec![
            MappedKey {
                line: 1,
                key: "grund_config_version".into(),
                spelling: "grund_config_version = 2".into(),
            },
            MappedKey {
                line: 4,
                key: "[reference] inline_style".into(),
                spelling: "[schema.notes.text] must".into(),
            },
        ],
        written: vec![
            "[schema.notes.lines] may = 1".into(),
            "[presentation] rules = \"root\"".into(),
        ],
        members: vec![
            "api/grund.toml is version 1; migrate it with grund config migrate --to 2 api".into(),
            "web/grund.toml is version 1; migrate it with grund config migrate --to 2 web".into(),
        ],
        gates: Some(GateResult::Equal),
        code_row_note: true,
        candidate: Some("grund_config_version = 2\n".into()),
        v1_lines: 24,
        wrote: true,
        ..Default::default()
    }
}

/// §FS-config-migrate.2.2 item 6: the closing line a refused run reads from
/// its one cause list, worded for a write or a preview.
fn closing(write: bool, causes: &str) -> String {
    if write {
        format!("not written: {causes}; grund.toml is unchanged\n")
    } else {
        format!("would not write: {causes}\n")
    }
}

fn render(report: &MigrateReport, full: bool) -> String {
    let mut sink = Vec::new();
    render_migrate_report(
        report,
        MigratePresentation::for_report(report, full),
        &mut sink,
    )
    .unwrap();
    String::from_utf8(sink).unwrap()
}

/// §FS-config-migrate.6.5: every retained field, original ranges once, advice last.
#[test]
fn compact_report_orders_all_retained_fields() {
    let report = successful_report();
    assert!(!report.refused());
    assert!(MigratePresentation::for_report(&report, false) == MigratePresentation::CompactWrite);
    assert_eq!(
        report.stdout(),
        report.candidate.as_deref(),
        "core eligibility is unchanged"
    );
    assert_eq!(
        render(&report, false),
        [OUTCOME, DROPS, NOTICES, MEMBERS, PROJECT_MAP, ADVICE].concat()
    );
}

/// §FS-config-migrate.6.5: independently absent optional fields add no blank lines.
#[test]
fn compact_report_omits_absent_optional_fields() {
    for mask in 0..16 {
        let mut report = successful_report();
        let mut expected = vec![OUTCOME];
        if mask & 1 == 0 {
            report.comments.clear();
        } else {
            expected.push(DROPS);
        }
        if mask & 2 == 0 {
            report.notices.clear();
        } else {
            expected.push(NOTICES);
        }
        if mask & 4 == 0 {
            report.members.clear();
        } else {
            expected.push(MEMBERS);
        }
        report.code_row_note = mask & 8 != 0;
        if report.code_row_note {
            expected.push(PROJECT_MAP);
        }
        expected.push(ADVICE);
        assert!(!report.refused());
        assert_eq!(
            render(&report, false),
            expected.concat(),
            "optional mask {mask}"
        );
    }
}

/// §FS-config-migrate.2.2, §FS-config-migrate.1.3: the same record restores the
/// complete order, including pre-write listing and passing-gate narration.
#[test]
fn full_report_preserves_legacy_order() {
    let mut report = successful_report();
    assert!(MigratePresentation::for_report(&report, true) == MigratePresentation::Complete);
    assert_eq!(
        render(&report, true),
        [
            HEADING,
            COMMENTS,
            NOTICES,
            MAPPINGS,
            DEFAULTS,
            MEMBERS,
            EQUAL,
            PROJECT_MAP,
            OUTCOME,
            DROPS,
            ADVICE
        ]
        .concat()
    );
    report.write = false;
    report.wrote = false;
    let expected = [
        PREVIEW,
        COMMENTS,
        NOTICES,
        MAPPINGS,
        DEFAULTS,
        MEMBERS,
        EQUAL,
        PROJECT_MAP,
    ]
    .concat();
    for full in [false, true] {
        assert!(MigratePresentation::for_report(&report, full) == MigratePresentation::Complete);
        assert_eq!(render(&report, full), expected);
        assert_eq!(report.stdout(), report.candidate.as_deref());
    }
}

/// §FS-config-migrate.2.2: setting refusals retain every anchor, detail and repair,
/// then comment listings and notices, and close with their causes, preview or write.
#[test]
fn refused_report_keeps_complete_diagnostics() {
    let mut report = successful_report();
    report.wrote = false;
    report.gates = None;
    report.candidate = None;
    report.mapped.clear();
    report.written.clear();
    report.members.clear();
    report.code_row_note = false;
    report.refusals = vec![
        MigrateRefusal {
            anchor: RefusalAnchor::Line(2),
            what: "[id] named_sections = false".into(),
            details: vec![
                "version 2 always recognizes named sections".into(),
                "the old setting cannot be kept".into(),
            ],
            repair: "set named_sections = true before migrating".into(),
        },
        MigrateRefusal {
            anchor: RefusalAnchor::Unset,
            what: "[reference] strict is unset".into(),
            details: vec!["version 2 always uses strict references".into()],
            repair: "set strict = true before migrating".into(),
        },
        MigrateRefusal {
            anchor: RefusalAnchor::File,
            what: "is a symbolic link".into(),
            details: vec!["replacement would remove the link".into()],
            repair: "migrate the file it points to".into(),
        },
    ];
    let refusals = "refused: grund.toml:2: [id] named_sections = false\n  version 2 always recognizes named sections\n  the old setting cannot be kept\n  repair: set named_sections = true before migrating\nrefused: grund.toml: [reference] strict is unset\n  version 2 always uses strict references\n  repair: set strict = true before migrating\nrefused: grund.toml is a symbolic link\n  replacement would remove the link\n  repair: migrate the file it points to\n";
    for write in [false, true] {
        report.write = write;
        for full in [false, true] {
            assert!(report.refused());
            assert_eq!(report.stdout(), None);
            assert!(
                MigratePresentation::for_report(&report, full) == MigratePresentation::Refused
            );
            assert_eq!(
                render(&report, full),
                [
                    if write { HEADING } else { PREVIEW },
                    refusals,
                    COMMENTS,
                    NOTICES,
                    closing(write, REFUSALS).as_str(),
                ]
                .concat()
            );
        }
    }
}

/// §FS-config-migrate.4.4, §FS-config-migrate.2.2, §FS-config-migrate.1.3:
/// unauthorized comment loss keeps the complete listing and closes with its
/// cause; only a `--full` preview shows the candidate and narrates its defaults.
#[test]
fn comment_refusal_keeps_complete_diagnostics() {
    let mut report = successful_report();
    report.wrote = false;
    report.drop_comments = false;
    for write in [false, true] {
        report.write = write;
        for full in [false, true] {
            assert!(report.refused());
            assert_eq!(
                report.stdout(),
                if write {
                    None
                } else {
                    report.candidate.as_deref()
                }
            );
            let narrated = full && !write;
            assert!(
                MigratePresentation::for_report(&report, full)
                    == if narrated {
                        MigratePresentation::Complete
                    } else {
                        MigratePresentation::Refused
                    }
            );
            assert_eq!(
                render(&report, full),
                [
                    if write { HEADING } else { PREVIEW },
                    COMMENTS,
                    NOTICES,
                    MAPPINGS,
                    if narrated { DEFAULTS } else { "" },
                    MEMBERS,
                    EQUAL,
                    if narrated { PROJECT_MAP } else { "" },
                    closing(write, COMMENT_REFUSAL).as_str(),
                ]
                .concat()
            );
        }
    }
}

/// §FS-config-migrate.2.2 items 5 and 6, §FS-config-migrate.1.3: a gate
/// difference has no candidate, so neither a preview nor a write narrates
/// defaults, with or without `--full`, and both close with the same cause.
fn assert_gate_difference(gate: Gate, difference: &str, cause: &str) {
    let mut report = successful_report();
    report.wrote = false;
    report.gates = Some(GateResult::Differs {
        gate,
        lines: vec!["- before".into(), "+ after".into()],
    });
    for write in [false, true] {
        report.write = write;
        for full in [false, true] {
            assert!(report.refused());
            assert_eq!(report.stdout(), None);
            assert!(
                MigratePresentation::for_report(&report, full) == MigratePresentation::Refused
            );
            assert_eq!(
                render(&report, full),
                [
                    if write { HEADING } else { PREVIEW },
                    COMMENTS,
                    NOTICES,
                    MAPPINGS,
                    MEMBERS,
                    difference,
                    "- before\n+ after\n",
                    closing(write, cause).as_str(),
                ]
                .concat()
            );
        }
    }
}

/// §FS-config-migrate.5.4: record differences remain complete, preview or write.
#[test]
fn records_gate_difference_keeps_complete_diagnostics() {
    assert_gate_difference(
        Gate::Records,
        "differs: the records\n",
        "the records differ (inspect the comparison above)",
    );
}

/// §FS-config-migrate.5.4: check-report differences remain complete, preview or write.
#[test]
fn report_gate_difference_keeps_complete_diagnostics() {
    assert_gate_difference(
        Gate::Report,
        "differs: grund check --suggestions on this tree\n",
        "grund check --suggestions differs (inspect the comparison above)",
    );
}
