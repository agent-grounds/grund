//! Who authored a finding (§FS-rules.6, §FS-rules.7.6): the origin set that
//! becomes the record's `authority` field, and the same set joined once for the
//! message tail. One value carries both, so the field and the words a reader
//! sees can never name a different set.

use std::collections::BTreeSet;
use std::fmt;

pub(super) struct Authority {
    /// The bytewise-sorted origins `check --only-rule` queries (§FS-rules.8).
    pub(super) origins: Vec<String>,
    tail: String,
}

impl Authority {
    /// One semantic group's contributing origins, in the order the `BTreeSet`
    /// already holds them — which is the order §FS-rules.6 fixes for the tail.
    pub(super) fn new(origins: BTreeSet<String>) -> Self {
        let origins = origins.into_iter().collect::<Vec<_>>();
        let tail = origins.join(", ");
        Self { origins, tail }
    }
}

impl fmt::Display for Authority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.tail)
    }
}

/// The authority of a diagnostic *about* one rule rather than an evaluation *by*
/// a group (§FS-rules.7.6). An `invalid-rule` never reaches the group join, so
/// there is no set to sort and no tail to render — and naming its rule anyway is
/// what keeps a trial sentence whose subject does not resolve inside a scoped
/// report instead of letting a typo read as `success`
/// (§DF-rule-authority-is-a-field.2). Both sites that raise one use this, so the
/// engine's own and the checker's configured-rule refusal cannot disagree.
pub(crate) fn one_rules_authority(origin: &str) -> Vec<String> {
    vec![origin.to_string()]
}
