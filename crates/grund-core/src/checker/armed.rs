//! The one internal table §FS-check.1.4.1 names: for each code the §FS-errors.5.5
//! catalog enables by a switch, the switches its *Enabled by* cell names, and how
//! each switch is spelled in the config version a run read. It sits beside the
//! code vocabulary in `selection.rs`, which reads it to say a selected check was
//! off, and no command lists it.

use crate::model::{Armed, Switch};

/// What turns a code on: the switches its catalog cell names, all of which it
/// needs, or the replacement a spent code points to (§FS-check.1.4.1).
enum Enablement {
    Switches(&'static [Switch]),
    /// Spent from 0.19.0, with the code that reports it now where there is one.
    Spent(Option<&'static str>),
}

use Enablement::{Spent, Switches};

/// The catalog's *Enabled by* column, for every code whose cell is a switch, in
/// code order (§FS-errors.5.5). A code with `—`, or enabled by the very thing it
/// judges, is always armed and has no row (§FS-check.1.4.1).
const CODE_SWITCHES: &[(&str, Enablement)] = &[
    ("chapter-cardinality", Switches(&[Switch::Rules])),
    ("citation-cardinality", Switches(&[Switch::Rules])),
    (
        "discouraged-citation",
        Switches(&[Switch::Suggestions, Switch::ShouldNotDirection]),
    ),
    ("empty-citation-obligation", Switches(&[Switch::Obligation])),
    (
        "escaped-citation-resolves",
        Switches(&[Switch::Suggestions]),
    ),
    ("excluded-stub-target", Spent(Some("broken-stub"))),
    ("forbidden-citation", Switches(&[Switch::MustNotDirection])),
    ("full-scope-ignored", Switches(&[Switch::Full])),
    ("invalid-field-value", Switches(&[Switch::FieldOneOf])),
    ("invalid-rule", Switches(&[Switch::Rules])),
    ("invalid-value-binding", Switches(&[Switch::Values])),
    ("invalid-value-declaration", Switches(&[Switch::Values])),
    ("misplaced-declaration", Switches(&[Switch::KindHome])),
    ("misplaced-field", Switches(&[Switch::FieldPlacement])),
    ("missing-citation", Switches(&[Switch::MustDirection])),
    ("missing-field", Switches(&[Switch::Field])),
    ("missing-snapshot", Switches(&[Switch::Fetch])),
    ("mistitled-field", Switches(&[Switch::FieldTitle])),
    (
        "optional-member-absent",
        Switches(&[Switch::OptionalMember]),
    ),
    ("orphan-section", Switches(&[Switch::NamedSections])),
    ("out-of-scope-dangling", Switches(&[Switch::Full])),
    (
        "out-of-scope-local-section-citation",
        Switches(&[Switch::Full]),
    ),
    ("out-of-scope-missing-section", Switches(&[Switch::Full])),
    ("out-of-scope-shorthand-citation", Switches(&[Switch::Full])),
    ("out-of-scope-unknown-project", Switches(&[Switch::Full])),
    ("oversized-lead", Switches(&[Switch::LeadWords])),
    ("section-heading-level", Switches(&[Switch::HeadingDepth])),
    ("stub-shape", Spent(None)),
    (
        "suggested-citation",
        Switches(&[Switch::Suggestions, Switch::ShouldDirection]),
    ),
    ("uncited-unit", Switches(&[Switch::Rules])),
    ("undeclared-chapter", Switches(&[Switch::ClosedKind])),
    ("ungrounded", Switches(&[Switch::Grounding])),
    ("unreached-declaration", Switches(&[Switch::Rules])),
    ("value-mismatch", Switches(&[Switch::Values])),
];

/// How each switch is spelled, completing "the check is off until …": under
/// version 1, then version 2. `None` is a switch that version cannot hold, which
/// the line answers with the migration instead (§FS-check.1.4.1).
const SWITCH_ADVICE: &[(Switch, Option<&str>, Option<&str>)] = &[
    (
        Switch::LeadWords,
        Some("[reference] lead_size_warning = { max = <N>, unit = \"words\" } is set"),
        Some("[schema.leads.words] warn = <N> is set"),
    ),
    (
        Switch::Rules,
        Some("rules = true is set on a [[kinds]] row or --rule is passed"),
        Some("form = \"rule\" is set on a [schema.kinds] table or --rule is passed"),
    ),
    (
        Switch::MustDirection,
        Some("[citations.<kind>] must or a must rule is set"),
        Some("[rules.citations.<kind>] must or a must rule is set"),
    ),
    (
        Switch::MustNotDirection,
        Some("[citations.<kind>] must-not or a must not rule is set"),
        Some("[rules.citations.<kind>] must-not or a must not rule is set"),
    ),
    (
        Switch::Obligation,
        Some("[citations.<kind>] must or should, or a must or should rule, is set"),
        Some("[rules.citations.<kind>] must or should, or a must or should rule, is set"),
    ),
    (
        Switch::ShouldDirection,
        Some("[citations.<kind>] should or a should rule is set"),
        Some("[rules.citations.<kind>] should or a should rule is set"),
    ),
    (
        Switch::ShouldNotDirection,
        Some("[citations.<kind>] should-not or a should not rule is set"),
        Some("[rules.citations.<kind>] should-not or a should not rule is set"),
    ),
    (
        Switch::Suggestions,
        Some("--suggestions is passed"),
        Some("--suggestions is passed"),
    ),
    (
        Switch::Full,
        Some("--full is passed"),
        Some("--full is passed"),
    ),
    (
        Switch::Values,
        Some("values = true is set on a [[kinds]] row"),
        Some("form = \"value\" is set on a [schema.kinds] table"),
    ),
    (
        Switch::Field,
        None,
        Some("[schema.kinds.<kind>.fields.<field>] presence = \"must\" is set"),
    ),
    (
        Switch::FieldOneOf,
        None,
        Some("[schema.kinds.<kind>.fields.<field>] one_of is set"),
    ),
    (
        Switch::FieldPlacement,
        None,
        Some("[schema.kinds.<kind>.fields.<field>] position or after is set"),
    ),
    (
        Switch::FieldTitle,
        None,
        Some("[schema.kinds.<kind>.fields.<field>] title is set"),
    ),
    (
        Switch::ClosedKind,
        None,
        Some("[schema.kinds.<kind>] closed = \"must\" is set"),
    ),
    // Version 2 always names sections, so it never asks.
    (
        Switch::NamedSections,
        Some("[id] named_sections = true is set"),
        None,
    ),
    (
        Switch::HeadingDepth,
        Some("[id] section_heading_levels = \"strict\" is set"),
        Some("[schema] heading_depth = \"must\" is set"),
    ),
    (
        Switch::Grounding,
        Some("[reference] require_grounding = true is set or --require-grounding is passed"),
        Some("a must rung of [rules.citations.grounding] is set or --require-grounding is passed"),
    ),
    (
        Switch::OptionalMember,
        Some("[workspace] optional_members is set"),
        Some("[workspace] optional_members is set"),
    ),
    (
        Switch::Fetch,
        Some("fetch is set on a [[kinds]] row"),
        Some("fetch is set on a [schema.kinds] table"),
    ),
    (
        Switch::KindHome,
        Some("folder or file is set on a [[kinds]] row"),
        Some("folders or files is set on a [schema.kinds] table"),
    ),
];

const MIGRATE: &str =
    "a version-1 config cannot turn the check on; migrate it with grund config migrate --to 2";

/// What one run armed, and the config version its advice is spelled in: the
/// record the hidden check path hands the CLI's selection (§FS-check.1.4.1).
#[doc(hidden)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArmedChecks {
    pub(crate) armed: Armed,
    pub(crate) version: u32,
}

impl ArmedChecks {
    pub(crate) fn new(armed: Armed, version: u32) -> Self {
        Self { armed, version }
    }

