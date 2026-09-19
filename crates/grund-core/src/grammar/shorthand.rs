//! The lexical half of the number-only citation shorthand (§FS-check.1.2,
//! §FS-fmt.2.4, §DF-number-only-citation-shorthand): what the shape *is*, and
//! what one project's own declaration set makes of a token written in it.
//!
//! The categories in §AR-core-module-layout.1 cut by *stage* — scanner, checker,
//! fmt, api. This rule is one contract that has to hold identically at every one
//! of them: the shape recognized in a file, the shape accepted as a CLI
//! argument, the shape reported, and the shape rewritten are the same shape, and
//! a divergence between any two of them is the defect the rule exists to fix.
//! What keeps them from diverging is that all four read *this* file for what the
//! shape is and for which declarations a token names.
//!
//! The seam the rule does split along is **recognition against resolution**
//! (§AR-system.2.1, §AR-system.2.10). Reading a token — the widened ID-argument
//! parser, the `(kind, number)` candidate rule, the index a pass builds so it
//! pays one walk instead of one per site — is a function of the compiled grammar
//! and a declaration set, so it is here. Resolving one against the *catalog of a
//! run* — every loaded project's declarations, whose grammar renders the
//! canonical form, whether this project's policy lets a persisted spelling stand
//! — needs the loaded project set, so it is `resolver/shorthand.rs`; and the one
//! finding a shorthand site earns is a rule, so it is `checker/shorthand.rs`
//! (§FS-check.3.13, §AR-checker.2.12). The unqualified whole-file resolution
//! below stays here because it runs inside one project's own walk, against the
//! declarations that walk just produced (§AR-scanner.2.6.6).
//!
//! What is *not* the rule sits in `id_format.rs`: the `[id] format` template and
//! the post-match tests for where a token of that grammar ends — questions about
//! the shape, which the rule here then serves.
//!
//! Everything here is crate-private and reached through what `grammar/mod.rs`
//! re-exports; the public embedding surface stays in `api/`
//! (§AR-core-module-layout.2).

use anyhow::{Result, anyhow};
use std::collections::BTreeMap;

use super::compiled::Grammar;
use super::ids::{parse_id, parse_id_arg};
use crate::model::{Citation, Declaration, Findings, Id};

/// One parsed ID token: the `Id`, its optional section path, and whether it was
/// written in the number-only shorthand (§FS-check.1.2). A shorthand `Id` carries
/// `slug: None` until the resolution pass fills it in (§AR-scanner.2.6.5).
pub(crate) struct ParsedId {
    pub(crate) id: Id,
    pub(crate) section: Option<String>,
    pub(crate) shorthand: bool,
}

/// `parse_id_arg` widened to also accept the number-only shorthand
/// (§FS-check.1.2.6). The full grammar is tried first and its result is returned
/// unconditionally when it matches, which is what makes "the full ID always wins"
/// (§DF-number-only-citation-shorthand.2.6) true by construction rather than by a
/// separate check.
pub(crate) fn parse_id_arg_with_shorthand(raw: &str, grammar: &Grammar) -> Result<ParsedId> {
    let full = match parse_id_arg(raw, grammar) {
        Ok((id, section)) => {
            return Ok(ParsedId {
                id,
                section,
                shorthand: false,
            });
        }
        Err(err) => err,
    };
    // The whole argument must be the shorthand — a trailing tail is not a
    // shorthand with junk after it, it is a token this grammar does not accept.
    let caps = grammar
        .shorthands()
        .find_map(|shorthand| {
            shorthand
                .prefix_re()
                .captures(raw)
                .filter(|caps| caps.get(0).is_some_and(|found| found.end() == raw.len()))
        })
        .ok_or(full)?;
    let id = parse_id(&caps, grammar).ok_or_else(|| anyhow!("invalid ID `{raw}`"))?;
    Ok(ParsedId {
        id,
        section: caps.name("sec").map(|m| m.as_str().to_string()),
        shorthand: true,
    })
}

