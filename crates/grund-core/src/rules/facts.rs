//! Producer-neutral in-memory fact schema (§FS-rules.5.1, §AR-rules.3).

use super::RuleAnchor;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct NodeKey(pub(crate) String);
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SiteKey(pub(crate) String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Completeness {
    Complete,
    Incomplete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FactHeader {
    pub(crate) schema: u32,
    pub(crate) project: String,
    pub(crate) producer: String,
    pub(crate) completeness: Completeness,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NodeMeta {
    pub(crate) label: String,
    pub(crate) anchor: RuleAnchor,
}
/// A site's display metadata. Mixed paths need no separate mark (§FS-rules.7.12).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SiteMeta {
    pub(crate) label: String,
    pub(crate) anchor: RuleAnchor,
}

/// Every declaration/chapter exists independently of citations and every
/// physical citation owns a distinct site key (§FS-rules.5.1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RuleFacts {
    pub(crate) header: FactHeader,
    pub(crate) decl: Vec<(NodeKey, String)>,
    /// Each chapter unit's node, path and display name; a front-matter unit has no
    /// display name, so a chapter-presence rule never counts or lists it
    /// (§FS-fields.front-matter.surfaces).
    pub(crate) chapter: Vec<(NodeKey, String, Option<String>)>,
    pub(crate) contains: Vec<(NodeKey, NodeKey)>,
    pub(crate) cites: Vec<(SiteKey, NodeKey, NodeKey)>,
    pub(crate) site_in: Vec<(SiteKey, NodeKey)>,
    pub(crate) nodes: BTreeMap<NodeKey, NodeMeta>,
    pub(crate) sites: BTreeMap<SiteKey, SiteMeta>,
}

/// A direct heading of a declaration that is not a chapter: a numbered section or an
/// unmarked heading. No rule counts it; a too-low chapter count shows the named form
/// that would count it (§FS-rules.7.2.1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NonChapter {
    pub(crate) line: usize,
    pub(crate) level: usize,
    pub(crate) text: String,
}

/// Each declaration node's direct non-chapter headings, in source order.
pub(crate) type NonChapters = BTreeMap<NodeKey, Vec<NonChapter>>;
