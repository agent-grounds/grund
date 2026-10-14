//! What each `[citations]` entry reported itself, handed to the engine beside the
//! rule it expresses so that config-to-rule deduplication is decided on the
//! findings, unit by unit (§FS-rules.6).

use super::citation_prohibitions::check_citation_prohibitions;
use super::citations::{DirectionChannel, DirectionSink, check_citation_obligations};
use crate::config::{CitationDisjunction, CitationLevel, Frame, Rules, Schema};
use crate::model::{Catalog, Diagnostic};
use crate::rules::engine::{
    Direction, citation_precedence, direction_rule, duplicates_a_direction,
};
use crate::rules::sentence::ParsedRule;

/// §FS-rules.6: each `[citations]` entry as an engine rule and, once a rule
/// duplicates one, every finding the entry reported itself, so the engine drops a
/// rule's finding only where its direction reported the same. The direction
/// passes are asked again here, as field pairing asks the field pass again,
/// rather than read back out of the report, which does not say which entry
/// reported what.
pub(super) fn directions(
    findings: &Catalog,
    rules: &Rules,
    schema: &Schema,
    frame: Frame<'_>,
    parsed_rules: &[ParsedRule],
) -> Vec<Direction> {
    let precedence = citation_precedence(&rules.citations);
    let duplicated = rules.citations.declared && duplicates_a_direction(parsed_rules, &precedence);
    let mut directions = Reported(
        precedence
            .into_iter()
            .map(|rule| (rule, Vec::new()))
            .collect(),
    );
    if duplicated {
        check_citation_obligations(findings, rules, schema, frame, &mut directions);
        check_citation_prohibitions(findings, rules, schema, &mut directions);
    }
    directions.0
}

/// The direction passes' findings, each under the entries that reported it.
struct Reported(Vec<Direction>);

impl DirectionSink for Reported {
    fn push(
        &mut self,
        _: DirectionChannel,
        kind: &str,
        level: CitationLevel,
        entries: &[&CitationDisjunction],
        diagnostic: Diagnostic,
    ) {
        for entry in entries {
            if let Some(rule) = direction_rule(kind, level, entry) {
                self.0.push((rule, vec![diagnostic.clone()]));
            }
        }
    }
}
