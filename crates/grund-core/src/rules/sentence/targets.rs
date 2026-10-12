//! Namespace-aware object-target recognition (§FS-rules.2,
//! §FS-config.3.9.3.1). Sentence parsing owns this vocabulary check; it reads no
//! facts and performs no evaluation.

use super::forms::{Form, Refusal, refuse};
use super::{RuleTargets, RuleVocabulary, TargetMode};
use crate::config::{
    NamespaceMatch, parse_citation_target_entry, render_citation_target, unverifiable_reason,
};

/// Recognize one object target. An unresolved namespaced kind in a scope that
/// holds no workspace vocabulary is not a refusal: its reason is recorded in
/// `unverifiable` and the target renders as authored, so the sentence still
/// parses and every other check the scope can make is still made
/// (§FS-rules.4.1). The first such reason is the one kept, which is the target
/// whose refusal the reader used to see. A refused object kind is never
/// guessed, so its refusal offers no form and is missing a kind
/// (§FS-rule-refusals.sentences.rewrite.no-form).
fn known_target(
    target: &str,
    vocab: &RuleVocabulary,
    unverifiable: &mut Option<String>,
) -> Result<String, Refusal> {
    let missing = Form::None { kind: true };
    let parsed =
        parse_citation_target_entry(target).map_err(|message| refuse(message, missing.clone()))?;
    let known = match &parsed.namespace {
        NamespaceMatch::Local => vocab.target_kinds.contains(&parsed.kind),
        NamespaceMatch::Alias(alias) => vocab
            .target_namespaces
            .get(alias)
            .is_some_and(|kinds| kinds.contains(&parsed.kind)),
        NamespaceMatch::Any => {
            vocab.target_kinds.contains(&parsed.kind)
                || vocab
                    .target_namespaces
                    .values()
                    .any(|kinds| kinds.contains(&parsed.kind))
        }
    };
    if known {
        return Ok(render_citation_target(&parsed));
    }
    let kind = parsed.kind.clone();
    let qualifier = match &parsed.namespace {
        NamespaceMatch::Alias(alias) => format!(" in namespace \"{alias}\""),
        NamespaceMatch::Any => " in any workspace namespace".to_string(),
        NamespaceMatch::Local => String::new(),
    };
    // §FS-rules.4.1.1: a namespaced object kind the run holds no workspace
    // vocabulary for is one this scope cannot judge either way.
    let alias = match &parsed.namespace {
        NamespaceMatch::Alias(alias) => Some(alias.as_str()),
        NamespaceMatch::Any => None,
        NamespaceMatch::Local => {
            // §FS-errors.3.8: a bare object shares the subject's scope, so a kind
            // configured there without IDs says so.
            let legacy = format!("unknown kind \"{kind}\"");
            return Err(refuse(vocab.kind_refusal(legacy, &kind), missing));
        }
    };
    if vocab.workspace_in_scope() {
        return Err(refuse(
            format!("unknown kind \"{kind}\"{qualifier}"),
            missing,
        ));
    }
    // §FS-errors.3.7.1: the true clause alone, the final template.
    unverifiable.get_or_insert(unverifiable_reason(alias, &kind));
    Ok(render_citation_target(&parsed))
}

pub(super) fn kind_targets(
    text: &str,
    mode: TargetMode,
    vocab: &RuleVocabulary,
    unverifiable: &mut Option<String>,
) -> Result<RuleTargets, Refusal> {
    let mut values = text
        .split(" or ")
        .map(|target| known_target(target, vocab, unverifiable))
        .collect::<Result<Vec<_>, _>>()?;
    values.sort();
    values.dedup();
    Ok(RuleTargets::Kinds { values, mode })
}
