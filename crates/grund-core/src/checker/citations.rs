//! The citation-direction half of the checker (§FS-config.3.9), in a file of its
//! own beside `references.rs` and `sections.rs`
//! (§AR-core-module-layout.1): the obligation pass (§FS-check.3.11), the
//! prohibition pass (§FS-check.3.12), and the two questions both of them ask —
//! what kind of place a citation sits in, and whether it matches a rule's
//! target. `report.rs` keeps the declaration-shape rules and the diagnostic
//! helpers they share.

use std::collections::BTreeMap;
use std::path::Path;

use super::obligation_units::{
    ObligationUnit, file_is_obligation_unit, non_citable_kind_names, obligation_units,
};
use crate::config::{
    CitationDisjunction, CitationLevel, CitationTarget, Config, KindCitationRules, KindConfig,
    NamespaceMatch, render_citation_target,
};
use crate::model::{
    CITATION_DIRECTION_REPAIR, Catalog, CheckReport, Citation, Diagnostic, E2eSpecRef, Id,
    paths_same_location,
};
use crate::scanner::file_home_kind;

/// How a citing kind is named in a finding (§FS-check.3.11, §FS-check.3.12.1): a
/// citable kind by its name, which is the prefix of every ID in it; a
/// non-citable one by its home, which is all a reader of the message could go
/// and look at. `code` keeps its own name — it is the one non-citable kind with
/// no place, being the complement of every home there is.
fn citing_side_label(config: &Config, kind: &str) -> String {
    config
        .kinds
        .iter()
        .find(|configured| configured.kind == kind && !configured.citable)
        .and_then(KindConfig::place_label)
        .unwrap_or_else(|| kind.to_string())
}

/// §AR-checker.2.9 / §FS-check.3.11: every top-level declaration of a citing
/// kind with a `must` / `should` obligation must carry, in its body, a citation
/// satisfying each obligation entry. `must` misses are `missing-citation`
/// errors; `should` misses are `suggested-citation` suggestions.
///
/// Why the citation indexes are built up front: the per-declaration and per-case
/// rescans they replace were O(kinds × declarations × citations) and dominated
/// `grund check` on a large tree.
pub(super) fn check_citation_obligations(
    findings: &Catalog,
    config: &Config,
    report: &mut CheckReport,
) {
    // Index every citation once, up front, so each citing kind's obligation pass
    // is a map lookup rather than a fresh O(citations) scan per declaration
    // (§AR-benchmarks).
    let mut by_decl: BTreeMap<&Id, Vec<&Citation>> = BTreeMap::new();
    let mut by_file: BTreeMap<(&str, &Path), Vec<&Citation>> = BTreeMap::new();
    // Resolved once, not per citation: the per-file question below is asked of
    // every citation in the tree, and answering it by scanning `[[kinds]]` each
    // time would make the pass O(citations × kinds) for no gain (§AR-benchmarks).
    let non_citable = non_citable_kind_names(config);
    let homeless = config.homeless_kind();
    for cite in &findings.citations {
        if let Some(id) = &cite.enclosing_declaration {
            by_decl.entry(id).or_default().push(cite);
        }
        if file_is_obligation_unit(&non_citable, homeless, cite) {
            by_file
                .entry((cite.source_kind.as_str(), cite.file.as_path()))
                .or_default()
                .push(cite);
        }
    }
    // Bucket the (usually zero — fixture trees are carved out of `[scan]`)
    // citations that live under an E2E case directory to their nearest case, so
    // the E2E obligation never `starts_with`-scans every citation per case.
    let mut e2e_by_case: BTreeMap<&Path, Vec<&Citation>> = BTreeMap::new();
    for decls in findings.declarations.values() {
        for decl in decls {
            if decl.e2e_case.is_some() {
                e2e_by_case.entry(decl.file.as_path()).or_default();
            }
        }
    }
    if !e2e_by_case.is_empty() {
        for cite in &findings.citations {
            for ancestor in cite.file.ancestors() {
                if let Some(bucket) = e2e_by_case.get_mut(ancestor) {
                    bucket.push(cite);
                    break;
                }
            }
        }
    }

    for (citing_kind, rules) in &config.citations.per_kind {
        if rules.must.is_empty() && rules.should.is_empty() {
            continue;
        }
        let units = obligation_units(
            citing_kind,
            config,
            findings,
            &by_decl,
            &by_file,
            &e2e_by_case,
        );
        // §FS-check.2.2.1.1: a walked folder with real non-entry content must not
        // silently pass when this obligation has no unit to evaluate.
        if units.is_empty()
            && let Some(warning) =
                empty_citation_obligation_warning(config, findings, citing_kind, rules)
        {
            report.warnings.push(warning);
        }
        for unit in units {
            for entry in &rules.must {
                if !entry.targets.iter().any(|t| unit.satisfies(t)) {
                    report.errors.push(obligation_diagnostic(
                        "missing-citation",
                        config,
                        &unit,
                        entry,
                        "must",
                    ));
                }
            }
            for entry in &rules.should {
                if !entry.targets.iter().any(|t| unit.satisfies(t)) {
                    report.suggestions.push(obligation_diagnostic(
                        "suggested-citation",
                        config,
                        &unit,
                        entry,
                        "should",
                    ));
                }
            }
        }
    }
}

