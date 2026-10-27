//! Test module: what a narrowed `check` says when what it selected judged
//! nothing (§FS-check.1.4.1, §FS-check.1.4) — the selector's carry of
//! `empty-citation-obligation`, its off lines, and the switches held to the
//! clauses of the §FS-errors.5.5 catalog that spell them.
//!
//! The behaviour is pinned black-box by the `check-only-*` and
//! `check-rule-applies-to-nothing-*` e2e cases; these are the seams under them.

use std::collections::BTreeSet;
use std::path::Path;

use super::armed::{
    ArmedChecks, advised, advised_codes, switch_clause, switch_of, switching_clauses,
};
use super::selection::CheckFindingSelection;
use crate::model::{Armed, Code, Finding, Report, Switch};

fn finding(code: &'static str, authority: &[&str]) -> Finding {
    Finding {
        severity: "warning",
        code,
        path: None,
        line: None,
        column: None,
        message: code.to_string(),
        sites: Vec::new(),
        authority: authority.iter().map(|origin| origin.to_string()).collect(),
    }
}

fn armed(switches: &[Switch], version: u32) -> ArmedChecks {
    let mut armed = Armed::default();
    for switch in switches {
        armed.arm(*switch);
    }
    ArmedChecks::new(armed, version)
}

fn selection(only: &[&str], ignore: &[&str]) -> CheckFindingSelection {
    let mut selection = CheckFindingSelection::default();
    for code in only {
        selection.add_only(code).expect("a catalog code");
    }
    for code in ignore {
        selection.add_ignore(code).expect("a catalog code");
    }
    selection
}

fn messages(report: &Report) -> Vec<&str> {
    report
        .warnings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect()
}

/// §FS-check.1.4: a rule-produced code carries every `empty-citation-obligation`
/// row as it carries `invalid-rule`; a code no rule produces carries neither,
/// and `--ignore empty-citation-obligation` still wins.
#[test]
fn a_rule_produced_code_carries_the_empty_obligation_line() {
    assert!(selection(&["missing-citation"], &[]).retains("empty-citation-obligation", &[]));
    assert!(selection(&["suggested-citation"], &[]).retains("empty-citation-obligation", &[]));
    assert!(!selection(&["dangling"], &[]).retains("empty-citation-obligation", &[]));
    assert!(
        !selection(&["missing-citation"], &["empty-citation-obligation"])
            .retains("empty-citation-obligation", &[])
    );
}

/// §FS-check.1.4.1: a selected code no gate armed earns one line naming its
/// switch in the version the run read; an armed one earns none.
#[test]
fn an_unarmed_selected_code_earns_one_off_line() {
    let only = selection(&["oversized-lead"], &[]);
    let report = Report::default();

    let off = only.select(&report, &armed(&[], 2));
    assert_eq!(
        messages(&off),
        [
            "--only oversized-lead judged nothing — the check is off until [schema.leads.words] warn = <N> is set"
        ]
    );
    assert_eq!(off.warnings[0].code, "selected-check-off");
    assert!(off.warnings[0].authority.is_empty() && off.warnings[0].path.is_none());

    let v1 = only.select(&report, &armed(&[], 1));
    assert_eq!(
        messages(&v1),
        [
            "--only oversized-lead judged nothing — the check is off until [reference] lead_size_warning = { max = <N>, unit = \"words\" } is set"
        ]
    );

    assert!(
        only.select(&report, &armed(&[Switch::LeadWords], 2))
            .warnings
            .is_empty()
    );
}

/// §FS-check.1.4.1: `--ignore` of the code, or of `selected-check-off`, removes
/// the line, and a code with no switch never earns one.
#[test]
fn ignore_removes_the_off_line() {
    let report = Report::default();
    let none = armed(&[], 2);
    assert!(
        selection(&["oversized-lead"], &["oversized-lead"])
            .select(&report, &none)
            .warnings
            .is_empty()
    );
    assert!(
        selection(&["oversized-lead"], &["selected-check-off"])
            .select(&report, &none)
            .warnings
            .is_empty()
    );
    assert!(
        selection(&["dangling"], &[])
            .select(&report, &none)
            .warnings
            .is_empty()
    );
}

