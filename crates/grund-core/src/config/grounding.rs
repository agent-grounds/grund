//! The grounding half of the config (§FS-config.3.4.8): the two keys that say
//! *whether* a place's files must cite a declared ID and *how finely* that is
//! asked, their `[reference]` defaults, and the per-row resolution every reader
//! of them goes through. The v1 spelling of the keys is `v1/grounding.rs`, and
//! the rules that reject a combination they cannot describe run once on the
//! lowered project in `validate.rs` (§AR-config.4).
//!
//! They live here rather than with the rest of `[[kinds]]` because they are one
//! contract spanning two sections: each key is written either on a row or in
//! `[reference]` as the default for every row, and a validation rule reaches
//! across both (`[reference] grounding_level` is an error when no row turns
//! grounding on). Keeping the pair in one file is what stops that rule from
//! being written twice.

use super::kind::KindConfig;
use super::record::{Config, declared_homeless_kind};

/// The effective grounding level of the row named `kind` (§FS-config.3.4.8) —
/// including the homeless kind, whose row a config need not have declared.
///
/// A `[[kinds]]` lookup over the pair below, and the one reading of it a caller
/// does by kind name rather than by row: the scanner asks it per file
/// (§AR-scanner.2.7.1) and the checker per citing kind (§FS-check.3.11.3), which is
/// why it sits with the keys rather than with either of them.
pub(crate) fn grounding_level_for_kind(config: &Config, kind: &str) -> usize {
    config
        .kinds
        .iter()
        .find(|configured| configured.kind == kind)
        .map(|configured| config.kind_grounding(configured).1)
        .unwrap_or_else(|| config.homeless_grounding().1)
}

impl Config {
    /// The effective grounding pair for one `[[kinds]]` row (§FS-config.3.4.8.3):
    /// the row's word where it has one, else the `[reference]` default — which is
    /// also what `grund check --require-grounding` sets, so an explicit row
    /// `false` wins over the flag.
    pub(crate) fn kind_grounding(&self, kind: &KindConfig) -> (bool, usize) {
        (
            kind.require_grounding.unwrap_or(self.require_grounding),
            kind.grounding_level.unwrap_or(self.grounding_level),
        )
    }

    /// The effective pair for the homeless kind (§FS-config.3.9.2.3) — its declared
    /// row when the table has one, else the `[reference]` defaults, since an
    /// undeclared complement has no row to write them on.
    pub(crate) fn homeless_grounding(&self) -> (bool, usize) {
        declared_homeless_kind(&self.kinds)
            .map(|kind| self.kind_grounding(kind))
            .unwrap_or((self.require_grounding, self.grounding_level))
    }

    /// Whether any place is grounded at all — the early out that keeps the whole
    /// pass off a tree that asked for none (§FS-check.3.6).
    pub fn grounding_enabled(&self) -> bool {
        self.require_grounding
            || self
                .kinds
                .iter()
                .any(|kind| kind.require_grounding == Some(true))
    }

    /// The `[[kinds]]` grounding lines `grund config show` prints for one row
    /// (§FS-config.4.2.1): each key only where the row's effective value differs
    /// from the effective global, which is printed under `[reference]`. A row
    /// that inherits both prints neither, and the shown config loads back to the
    /// same effective values — printing a key on every row would be noise, and
    /// printing a level under a row that turned grounding off would not load at
    /// all (§FS-config.3.4.8.6).
    pub fn kind_grounding_toml_lines(&self, kind: &KindConfig) -> Vec<String> {
        let (require, level) = self.kind_grounding(kind);
        let mut lines = Vec::new();
        if require != self.require_grounding {
            lines.push(format!("require_grounding = {require}"));
        }
        if level != self.grounding_level {
            lines.push(format!("grounding_level = {level}"));
        }
        lines
    }
}
