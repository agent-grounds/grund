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
    /// A `must` or `should` obligation, an entry's or a rule's, on a scanned kind
    /// with a folder home: no other kind's obligation can apply to nothing.
    Obligation,
    /// A `should` entry or rule.
    ShouldDirection,
    /// A `should-not` entry or rule.
    ShouldNotDirection,
    /// `--suggestions`.
    Suggestions,
    /// `--full`.
    Full,
    /// Value authority: a kind's values slot, or a value tag the scan found.
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
    /// A v2 kind's front-matter field (§FS-fields.front-matter).
    FrontMatterField,
    /// Named sections.
    NamedSections,
    /// A heading-depth mode that checks.
    HeadingDepth,
    /// A place grounding reaches: its row's own word, else the default that
    /// `[reference]` or `--require-grounding` sets, or a softer v2 rung.
    Grounding,
    /// `[workspace] optional_members`, in any block the run expands.
    OptionalMember,
    /// A fetched kind whose resolution is `should`, v2's `warn`: `fetch` alone
    /// resolves `must`, which selects the `dangling` error instead.
    SnapshotWarning,
    /// A configured kind home.
    KindHome,
    /// A whole-value kind, whose home may hold YAML value sources
    /// (§FS-values.2.2.4).
    ValueHome,
}

/// The switches one run armed, in a workspace the union of every project's,
/// and the config versions its projects read, which the advice is spelled in
/// (§FS-check.1.4.1).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Armed {
    switches: BTreeSet<Switch>,
    versions: BTreeSet<u32>,
}

impl Armed {
    pub(crate) fn arm(&mut self, switch: Switch) {
        self.switches.insert(switch);
    }

    /// Record the config version a checked project read.
    pub(crate) fn read(&mut self, version: u32) {
        self.versions.insert(version);
    }

    pub(crate) fn absorb(&mut self, other: &mut Armed) {
        self.switches.append(&mut other.switches);
        self.versions.append(&mut other.versions);
    }

    pub fn contains(&self, switch: Switch) -> bool {
        self.switches.contains(&switch)
    }

    pub(crate) fn versions(&self) -> &BTreeSet<u32> {
        &self.versions
    }
}
