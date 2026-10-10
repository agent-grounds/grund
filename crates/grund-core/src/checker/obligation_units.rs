//! What a citation obligation is evaluated against (§AR-checker.2.9,
//! §FS-check.3.11): one unit per non-stub declaration, per `E2E` case, or per
//! file for `code` and every non-citable kind, each holding the citations that
//! count toward it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::citations::{citation_matches_target, e2e_spec_ref_matches_target};
use super::grounding::file_obligation_units;
use crate::config::{CitationTarget, Config};
use crate::grammar::render_id;
use crate::model::{Catalog, Citation, E2eSpecRef, Id};

/// One thing an obligation is evaluated against (§AR-checker.2.9): a declaration
/// body, a `code` source file, or an `E2E` case — together with the citations
/// that count toward it, the `path:line` a finding anchors at, and the subject
/// `id` (a declaration) or `None` (a `code` source file).
pub(super) struct ObligationUnit<'a> {
    pub(super) id: Option<&'a Id>,
    /// The place a non-citable kind's unit is named by (§FS-check.3.11.2) — its
    /// home, since the unit is a file in it and the kind has no ID to print.
    pub(super) place: Option<String>,
    pub(super) path: PathBuf,
    pub(super) line: usize,
    pub(super) citations: Vec<&'a Citation>,
    pub(super) e2e_spec_refs: Vec<&'a E2eSpecRef>,
}

impl ObligationUnit<'_> {
    pub(super) fn satisfies(&self, target: &CitationTarget) -> bool {
        self.citations
            .iter()
            .any(|cite| citation_matches_target(cite, target))
            || self
                .e2e_spec_refs
                .iter()
                .any(|spec_ref| e2e_spec_ref_matches_target(spec_ref, target))
    }

    pub(super) fn subject(&self, config: &Config) -> String {
        match (self.id, &self.place) {
            (Some(id), _) => render_id(&config.grammar, id),
            (None, Some(place)) => place.clone(),
            (None, None) => "source file".to_string(),
        }
    }
}

/// Whether this citation site counts toward a *per-file* obligation unit
/// (§FS-check.3.11). Two kinds of citing side have no declaration to attach an
/// obligation to, and both answer with the file:
///
/// * the **homeless kind** (`code`, or whatever the project named it,
///   §FS-config.3.9.2.4) — every citation outside a configured home, source files
///   only. A README or a changelog is a document, and §FS-check.3.6 exempts it
///   for the same reason.
/// * a **homed non-citable kind** — every scanned file in its home, `.md`
///   included. Inheriting the Markdown exemption here would make `must` inert on
///   the kinds that are usually all Markdown, which is most of them: the
///   exemption reasons about implementation-versus-document, and a home the
///   maintainer named is neither guess.
pub(super) fn file_is_obligation_unit(
    non_citable: &BTreeSet<&str>,
    homeless: &str,
    cite: &Citation,
) -> bool {
    if cite.source_kind == homeless {
        return cite.file.extension().and_then(|ext| ext.to_str()) != Some("md");
    }
    non_citable.contains(cite.source_kind.as_str())
}

/// The configured kinds that declare no IDs (§FS-config.3.4.1), by name.
pub(super) fn non_citable_kind_names(config: &Config) -> BTreeSet<&str> {
    config
        .kinds
        .iter()
        .filter(|kind| !kind.citable)
        .map(|kind| kind.kind.as_str())
        .collect()
}

/// The evaluation units for one citing kind's obligations (§FS-config.3.9):
/// per file for `code` and for every non-citable kind, per case (over the case's
/// scanned-file citations) for `E2E`, per non-stub declaration otherwise. Reads
/// the citation indexes built once in [`check_citation_obligations`] rather than
/// rescanning.
///
/// Why an E2E case with no matching evidence is still a unit: normal root scans
/// skip fixture trees, so dropping the empty cases would quietly stop `must` from
/// being a hard gate on exactly the cases that carry no evidence yet.
pub(super) fn obligation_units<'a>(
    citing_kind: &str,
    config: &Config,
    findings: &'a Catalog,
    by_decl: &BTreeMap<&'a Id, Vec<&'a Citation>>,
    by_file: &BTreeMap<(&'a str, &'a Path), Vec<&'a Citation>>,
    e2e_by_case: &BTreeMap<&'a Path, Vec<&'a Citation>>,
) -> Vec<ObligationUnit<'a>> {
    if citing_kind == config.homeless_kind() || non_citable_kind_names(config).contains(citing_kind)
    {
        // §FS-check.3.11.3: a kind with no declarations answers with its files,
        // cut by the row's `grounding_level` — the same unit §FS-check.3.6 asks
        // for grounding, in `grounding.rs`.
        return file_obligation_units(citing_kind, config, findings, by_file);
    }

    let mut units = Vec::new();
    for (id, decls) in &findings.declarations {
        if id.kind != citing_kind {
            continue;
        }
        for decl in decls {
            if decl.is_stub {
                continue;
            }
            if let Some(case) = &decl.e2e_case {
                // §FS-config.3.9: an E2E obligation evaluates over the case's
                // manifest refs and scanned files when explicit scope includes
                // them; a case with no matching evidence is still a unit.
                let citations = e2e_by_case
                    .get(decl.file.as_path())
                    .cloned()
                    .unwrap_or_default();
                let e2e_spec_refs = case.spec_refs.iter().collect();
                units.push(ObligationUnit {
                    id: Some(id),
                    place: None,
                    path: decl.file.clone(),
                    line: decl.line,
                    citations,
                    e2e_spec_refs,
                });
            } else {
                let citations = by_decl.get(id).cloned().unwrap_or_default();
                units.push(ObligationUnit {
                    id: Some(id),
                    place: None,
                    path: decl.file.clone(),
                    line: decl.line,
                    citations,
                    e2e_spec_refs: Vec::new(),
                });
            }
        }
    }
    units
}
