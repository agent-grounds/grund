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
use super::project::{Rules, Schema};
use super::record::Config;
use super::run::Run;

/// The effective grounding level of the row named `kind` (§FS-config.3.4.8) —
/// including the homeless kind, whose row a config need not have declared.
///
/// A lookup over the schema's rows and the rules' grounding pair, and the one
/// reading of it a caller does by kind name rather than by row: `compile` asks
/// it per row for the scan demand (§AR-config.6.1) and the checker per citing
/// kind (§FS-check.3.11.3), which is why it sits with the keys rather than with
/// either of them.
pub(crate) fn grounding_level_for_kind(schema: &Schema, rules: &Rules, kind: &str) -> usize {
    let grounding = &rules.grounding;
    let level = |name: &str| {
        grounding
            .kinds
            .get(name)
            .and_then(|row| row.level)
            .unwrap_or(grounding.level)
    };
    if schema.rows.iter().any(|row| row.name == kind) {
        return level(kind);
    }
    // §FS-config.3.9.2.3: the homeless kind's declared row, else the default.
    schema
        .rows
        .iter()
        .find(|row| row.is_complement())
        .map_or(grounding.level, |row| level(&row.name))
}

/// The effective `require_grounding` default (§FS-config.3.4.8.3): the
/// `[reference]` key, or `grund check --require-grounding`, which sets the same
/// default for one run (§FS-check.3.5) — so an explicit row `false` still wins.
pub(crate) fn requires_grounding_by_default(rules: &Rules, run: &Run) -> bool {
    rules.grounding.require || run.scope.require_grounding
}

/// The effective grounding pair for the row named `kind` (§FS-config.3.4.8.3):
/// the row's word where it has one, else the default. The records' reading of
/// `Config::kind_grounding`, for a stage handed no façade (§AR-config.5).
pub(crate) fn row_grounding(rules: &Rules, run: &Run, kind: &str) -> (bool, usize) {
    let row = rules.grounding.kinds.get(kind);
    (
        row.and_then(|row| row.require)
            .unwrap_or_else(|| requires_grounding_by_default(rules, run)),
        row.and_then(|row| row.level)
            .unwrap_or(rules.grounding.level),
    )
}

/// The effective pair for the homeless kind (§FS-config.3.9.2.3) — its declared
/// row when the schema has one, else the defaults.
pub(crate) fn homeless_row_grounding(schema: &Schema, rules: &Rules, run: &Run) -> (bool, usize) {
    match schema.rows.iter().find(|row| row.is_complement()) {
        Some(row) => row_grounding(rules, run, &row.name),
        None => (
            requires_grounding_by_default(rules, run),
            rules.grounding.level,
        ),
    }
}

/// Whether any place is grounded at all (§FS-check.3.6): the records' reading of
/// `Config::grounding_enabled`.
pub(crate) fn any_place_grounded(schema: &Schema, rules: &Rules, run: &Run) -> bool {
    requires_grounding_by_default(rules, run)
        || schema.rows.iter().any(|row| {
            rules
                .grounding
                .kinds
                .get(&row.name)
                .and_then(|row| row.require)
                == Some(true)
        })
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
