//! The release windows in which a finding that only a newly counted
//! numbered-section citation (§FS-rules.7.8), or only a target newly matched at
//! the rule's own nested alias (§FS-rules.7.9), produces warns rather than
//! fails. Those two, the site mark they read and the second field of
//! `evaluate` go away at 0.18.0; the window of a rule declared through a stub
//! whose target the scan does not read (§FS-rules.7.10), and of a citation or
//! chapter such a target lends (§FS-rules.7.11), go at 0.19.0, and so does the
//! window of a citation of a name under a number (§FS-rules.7.12,
//! §FS-citations.checks.mixed-path).

use super::super::facts::{NodeKey, RuleFacts, SiteKey};
use super::super::{Cardinality, RuleLevel, RuleTargets};
use super::index::FactIndex;
use crate::model::Diagnostic;
use once_cell::sync::Lazy;
use std::collections::BTreeSet;

/// The pending clause of §FS-distribution.4.2.3's closed vocabulary, after the
/// error's message unchanged, so the release guard reads the promise (§FS-rules.7.8).
const RAMP_CLAUSE: &str = "; a citation to a numbered section now counts, and this warning becomes an error in grund 0.18.0";

/// The same pending clause for the corrected pinned-target match (§FS-rules.7.9).
const MATCH_CLAUSE: &str = "; a target pinned at the rule's own nested alias path now matches, and this warning becomes an error in grund 0.18.0";

/// The same pending clause for a rule declared through a stub (§FS-rules.7.10).
const STUB_CLAUSE: &str = "; a rule declared through a stub now applies, and this warning becomes an error in grund 0.19.0";

/// The same pending clause for what a stub's target lends (§FS-rules.7.11), which
/// the `[citations]` directions name too (§FS-check.3.11, §FS-check.3.12).
pub(crate) const LENT_CLAUSE: &str = "; a citation or chapter a stub's target holds now counts, and this warning becomes an error in grund 0.19.0";

/// The release at which a citation of a name under a number stops warning: its
/// `missing-section` and `dangling` (§FS-citations.checks.mixed-path) and the
/// rule and `[citations]` findings only such citations raise (§FS-rules.7.12)
/// become errors, and the window and its clauses go. A unit test holds it ahead
/// of the crate version, as every pending clause is.
pub(super) const MIXED_PATH_FLIP: &str = "0.19.0";

/// The same pending clause for a citation of a name under a number, naming
/// [`MIXED_PATH_FLIP`] (§FS-citations.checks.mixed-path, §FS-rules.7.12).
pub(crate) static MIXED_CLAUSE: Lazy<String> = Lazy::new(|| {
    format!(
        "; a citation of a name under a number is now checked, and this warning becomes an \
         error in grund {MIXED_PATH_FLIP}"
    )
});

/// What an unrecorded mixed address adds to its error, which goes with the window
/// at [`MIXED_PATH_FLIP`] (§FS-section-lock.checks.unrecorded-section-citation).
pub(crate) const UNRECORDED_MIXED_CLAUSE: &str = "; grund 0.17 skipped a citation of a name under a number, and 0.18.0 records it in the section lock";

/// What one counted fact is new by: a numbered section newly counted
/// (§FS-rules.7.8), a target newly matched (§FS-rules.7.9), a site or chapter a
/// stub's target lends (§FS-rules.7.11), or a citation of a name under a number
/// (§FS-rules.7.12).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Marks {
    pub(super) counted: bool,
    pub(super) matched: bool,
    pub(super) lent: bool,
    pub(super) mixed: bool,
}

/// Which windows a finding falls in. Any may hold, and then the finding names
/// each, in the order of their subsections (§FS-rules.7.9, §FS-rules.7.11,
/// §FS-rules.7.12).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Windows {
    pub(super) counted: bool,
    pub(super) matched: bool,
    pub(super) lent: bool,
    pub(super) mixed: bool,
}

