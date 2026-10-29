//! The off-format reading (§FS-citations.recognized.off-format): where the
//! promotion step of `legacy.rs` finds no declaration behind a deferred
//! candidate, the candidate is recorded rather than dropped when it opens like
//! an ID of a configured kind that its kind's format does not spell and no
//! other reading claimed its marker. Still a catalog operation: the record is a
//! site for §FS-check.checks.off-format-citation and never a citation, so no
//! edge is taken for it (§FS-config.3.2.5).

use std::collections::HashSet;
use std::ffi::OsStr;

use crate::config::{Frame, Schema, kind_format_guidance};
use crate::grammar::parse_id_arg_with_shorthand;
use crate::model::{Catalog, LegacyCitationCandidate, OffFormatCitation};

/// The markers a citation or a local section form already holds, by file, line and
/// column. The file is keyed by its bytes, which one walk spells one way, since
/// hashing a `Path` walks its components.
type ClaimedMarkers<'a> = HashSet<(&'a OsStr, usize, usize)>;

/// §FS-citations.recognized.off-format: record each of `undeclared`, the
/// candidates no exact declaration of their target catalog backs, for which the
/// reading's three conditions hold. Each comes with its target — its own project
/// for an unqualified token, the alias's for a qualified one — whose grammar
/// decides the kind and the format, and whose config the clause the message names.
/// The markers other readings claim are gathered once for the whole pass, so
/// asking condition 1 costs a lookup rather than a walk of the catalog.
pub(crate) fn record_off_format_citations(
    undeclared: &[((&Schema, Frame<'_>), LegacyCitationCandidate)],
    findings: &mut Catalog,
) {
    if undeclared.is_empty() {
        return;
    }
    let claimed: ClaimedMarkers<'_> = findings
        .citations
        .iter()
        .map(|cite| (cite.file.as_os_str(), cite.line, cite.column))
        .chain(
            findings
                .local_section_citation_candidates
                .iter()
                .map(|local| (local.file.as_os_str(), local.line, local.column)),
        )
        .collect();
    let records = undeclared
        .iter()
        .filter_map(|(target, candidate)| off_format_citation(*target, candidate, &claimed))
        .collect::<Vec<_>>();
    findings.off_format_citations.extend(records);
}

/// §FS-citations.recognized.off-format: the record for one undeclared
/// `candidate`, when the reading's three conditions hold.
fn off_format_citation(
    (schema, frame): (&Schema, Frame<'_>),
    candidate: &LegacyCitationCandidate,
    claimed: &ClaimedMarkers<'_>,
) -> Option<OffFormatCitation> {
    let token = off_format_token(&candidate.tail);
    // 2. It opens with exactly one configured kind, a character that is no ASCII
    // letter or digit, and then one that is, so a prefix mention stays text.
    let (kind, _) = frame.grammar().legacy_kind_and_format(token)?;
    if !token[kind.len()..]
        .chars()
        .nth(1)
        .is_some_and(|ch| ch.is_ascii_alphanumeric())
    {
        return None;
    }
    // 1. No other reading claims its marker: a citation, resolved or dangling, a
    // shorthand or a local section form there keeps its own verdict.
    if claimed.contains(&(candidate.file.as_os_str(), candidate.line, candidate.column)) {
        return None;
    }
    // 3. Its kind's format does not spell it, whole or before a section path; a
    // number-only shorthand of that format is the shorthand pass's to judge.
    // Asked last because it parses, and every marker of the tree is a candidate.
    if parse_id_arg_with_shorthand(token, frame.grammar()).is_ok() {
        return None;
    }
    let qualified = candidate
        .namespace
        .as_deref()
        .map(|alias| format!("{alias}/"))
        .unwrap_or_default();
    Some(OffFormatCitation {
        file: candidate.file.clone(),
        line: candidate.line,
        column: candidate.column,
        token: format!("{qualified}{token}"),
        kind: kind.to_string(),
        format: kind_format_guidance(schema, frame, Some(kind)).with_patterns(schema),
    })
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
