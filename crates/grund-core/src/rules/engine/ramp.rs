//! The release windows in which a finding that only a target newly matched at
//! the rule's own nested alias (§FS-rules.7.9) or a citation of a name under a
//! number (§FS-rules.7.12, §FS-citations.checks.mixed-path) produces warns rather
//! than fails. Both, the site marks they read and the second field of `evaluate`
//! go away at 0.19.0. A numbered-section citation counts as any other site since
//! §FS-rules.7.8 landed, and a rule declared through a stub and what a stub's
//! target lends since §FS-rules.7.10 and §FS-rules.7.11 closed, so none of them
//! marks anything here.

use super::super::facts::{NodeKey, RuleFacts, SiteKey};
use super::super::{Cardinality, RuleLevel, RuleTargets};
use super::index::FactIndex;
use crate::model::Diagnostic;
use once_cell::sync::Lazy;
use std::collections::BTreeSet;

/// The pending clause of §FS-distribution.4.2.3's closed vocabulary, after the
/// error's message unchanged, so the release guard reads the promise: the one for
/// the corrected pinned-target match (§FS-rules.7.9).
const MATCH_CLAUSE: &str = "; a target pinned at the rule's own nested alias path now matches, and this warning becomes an error in grund 0.19.0";

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

/// What one counted fact is new by: a target newly matched (§FS-rules.7.9), or a
/// citation of a name under a number (§FS-rules.7.12).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Marks {
    pub(super) matched: bool,
    pub(super) mixed: bool,
}

/// Which windows a finding falls in. Either may hold, and then the finding names
/// each, in the order of their subsections (§FS-rules.7.9, §FS-rules.7.12).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Windows {
    pub(super) matched: bool,
    pub(super) mixed: bool,
}

impl Windows {
    /// Only a required-level finding owes a ramp (§FS-rules.7.9, §FS-rules.7.12).
    pub(super) fn at(level: RuleLevel, marks: Marks) -> Self {
        if level == RuleLevel::Required {
            Self {
                matched: marks.matched,
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
    /// by itself (§FS-rules.7.9, §FS-rules.7.12). A numbered-section
    /// site is an ordinary site, so a count it alone breaks is an error (§FS-rules.7.8).
    pub(super) fn of_count(
        level: RuleLevel,
        cardinality: Cardinality,
        sites: impl IntoIterator<Item = Marks>,
    ) -> Self {
        let sites: Vec<Marks> = sites.into_iter().collect();
        let holds_without = |marked: &dyn Fn(&Marks) -> bool| {
            cardinality.contains(sites.iter().filter(|site| !marked(site)).count())
        };
        if level != RuleLevel::Required || !holds_without(&|site| site.matched || site.mixed) {
            return Self::default();
        }
        let alone = Self {
            matched: holds_without(&|site| site.matched),
            mixed: holds_without(&|site| site.mixed),
        };
        if alone.any() {
            return alone;
        }
        Self {
            matched: sites.iter().any(|site| site.matched),
            mixed: sites.iter().any(|site| site.mixed),
        }
    }

    /// A per-target finding for a target only the corrected match takes into the
    /// universe exists only now, whatever its count, so it is always in
    /// §FS-rules.7.9's window, and in §FS-rules.7.12's only when its count holds
    /// without that window's sites.
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
            matched: true,
            mixed: holds_without(&|site| site.mixed),
        };
        Self::at(level, marks)
    }

    fn any(self) -> bool {
        self.matched || self.mixed
    }
}

/// The marks of one site: mixed read off its facts, newly matched as the caller
/// found it.
pub(super) fn site_marks(facts: &RuleFacts, site: &SiteKey, matched: bool) -> Marks {
    let meta = facts.sites.get(site);
    Marks {
        matched,
        mixed: meta.is_some_and(|meta| meta.mixed),
    }
}

/// Whether the matcher before §FS-rules.7.9's correction took `kind` for one of
/// the targets: it split each entry at its first `/`, so a nested alias path
/// pinned itself as alias `group` and kind `api/AR` and matched nothing.
pub(super) fn matched_before(targets: &RuleTargets, kind: &str, facts: &RuleFacts) -> bool {
    let RuleTargets::Kinds { values, .. } = targets else {
        return false;
    };
    // coordinate-lexer: the old first-`/` matcher, kept for the §FS-rules.7.9 ramp to 0.19.0
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
/// (§FS-rules.7.9, §FS-rules.7.12).
pub(super) fn route<'a>(
    windows: Windows,
    out: &'a mut Vec<Diagnostic>,
    ramp: &'a mut Vec<Diagnostic>,
    mut message: String,
) -> (&'a mut Vec<Diagnostic>, String) {
    if !windows.any() {
        return (out, message);
    }
    if windows.matched {
        message.push_str(MATCH_CLAUSE);
    }
    if windows.mixed {
        message.push_str(&MIXED_CLAUSE);
    }
    (ramp, message)
}