/// §FS-check.2.2.1.1: identify the one run-level warning for a walked folder whose
/// explicit obligation has real scanned content but no ordinary obligation unit.
/// The warning is derived from the same normalized home classifier the scanner
/// uses for citation-source attribution, so explicit paths and symlink spellings
/// stay inside the same home boundary.
fn empty_citation_obligation_warning(
    config: &Config,
    findings: &Catalog,
    citing_kind: &str,
    rules: &KindCitationRules,
) -> Option<Diagnostic> {
    let kind = config.kinds.iter().find(|kind| kind.kind == citing_kind)?;
    let folder = kind.folder.as_deref()?;
    if !kind.scan
        || !findings.scanned_files.iter().any(|file| {
            file_home_kind(file, config.schema(), config.frame()).as_deref() == Some(citing_kind)
                && !kind_entry_file(file, config, kind)
        })
    {
        return None;
    }

    let level = if rules.must.is_empty() {
        "should"
    } else {
        "must"
    };
    let place = format!("{folder}/");
    let message = if kind.citable {
        format!(
            "[citations.{citing_kind}] {level} applies to nothing — {place} declares no {citing_kind} ID; did you mean `citable = false`?"
        )
    } else {
        // §FS-check.2.2.1.2: the row-key half is advice, so it is given only where it is
        // still advice — a row already grounding (§FS-config.3.4.8) has made that
        // setting, and this run is already reporting what it caught (§FS-check.3.6).
        let tail = if config.kind_grounding(kind).0 {
            String::new()
        } else {
            format!("; set require_grounding = true on the {place} row to make that an error")
        };
        format!(
            "[citations.{citing_kind}] {level} applies to nothing — no scanned file in {place} carries a citation{tail}"
        )
    };
    Some(Diagnostic {
        code: "empty-citation-obligation",
        path: None,
        line: None,
        column: None,
        message,
        sites: Vec::new(),
        authority: Vec::new(),
    })
}

/// Whether `file` is the entry file excluded from a folder's content count:
/// the effective citable index, or literal `README.md` for a non-citable home
/// (§FS-check.2.2.1.1). `index = false` naturally has no entry path.
fn kind_entry_file(file: &Path, config: &Config, kind: &KindConfig) -> bool {
    let entry = if kind.citable {
        kind.index_path()
    } else {
        kind.folder
            .as_deref()
            .map(|folder| Path::new(folder).join("README.md"))
    };
    entry.is_some_and(|entry| paths_same_location(file, &config.root.join(entry)))
}

fn obligation_diagnostic(
    code: &'static str,
    config: &Config,
    unit: &ObligationUnit<'_>,
    entry: &CitationDisjunction,
    verb_level: &str,
) -> Diagnostic {
    Diagnostic {
        code,
        path: Some(unit.path.clone()),
        line: Some(unit.line),
        column: None,
        message: format!(
            "{} {verb_level} cite {} (citation direction)",
            unit.subject(config),
            render_target_phrase(entry)
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// §AR-checker.2.10 / §FS-check.3.12: a citation site whose citing kind prohibits
/// its target is a `forbidden-citation` error (`must-not`) or a
/// `discouraged-citation` suggestion (`should-not`). The error carries
/// §FS-check.3.12's repair suffix; the suggestion does not (§FS-check.2.3).
pub(super) fn check_citation_prohibitions(
    findings: &Catalog,
    config: &Config,
    report: &mut CheckReport,
) {
    for cite in &findings.citations {
        match citation_site_level(config, cite) {
            Some(CitationLevel::MustNot) => report.errors.push(prohibition_diagnostic(
                "forbidden-citation",
                config,
                cite,
                "must not",
                CITATION_DIRECTION_REPAIR,
            )),
            Some(CitationLevel::ShouldNot) => report.suggestions.push(prohibition_diagnostic(
                "discouraged-citation",
                config,
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
    config: &Config,
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
            citing_side_label(config, &cite.source_kind),
            render_citation_target(&target)
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// The direction level a citation site resolves to (§FS-config.3.9.4): the
/// explicit list it matches under its citing kind's rules, else the per-kind
/// `default`, else the global `default`, else `may`.
fn citation_site_level(config: &Config, cite: &Citation) -> Option<CitationLevel> {
    let rules = config.citations.per_kind.get(&cite.source_kind);
    if let Some(rules) = rules {
        let lists = [
            (CitationLevel::Must, &rules.must),
            (CitationLevel::Should, &rules.should),
            (CitationLevel::May, &rules.may),
            (CitationLevel::ShouldNot, &rules.should_not),
            (CitationLevel::MustNot, &rules.must_not),
        ];
        for (level, disjunctions) in lists {
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
        if let Some(default) = rules.default {
            return Some(default);
        }
    }
    config.citations.global_default
}

/// Whether a citation matches a rule target: same cited kind, and a namespace
/// qualifier that covers the citation's namespace (§FS-config.3.9.3).
pub(super) fn citation_matches_target(cite: &Citation, target: &CitationTarget) -> bool {
    if cite.id.kind != target.kind {
        return false;
    }
    match &target.namespace {
        NamespaceMatch::Any => true,
        NamespaceMatch::Local => cite.namespace.is_none(),
        NamespaceMatch::Alias(alias) => cite.namespace.as_deref() == Some(alias.as_str()),
    }
}

pub(super) fn e2e_spec_ref_matches_target(spec_ref: &E2eSpecRef, target: &CitationTarget) -> bool {
    if spec_ref.kind != target.kind {
        return false;
    }
    match &target.namespace {
        NamespaceMatch::Any => true,
        NamespaceMatch::Local => spec_ref.namespace.is_none(),
        NamespaceMatch::Alias(alias) => spec_ref.namespace.as_deref() == Some(alias.as_str()),
    }
}

/// Render a disjunction as a human phrase for a finding message: kinds joined by
/// " or " (§FS-init.2.3.5.4 uses the same phrasing in the agent entrypoint).
fn render_target_phrase(entry: &CitationDisjunction) -> String {
    entry
        .targets
        .iter()
        .map(render_citation_target)
        .collect::<Vec<_>>()
        .join(" or ")
}
