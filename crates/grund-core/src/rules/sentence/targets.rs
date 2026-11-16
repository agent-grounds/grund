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

/// The object kinds after a predicate's count, and where the keyword `distinct` was written
/// among them (§FS-rules.3.2.1). Each predicate decides what a `distinct` means for it, once
/// every object kind is accepted, so a `distinct` refusal is the last of its stage
/// (§FS-rule-refusals.sentences.order).
pub(super) struct Objects {
    /// The accepted object kinds, sorted and deduplicated.
    pub(super) values: Vec<String>,
    /// `distinct` right after the count, covering the whole `or` list.
    pub(super) counted: bool,
    /// `distinct` before a later object kind (§FS-rule-refusals.sentences.rewrite.same-units).
    pub(super) later: bool,
    /// The object kinds as typed with every keyword `distinct` dropped, which is the text a
    /// form that drops it offers (§FS-rule-refusals.sentences.rewrite.by-refusal).
    pub(super) plain: String,
}

impl Objects {
    /// Whether `distinct` is written anywhere among the object kinds.
    pub(super) fn distinct(&self) -> bool {
        self.counted || self.later
    }

    /// The targets, counted in `mode`.
    pub(super) fn targets(self, mode: TargetMode) -> RuleTargets {
        RuleTargets::Kinds {
            values: self.values,
            mode,
        }
    }
}

const DISTINCT: &str = "distinct";

/// Read the object kinds `text` names, the keyword `distinct` included (§FS-rules.3.2.1).
/// Whether a leading `distinct ` is the keyword is decided by the kind table of the rule's
/// scope, never by the scan: where `distinct` is no citable kind of the rule's own project it
/// always is, so `distinct SRCX` is refused as `unknown kind "SRCX"` and `distinct mem/SRC`
/// gets what `mem/SRC` gets; where it is one, only when what follows it is accepted, so
/// `at least 3 distinct.` and every sentence it refused keep their readings
/// (§DF-distinct-target-counts.2.3).
pub(super) fn objects(
    text: &str,
    vocab: &RuleVocabulary,
    unverifiable: &mut Option<String>,
) -> Result<Objects, Refusal> {
    if let Some(rest) = keyword(text) {
        if !configured(vocab) {
            return read_list(rest, vocab, unverifiable).map(|objects| Objects {
                counted: true,
                ..objects
            });
        }
        let mut scratch = unverifiable.clone();
        if let Ok(objects) = read_list(rest, vocab, &mut scratch) {
            *unverifiable = scratch;
            return Ok(Objects {
                counted: true,
                ..objects
            });
        }
    }
    read_list(text, vocab, unverifiable)
}

/// What follows a `distinct ` that opens `text`, where something does.
fn keyword(text: &str) -> Option<&str> {
    text.strip_prefix(DISTINCT)?
        .strip_prefix(' ')
        .filter(|rest| !rest.is_empty())
}

/// Whether `distinct` is a citable kind of the rule's own project (§FS-rules.3.2.1).
fn configured(vocab: &RuleVocabulary) -> bool {
    vocab.objects.verdict(KindScope::Project(""), DISTINCT) == KindVerdict::Citable
}

/// The `or` list of object kinds, a later object kind's leading `distinct` read by the same
/// scope rule as the count's, one kind at a time. The first kind is the one after the count,
/// whose `distinct` [`objects`] has already decided, so it is read as written.
fn read_list(
    text: &str,
    vocab: &RuleVocabulary,
    unverifiable: &mut Option<String>,
) -> Result<Objects, Refusal> {
    let mut values = Vec::new();
    let mut plain = Vec::new();
    let mut later = false;
    for (at, target) in text.split(" or ").enumerate() {
        let marked = keyword(target).filter(|_| at > 0).and_then(|rest| {
            if !configured(vocab) {
                return Some((rest, known_target(rest, vocab, unverifiable)));
            }
            let mut scratch = unverifiable.clone();
            let read = known_target(rest, vocab, &mut scratch).ok()?;
            *unverifiable = scratch;
            Some((rest, Ok(read)))
        });
        let (written, value) = match marked {
            Some((rest, value)) => {
                later = true;
                (rest, value?)
            }
            None => (target, known_target(target, vocab, unverifiable)?),
        };
        values.push(value);
        plain.push(written);
    }
    values.sort();
    values.dedup();
    Ok(Objects {
        values,
        counted: false,
        later,
        plain: plain.join(" or "),
    })
}