/// Resolve a CLI `<ID>[.<section>]` argument that may be written in the
/// number-only shorthand (§FS-check.1.2.6). A query persists nothing, so the
/// shorthand is simply expanded here rather than reported the way a shorthand in
/// a file is (§DF-number-only-citation-shorthand.2.2) — this is what lets a
/// clicked `§FS-042` open in a terminal (§FS-integrations.3.1.7).
///
/// A shorthand matching several declarations is the same class of query failure
/// as an ambiguous full ID (§FS-show.2.2.1.1); one matching none keeps its
/// shorthand `Id`, so the caller's own "not found" path reports it as written
/// instead of a second message saying the same thing.
/// Why an `<ID>` argument could not be turned into one declaration
/// (§FS-show.2.2.1, §FS-refs.4).
///
/// The two cases want different help. `Unparsable` is the classic stumble — an
/// argument shaped for a different repo's `[id] format` — and the format hint is
/// exactly what resolves it. `Ambiguous` means the argument *did* parse and named
/// several declarations; repeating the format there would be advice for a problem
/// the caller does not have, so the candidate list stands alone.
#[derive(Debug)]
pub(crate) enum IdArgError {
    Unparsable(anyhow::Error),
    Ambiguous(anyhow::Error),
}

impl IdArgError {
    fn error(&self) -> &anyhow::Error {
        match self {
            Self::Unparsable(err) | Self::Ambiguous(err) => err,
        }
    }

    /// Whether the caller should follow this with its `[id] format` hint.
    pub(crate) fn wants_format_hint(&self) -> bool {
        matches!(self, Self::Unparsable(_))
    }
}

impl std::fmt::Display for IdArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#}", self.error())
    }
}

/// Every declaration whose kind and number match a shorthand `Id`, in the
/// deterministic `BTreeMap` key order the report needs (§FS-check.3.13.3). One
/// match is the resolution; zero or several is what the checker reports.
///
/// One-shot lookup, for the sites that answer a single question — a CLI
/// argument, one report line. A pass that asks per citation must build a
/// `ShorthandIndex` instead: this walks every declaration in the project.
pub(crate) fn shorthand_candidates<'a>(
    id: &Id,
    declarations: &'a BTreeMap<Id, Vec<Declaration>>,
) -> Vec<&'a Id> {
    declarations
        .keys()
        .filter(|declared| shorthand_names(declared, id))
        .collect()
}

/// Whether `declared` is a declaration the shorthand `id` could name: same kind,
/// same number, and a slug to stand in for (§FS-check.1.2). The slug test is what
/// keeps a partially-parsed `Id` out of its own candidate set.
pub(crate) fn shorthand_names(declared: &Id, id: &Id) -> bool {
    declared.kind == id.kind && declared.num == id.num && declared.slug.is_some()
}

/// Declarations grouped by the `(kind, number)` pair a shorthand names
/// (§FS-check.1.2), so a pass that resolves many sites pays one walk of the
/// declaration set instead of one per site.
///
/// Without it `check` and `fmt --write` are O(sites × declarations), which is
/// quadratic in exactly the case this rule exists to serve: a repository full of
/// shorthands being migrated to canonical form. Measured on a synthetic tree at
/// 8k declarations and 8k shorthand citations, the linear-scan form spent 3.6s in
/// `check` where a canonical tree spent 0.11s (§GOAL-fast-feedback).
pub(crate) struct ShorthandIndex<'a> {
    by_number: BTreeMap<(&'a str, Option<u32>), Vec<&'a Id>>,
}

