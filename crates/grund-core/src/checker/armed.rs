//! What turns a selected code on, read from the rendered §FS-errors.5.5 catalog
//! (§FS-check.1.4.1): each switch a gate records is one clause of the catalog's
//! *Enabled by* column, and the line a selected check the run left off earns is
//! spelled in that clause's own words, so the line and `grund config explain`
//! read one copy (§AR-config.9.1). It sits beside the selection in
//! `selection.rs`, which reads it to say a selected check was off.

use crate::model::remedies;
use std::collections::BTreeSet;

use crate::model::catalog::{self, ClauseBy, CodeSeverity};
use crate::model::{Armed, Code, Switch};

/// Where each switch is spelled: a row of the catalog and the position of its
/// clause there. A clause of any row that reads alike in both versions is the
/// same switch, so this names each switch once and holds none of its words.
const SWITCH_CLAUSES: &[(Switch, Code, usize)] = &[
    (Switch::LeadWords, Code::OVERSIZED_LEAD, 0),
    (Switch::Rules, Code::INVALID_RULE, 0),
    (Switch::MustDirection, Code::MISSING_CITATION, 0),
    (Switch::MustNotDirection, Code::FORBIDDEN_CITATION, 0),
    (Switch::Obligation, Code::EMPTY_CITATION_OBLIGATION, 0),
    (Switch::ShouldDirection, Code::SUGGESTED_CITATION, 1),
    (Switch::ShouldNotDirection, Code::DISCOURAGED_CITATION, 1),
    (Switch::Suggestions, Code::ESCAPED_CITATION_RESOLVES, 0),
    (Switch::Full, Code::OUT_OF_SCOPE_DANGLING, 0),
    (Switch::Values, Code::VALUE_MISMATCH, 0),
    (Switch::Field, Code::MISSING_FIELD, 0),
    (Switch::FieldOneOf, Code::INVALID_FIELD_VALUE, 0),
    (Switch::FieldPlacement, Code::MISPLACED_FIELD, 0),
    (Switch::FieldTitle, Code::MISTITLED_FIELD, 0),
    (Switch::ClosedKind, Code::UNDECLARED_CHAPTER, 0),
    (Switch::NamedSections, Code::ORPHAN_SECTION, 0),
    (Switch::HeadingDepth, Code::SECTION_HEADING_LEVEL, 0),
    (Switch::Grounding, Code::UNGROUNDED, 0),
    (Switch::OptionalMember, Code::OPTIONAL_MEMBER_ABSENT, 0),
    (Switch::SnapshotWarning, Code::MISSING_SNAPSHOT, 0),
    (Switch::KindHome, Code::MISPLACED_DECLARATION, 0),
];

/// Why a version-1 config is answered by the migration alone (§FS-check.1.4.1).
const MIGRATE: &str = "a version-1 config cannot turn the check on";

/// Why a workspace whose projects read both versions is answered by the migration
/// in place of a key one of them cannot carry (§FS-check.1.4.1).
const MIGRATE_MIXED: &str =
    "the run's projects read config versions 1 and 2, which do not turn it on alike";

/// What one run armed, and the config versions its advice is spelled in: the
/// record the hidden check path hands the CLI's selection (§FS-check.1.4.1).
#[doc(hidden)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArmedChecks {
    pub(crate) armed: Armed,
    pub(crate) versions: BTreeSet<u32>,
}

impl ArmedChecks {
    /// `version` is the run's own, which spells the advice where the run checked
    /// no project to read one from.
    pub(crate) fn new(armed: Armed, version: u32) -> Self {
        let mut versions = armed.versions().clone();
        if versions.is_empty() {
            versions.insert(version);
        }
        Self { armed, versions }
    }

    /// §FS-check.1.4.1: why `--only <code>` judged nothing, or `None` when the
    /// run armed it. The advice names only what the run lacks of the cell.
    pub(super) fn off_advice(&self, code: &str) -> Option<String> {
        let code = Code::named(code)?;
        let read = *self.versions.first()?;
        if let CodeSeverity::Spent(spent) = &catalog::row(code).side(read).severity {
            return Some(format!("the check is {spent}"));
        }
        let lacking: Vec<Option<&str>> = switches(code)
            .into_iter()
            .filter(|switch| !self.armed.contains(*switch))
            .map(|switch| spelling(switch, &self.versions))
            .collect();
        if lacking.is_empty() {
            return None;
        }
        // A switch a version read cannot hold, or two versions spell apart, is
        // answered by the migration alone.
        let Some(parts) = lacking.into_iter().collect::<Option<Vec<_>>>() else {
            // §FS-errors.5.5.1: the cases of the row's Fix, the one taking
            // config-version-1's words by reference.
            return Some(if self.versions.len() > 1 {
                format!(
                    "{MIGRATE_MIXED}; {}",
                    remedies::selected_check_off_mixed_versions()
                )
            } else {
                format!("{MIGRATE}; {}", remedies::selected_check_off_one_version())
            });
        };
        Some(remedies::selected_check_off_off(parts.join(" and ")))
    }
}

