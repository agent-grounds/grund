//! Text-only index-entry explanations (§FS-refs.3.1.1, §FS-refs.3.3.1), carried
//! beside the published records so JSON and library record shapes stay stable.

use std::collections::{BTreeMap, BTreeSet};

use super::refs_query::hit_path;
use crate::checker::KindIndexFiles;
use crate::model::{Citation, Id};
use crate::queries::kind_index_entries_for;
use crate::resolver::{WorkspaceContext, WorkspaceProject};

/// The exact excluded sites among `hits`, spelled as refs spells its hits.
/// Classification comes from the same semantic source as check and list
/// (§FS-check.4.1.2), including canonical enrollment and value-binding
/// exceptions. Every entry sits in a configured kind index, so only a hit in
/// one is classified, by a classifier restricted to those hits' IDs: refs pays
/// for the sites it prints, not for every index (§GOAL-fast-feedback).
pub(super) fn index_entry_sites<'c>(
    context: &WorkspaceContext,
    hits: impl IntoIterator<Item = (&'c WorkspaceProject, &'c Citation)>,
) -> BTreeSet<(String, usize, usize)> {
    let mut files: BTreeMap<usize, KindIndexFiles> = BTreeMap::new();
    let mut candidates: BTreeMap<usize, Vec<&Citation>> = BTreeMap::new();
    for (project, citation) in hits {
        let Some(index) = context
            .projects
            .iter()
            .position(|candidate| std::ptr::eq(candidate, project))
        else {
            continue;
        };
        let config = &project.config;
        let files = files
            .entry(index)
            .or_insert_with(|| KindIndexFiles::new(config.schema(), config.frame()));
        if citation.namespace.is_none() && files.contains(&citation.file) {
            candidates.entry(index).or_default().push(citation);
        }
    }
    let mut sites = BTreeSet::new();
    for (index, citations) in candidates {
        let project = &context.projects[index];
        let ids: BTreeSet<Id> = citations
            .iter()
            .map(|citation| citation.id.clone())
            .collect();
        let entries = kind_index_entries_for(project, &ids);
        for citation in citations {
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
