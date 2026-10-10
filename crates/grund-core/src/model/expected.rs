//! §AR-checker.1.3: what presentation decided, as bytes. The writers render it
//! before the checker's relational half runs (§AR-system.2.11), and the checker
//! compares it with disk and renders nothing (§AR-checker.2.7, §AR-checker.2.16),
//! so a presentation-only change can move a drift finding through these bytes
//! and no other verdict (§FS-check.3.5.4).

use std::collections::BTreeMap;
use std::path::PathBuf;

use super::records::Id;

/// §AR-checker.1.3: the managed-block version this config asks for, every
/// agent entrypoint in the order the check compares them, and the canonical
/// link target of each index-file citation external enrollment inspects.
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
}

/// One generated `###` section: its heading, the noun a drift message names it
/// by, and the bytes `grund init` would write (§FS-check.3.5).
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedSection {
    pub heading: &'static str,
    pub noun: &'static str,
    pub bytes: String,
}
