//! Namespace-aware object-target recognition (§FS-rules.2,
//! §FS-config.3.9.3.1). Sentence parsing owns this vocabulary check; it reads no
//! facts and performs no evaluation.

use super::{RuleParseError, RuleTargets, RuleVocabulary, TargetMode, error, unverifiable_here};
use crate::config::{NamespaceMatch, parse_citation_target_entry, render_citation_target};

fn known_target(target: &str, vocab: &RuleVocabulary) -> Result<String, RuleParseError> {
    let parsed = parse_citation_target_entry(target).map_err(|message| {
        error(format!(
            "{message}; accepted form: Each FS must cite at least one GOAL."
        ))
    })?;
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
    let kind = parsed.kind;
    let qualifier = match &parsed.namespace {
        NamespaceMatch::Alias(alias) => format!(" in namespace \"{alias}\""),
        NamespaceMatch::Any => " in any workspace namespace".to_string(),
        NamespaceMatch::Local => String::new(),
    };
    let legacy = format!(
        "unknown kind \"{kind}\"{qualifier}; accepted form: Each FS must cite at least one GOAL."
    );
    // §FS-rules.4.1.1: a namespaced object kind the run holds no workspace
    // vocabulary for is one this scope cannot judge either way.
    match unverifiable_reason(&parsed.namespace, &kind, vocab) {
        // §FS-errors.3.7: the legacy reason stays a verbatim contiguous prefix
        // for the two compatibility releases, with the true clause after it.
        Some(reason) => Err(unverifiable_here(format!(
            "{legacy} \u{2014} {reason} \u{2014} check from the workspace root{RULE_ALIAS_RAMP_TAIL}"
        ))),
        None => Err(error(legacy)),
    }
}

/// `; this wording changes in grund 0.15.0` — the ramp §FS-errors.3.7 carries
/// until §FS-errors.3.7.1's final reasons replace the compatibility prefix.
const RULE_ALIAS_RAMP_TAIL: &str = "; this wording changes in grund 0.15.0";

/// Why an unresolved object kind is §FS-rules.4.1's unverifiable case rather
/// than an invalid rule, or `None` when it is an invalid rule. The reason names
/// what the scope cannot reach, never a candidate it cannot have
/// (§FS-check.3.8.3).
fn unverifiable_reason(
    namespace: &NamespaceMatch,
    kind: &str,
    vocab: &RuleVocabulary,
) -> Option<String> {
    if vocab.workspace_in_scope() {
        return None;
    }
    match namespace {
        NamespaceMatch::Alias(alias) => Some(format!(
            "unknown project alias {alias}; no workspace is in scope here, so the alias cannot be resolved"
        )),
        NamespaceMatch::Any => Some(format!(
            "no workspace is in scope here, so no namespace can be searched for {kind}"
        )),
        NamespaceMatch::Local => None,
    }
}

pub(super) fn kind_targets(
    text: &str,
    mode: TargetMode,
    vocab: &RuleVocabulary,
) -> Result<RuleTargets, RuleParseError> {
    let mut values = text
        .split(" or ")
        .map(|target| known_target(target, vocab))
        .collect::<Result<Vec<_>, _>>()?;
    values.sort();
    values.dedup();
    Ok(RuleTargets::Kinds { values, mode })
}
