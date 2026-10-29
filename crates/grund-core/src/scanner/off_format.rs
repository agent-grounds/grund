//! The off-format reading (§FS-citations.recognized.off-format): where the
//! promotion step of `legacy.rs` finds no declaration behind a deferred
//! candidate, the candidate is recorded rather than dropped when it opens like
//! an ID of a configured kind that its kind's format does not spell and no
//! other reading claimed its marker. Still a catalog operation: the record is a
//! site for §FS-check.checks.off-format-citation and never a citation, so no
//! edge is taken for it (§FS-config.3.2.5).

use crate::config::{Frame, Schema, kind_format_guidance};
use crate::grammar::parse_id_arg_with_shorthand;
use crate::model::{Catalog, LegacyCitationCandidate, OffFormatCitation};

/// §FS-citations.recognized.off-format: record `candidate`, which no exact
/// declaration of the target catalog backs, when the reading's three conditions
/// hold. The target is the project the token names — its own for an unqualified
/// token, the alias's for a qualified one — and its grammar decides the kind and
/// the format, and its config the clause the message names.
pub(super) fn record_off_format_citation(
    (schema, frame): (&Schema, Frame<'_>),
    candidate: &LegacyCitationCandidate,
    findings: &mut Catalog,
) {
    let token = off_format_token(&candidate.tail);
    // 2. It opens with exactly one configured kind, a character that is no ASCII
    // letter or digit, and then one that is, so a prefix mention stays text.
    let Some((kind, _)) = frame.grammar().legacy_kind_and_format(token) else {
        return;
    };
    if !token[kind.len()..]
        .chars()
        .nth(1)
        .is_some_and(|ch| ch.is_ascii_alphanumeric())
    {
        return;
    }
    // 3. Its kind's format does not spell it, whole or before a section path; a
    // number-only shorthand of that format is the shorthand pass's to judge.
    if parse_id_arg_with_shorthand(token, frame.grammar()).is_ok() {
        return;
    }
    // 1. No other reading claims its marker: a citation, resolved or dangling, a
    // shorthand or a local section form there keeps its own verdict. Asked last
    // because it walks the catalog and every marker of the tree is a candidate.
    let at_marker = |file: &std::path::Path, line: usize, column: usize| {
        file == candidate.file && line == candidate.line && column == candidate.column
    };
    if findings
        .citations
        .iter()
        .any(|cite| at_marker(&cite.file, cite.line, cite.column))
        || findings
            .local_section_citation_candidates
            .iter()
            .any(|local| at_marker(&local.file, local.line, local.column))
    {
        return;
    }
    let qualified = candidate
        .namespace
        .as_deref()
        .map(|alias| format!("{alias}/"))
        .unwrap_or_default();
    findings.off_format_citations.push(OffFormatCitation {
        file: candidate.file.clone(),
        line: candidate.line,
        column: candidate.column,
        token: format!("{qualified}{token}"),
        kind: kind.to_string(),
        format: kind_format_guidance(schema, frame, Some(kind)).with_patterns(schema),
    });
}

/// §FS-citations.recognized.off-format: the token, read from the character after
/// the marker and any alias over ASCII letters, digits, `-`, `_` and `.`, so a
/// section path is part of it, less one `.` that ends the run as sentence
/// punctuation.
pub(super) fn off_format_token(tail: &str) -> &str {
    let end = tail
        .find(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.')))
        .unwrap_or(tail.len());
    let run = &tail[..end];
    run.strip_suffix('.').unwrap_or(run)
}
