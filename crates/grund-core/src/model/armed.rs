//! What a run turned on (§FS-check.1.4.1): each gate that decides whether a
//! check runs records the switch it found set, so "is this check on" is the one
//! fact that gate already decides rather than a predicate re-read from config.
//! The record rides beside the report's three channels (§AR-checker.1) and only
//! the CLI's selection reads it; the published report never carries it.

use std::collections::BTreeSet;

/// One part of a §FS-errors.5.5 *Enabled by* cell: a config key, a flag, or a
/// declaration whose presence turns a check on. A code is armed when every
/// switch its row names is armed (§FS-check.1.4.1).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Switch {
    /// `[schema.leads.words]`, v1's `[reference] lead_size_warning`.
    LeadWords,
    /// A rule kind or a `--rule` sentence.
    Rules,
    /// A `must` or `warn` citation-direction entry, or a `must` rule.
    MustDirection,
    /// A `must-not` or `warn-not` entry, or a `must not` rule.
    MustNotDirection,
    /// A `must` or `should` obligation, an entry's or a rule's.
    Obligation,
    /// A `should` entry or rule.
    ShouldDirection,
    /// A `should-not` entry or rule.
    ShouldNotDirection,
    /// `--suggestions`.
    Suggestions,
    /// `--full`.
    Full,
    /// A value kind.
    Values,
    /// A v2 field a declaration can miss.
    Field,
    /// `one_of` on a v2 field.
    FieldOneOf,
    /// `position` or `after` on a v2 field.
    FieldPlacement,
    /// `title` on a v2 field.
    FieldTitle,
    /// `closed` on a v2 kind.
    ClosedKind,
    /// Named sections.
    NamedSections,
    /// A heading-depth mode that checks.
    HeadingDepth,
    /// A grounded place, by config or `--require-grounding`.
    Grounding,
    /// `[workspace] optional_members`.
    OptionalMember,
    /// `fetch` on a kind.
    Fetch,
    /// A configured kind home.
    KindHome,
}

/// The switches one run armed, in a workspace the union of every project's
/// (§FS-check.1.4.1).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Armed(BTreeSet<Switch>);

impl Armed {
    pub(crate) fn arm(&mut self, switch: Switch) {
        self.0.insert(switch);
    }

    pub(crate) fn absorb(&mut self, other: &mut Armed) {
        self.0.append(&mut other.0);
    }

    pub fn contains(&self, switch: Switch) -> bool {
        self.0.contains(&switch)
    }
}
