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
use super::project::{Grounding, Rules, Rung, Schema, Strength};
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
    // §FS-config.3.9.2.3: the homeless kind's declared row, else the default — or
    // the ladder v2 writes for the undeclared complement by name.
    schema
        .rows
        .iter()
        .find(|row| row.is_complement())
        .map_or_else(|| level(schema.complement_name()), |row| level(&row.name))
}

/// The unit a grounding level spells (§FS-config-v2.rules.grounding): `file`
/// for the file, `h2` to `h6` for a heading level. One spelling for the v2
/// reader, its renderer and the schema view's badge (§FS-schema.6).
pub(crate) fn grounding_unit(level: usize) -> String {
    if level <= 1 {
        "file".to_string()
    } else {
        format!("h{level}")
    }
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
    row_pair(rules, requires_grounding_by_default(rules, run), kind)
}

/// The effective pair for the row named `kind` as the configuration alone says
/// it, with no run's `--require-grounding`: the `must` rung the schema view
/// badges (§FS-schema.6).
pub(crate) fn configured_row_grounding(rules: &Rules, kind: &str) -> (bool, usize) {
    row_pair(rules, rules.grounding.require, kind)
}

/// The row's word where it has one, else `require_default` and the default level.
fn row_pair(rules: &Rules, require_default: bool, kind: &str) -> (bool, usize) {
    let row = rules.grounding.kinds.get(kind);
    (
        row.and_then(|row| row.require).unwrap_or(require_default),
        row.and_then(|row| row.level)
            .unwrap_or(rules.grounding.level),
    )
}

/// The effective pair for the homeless kind (§FS-config.3.9.2.3) — its declared
/// row when the schema has one, else the defaults. A v2 ladder written for the
/// undeclared complement by name is its row's word (§FS-config-v2.rules.grounding).
pub(crate) fn homeless_row_grounding(schema: &Schema, rules: &Rules, run: &Run) -> (bool, usize) {
    match schema.rows.iter().find(|row| row.is_complement()) {
        Some(row) => row_grounding(rules, run, &row.name),
        None => row_grounding(rules, run, schema.complement_name()),
    }
}

/// Whether any grounding rung, at any strength, reaches the files of the row
/// named `kind` (§FS-init.2.3.5.2.2): its effective `must` pair, or a v2 rung
/// below `must` that governs it (§FS-config-v2.rules.grounding). A row for which
/// this is false grounds nothing, so a file there that cites nothing is never
/// judged (§FS-check.3.11).
pub(crate) fn row_grounds_at_any_strength(rules: &Rules, run: &Run, kind: &str) -> bool {
    row_grounding(rules, run, kind).0 || !soft_grounding_rungs(rules, kind).is_empty()
}

/// The one setting that makes a file of the row named `kind` that cites nothing a
/// finding, spelled in config `version` (§FS-init.2.3.5.2.2): version 2's
/// grounding ladder, version 1's row key. `code` wraps each config fragment, so
/// the Citation directions bullet and the place-refusal hint
/// (§FS-rule-refusals.sentences.rewrite.no-form.place) spell it from this alone.
pub(crate) fn grounding_setting(kind: &str, version: u32, code: fn(&str) -> String) -> String {
    if version >= 2 {
        code(&format!(
            "[rules.citations.{kind}.grounding] must = \"file\""
        ))
    } else {
        format!(
            "{} on its {} row",
            code("require_grounding = true"),
            code("[[kinds]]")
        )
    }
}

/// Whether any place is grounded at all (§FS-check.3.6), the checker's early out
/// and the gate that arms `ungrounded` (§FS-check.1.4.1): a row, or the homeless
/// kind, whose effective pair requires grounding, so a default that every row sets
/// `false` grounds nothing (§FS-config.3.4.8.3). A v2 ladder rung below `must`
/// grounds too (§FS-config-v2.rules.grounding). `Config::grounding_enabled` asks
/// the looser question of whether anything sets the key at all.
pub(crate) fn any_place_grounded(schema: &Schema, rules: &Rules, run: &Run) -> bool {
    schema
        .rows
        .iter()
        .any(|row| row_grounding(rules, run, &row.name).0)
        || homeless_row_grounding(schema, rules, run).0
        || soft_rungs_written(&rules.grounding)
}

/// Whether any v2 ladder rung below `must` asks for citations
/// (§FS-config-v2.rules.grounding); never in v1, whose grounding is the pair alone.
fn soft_rungs_written(grounding: &Grounding) -> bool {
    ladders(grounding).any(|rung| matches!(rung.strength, Strength::Warn | Strength::Should))
}

/// The finest unit any rung governing the row named `kind` asks for: its
/// grounding level, or a finer v2 rung below `must` (§FS-config-v2.rules.grounding).
/// What the scanner records structure to (§AR-scanner.2.7.1).
pub(crate) fn finest_grounding_level_for_kind(schema: &Schema, rules: &Rules, kind: &str) -> usize {
    soft_grounding_rungs(rules, kind)
        .into_iter()
        .map(|(_, level)| level)
        .fold(grounding_level_for_kind(schema, rules, kind), usize::max)
}

/// The rungs below `must` that govern the files of the row named `kind`
/// (§FS-config-v2.rules.grounding): the row's own ladder where it wrote one,
/// which replaces the project's whole, else the project's. The `must` rung is
/// the pair [`row_grounding`] answers; a `may` rung checks nothing. Empty for
/// every v1 config, whose grounding is the pair alone.
pub(crate) fn soft_grounding_rungs(rules: &Rules, kind: &str) -> Vec<(Strength, usize)> {
    let grounding = &rules.grounding;
    let ladder = grounding
        .kinds
        .get(kind)
        .and_then(|row| row.ladder.as_ref())
        .or(grounding.ladder.as_ref());
    ladder
        .into_iter()
        .flatten()
        .filter(|rung| matches!(rung.strength, Strength::Warn | Strength::Should))
        .map(|rung| (rung.strength, rung.level))
        .collect()
}

/// Every v2 ladder rung the project wrote, the project's and each row's
/// (§FS-config-v2.rules.grounding).
fn ladders(grounding: &Grounding) -> impl Iterator<Item = &Rung> {
    grounding
        .ladder
        .iter()
        .chain(
            grounding
                .kinds
                .values()
                .filter_map(|row| row.ladder.as_ref()),
        )
        .flatten()
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

    /// Whether anything turns grounding on — the key, the flag, a row or a v2 ladder
    /// rung below `must` (§FS-config-v2.rules.grounding) — which is when a
    /// `[reference] grounding_level` loads back (§FS-config.3.4.8). The checker's
    /// early out, [`any_place_grounded`], also asks that a place still inherits it.
    pub fn grounding_enabled(&self) -> bool {
        self.require_grounding
            || self
                .kinds
                .iter()
                .any(|kind| kind.require_grounding == Some(true))
            || homeless_row_grounding(self.schema(), self.rules(), self.run()).0
            || soft_rungs_written(&self.rules().grounding)
    }

    /// The rungs below `must` for the row named `kind`: [`soft_grounding_rungs`]
    /// read through the façade, for the lowering cases.
    #[cfg(test)]
    pub(crate) fn soft_grounding_rungs(&self, kind: &str) -> Vec<(Strength, usize)> {
        soft_grounding_rungs(self.rules(), kind)
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
