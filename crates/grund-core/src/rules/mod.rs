//! Controlled-English chapter rules (§FS-rules, §AR-rules).
//!
//! The boundary exposes normalized sentences, producer-neutral facts, the
//! Markdown adapter, and engine diagnostics as separate modules. The checker
//! composes them; none calls another producer.

pub(crate) mod engine;
pub(crate) mod facts;
pub(crate) mod markdown;
pub(crate) mod sentence;

/// Repository-relative anchor shared by the two boundary representations
/// (§AR-rules.2, §AR-rules.3).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct RuleAnchor {
    pub(crate) path: String,
    pub(crate) line: usize,
    pub(crate) column: Option<usize>,
}

// The engine consumes the normalized representation through the component
// boundary, never through the sentence front end itself (§AR-rules.2,
// §AR-rules.4).
pub(crate) use sentence::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    TargetMode,
};

#[cfg(test)]
mod tests_boundaries;
