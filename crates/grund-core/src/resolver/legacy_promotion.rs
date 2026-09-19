//! Off-grammar promotion across the **loaded set** (§AR-system.2.10): a
//! qualified candidate the scanner deferred, reconciled against the catalog of
//! the project its alias names (§FS-workspace.4.1, §FS-workspace.8,
//! §FS-config.3.2.6).
//!
//! The single-project half of the same reconciliation is the scanner's own, in
//! `scanner/legacy.rs`, and stays there: it runs on one `Findings` inside the
//! walk that produced it. This half needs *every* project's findings at once, so
//! it runs where the loaded set exists and reads the per-candidate promotion
//! downward (§AR-resolver.placement, §AR-system.4). It is deliberately still a
//! catalog operation: the authoring grammar stays strict and no unbacked token
//! becomes a citation.

use super::context::WorkspaceProject;
use crate::scanner::{
    configured_catalog_ids, legacy_catalog_ids, promote_legacy_candidate, sort_citations,
};

/// The workspace half of the same reconciliation (§FS-workspace.4.1,
/// §FS-workspace.8): a qualified candidate consults only the alias-selected
/// project's catalog and effective section grammar.
pub(crate) fn promote_qualified_legacy_citations(projects: &mut [WorkspaceProject]) {
    let catalogs = projects
        .iter()
        .map(|project| {
            (
                project.alias.clone(),
                project.config.clone(),
                configured_catalog_ids(&project.findings.declarations),
                legacy_catalog_ids(&project.findings.declarations),
            )
        })
        .collect::<Vec<_>>();

    for project in projects {
        let candidates = std::mem::take(&mut project.findings.legacy_citation_candidates);
        for candidate in candidates {
            let Some(alias) = candidate.namespace.as_deref() else {
                continue;
            };
            let Some((_, target_config, configured_catalog, catalog)) =
                catalogs.iter().find(|(target, _, _, _)| target == alias)
            else {
                continue;
            };
            promote_legacy_candidate(
                &project.config,
                target_config,
                configured_catalog,
                catalog,
                candidate,
                &mut project.findings.citations,
            );
        }
        sort_citations(&mut project.findings.citations);
    }
}
