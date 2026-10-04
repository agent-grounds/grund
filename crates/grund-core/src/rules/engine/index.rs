//! Snapshot-local joins shared by selectors and rule families (§AR-rules.3.1,
//! §FS-rules.5.3). Keys remain opaque; ordered buckets retain fact order rather
//! than deriving ancestry or diagnostic order from a producer's key spelling.
//! Ordered maps use the keys' existing ordering without changing RuleFacts;
//! lookups cost logarithmic time instead of scanning whole relations.

use super::super::facts::{NodeKey, RuleFacts, SiteKey};
use std::collections::{BTreeMap, BTreeSet};

/// A borrowed physical citation row, including its immediate source (§FS-rules.5.1).
pub(super) type CitationRow = (SiteKey, NodeKey, NodeKey);

/// Private evaluation data, never a producer or serialized boundary (§AR-rules.3.1).
pub(super) struct FactIndex<'a> {
    pub(super) declarations: BTreeMap<&'a str, Vec<&'a NodeKey>>,
    pub(super) labels: BTreeMap<&'a str, Vec<&'a NodeKey>>,
    chapters: BTreeMap<&'a NodeKey, Vec<usize>>,
    pub(super) paths: BTreeMap<&'a str, Vec<&'a NodeKey>>,
    outgoing: BTreeMap<&'a NodeKey, Vec<&'a CitationRow>>,
    incoming: BTreeMap<&'a NodeKey, Vec<&'a CitationRow>>,
    kinds: BTreeMap<&'a NodeKey, &'a str>,
    parents: BTreeMap<&'a NodeKey, Vec<&'a NodeKey>>,
    owners: BTreeMap<&'a NodeKey, Option<&'a NodeKey>>,
}

impl<'a> FactIndex<'a> {
    /// Index the immutable relations once (§FS-rules.5.3, §AR-rules.3.1).
    /// Bucket vectors retain declaration, chapter, node-metadata and citation
    /// iteration order. Existence joins deduplicate edges, never physical sites
    /// or citation rows (§FS-rules.5.1).
    pub(super) fn new(facts: &'a RuleFacts) -> Self {
        let mut index = Self {
            declarations: BTreeMap::new(),
            labels: BTreeMap::new(),
            chapters: BTreeMap::new(),
            paths: BTreeMap::new(),
            outgoing: BTreeMap::new(),
            incoming: BTreeMap::new(),
            kinds: BTreeMap::new(),
            parents: BTreeMap::new(),
            owners: BTreeMap::new(),
        };
        let mut nodes = BTreeSet::new();
        for (node, kind) in &facts.decl {
            index.declarations.entry(kind).or_default().push(node);
            index.kinds.entry(node).or_insert(kind);
            nodes.insert(node);
        }
        for (node, meta) in &facts.nodes {
            index.labels.entry(&meta.label).or_default().push(node);
            nodes.insert(node);
        }
        for (parent, child) in &facts.contains {
            index.parents.entry(child).or_default().push(parent);
            nodes.extend([parent, child]);
        }
        for (row, (node, path, _)) in facts.chapter.iter().enumerate() {
            index.paths.entry(path).or_default().push(node);
            nodes.insert(node);
            if let Some(parents) = index.parents.get(node) {
                for parent in parents.iter().copied().collect::<BTreeSet<_>>() {
                    index.chapters.entry(parent).or_default().push(row);
                }
            }
        }
        let mut units: BTreeMap<&SiteKey, BTreeSet<&NodeKey>> = BTreeMap::new();
        for (site, unit) in &facts.site_in {
            units.entry(site).or_default().insert(unit);
            nodes.insert(unit);
        }
        for citation @ (site, from, target) in &facts.cites {
            index.incoming.entry(target).or_default().push(citation);
            if let Some(containing) = units.get(site) {
                for unit in containing {
                    index.outgoing.entry(unit).or_default().push(citation);
                }
            }
            nodes.extend([from, target]);
        }
        for node in nodes {
            index.resolve_owner(node);
        }
        index
    }

    /// Memoize each containment chain (§AR-rules.3.1), stopping at the nearest
    /// declaration (§FS-rules.5.1). The first parent preserves the evaluator's
    /// existing ownership walk even when a producer supplies multiple parents.
    fn resolve_owner(&mut self, node: &'a NodeKey) {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut current = node;
        let owner = loop {
            if let Some(owner) = self.owners.get(current) {
                break *owner;
            }
            if self.kinds.contains_key(current) {
                break Some(current);
            }
            if !seen.insert(current) {
                break None;
            }
            chain.push(current);
            let Some(parent) = self.parents.get(current).and_then(|p| p.first()) else {
                break None;
            };
            current = parent;
        };
        self.owners.entry(current).or_insert(owner);
        for node in chain {
            self.owners.insert(node, owner);
        }
    }

    /// Lookup ownership without another global containment walk (§FS-rules.5.3).
    pub(super) fn owner(&self, node: &NodeKey) -> Option<&'a NodeKey> {
        self.owners.get(node).copied().flatten()
    }

    /// Recover the kind of the immediate source's owner (§FS-rules.3.4).
    pub(super) fn declaration_kind(&self, node: &NodeKey) -> Option<&'a str> {
        self.owner(node)
            .and_then(|owner| self.kinds.get(owner).copied())
    }

    /// Every declared unit of a kind, including uncited ones (§FS-rules.5.1).
    pub(super) fn declarations_of_kind(&self, kind: &str) -> &[&'a NodeKey] {
        self.declarations
            .get(kind)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Direct chapter rows only; descendants do not count as children (§FS-rules.3.1).
    pub(super) fn chapters_of(&self, node: &NodeKey) -> &[usize] {
        self.chapters.get(node).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Physical citation rows contained in this unit, once per row (§FS-rules.5.1).
    pub(super) fn citations_in(&self, node: &NodeKey) -> &[&'a CitationRow] {
        self.outgoing.get(node).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Incoming rows match the immediate target, without owner promotion (§FS-rules.3.4).
    pub(super) fn citations_to(&self, node: &NodeKey) -> &[&'a CitationRow] {
        self.incoming.get(node).map(Vec::as_slice).unwrap_or(&[])
    }
}
