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
    pub(crate) chapter: Vec<(NodeKey, String, String)>,
    pub(crate) contains: Vec<(NodeKey, NodeKey)>,
    pub(crate) cites: Vec<(SiteKey, NodeKey, NodeKey)>,
    pub(crate) site_in: Vec<(SiteKey, NodeKey)>,
    pub(crate) nodes: BTreeMap<NodeKey, NodeMeta>,
    pub(crate) sites: BTreeMap<SiteKey, SiteMeta>,
}
