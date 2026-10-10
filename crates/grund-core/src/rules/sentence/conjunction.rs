//! The conjunction refusal of §FS-rules.3.5.5.

use super::split_modality;

/// §FS-rules.3.5.5: a sentence whose ` and ` is followed by `must ` or
/// `should ` joins clauses, and is answered with one sentence per clause, each
/// later clause given the first clause's subject (§FS-rules.3.6). Runs before
/// the modality split, so no fragment of the sentence is read as a kind; each
/// form is then read again like any other (§FS-rules.3.5.4.3).
pub(super) fn conjunction_forms(sentence: &str) -> Option<Vec<String>> {
    let mut clauses = Vec::new();
    let mut start = 0;
    for (at, joiner) in sentence.match_indices(" and ") {
        let next = &sentence[at + joiner.len()..];
        if next.starts_with("must ") || next.starts_with("should ") {
            clauses.push(&sentence[start..at]);
            start = at + joiner.len();
        }
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