impl Windows {
    /// Only a required-level finding owes a ramp (§FS-rules.7.8, §FS-rules.7.9,
    /// §FS-rules.7.11, §FS-rules.7.12).
    pub(super) fn at(level: RuleLevel, marks: Marks) -> Self {
        if level == RuleLevel::Required {
            Self {
                counted: marks.counted,
                matched: marks.matched,
                lent: marks.lent,
                mixed: marks.mixed,
            }
        } else {
            Self::default()
        }
    }

    /// A required count that fails with the marked sites and holds without them is
    /// ramped; one that fails without them stays an error. It names a window when
    /// dropping that window's sites alone lets the count hold, and every window
    /// with a marked site when none alone does, since then each change produces it
    /// by itself (§FS-rules.7.8, §FS-rules.7.9, §FS-rules.7.11, §FS-rules.7.12).
    pub(super) fn of_count(
        level: RuleLevel,
        cardinality: Cardinality,
        sites: impl IntoIterator<Item = Marks>,
    ) -> Self {
        let sites: Vec<Marks> = sites.into_iter().collect();
        let holds_without = |marked: &dyn Fn(&Marks) -> bool| {
            cardinality.contains(sites.iter().filter(|site| !marked(site)).count())
        };
        if level != RuleLevel::Required
            || !holds_without(&|site| site.counted || site.matched || site.lent || site.mixed)
        {
            return Self::default();
        }
        let alone = Self {
            counted: holds_without(&|site| site.counted),
            matched: holds_without(&|site| site.matched),
            lent: holds_without(&|site| site.lent),
            mixed: holds_without(&|site| site.mixed),
        };
        if alone.any() {
            return alone;
        }
        Self {
            counted: sites.iter().any(|site| site.counted),
            matched: sites.iter().any(|site| site.matched),
            lent: sites.iter().any(|site| site.lent),
            mixed: sites.iter().any(|site| site.mixed),
        }
    }

    /// A per-target finding for a target only the corrected match takes into the
    /// universe exists only now, whatever its count, so it is always in
    /// §FS-rules.7.9's window, and in §FS-rules.7.8's, §FS-rules.7.11's or
    /// §FS-rules.7.12's only when its count holds without that window's sites.
    pub(super) fn of_new_target(
        level: RuleLevel,
        cardinality: Cardinality,
        sites: impl IntoIterator<Item = Marks>,
    ) -> Self {
        let sites: Vec<Marks> = sites.into_iter().collect();
        let holds_without = |marked: &dyn Fn(&Marks) -> bool| {
            cardinality.contains(sites.iter().filter(|site| !marked(site)).count())
        };
        let marks = Marks {
            counted: holds_without(&|site| site.counted),
            matched: true,
            lent: holds_without(&|site| site.lent),
            mixed: holds_without(&|site| site.mixed),
        };
        Self::at(level, marks)
    }

    /// A finding about a chapter a stub's target lends exists only because it is
    /// lent, whatever its count (§FS-rules.7.11).
    pub(super) fn of_subject(self, level: RuleLevel, lent: bool) -> Self {
        Self {
            lent: self.lent || (lent && level == RuleLevel::Required),
            ..self
        }
    }

    fn any(self) -> bool {
        self.counted || self.matched || self.lent || self.mixed
    }
}

/// The marks of one site: newly counted, lent and mixed read off its facts, newly
/// matched as the caller found it.
pub(super) fn site_marks(facts: &RuleFacts, site: &SiteKey, matched: bool) -> Marks {
    let meta = facts.sites.get(site);
    Marks {
        counted: meta.is_some_and(|meta| meta.newly_counted),
        matched,
        lent: meta.is_some_and(|meta| meta.lent),
        mixed: meta.is_some_and(|meta| meta.mixed),
    }
}

/// Whether a stub's target lends the node, a chapter (§FS-rules.7.11).
pub(super) fn node_lent(facts: &RuleFacts, node: &NodeKey) -> bool {
    facts.nodes.get(node).is_some_and(|meta| meta.lent)
}

