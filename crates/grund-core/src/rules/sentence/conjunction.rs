//! The conjunction refusal of §FS-rule-refusals.sentences.conjunction.

use super::modality::split_modality;

/// §FS-rule-refusals.sentences.conjunction: a sentence whose ` and ` is followed by `must ` or
/// `should ` joins clauses, and is answered with one sentence per clause, each
/// later clause given the first clause's subject (§FS-rules.3.6). Runs before
/// the modality split, so no fragment of the sentence is read as a kind; each
/// form is then read again like any other (§FS-rule-refusals.sentences.rewrite.reparsed).
pub(super) fn conjunction_forms(sentence: &str) -> Option<Vec<String>> {
    let mut clauses = Vec::new();
    let mut start = 0;
    for at in joins(sentence) {
        clauses.push(&sentence[start..at]);
        start = at + " and ".len();
    }
    let (first, rest) = clauses.split_first()?;
    let subject = split_modality(first).map_or(*first, |(subject, ..)| subject);
    Some(
        std::iter::once(format!("{first}."))
            .chain(
                rest.iter()
                    .copied()
                    .chain([&sentence[start..]])
                    .map(|clause| format!("{subject} {clause}.")),
            )
            .collect(),
    )
}

/// The sentence's first clause: all of it, or what precedes its first join
/// (§FS-rule-refusals.sentences.conjunction), the clause the subject is read
/// from (§FS-rule-refusals.sentences.order).
pub(super) fn first_clause(sentence: &str) -> &str {
    joins(sentence)
        .next()
        .map_or(sentence, |at| &sentence[..at])
}

/// Where each ` and ` that joins clauses starts: one followed by `must ` or
/// `should `.
fn joins(sentence: &str) -> impl Iterator<Item = usize> + '_ {
    sentence.match_indices(" and ").filter_map(|(at, joiner)| {
        let next = &sentence[at + joiner.len()..];
        (next.starts_with("must ") || next.starts_with("should ")).then_some(at)
    })
}
