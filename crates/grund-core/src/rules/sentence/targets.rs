//! Namespace-aware object-target recognition (§FS-rules.2,
//! §FS-config.3.9.3.1). Sentence parsing owns this vocabulary check; it reads no
//! facts and performs no evaluation.

use super::forms::{Form, Refusal, refuse};
use super::whose::WHOSE;
use super::{RuleTargets, RuleVocabulary, TargetMode};
use crate::config::{
    KindScope, KindVerdict, NamespaceMatch, Quote, parse_citation_target_entry,
    render_citation_target, unverifiable_reason,
};

/// Recognize one object target, by the verdict the vocabulary's kind table
/// gives it in its scope (§AR-config.1.5): the local project for a
/// bare `KIND`, the aliased one for `alias/KIND`, every one for `*/KIND`. An
/// unresolved namespaced kind in a scope that holds no workspace vocabulary is
/// not a refusal: its reason is recorded in `unverifiable` and the target
/// renders as authored, so the sentence still parses and every other check the
/// scope can make is still made (§FS-rules.4.1). The first such reason is the
/// one kept, which is the target whose refusal the reader used to see. A
/// refused object kind is never guessed, so its refusal offers no form and is
/// missing a kind (§FS-rule-refusals.sentences.rewrite.no-form).
fn known_target(
    target: &str,
    vocab: &RuleVocabulary,
    unverifiable: &mut Option<String>,
) -> Result<String, Refusal> {
    let missing = Form::None { kind: true };
    let parsed =
        parse_citation_target_entry(target).map_err(|message| refuse(message, missing.clone()))?;
    let kind = parsed.kind.as_str();
    let (scope, alias, qualifier) = match &parsed.namespace {
        NamespaceMatch::Local => (KindScope::Project(""), None, String::new()),
        NamespaceMatch::Alias(alias) => (
            KindScope::Pinned(alias),
            Some(alias.as_str()),
            format!(" in namespace \"{alias}\""),
        ),
        NamespaceMatch::Any => (
            KindScope::Any,
            None,
            " in any workspace namespace".to_string(),
        ),
    };
    let legacy = format!("unknown kind \"{kind}\"{qualifier}");
    let reason = match vocab.objects.verdict(scope, kind) {
        KindVerdict::Citable => return Ok(render_citation_target(&parsed)),
        // §FS-rules.4.1.1: a namespaced object kind the run holds no workspace
        // vocabulary for is one this scope cannot judge either way; §FS-errors.3.7.1
        // gives the true clause alone, the final template.
        KindVerdict::Unverifiable => {
            unverifiable.get_or_insert(unverifiable_reason(alias, kind));
            return Ok(render_citation_target(&parsed));
        }
        // §FS-errors.3.8.1: a kind its scope configures without IDs says so alone.
        KindVerdict::NonCitable(entry) => entry.non_citable_reason(Quote::Double),
        // §FS-errors.2.5.1: `code` with no row is refused as not citable too.
        KindVerdict::Unknown => {
            vocab
                .objects
                .unknown_or_complement(scope, kind, legacy, Quote::Double)
        }
        KindVerdict::UnknownAlias => legacy,
    };
    // §FS-rule-refusals.sentences.whose: an object kind followed by a clause appends why.
    let reason = match kind.split_once(WHOSE) {
        Some((head, _)) if vocab.objects.verdict(scope, head) == KindVerdict::Citable => {
            format!("{reason} — a whose clause narrows only a rule's subject")
        }
        _ => reason,
    };
    Err(refuse(reason, missing))
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
