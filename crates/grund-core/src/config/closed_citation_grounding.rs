//! Facts for §FS-check.checks.closed-citation-grounding, shared by diagnostics
//! and the migration report gate (§FS-config-migrate.5.2).

use super::grounding::{
    configured_row_grounding, row_closed_to_every_citation, row_grounding,
    row_grounds_at_any_strength, soft_grounding_rungs,
};
use super::{Project, Run, Strength};

/// Where the grounding that reaches a closed home comes from
/// (§FS-check.checks.closed-citation-grounding). Configuration wins over the flag.
#[derive(Clone, Copy)]
pub(crate) enum GroundingSource {
    Row,
    Project,
    Flag,
}

/// The effective conflict, independent of the version spelling it
/// (§FS-check.checks.closed-citation-grounding).
pub(crate) struct ClosedCitationGrounding {
    pub(crate) kind: String,
    pub(crate) strength: Strength,
    pub(crate) level: usize,
    pub(crate) source: GroundingSource,
    pub(crate) row_fallback: bool,
}

/// One conflict per scanned non-citable home, using the same closed and grounded
/// predicates as Citation directions (§FS-check.checks.closed-citation-grounding).
pub(crate) fn closed_citation_grounding(
    project: &Project,
    run: &Run,
) -> Vec<ClosedCitationGrounding> {
    if !project.workspace.include_root {
        return Vec::new();
    }
    let rules = &project.rules;
    project
        .schema
        .rows
        .iter()
        .filter_map(|row| {
            if row.kind.is_some()
                || row.is_complement()
                || !row.places.iter().any(|place| place.scanned)
                || !row_closed_to_every_citation(rules, &row.name)
                || !row_grounds_at_any_strength(rules, run, &row.name)
            {
                return None;
            }
            let (must, level) = configured_row_grounding(rules, &row.name);
            let configured = if must {
                Some((Strength::Must, level))
            } else {
                soft_grounding_rungs(rules, &row.name)
                    .into_iter()
                    .min_by_key(|(strength, _)| *strength as u8)
            };
            let own = rules.grounding.kinds.get(&row.name);
            let (strength, level, source) = match configured {
                Some((strength, level)) => (
                    strength,
                    level,
                    if own.is_some_and(|own| own.ladder.is_some() || own.require == Some(true)) {
                        GroundingSource::Row
                    } else {
                        GroundingSource::Project
                    },
                ),
                None => (
                    Strength::Must,
                    row_grounding(rules, run, &row.name).1,
                    GroundingSource::Flag,
                ),
            };
            Some(ClosedCitationGrounding {
                kind: row.name.clone(),
                strength,
                level,
                source,
                row_fallback: rules
                    .citations
                    .per_kind
                    .get(&row.name)
                    .is_some_and(|row| row.default.is_some()),
            })
        })
        .collect()
}