/// Whether the matcher before §FS-rules.7.9's correction took `kind` for one of
/// the targets: it split each entry at its first `/`, so a nested alias path
/// pinned itself as alias `group` and kind `api/AR` and matched nothing.
pub(super) fn matched_before(targets: &RuleTargets, kind: &str, facts: &RuleFacts) -> bool {
    let RuleTargets::Kinds { values, .. } = targets else {
        return false;
    };
    // coordinate-lexer: the old first-`/` matcher, kept for the §FS-rules.7.9 ramp to 0.18.0
    values.iter().any(|target| match target.split_once('/') {
        None => target == kind,
        Some(("*", target_kind)) => {
            kind == target_kind || kind.ends_with(&format!("/{target_kind}"))
        }
        Some((project, target_kind)) => {
            kind == format!("{project}/{target_kind}")
                || (project == facts.header.project && kind == target_kind)
        }
    })
}

/// The declarations the matcher before the correction took for the targets, to
/// tell a newly matched site from one already matched (§FS-rules.7.9).
pub(super) fn nodes_matched_before(
    targets: &RuleTargets,
    facts: &RuleFacts,
    index: &FactIndex<'_>,
) -> BTreeSet<NodeKey> {
    index
        .declarations
        .iter()
        .filter(|(kind, _)| matched_before(targets, kind, facts))
        .flat_map(|(_, nodes)| nodes.iter().map(|node| (*node).clone()))
        .collect()
}

/// Pick the channel for one finding and word its message for it: a ramped
/// finding keeps its code and its message, then names its error release
/// (§FS-rules.7.8, §FS-rules.7.9, §FS-rules.7.11, §FS-rules.7.12).
pub(super) fn route<'a>(
    windows: Windows,
    out: &'a mut Vec<Diagnostic>,
    ramp: &'a mut Vec<Diagnostic>,
    mut message: String,
) -> (&'a mut Vec<Diagnostic>, String) {
    if !windows.any() {
        return (out, message);
    }
    if windows.counted {
        message.push_str(RAMP_CLAUSE);
    }
    if windows.matched {
        message.push_str(MATCH_CLAUSE);
    }
    if windows.lent {
        message.push_str(LENT_CLAUSE);
    }
    if windows.mixed {
        message.push_str(&MIXED_CLAUSE);
    }
    (ramp, message)
}

/// §FS-rules.7.10: a required-level finding that only rules declared through a
/// stub whose target the scan does not read produce — every rule its authority
/// names is one of `stubbed` — moves from `errors` to `warnings`, and a finding
/// already ramped by §FS-rules.7.8 or §FS-rules.7.9 keeps its clauses and gains
/// this one after them, and before §FS-rules.7.11's and §FS-rules.7.12's, in
/// subsection order. A rule's
/// own `invalid-rule` is about the rule, not by it, and stays an error as it is for
/// an inline rule.
pub(crate) fn route_stubbed_rules(
    stubbed: &BTreeSet<String>,
    errors: &mut Vec<Diagnostic>,
    warnings: &mut Vec<Diagnostic>,
) {
    if stubbed.is_empty() {
        return;
    }
    let only_stubbed = |diagnostic: &Diagnostic| {
        diagnostic.code != "invalid-rule"
            && !diagnostic.authority.is_empty()
            && diagnostic
                .authority
                .iter()
                .all(|origin| stubbed.contains(origin))
    };
    for warning in warnings.iter_mut().filter(|warning| only_stubbed(warning)) {
        push_stub_clause(&mut warning.message);
    }
    let (ramped, kept): (Vec<_>, Vec<_>) =
        std::mem::take(errors).into_iter().partition(only_stubbed);
    *errors = kept;
    warnings.extend(ramped.into_iter().map(|mut diagnostic| {
        push_stub_clause(&mut diagnostic.message);
        diagnostic
    }));
}

/// §FS-rules.7.11, §FS-rules.7.12: a finding the engine already ramped for what a
/// stub's target lends, or for a citation of a name under a number, keeps those
/// clauses last, so the clauses read in subsection order.
fn push_stub_clause(message: &mut String) {
    let mut later = String::new();
    for clause in [MIXED_CLAUSE.as_str(), LENT_CLAUSE] {
        if let Some(head) = message.strip_suffix(clause) {
            later.insert_str(0, clause);
            message.truncate(head.len());
        }
    }
    message.push_str(STUB_CLAUSE);
    message.push_str(&later);
}
