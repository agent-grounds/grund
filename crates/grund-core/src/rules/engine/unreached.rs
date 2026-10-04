//! The declaration a chapter-scoped citation rule cannot reach
//! (§FS-rules.checks.unreached-declaration): the second premise §FS-rules.5.2
//! gives a chapter subject, reported as an error on the ordinary `must`
//! channel (§FS-rules.7).

use super::super::facts::{NodeKey, RuleFacts};
use super::super::{RuleLevel, RulePolarity, RuleRelation, RuleSubject};
use super::authority::Authority;
use super::index::FactIndex;
use super::{SemanticRule, label, push_node};
use crate::model::Diagnostic;

/// §FS-rules.checks.unreached-declaration: the ramp the warning promised closed
/// in `0.16.0`, so the message names that release in the landed clause of
/// §FS-distribution.4.2.3's closed vocabulary and the release guard reads the
/// floor out of this one string instead of the deadline.
const LANDED_CLAUSE: &str = "; this became an error in grund 0.16.0";

/// §FS-rules.checks.unreached-declaration: one finding per semantic rule group
/// per unreached declaration, located at the declaration's title line because
/// the chapter title that would otherwise anchor it is the thing that is
/// missing. It goes on the one channel its level already has — the required
/// level's errors, the recommended level's suggestions (§FS-rules.7).
///
/// Only the three positive citation families reach the premise. Chapter
/// presence never takes a chapter subject at all, and a prohibition stays
/// silent because a citation site inside the chapter body is deleted along with
/// that body — for `must not cite` the forbidden site genuinely no longer
/// exists, which is why the same argument does not excuse the other three
/// (§FS-rules.5.2).
pub(super) fn report_unreached(
    rule: &SemanticRule,
    authority: &Authority,
    facts: &RuleFacts,
    index: &FactIndex<'_>,
    selected: &[NodeKey],
    out: &mut Vec<Diagnostic>,
) {
    if rule.polarity != RulePolarity::Positive
        || !matches!(rule.relation, RuleRelation::Cite | RuleRelation::BeCitedBy)
    {
        return;
    }
    let RuleSubject::ChapterOfKind { kind, name } = &rule.subject else {
        return;
    };
    // §FS-rules.checks.unreached-declaration: the recommended level drops the
    // landed clause, because a suggestion never moved the exit status at any
    // release and so owed no promotion.
    let landed = if rule.level == RuleLevel::Required {
        LANDED_CLAUSE
    } else {
        ""
    };
    for declaration in unreached_declarations(kind, selected, facts, index) {
        let message = format!(
            "{} has no {name} chapter, so {authority} cannot reach it; add the chapter, \
             or narrow the rule to the declarations that have one{landed}",
            label(facts, &declaration)
        );
        push_node(
            out,
            facts,
            &declaration,
            "unreached-declaration",
            message,
            authority,
        );
    }
}

/// §FS-rules.5.2's `unreached(d, N)`: the local declarations of the kind that
/// own none of the chapters the subject selected.
///
/// It is read as the complement of the selection rather than as a second join
/// over `chapter`, which is what §FS-rules.5.2's `chapter_of` says as well: that
/// clause is written over the subject selector's section-component join and not
/// over the presence family's display-name count (§FS-rules.3.1). So
/// "contributes no unit" and "is unreached" are one set by construction
/// (§FS-rules.2) — no declaration can both hand the relation a unit and be
/// reported as out of the rule's reach.
///
/// §FS-rules.5.3: form the complement using each declaration's direct chapter
/// bucket, sharing selector data rather than joining global relations again.
fn unreached_declarations(
    kind: &str,
    selected: &[NodeKey],
    facts: &RuleFacts,
    index: &FactIndex<'_>,
) -> Vec<NodeKey> {
    let selected = selected.iter().collect::<std::collections::BTreeSet<_>>();
    index
        .declarations_of_kind(kind)
        .iter()
        .filter(|node| {
            !index
                .chapters_of(node)
                .iter()
                .any(|row| selected.contains(&facts.chapter[*row].0))
        })
        .map(|node| (*node).clone())
        .collect()
}