impl<'a> ShorthandIndex<'a> {
    /// Index canonical declarations plus exact persisted spellings which are
    /// themselves valid shorthand tokens under the effective grammar
    /// (§FS-config.3.2.6). The latter must join the target set so a shorthand
    /// collision cannot silently resolve to its conforming neighbor.
    pub(crate) fn build(grammar: &Grammar, declarations: impl IntoIterator<Item = &'a Id>) -> Self {
        let mut by_number: BTreeMap<(&'a str, Option<u32>), Vec<&'a Id>> = BTreeMap::new();
        for declared in declarations {
            let Some(number) = shorthand_index_number(grammar, declared) else {
                continue;
            };
            by_number
                .entry((declared.kind.as_str(), number))
                .or_default()
                .push(declared);
        }
        Self { by_number }
    }

    /// The declarations `id` could name, in `BTreeMap` key order — the same order
    /// `shorthand_candidates` produces, so the report and the resolution agree.
    pub(crate) fn candidates<'b>(&'b self, id: &'b Id) -> &'b [&'a Id] {
        self.by_number
            .get(&(id.kind.as_str(), id.num))
            .map_or(&[][..], |found| found.as_slice())
    }

    /// The single declaration `id` names, or `None` when zero or several match —
    /// the only outcome that resolves (§DF-number-only-citation-shorthand.2.7).
    pub(crate) fn unique(&self, id: &Id) -> Option<&'a Id> {
        match self.candidates(id) {
            [unique] => Some(unique),
            _ => None,
        }
    }
}

/// The shorthand index a declared ID answers to (§DF-number-only-citation-shorthand,
/// §FS-config.3.2.6): its own number, or — for a persisted off-grammar spelling that
/// is itself a valid shorthand token — the number that spelling parses to. `None`
/// for a declaration no shorthand can name.
fn shorthand_index_number(grammar: &Grammar, declared: &Id) -> Option<Option<u32>> {
    match declared.legacy_spelling() {
        Some(spelling) => parse_id_arg_with_shorthand(spelling, grammar)
            .ok()
            .filter(|parsed| parsed.shorthand && parsed.section.is_none())
            .map(|parsed| parsed.id.num),
        None if declared.slug.is_some() => Some(declared.num),
        None => None,
    }
}

/// §AR-scanner.2.6.6: rewrite each shorthand citation's `Id` to the declaration it
/// names, once the whole project's declarations are known. This is the step that
/// makes a resolved shorthand invisible to everything downstream — the checker,
/// `refs`, `cover`, the unused warning, and the LSP snapshot all read a canonical
/// `Id` and never learn the shorthand existed.
///
/// Zero or several matches leave `slug: None`, which is exactly the state
/// §FS-check.3.13.3 reports as unknown or ambiguous.
///
/// The escaped citations are resolved too. Without that, `<§>FS-042` escaping a
/// real declaration is silently exempt from a check that catches
/// `<§>FS-042-user-login`.
pub(crate) fn resolve_shorthand_citations(grammar: &Grammar, findings: &mut Findings) {
    let pending = |citations: &[Citation]| {
        citations
            .iter()
            .any(|cite| cite.shorthand && cite.namespace.is_none())
    };
    if !pending(&findings.citations) && !pending(&findings.escaped_citations) {
        return;
    }
    // Snapshot the declaration keys first: the loop below mutates `citations`
    // while it reads `declarations`, and only local (unqualified) shorthands can
    // resolve here — a qualified one is resolved against its own namespace by
    // the resolver's cross-namespace pass.
    let declared: Vec<Id> = findings.declarations.keys().cloned().collect();
    let index = ShorthandIndex::build(grammar, declared.iter());
    // §FS-check.2.3.1: an escape earns its "this resolves — did you mean it to be
    // live?" suggestion only by carrying an `Id` that is actually declared, and a
    // shorthand's `Id` never is until it is rewritten here.
    for cite in findings
        .citations
        .iter_mut()
        .chain(findings.escaped_citations.iter_mut())
    {
        if !cite.shorthand || cite.namespace.is_some() || cite.id.slug.is_some() {
            continue;
        }
        if let Some(unique) = index.unique(&cite.id) {
            cite.id = unique.clone();
        }
    }
}
