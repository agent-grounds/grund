//! §AR-checker.1.3: what presentation decided, as bytes. The writers render it
//! before the checker's relational half runs (§AR-system.2.11), and the checker
//! compares it with disk and renders nothing (§AR-checker.2.7, §AR-checker.2.16),
//! so a presentation-only change can move a drift finding through these bytes
//! (§FS-check.3.5.4), and no other verdict.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::records::Id;

/// §AR-checker.1.3: the managed-block version this config asks for, every
/// agent entrypoint in the order the check compares them, the canonical link
/// from each index to the IDs it cites and the declarations its folders own,
/// and the scoped guidance files no row lists.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Expected {
    /// The block version a current managed block carries (§FS-init.2.3.7).
    pub block_version: u32,
    /// `AGENTS.md` first when it exists, then each companion (§FS-check.3.5).
    pub entrypoints: Vec<ExpectedEntrypoint>,
    /// The companion probe's io error, kept as data so the check reports it
    /// where it reported it before (§FS-check.3.5).
    pub entrypoint_probe_error: Option<(PathBuf, String)>,
    /// Whether the two workspace sections are compared: only at the outermost
    /// root of a full checkout, and elsewhere the mark that leaves them out of
    /// both sides (§FS-check.3.5.5).
    pub workspace_sections: bool,
    /// `fmt`'s bare-ID link destination keyed by (index file, ID), for each ID
    /// the index cites and each declaration its folders own (§FS-check.3.18.3,
    /// §FS-check.3.18.5.2, §DF-index-entry-form.2.7).
    pub index_targets: BTreeMap<(PathBuf, Id), String>,
    /// The scoped `AGENTS.md` files and their companions that no row's `files`
    /// lists: guidance about a directory's units, never a unit of it
    /// (§FS-init.2.3.5.11).
    pub non_units: BTreeSet<PathBuf>,
}

/// §AR-checker.1.3: one entrypoint and the sections its managed block should
/// carry, each rendered for this file's conversation surface (§FS-init.2.3.6.1):
/// a root entrypoint's whole block, cut into the parts a stale finding names
/// (§FS-check.3.5.3, §FS-check.3.5.4), and a scoped one's generated sections.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedEntrypoint {
    pub path: PathBuf,
    /// Whether a missing block is a finding here (§FS-check.3.5).
    pub require_block: bool,
    pub sections: Vec<ExpectedSection>,
    /// What the file's whole managed block should be, for a scoped entrypoint
    /// (§FS-init.2.3.5.12): the block of a destination of the live render, or
    /// `None` for an orphan the list on disk names (§FS-init.2.3.5.13.1). Root
    /// entrypoints are held to their sections, which cover the whole block.
    pub scoped: Option<ScopedExpectation>,
}

/// §FS-init.2.3.5.8: what a scoped entrypoint is held to.
#[derive(Clone, Debug, PartialEq)]
pub enum ScopedExpectation {
    /// A destination's block, byte for byte.
    Block(String),
    /// An orphan: any block it still carries is stale.
    Orphan,
}

/// One section of a managed block: its heading, the noun a drift message names
/// it by, and the bytes `grund init` would write (§FS-check.3.5.3).
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedSection {
    pub heading: String,
    pub noun: String,
    pub bytes: String,
}

/// The five generated sections, by their nouns: the ones `config migrate`'s
/// guidance gate refuses a change to (§FS-config-migrate.5.3).
pub(crate) const GENERATED_SECTIONS: [&str; 5] = [
    "citation directions",
    "clickable citations",
    "chapter rules",
    "declaration shapes",
    "scoped guidance",
];