    /// §FS-check.1.4.1: why `--only <code>` judged nothing, or `None` when the
    /// run armed it. The advice names only what the run lacks of the cell.
    pub(super) fn off_advice(&self, code: &str) -> Option<String> {
        let (_, enablement) = CODE_SWITCHES.iter().find(|(row, _)| *row == code)?;
        let switches = match enablement {
            Spent(Some(replacement)) => {
                return Some(format!(
                    "the check is spent from grund 0.19.0; select {replacement} instead"
                ));
            }
            Spent(None) => {
                return Some("the check is spent from grund 0.19.0 and never fires".into());
            }
            Switches(switches) => switches,
        };
        let lacking: Vec<Option<&str>> = switches
            .iter()
            .filter(|switch| !self.armed.contains(**switch))
            .map(|switch| spelling(*switch, self.version))
            .collect();
        if lacking.is_empty() {
            return None;
        }
        // A switch this version cannot hold is answered by the migration alone.
        let Some(parts) = lacking.into_iter().collect::<Option<Vec<_>>>() else {
            return Some(MIGRATE.into());
        };
        Some(format!("the check is off until {}", parts.join(" and ")))
    }
}

fn spelling(switch: Switch, version: u32) -> Option<&'static str> {
    let (_, v1, v2) = SWITCH_ADVICE
        .iter()
        .find(|(row, ..)| *row == switch)
        .expect("every switch has an advice row");
    if version >= 2 { *v2 } else { *v1 }
}

#[cfg(test)]
pub(super) fn table_codes() -> impl Iterator<Item = (&'static str, Option<&'static [Switch]>)> {
    CODE_SWITCHES.iter().map(|(code, enablement)| {
        (
            *code,
            match enablement {
                Switches(switches) => Some(*switches),
                Spent(_) => None,
            },
        )
    })
}

#[cfg(test)]
pub(super) fn advised(switch: Switch) -> (Option<&'static str>, Option<&'static str>) {
    (spelling(switch, 1), spelling(switch, 2))
}
