//! Where a sentence's subject ends (§FS-rules.3.6): the modality split a rule,
//! a conjunction and a pasted selector all read the subject through.

use super::{RuleLevel, RulePolarity};

/// A sentence split at its modality: the subject, the modality, the predicate.
pub(super) type Split<'a> = (&'a str, RuleLevel, RulePolarity, &'a str);

/// The accepted modalities, in the order the split looks for them (§FS-rules.3.6).
pub(super) const MODALITIES: [(&str, RuleLevel, RulePolarity); 4] = [
    (" must not ", RuleLevel::Required, RulePolarity::Prohibiting),
    (
        " should not ",
        RuleLevel::Recommended,
        RulePolarity::Prohibiting,
    ),
    (" must ", RuleLevel::Required, RulePolarity::Positive),
    (" should ", RuleLevel::Recommended, RulePolarity::Positive),
];

/// Split a sentence into its subject, its modality and its predicate. The
/// modality is found in a fixed order rather than at the earliest position
/// (§FS-rules.3.6), and a rule and a pasted selector both split here, so they
/// read the same subject from the same sentence (§FS-rule-refusals.selectors).
pub(super) fn split_modality(sentence: &str) -> Option<Split<'_>> {
    MODALITIES
        .into_iter()
        .find_map(|(modality, level, polarity)| {
            let (subject, predicate) = sentence.split_once(modality)?;
            Some((subject, level, polarity, predicate))
        })
}