/// One clause of a row as both versions spell it: `None` on a side that is
/// always on or that the version does not have. A clause no key or flag turns
/// on is a condition the check judges, so it is no switch (§FS-errors.5.5.1).
fn clause(code: Code, index: usize) -> Option<[Option<&'static str>; 2]> {
    let row = catalog::row(code);
    let mut switched = false;
    let spelled = [2, 1].map(|version| {
        let clause = row.side(version).enabled.clauses().get(index)?;
        switched |= clause.by != ClauseBy::Condition;
        Some(clause.text.as_str())
    });
    switched.then_some(spelled)
}

/// The switches a code's *Enabled by* cell names, all of which it needs: none
/// for a code that is always on or turned on by what it judges.
fn switches(code: Code) -> Vec<Switch> {
    let row = catalog::row(code);
    let count = [&row.v2, &row.v1]
        .map(|side| side.enabled.clauses().len())
        .into_iter()
        .max()
        .unwrap_or(0);
    (0..count)
        .filter_map(|index| clause(code, index))
        .map(|spelled| {
            SWITCH_CLAUSES
                .iter()
                .find(|(_, at, index)| clause(*at, *index) == Some(spelled))
                .map(|(switch, ..)| *switch)
                .expect("every switching clause of the catalog is a switch")
        })
        .collect()
}

/// The one spelling of `switch` every version in `versions` shares, or `None`
/// where one of them cannot hold it or two spell it apart (§FS-check.1.4.1).
fn spelling(switch: Switch, versions: &BTreeSet<u32>) -> Option<&'static str> {
    let (_, code, index) = SWITCH_CLAUSES
        .iter()
        .find(|(row, ..)| *row == switch)
        .expect("every switch names its clause");
    let [v2, v1] = clause(*code, *index).expect("a switch's clause is one a key or flag sets");
    let mut spelled = versions
        .iter()
        .map(|version| if *version >= 2 { v2 } else { v1 });
    let first = spelled.next().flatten()?;
    spelled.all(|other| other == Some(first)).then_some(first)
}

/// Every code whose cell a key or flag turns on, with the switches it needs, or
/// that is spent: the codes the off line can advise about.
#[cfg(test)]
pub(super) fn advised_codes() -> impl Iterator<Item = (Code, Vec<Switch>)> {
    Code::ALL.iter().filter_map(|code| {
        let switches = switches(*code);
        let spent = matches!(catalog::row(*code).v2.severity, CodeSeverity::Spent(_));
        (spent || !switches.is_empty()).then_some((*code, switches))
    })
}

#[cfg(test)]
pub(super) fn advised(switch: Switch) -> (Option<&'static str>, Option<&'static str>) {
    (
        spelling(switch, &BTreeSet::from([1])),
        spelling(switch, &BTreeSet::from([2])),
    )
}

/// Every clause a key or flag turns on, by the row and position it stands at.
#[cfg(test)]
pub(super) fn switching_clauses() -> impl Iterator<Item = (Code, usize)> {
    Code::ALL.iter().flat_map(|code| {
        let row = catalog::row(*code);
        let count = row
            .v2
            .enabled
            .clauses()
            .len()
            .max(row.v1.enabled.clauses().len());
        (0..count)
            .filter(|index| clause(*code, *index).is_some())
            .map(|index| (*code, index))
    })
}

#[cfg(test)]
pub(super) fn switch_clause(switch: Switch) -> Option<(Code, usize)> {
    SWITCH_CLAUSES
        .iter()
        .find(|(row, ..)| *row == switch)
        .map(|(_, code, index)| (*code, *index))
}

#[cfg(test)]
pub(super) fn switch_of(code: Code, index: usize) -> Vec<Switch> {
    let spelled = clause(code, index);
    SWITCH_CLAUSES
        .iter()
        .filter(|(_, at, at_index)| spelled.is_some() && clause(*at, *at_index) == spelled)
        .map(|(switch, ..)| *switch)
        .collect()
}
