//! The release window in which a finding that a newly counted numbered-section
//! citation alone produces warns rather than fails (§FS-rules.7.8). This module,
//! the site mark it reads and the second field of `evaluate` go away at 0.18.0.

use super::super::facts::{RuleFacts, SiteKey};
use super::super::{Cardinality, RuleLevel};
use crate::model::Diagnostic;

/// The pending clause of §FS-distribution.4.2.3's closed vocabulary, after the
/// error's message unchanged, so the release guard reads the promise (§FS-rules.7.8).
const RAMP_CLAUSE: &str = "; a citation to a numbered section now counts, and this warning becomes an error in grund 0.18.0";

/// The site counts only because §FS-rules.5.1 now reads its numbered section.
pub(super) fn newly_counted(facts: &RuleFacts, site: &SiteKey) -> bool {
    facts.sites.get(site).is_some_and(|meta| meta.newly_counted)
}

/// A required-level count that fails with the newly counted sites and holds
/// without them is ramped; one that fails without them stays an error, and the
/// recommended level owes no ramp at all (§FS-rules.7.8).
pub(super) fn count_is_ramped(level: RuleLevel, cardinality: Cardinality, without: usize) -> bool {
    level == RuleLevel::Required && cardinality.contains(without)
}

/// Pick the channel for one finding and word its message for it: a ramped
/// finding keeps its code and its message, then names its error release
/// (§FS-rules.7.8).
pub(super) fn route<'a>(
    ramped: bool,
    out: &'a mut Vec<Diagnostic>,
    ramp: &'a mut Vec<Diagnostic>,
    message: String,
) -> (&'a mut Vec<Diagnostic>, String) {
    if ramped {
        (ramp, format!("{message}{RAMP_CLAUSE}"))
    } else {
        (out, message)
    }
}