/// §FS-check.1.4.1: the lines come in code order, whatever order the flags came
/// in, and name only what the run lacks of each cell.
#[test]
fn off_lines_keep_code_order_and_name_only_the_missing_part() {
    let report = Report::default();
    let only = selection(&["suggested-citation", "out-of-scope-dangling"], &[]);
    assert_eq!(
        messages(&only.select(&report, &armed(&[Switch::ShouldDirection], 2))),
        [
            "--only out-of-scope-dangling judged nothing — the check is off until --full is passed",
            "--only suggested-citation judged nothing — the check is off until --suggestions is passed",
        ]
    );
}

/// §FS-check.1.4.1: the line is about the selection, so the authority axis of
/// `--only-rule` passes it while it drops a finding no rule authored.
#[test]
fn the_off_line_passes_the_authority_axis() {
    let mut only = selection(&["oversized-lead"], &[]);
    only.scope_to_trial_rule();
    assert!(only.retains("selected-check-off", &[]));
    let report = Report {
        warnings: vec![finding("oversized-lead", &[])],
        ..Report::default()
    };
    let selected = only.select(&report, &armed(&[], 2));
    assert_eq!(selected.warnings.len(), 1);
    assert_eq!(selected.warnings[0].code, "selected-check-off");
}

/// §FS-check.1.4.1: a switch a version-1 file cannot hold names the migration,
/// and a spent code names its replacement or says it never fires.
#[test]
fn version_one_and_spent_codes_have_their_own_advice() {
    let report = Report::default();
    assert_eq!(
        messages(&selection(&["missing-field"], &[]).select(&report, &armed(&[], 1))),
        [
            "--only missing-field judged nothing — a version-1 config cannot turn the check on; migrate it with grund config migrate --to 2"
        ]
    );
    assert_eq!(
        messages(&selection(&["undeclared-chapter"], &[]).select(&report, &armed(&[], 1))),
        [
            "--only undeclared-chapter judged nothing — a version-1 config cannot turn the check on; migrate it with grund config migrate --to 2"
        ]
    );
    let spent = selection(&["excluded-stub-target", "stub-shape"], &[]);
    assert_eq!(
        messages(&spent.select(&report, &armed(&[], 2))),
        [
            "--only excluded-stub-target judged nothing — the check is spent from grund 0.19.0; select broken-stub instead",
            "--only stub-shape judged nothing — the check is spent from grund 0.19.0 and never fires",
        ]
    );
}

/// §FS-check.1.4.1: a workspace whose projects read both versions names a
/// switch only where both spell it alike, a flag here, and otherwise the
/// migration of its version-1 projects, whether version 1 spells the switch
/// another way or cannot hold it at all.
#[test]
fn a_workspace_of_both_versions_names_only_a_shared_spelling() {
    let mut read = Armed::default();
    read.read(1);
    read.read(2);
    let mixed = ArmedChecks::new(read, 2);
    let only = selection(
        &["oversized-lead", "missing-field", "out-of-scope-dangling"],
        &[],
    );
    let migrate = "the run's projects read config versions 1 and 2, which do not turn it on alike; migrate the version-1 ones with grund config migrate --to 2";
    assert_eq!(
        messages(&only.select(&Report::default(), &mixed)),
        [
            format!("--only missing-field judged nothing — {migrate}"),
            "--only out-of-scope-dangling judged nothing — the check is off until --full is passed"
                .to_string(),
            format!("--only oversized-lead judged nothing — {migrate}"),
        ]
    );
}

/// Codes the catalog enables by the very thing they judge: a run with nothing
/// for them did pass, so they are always armed and have no row (§FS-check.1.4.1).
/// `inline-citation-style` is on by default, and the code of the line itself
/// needs no switch.
const ALWAYS_ARMED: &[&str] = &[
    "config-version-1",
    "inline-citation-style",
    "invalid-section-lock",
    "section-address-changed",
    "selected-check-off",
    "unread-scan-root",
    "unrecorded-section-citation",
];

