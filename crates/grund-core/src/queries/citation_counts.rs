use std::collections::{BTreeMap, BTreeSet};

use crate::checker::KindIndexEntries;
use crate::model::Id;
use crate::resolver::{
    SettledHeadings, WorkspaceContext, WorkspaceProject, index_link_targets, index_link_targets_for,
};

/// The two per-target-alias citation counts `grund list` needs, built in one
/// pass (§FS-list.3.2.1, §DF-index-not-an-inbound-citation).
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
/// explanation, so `refs` stays the total and the sentence lives in §FS-list.3.2.1.
///
/// §FS-imports.graph.diamond: a tree two alias paths reach is counted once, under
/// its canonical alias path, and each of its alias paths reads that one count.
pub(crate) struct ListCitationCounts<'a> {
    refs: BTreeMap<&'a str, BTreeMap<&'a Id, usize>>,
    used: BTreeMap<&'a str, BTreeMap<&'a Id, usize>>,
    empty: BTreeMap<&'a Id, usize>,
    canonical: BTreeMap<&'a str, &'a str>,
}

impl<'a> ListCitationCounts<'a> {
    pub(crate) fn new(context: &'a WorkspaceContext) -> Self {
        let projects = &context.projects;
        let canonical = projects
            .iter()
            .filter(|copy| !copy.reports_sites())
            .filter_map(|copy| {
                let same = |owner: &&WorkspaceProject| {
                    owner.config.root == copy.config.root && owner.reports_sites()
                };
                let owner = projects.iter().find(same)?;
                Some((copy.alias.as_str(), owner.alias.as_str()))
            })
            .collect();
        let mut counts = Self {
            refs: BTreeMap::new(),
            used: BTreeMap::new(),
            empty: BTreeMap::new(),
            canonical,
        };
        for source in projects.iter().filter(|source| source.reports_sites()) {
            let index_entries = kind_index_entries(source);
            // §FS-refs.2: `list` counts the set `refs` lists, lent citations included.
            for citation in source.findings.edges() {
                let target_alias = counts.tree(match &citation.namespace {
                    Some(ns) => ns.as_str(),
                    None => source.alias.as_str(),
                });
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
        self.refs.get(self.tree(alias)).unwrap_or(&self.empty)
    }

    /// The alias path a tree's count is kept under (§FS-imports.graph.diamond).
    fn tree<'s>(&self, alias: &'s str) -> &'s str
    where
        'a: 's,
    {
        self.canonical.get(alias).copied().unwrap_or(alias)
    }

    /// The same count minus each kind's own index entries — what `--unused`
    /// asks, so a declaration only its index names still counts as unused.
    pub(crate) fn used_for(&self, alias: &str) -> &BTreeMap<&'a Id, usize> {
        self.used.get(self.tree(alias)).unwrap_or(&self.empty)
    }
}

/// `source`'s own kind-index entries, the citations `--unused` discounts
/// (§FS-list.1.3, §DF-index-not-an-inbound-citation).
pub(crate) fn kind_index_entries(source: &WorkspaceProject) -> KindIndexEntries {
    // §AR-checker.2.16: the enrollment destination, as `Expected` carries it.
    let index_targets = index_link_targets(
        &source.config.project().presentation,
        source.config.schema(),
        source.config.frame(),
        &source.findings,
        &SettledHeadings::default(),
    );
    KindIndexEntries::new(
        &source.findings,
        source.config.schema(),
        source.config.frame(),
        &index_targets,
    )
}

/// The kind-index entries among citations of `ids`, the sites text refs labels
/// (§FS-refs.3.1.1), by check's defining classifier (§FS-check.4.1.2) restricted
/// to those IDs, so a refs run links no other declaration (§GOAL-fast-feedback).
pub(crate) fn kind_index_entries_for(
    source: &WorkspaceProject,
    ids: &BTreeSet<Id>,
) -> KindIndexEntries {
    // §AR-checker.2.16: the enrollment destination, as `Expected` carries it.
    let index_targets = index_link_targets_for(
        &source.config.project().presentation,
        source.config.schema(),
        source.config.frame(),
        &source.findings,
        &SettledHeadings::default(),
        Some(ids),
    );
    KindIndexEntries::restricted(
        &source.findings,
        source.config.schema(),
        source.config.frame(),
        &index_targets,
        ids,
    )
}
