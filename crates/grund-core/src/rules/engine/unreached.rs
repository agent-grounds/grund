//! The declaration a chapter-scoped citation rule cannot reach
//! (§FS-rules.checks.unreached-declaration): the second premise §FS-rules.5.2
//! gives a chapter subject, and the ramp that carries the finding on the
//! warnings channel until `0.16.0` (§FS-rules.7.7).

use super::super::facts::{NodeKey, RuleFacts};
use super::super::{RuleLevel, RulePolarity, RuleRelation, RuleSubject};
use super::authority::Authority;
use super::{SemanticRule, label, push_node};
use crate::model::Diagnostic;

/// §FS-rules.7.7: the warning promises its own promotion in the pending clause
/// of §FS-distribution.4.2.3's closed vocabulary, so the release guard reads the
/// deadline out of this one string. It goes with the subsection it implements
/// when the ramp closes and the finding joins the errors.
const RAMP_CLAUSE: &str = "; this warning becomes an error in grund 0.16.0";

/// §FS-rules.checks.unreached-declaration: one finding per semantic rule group
/// per unreached declaration, located at the declaration's title line because
/// the chapter title that would otherwise anchor it is the thing that is
/// missing. The caller picks the channel, which is what keeps the required
/// level's warning a property of the ramp rather than of the finding.
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
    // §FS-rules.7.7: the recommended level owes no ramp, because a suggestion
    // never moves the exit status at any release.
    let ramp = if rule.level == RuleLevel::Required {
        RAMP_CLAUSE
    } else {
        ""
    };
    for declaration in unreached_declarations(kind, selected, facts) {
        let message = format!(
            "{} has no {name} chapter, so {authority} cannot reach it; add the chapter, \
             or narrow the rule to the declarations that have one{ramp}",
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
fn unreached_declarations(kind: &str, selected: &[NodeKey], facts: &RuleFacts) -> Vec<NodeKey> {
    facts
        .decl
        .iter()
        .filter(|(_, declared)| declared == kind)
        .map(|(node, _)| node)
        .filter(|node| {
            !selected.iter().any(|chapter| {
                facts
                    .contains
                    .iter()
                    .any(|(parent, child)| &parent == node && child == chapter)
            })
        })
        .cloned()
        .collect()
}
