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
/// target of each index-file citation external enrollment inspects, and the
/// scoped guidance files no row lists.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Expected {
    /// The block version a current managed block carries (§FS-init.2.3.7).
    pub block_version: u32,
    /// `AGENTS.md` first when it exists, then each companion (§FS-check.3.5).
    pub entrypoints: Vec<ExpectedEntrypoint>,
    /// The companion probe's io error, kept as data so the check reports it
    /// where it reported it before (§FS-check.3.5).
    pub entrypoint_probe_error: Option<(PathBuf, String)>,
    /// `fmt`'s bare-ID link destination keyed by (index file, ID)
    /// (§FS-check.3.18.3, §DF-index-entry-form.2.7).
    pub index_targets: BTreeMap<(PathBuf, Id), String>,
    /// The scoped `AGENTS.md` files and their companions that no row's `files`
    /// lists: guidance about a directory's units, never a unit of it
    /// (§FS-init.2.3.5.11).
    pub non_units: BTreeSet<PathBuf>,
}

/// §AR-checker.1.3: one entrypoint and the config-derived sections its managed
/// block should carry, each rendered for this file's conversation surface
/// (§FS-init.2.3.6.1).
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedEntrypoint {
    pub path: PathBuf,
    /// Whether a missing block is a finding here (§FS-check.3.5).
    pub require_block: bool,
    pub sections: Vec<ExpectedSection>,
    /// What the file's whole managed block should be, for a scoped entrypoint
    /// (§FS-init.2.3.5.12): the block of a destination of the live render, or
    /// `None` for an orphan the list on disk names (§FS-init.2.3.5.13.1). Root
    /// entrypoints are held to their sections alone.
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

/// One generated `###` section: its heading, the noun a drift message names it
/// by, and the bytes `grund init` would write (§FS-check.3.5).
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedSection {
    pub heading: &'static str,
    pub noun: &'static str,
    pub bytes: String,
}
