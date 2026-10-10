//! The prohibition half of the citation-direction checker (§FS-check.3.12), in a
//! file of its own beside `citations.rs`, whose obligation pass it mirrors: each
//! citation site resolves to one level, and `must-not`, `warn-not` and
//! `should-not` each report on their own channel (§FS-config-v2.rules.strengths).

use super::citations::{citation_matches_target, citing_side_label};
use crate::config::{
    CitationLevel, CitationTarget, NamespaceMatch, Rules, Schema, render_citation_target,
};
use crate::model::{CITATION_DIRECTION_REPAIR, Catalog, CheckReport, Citation, Diagnostic};

/// §AR-checker.2.10 / §FS-check.3.12: a citation site whose citing kind prohibits
/// its target is a `forbidden-citation` error (`must-not`) or a
/// `discouraged-citation` suggestion (`should-not`). The error carries
/// §FS-check.3.12's repair suffix; the suggestion does not (§FS-check.2.3).
pub(super) fn check_citation_prohibitions(
    findings: &Catalog,
    rules: &Rules,
    schema: &Schema,
    report: &mut CheckReport,
) {
    for cite in &findings.citations {
        match citation_site_level(rules, cite) {
            Some(CitationLevel::MustNot) => report.errors.push(prohibition_diagnostic(
                "forbidden-citation",
                schema,
                cite,
                "must not",
                CITATION_DIRECTION_REPAIR,
            )),
            // §FS-config-v2.rules.strengths: the `must-not` finding, code, message
            // and repair, on the warning channel.
            Some(CitationLevel::WarnNot) => report.warnings.push(prohibition_diagnostic(
                "forbidden-citation",
                schema,
                cite,
                "must not",
                CITATION_DIRECTION_REPAIR,
            )),
            Some(CitationLevel::ShouldNot) => report.suggestions.push(prohibition_diagnostic(
                "discouraged-citation",
                schema,
                cite,
                "should not",
                "",
            )),
            _ => {}
        }
    }
}

fn prohibition_diagnostic(
    code: &'static str,
    schema: &Schema,
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
            citing_side_label(schema, &cite.source_kind),
            render_citation_target(&target)
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// The direction level a citation site resolves to (§FS-config.3.9.4): the
/// explicit list it matches under its citing kind's rules, else the per-kind
/// `default`, else the global `default`, else `may`.
fn citation_site_level(rules: &Rules, cite: &Citation) -> Option<CitationLevel> {
    if let Some(kind_rules) = rules.citations.per_kind.get(&cite.source_kind) {
        for (level, disjunctions) in kind_rules.lists() {
            for disjunction in disjunctions {
                if disjunction
                    .targets
                    .iter()
                    .any(|target| citation_matches_target(cite, target))
                {
                    return Some(level);
                }
            }
        }
        if let Some(default) = kind_rules.default {
            return Some(default);
        }
    }
    rules.citations.global_default
}
