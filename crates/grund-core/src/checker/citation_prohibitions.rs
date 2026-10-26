//! The prohibition half of the citation-direction checker (§FS-check.3.12), in a
//! file of its own beside `citations.rs`, whose obligation pass it mirrors: each
//! citation site resolves to one level, and `must-not`, `warn-not` and
//! `should-not` each report on their own channel (§FS-config-v2.rules.strengths).

use super::citations::{
    DirectionChannel, DirectionSink, citation_matches_target, citing_side_label,
};
use crate::config::{
    CitationDisjunction, CitationLevel, CitationTarget, Frame, NamespaceMatch, Owners, Rules,
    Schema, render_citation_target,
};
use crate::model::remedies;
use crate::model::{Catalog, Citation, Code, Diagnostic, Switch, citation_direction_repair};

/// §AR-checker.2.10 / §FS-check.3.12: a citation site whose citing kind prohibits
/// its target is a `forbidden-citation` error (`must-not`) or a
/// `discouraged-citation` suggestion (`should-not`). The error carries
/// §FS-check.3.12's ordinary repair suffix, finalized for required index entries
/// by §FS-check.3.12.2; the suggestion has none (§FS-check.2.3). A
/// citation a stub's target lends is a site of its kind at its line in the target,
/// judged as the scanned tree judges it (§FS-check.3.12, §FS-rules.7.11). A citation
/// of a name under a number is an ordinary site (§FS-rules.7.12).
pub(super) fn check_citation_prohibitions(
    findings: &Catalog,
    rules: &Rules,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut impl DirectionSink,
) {
    arm_prohibitions(rules, report);
    let owners = Owners::new(schema, frame);
    for cite in findings.edges() {
        let Some((level, entries)) = citation_site_level(rules, cite) else {
            continue;
        };
        let (channel, diagnostic) = match level {
            CitationLevel::MustNot => (
                DirectionChannel::Errors,
                prohibition_diagnostic(
                    Code::FORBIDDEN_CITATION,
                    &owners,
                    cite,
                    "must not",
                    &citation_direction_repair(),
                ),
            ),
            // §FS-config-v2.rules.strengths: the `must-not` finding, code, message
            // and repair, on the warning channel.
            CitationLevel::WarnNot => (
                DirectionChannel::Warnings,
                prohibition_diagnostic(
                    Code::FORBIDDEN_CITATION,
                    &owners,
                    cite,
                    "must not",
                    &citation_direction_repair(),
                ),
            ),
            CitationLevel::ShouldNot => (
                DirectionChannel::Suggestions,
                prohibition_diagnostic(
                    Code::DISCOURAGED_CITATION,
                    &owners,
                    cite,
                    "should not",
                    &format!(" — {}", remedies::discouraged_citation()),
                ),
            ),
            _ => continue,
        };
        report.push(channel, &cite.source_kind, level, &entries, diagnostic);
    }
}

/// §FS-check.1.4.1: every level a site could resolve to below, a listed entry's
/// or a default's, and so every prohibition the pass can report.
fn arm_prohibitions(rules: &Rules, report: &mut impl DirectionSink) {
    let citations = &rules.citations;
    let levels = citations
        .per_kind
        .values()
        .flat_map(|kind_rules| {
            kind_rules
                .lists()
                .into_iter()
                .filter(|(_, entries)| !entries.is_empty())
                .map(|(level, _)| level)
                .chain(kind_rules.default)
        })
        .chain(citations.global_default);
    for level in levels {
        match level {
            CitationLevel::MustNot | CitationLevel::WarnNot => report.arm(Switch::MustNotDirection),
            CitationLevel::ShouldNot => report.arm(Switch::ShouldNotDirection),
            _ => {}
        }
    }
}

fn prohibition_diagnostic(
    code: Code,
    owners: &Owners<'_>,
    cite: &Citation,
    verb: &str,
    repair: &str,
) -> Diagnostic {
    let target = CitationTarget {
        namespace: match &cite.namespace {
            None => NamespaceMatch::Local,
            Some(alias) => NamespaceMatch::Alias(alias.clone()),
        },
        kind: cite.id.kind.clone(),
    };
    Diagnostic {
        code,
        path: Some(cite.file.clone()),
        line: Some(cite.line),
        column: Some(cite.column),
        message: format!(
            "{} {verb} cite {} (citation direction){repair}",
            // §FS-check.3.12.1: a non-citable citing kind is named by its place —
            // the same label §FS-check.3.11.2 and the generated directions use,
            // because its name is a config handle and not a thing to read.
            citing_side_label(owners, &cite.source_kind, &cite.file),
            render_citation_target(&target)
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// The direction level a citation site resolves to (§FS-config.3.9.4): the
/// explicit list it matches under its citing kind's rules, with the entries of
/// that list it matches, else the per-kind `default`, else the global
/// `default`, else `may`; a default level has no entry behind it. The shared
/// repair selection reads this same resolution to name the applicable list
/// rather than reimplement precedence (§FS-check.3.12.2).
pub(super) fn citation_site_level<'r>(
    rules: &'r Rules,
    cite: &Citation,
) -> Option<(CitationLevel, Vec<&'r CitationDisjunction>)> {
    if let Some(kind_rules) = rules.citations.per_kind.get(&cite.source_kind) {
        for (level, disjunctions) in kind_rules.lists() {
            let entries = disjunctions
                .iter()
                .filter(|disjunction| {
                    disjunction
                        .targets
                        .iter()
                        .any(|target| citation_matches_target(cite, target))
                })
                .collect::<Vec<_>>();
            if !entries.is_empty() {
                return Some((level, entries));
            }
        }
        if let Some(default) = kind_rules.default {
            return Some((default, Vec::new()));
        }
    }
    rules
        .citations
        .global_default
        .map(|level| (level, Vec::new()))
}
