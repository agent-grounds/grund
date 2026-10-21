//! Text-only index-entry explanations (§FS-refs.3.1.1, §FS-refs.3.3.1), carried
//! beside the published records so JSON and library record shapes stay stable.

use std::collections::BTreeSet;

use super::refs_query::hit_path;
use crate::queries::kind_index_entries;
use crate::resolver::WorkspaceContext;

/// The exact excluded sites, spelled as refs spells its hits. Classification
/// comes from the same semantic source as check and list (§FS-check.4.1.2),
/// including canonical enrollment and value-binding exceptions.
pub(super) fn index_entry_sites(context: &WorkspaceContext) -> BTreeSet<(String, usize, usize)> {
    let mut sites = BTreeSet::new();
    for project in &context.projects {
        let entries = kind_index_entries(project);
        for citation in project.findings.edges() {
            if entries.is_index_entry(citation) {
                sites.insert((
                    hit_path(context, project, &citation.file),
                    citation.line,
                    citation.column,
                ));
            }
        }
    }
    sites
}
