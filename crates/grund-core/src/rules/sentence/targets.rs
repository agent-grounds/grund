//! Namespace-aware object-target recognition (§FS-rules.2,
//! §FS-config.3.9.3.1). Sentence parsing owns this vocabulary check; it reads no
//! facts and performs no evaluation.

use super::{RuleParseError, RuleTargets, RuleVocabulary, TargetMode, error};
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
        Ok(render_citation_target(&parsed))
    } else {
        let kind = parsed.kind;
        let qualifier = match parsed.namespace {
            NamespaceMatch::Alias(alias) => format!(" in namespace \"{alias}\""),
            NamespaceMatch::Any => " in any workspace namespace".to_string(),
            NamespaceMatch::Local => String::new(),
        };
        Err(error(format!(
            "unknown kind \"{kind}\"{qualifier}; accepted form: Each FS must cite at least one GOAL."
        )))
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
