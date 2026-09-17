use std::collections::BTreeMap;

use crate::checker::KindIndexEntries;
use crate::model::Id;
use crate::workspace::WorkspaceContext;

/// The two per-target-alias citation counts `grund list` needs, built in one
/// pass (§FS-list.3.2, §DF-index-not-an-inbound-citation).
///
/// §FS-workspace.8.3: each citation belongs to exactly one target project — its
/// `namespace` when qualified, the citing project when not — so a single pass
/// over every project's citations builds the per-target lookup. Doing this once
/// up front (rather than re-walking every project's citations per target) keeps
/// the count linear in the total citation set, not quadratic in the project
/// count.
///
/// The two counts differ by exactly the index entries. `refs` is the total
/// `grund refs` lists, which is the number the `refs` column prints; `used` drops
/// each kind's own index entries, which name every declaration in their folder
/// by construction, and is the number `--unused` selects on. One count would
/// have to be wrong for one of the two questions; two counts printed by two
/// commands that disagreed would be worse than one that needs a sentence of
/// explanation, so `refs` stays the total and the sentence lives in §FS-list.3.2.
pub(crate) struct ListCitationCounts<'a> {
    refs: BTreeMap<&'a str, BTreeMap<&'a Id, usize>>,
    used: BTreeMap<&'a str, BTreeMap<&'a Id, usize>>,
    empty: BTreeMap<&'a Id, usize>,
}

impl<'a> ListCitationCounts<'a> {
    pub(crate) fn new(context: &'a WorkspaceContext) -> Self {
        let mut counts = Self {
            refs: BTreeMap::new(),
            used: BTreeMap::new(),
            empty: BTreeMap::new(),
        };
        for source in &context.projects {
            let index_entries = KindIndexEntries::new(&source.findings, &source.config);
            for citation in &source.findings.citations {
                let target_alias: &str = match &citation.namespace {
                    Some(ns) => ns.as_str(),
                    None => source.alias.as_str(),
                };
                *counts
                    .refs
                    .entry(target_alias)
                    .or_default()
                    .entry(&citation.id)
                    .or_insert(0) += 1;
                if !index_entries.is_index_entry(citation) {
                    *counts
                        .used
                        .entry(target_alias)
                        .or_default()
                        .entry(&citation.id)
                        .or_insert(0) += 1;
                }
            }
        }
        counts
    }

    /// How many citations name each of `alias`'s IDs — the `refs` column.
    ///
    /// A `<§><alias>/<ID>` citation targets `<alias>`'s declaration, so it is
    /// attributed to the *target* project, not the citing one; that is what lets
    /// a workspace-root `grund list --unused` see members' declarations that only
    /// sibling projects cite.
    pub(crate) fn refs_for(&self, alias: &str) -> &BTreeMap<&'a Id, usize> {
        self.refs.get(alias).unwrap_or(&self.empty)
    }

    /// The same count minus each kind's own index entries — what `--unused`
    /// asks, so a declaration only its index names still counts as unused.
    pub(crate) fn used_for(&self, alias: &str) -> &BTreeMap<&'a Id, usize> {
        self.used.get(alias).unwrap_or(&self.empty)
    }
}