/// The §FS-errors.5.5 catalog rows, as `(code, severity, enabled by)`, each
/// cell found by its column's header.
fn catalog() -> Vec<(String, String, String)> {
    let page =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/functional-spec/FS-errors.md");
    let text = std::fs::read_to_string(page).expect("read the catalog");
    let start = text.find("### 5.5 ").expect("the catalog heading");
    let cells = |line: &str| -> Vec<String> {
        let line = line.trim().trim_matches('|');
        line.split('|')
            .map(|cell| cell.trim().to_string())
            .collect()
    };
    let mut lines = text[start..]
        .lines()
        .skip_while(|line| !line.starts_with("| Code |"));
    let header = cells(lines.next().expect("the catalog's header row"));
    let column = |name: &str| header.iter().position(|cell| cell == name).expect(name);
    let (severity, enabled) = (column("Severity"), column("Enabled by"));
    lines
        .take_while(|line| line.starts_with('|'))
        .filter_map(|line| {
            let row = cells(line);
            let code = row[0].strip_prefix('`')?.strip_suffix('`')?.to_string();
            Some((code, row[severity].clone(), row[enabled].clone()))
        })
        .collect()
}

/// §FS-check.1.4.1: the off line advises exactly the codes the catalog's own
/// page says a key or flag turns on, or that are spent — read from the
/// markdown, so the rendered rows and the reading of them agree with it.
#[test]
fn the_advice_covers_exactly_the_switched_catalog_codes() {
    let selectable: BTreeSet<&str> = Code::ALL.iter().map(|code| code.as_str()).collect();
    let expected: BTreeSet<String> = catalog()
        .into_iter()
        .filter(|(code, severity, enabled)| {
            selectable.contains(code.as_str())
                && !ALWAYS_ARMED.contains(&code.as_str())
                && (enabled != "—" || severity.contains("spent"))
        })
        .map(|(code, ..)| code)
        .collect();
    let advised: BTreeSet<String> = advised_codes()
        .map(|(code, _)| code.as_str().to_string())
        .collect();
    assert_eq!(advised, expected);
}

/// §FS-errors.5.5.1: one clause is one switch spelled in each version, so every
/// clause a key or flag turns on is exactly one switch, and every switch names
/// such a clause.
#[test]
fn every_switching_clause_is_exactly_one_switch() {
    for (code, index) in switching_clauses() {
        assert_eq!(
            switch_of(code, index).len(),
            1,
            "clause {index} of `{}`",
            code.as_str()
        );
    }
    for switch in SWITCHES {
        let (code, index) = switch_clause(*switch).expect("every switch names its clause");
        assert_eq!(switch_of(code, index), [*switch], "{switch:?}");
    }
}

/// Every switch a gate records (§FS-check.1.4.1).
const SWITCHES: &[Switch] = &[
    Switch::LeadWords,
    Switch::Rules,
    Switch::MustDirection,
    Switch::MustNotDirection,
    Switch::Obligation,
    Switch::ShouldDirection,
    Switch::ShouldNotDirection,
    Switch::Suggestions,
    Switch::Full,
    Switch::Values,
    Switch::Field,
    Switch::FieldOneOf,
    Switch::FieldPlacement,
    Switch::FieldTitle,
    Switch::ClosedKind,
    Switch::FrontMatterField,
    Switch::NamedSections,
    Switch::HeadingDepth,
    Switch::Grounding,
    Switch::OptionalMember,
    Switch::SnapshotWarning,
    Switch::KindHome,
    Switch::ValueHome,
];

/// §FS-check.1.4.1: every switch the catalog names is spelled under both config
/// versions, except the field switches a version-1 file cannot hold, which name
/// the migration, and named sections, which version 2 never turns off.
#[test]
fn every_switch_is_spelled_in_both_config_versions() {
    let mut v1_absent = BTreeSet::new();
    let mut v2_absent = BTreeSet::new();
    for switch in advised_codes()
        .flat_map(|(_, switches)| switches)
        .collect::<BTreeSet<_>>()
    {
        let (v1, v2) = advised(switch);
        if v1.is_none() {
            v1_absent.insert(switch);
        }
        if v2.is_none() {
            v2_absent.insert(switch);
        }
        for spelled in [v1, v2].into_iter().flatten() {
            assert!(
                spelled.contains(" is set") || spelled.contains(" is passed"),
                "{spelled}"
            );
        }
    }
    assert_eq!(
        v1_absent,
        BTreeSet::from([
            Switch::Field,
            Switch::FieldOneOf,
            Switch::FieldPlacement,
            Switch::FieldTitle,
            Switch::ClosedKind,
            Switch::FrontMatterField,
        ])
    );
    assert_eq!(v2_absent, BTreeSet::from([Switch::NamedSections]));
}
