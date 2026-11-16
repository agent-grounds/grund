//! The `cites` relation's count over one subject (§FS-rules.5.1–5.3): every site
//! a target matches counts, one pinned at the rule's own nested alias included
//! (§FS-config.3.9.3). Mixed sites are ordinary (§FS-rules.7.12). Every count above zero hands
//! the checker the citations it read (§FS-rules.7.3.1), through [`counted_citations`].

use super::super::facts::{NodeKey, RuleFacts, SiteMeta};
use super::super::{Cardinality, RuleLevel, RuleTargets, TargetMode};
use super::authority::Authority;
use super::index::{CitationRow, FactIndex};
use super::selectors::{citation_matches_targets, label, target_nodes, target_wording};
use super::{RuleFinding, SemanticRule, push_counted};
use crate::model::Code;
use crate::model::remedies;
use std::collections::BTreeMap;

/// The citations a count read, one per row whatever the count makes of them (§FS-rules.7.3.1):
/// both count arms, outbound and inbound, call this one helper over the rows they counted. Each is
/// the fact's own `SiteMeta`, its path, line, column and the citation as written (§AR-rules.5),
/// so no new type crosses to the checker, which names the path from the run's spellings and base.
pub(super) fn counted_citations<'a>(
    facts: &RuleFacts,
    rows: impl IntoIterator<Item = &'a CitationRow>,
) -> Vec<SiteMeta> {
    rows.into_iter()
        .filter_map(|(site, _, _)| facts.sites.get(site).cloned())
        .collect()
}

/// Count the unit's physical rows once (§FS-rules.5.1, §FS-rules.5.3), then
/// enumerate the declared target universe for per-target coverage (§FS-rules.5.2).
pub(super) fn evaluate_cites(
    rule: &SemanticRule,
    authority: &Authority,
    facts: &RuleFacts,
    index: &FactIndex<'_>,
    subjects: &[NodeKey],
    out: &mut Vec<RuleFinding>,
) {
    let RuleTargets::Kinds { mode, .. } = &rule.targets else {
        return;
    };
    let targets = target_nodes(&rule.targets, facts, index);
    for subject in subjects {
        if *mode == TargetMode::PerTarget {
            // Each owner's sites, mixed ones as ordinary ones (§FS-rules.7.12).
            let mut rows: BTreeMap<_, Vec<&CitationRow>> = BTreeMap::new();
            for row in index.citations_in(subject) {
                if let Some(owner) = index.owner(&row.2) {
                    rows.entry(owner).or_default().push(row);
                }
            }
            for target in &targets {
                let read = rows.get(target).map(Vec::as_slice).unwrap_or_default();
                let count = read.len();
                if !rule.cardinality.contains(count) {
                    let message = format!(
                        "{} cites {} {count} times; {authority} requires {}; {}",
                        label(facts, subject),
                        label(facts, target),
                        rule.cardinality.times_wording(),
                        remedies::citation_cardinality()
                    );
                    // §FS-rules.7.3.1: only this target's citations.
                    push_counted(
                        out,
                        facts,
                        subject,
                        Code::CITATION_CARDINALITY,
                        message,
                        authority,
                        counted_citations(facts, read.iter().copied()),
                    );
                }
            }
        } else {
            let read: Vec<&CitationRow> = index
                .citations_in(subject)
                .iter()
                .copied()
                .filter(|(_, _, target)| citation_matches_targets(target, &targets, index))
                .collect();
            let count = read.len();
            if !rule.cardinality.contains(count) {
                let ordinary = rule.cardinality == Cardinality::AT_LEAST_ONE && count == 0;
                let code = if ordinary {
                    if rule.level == RuleLevel::Required {
                        Code::MISSING_CITATION
                    } else {
                        Code::SUGGESTED_CITATION
                    }
                } else {
                    Code::CITATION_CARDINALITY
                };
                let message = if ordinary {
                    format!(
                        "{} {} cite {} ({authority}); {}",
                        label(facts, subject),
                        if rule.level == RuleLevel::Required {
                            "must"
                        } else {
                            "should"
                        },
                        target_wording(&rule.targets),
                        if rule.level == RuleLevel::Required {
                            remedies::missing_citation()
                        } else {
                            remedies::suggested_citation()
                        }
                    )
                } else {
                    format!(
                        "{} cites {} {count} times; {authority} requires {}; {}",
                        label(facts, subject),
                        target_wording(&rule.targets),
                        rule.cardinality.wording(),
                        remedies::citation_cardinality()
                    )
                };
                // §FS-rules.7.3.1: a zero count reads nothing, so it names nothing.
                let counted = counted_citations(facts, read);
                push_counted(out, facts, subject, code, message, authority, counted);
            }
        }
    }
}
