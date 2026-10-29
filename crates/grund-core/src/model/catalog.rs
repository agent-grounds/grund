//! The rendered §FS-errors.5.5 catalog, embedded: what each code means, the
//! severity it carries, what turns it on and what clears its finding, in each
//! config version, read by the grammar of §FS-errors.5.5.1 when
//! `scripts/render_check_codes.py` rendered it. `grund config explain` prints it
//! (§FS-config-explain.4), the off line of §FS-check.1.4.1 is built from it and a
//! message's remedy is filled from it, so they all read one copy (§AR-config.9.1).

use std::fmt::{Display, Write as _};

use once_cell::sync::Lazy;
use serde::Deserialize;

use super::codes::Code;

const RENDERED: &str = include_str!("../../assets/explain/check-codes.json");

static ROWS: Lazy<Vec<Row>> =
    Lazy::new(|| parse(RENDERED).expect("the rendered check-code catalog parses"));

/// One row of the catalog, with its backticks already dropped (§FS-errors.5.5.1).
#[derive(Debug, Deserialize)]
pub(crate) struct Row {
    pub(crate) code: String,
    pub(crate) meaning: String,
    pub(crate) ramp: Option<String>,
    #[serde(rename = "2")]
    pub(crate) v2: Side,
    #[serde(rename = "1")]
    pub(crate) v1: Side,
}

/// What a row says for a config file read as one version.
#[derive(Debug, Deserialize)]
pub(crate) struct Side {
    pub(crate) severity: CodeSeverity,
    pub(crate) enabled: CodeEnabled,
    pub(crate) fix: CodeFix,
    pub(crate) note: Option<String>,
}

/// A *Fix* cell's side, read and its references expanded, backticks kept
/// (§FS-errors.5.5.1).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CodeFix {
    /// One template, its `{slot}`s unfilled.
    Template { text: String },
    /// One template per remedy, in the cell's order.
    Cases { cases: Vec<CodeCase> },
    /// `none — <why>`: the words after `none — `.
    None { text: String },
    /// `not in version N`.
    Absent { text: String },
}

/// One labelled case of a *Fix* cell.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CodeCase {
    pub label: String,
    pub text: String,
}

/// A *Severity* cell's side, read (§FS-errors.5.5.1).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "kind", content = "text", rename_all = "lowercase")]
pub enum CodeSeverity {
    /// `error` or `warning`, whatever is configured.
    Fixed(String),
    /// `none — suggestion`, shown only with `--suggestions`.
    Suggestion(String),
    /// `none — spent …`: the words after `none — `.
    Spent(String),
    /// `by strength: <setting>`: the setting.
    Strength(String),
    /// A severity and the setting or rule strength that changes it.
    Setting(String),
    /// `not in version N`.
    Absent(String),
}

/// An *Enabled by* cell's side, read (§FS-errors.5.5.1).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CodeEnabled {
    /// `—`: on in every run.
    Always,
    /// `not in version N`.
    Absent { text: String },
    /// Clauses every one of which the check needs, and the remark after the
    /// last one's `; `.
    Clauses {
        clauses: Vec<CodeClause>,
        remark: Option<String>,
    },
}

/// One clause of an *Enabled by* cell and what it is turned on by.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CodeClause {
    pub text: String,
    pub by: ClauseBy,
}

/// §FS-errors.5.5.1: a clause that says `is set` is a setting's, `is passed` a
/// flag's, both either's, and neither a condition of the tree.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClauseBy {
    Setting,
    Flag,
    Either,
    Condition,
}

impl Row {
    /// The side a config file read as `version` reads.
    pub(crate) fn side(&self, version: u32) -> &Side {
        if version >= 2 { &self.v2 } else { &self.v1 }
    }
}

impl CodeEnabled {
    /// The clauses of this side, none where it is always on or absent.
    pub(crate) fn clauses(&self) -> &[CodeClause] {
        match self {
            CodeEnabled::Clauses { clauses, .. } => clauses,
            CodeEnabled::Always | CodeEnabled::Absent { .. } => &[],
        }
    }
}

/// The template `code`'s row gives in `version`, or its case `case`: the side of
/// that version, or the other where that one holds none. A side that is one
/// template answers every case the other side names.
fn template(code: Code, version: u32, case: Option<&str>) -> &'static str {
    let row = row(code);
    let pick = |fix: &'static CodeFix| match (fix, case) {
        (CodeFix::Template { text }, _) => Some(text.as_str()),
        (CodeFix::Cases { cases }, Some(label)) => cases
            .iter()
            .find(|known| known.label == label)
            .map(|known| known.text.as_str()),
        _ => None,
    };
    pick(&row.side(version).fix)
        .or_else(|| pick(&row.side(if version >= 2 { 1 } else { 2 }).fix))
        .expect("a rendered filler names a template its row has")
}

/// §FS-errors.5.5.1: `code`'s *Fix* template for `version`, or its case `case`,
/// with each `{slot}` filled from `slots`: the one way a message's remedy is
/// worded, which `remedies/` calls (§AR-config.9.1).
pub(crate) fn fill(
    code: Code,
    version: u32,
    case: Option<&str>,
    slots: &[(&str, &dyn Display)],
) -> String {
    let mut out = String::new();
    let mut rest = template(code, version, case);
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = open + rest[open..].find('}').expect("a rendered slot closes");
        let name = &rest[open + 1..close];
        let (_, value) = slots
            .iter()
            .find(|(slot, _)| *slot == name)
            .expect("a rendered filler passes every slot of its template");
        let _ = write!(out, "{value}");
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// A rendered catalog's rows, in code order.
pub(crate) fn parse(text: &str) -> Result<Vec<Row>, String> {
    serde_json::from_str(text).map_err(|error| error.to_string())
}

/// The embedded catalog's row for `code`; every handle has one.
pub(crate) fn row(code: Code) -> &'static Row {
    let index = Code::ALL
        .binary_search(&code)
        .expect("a handle is in the sorted list");
    &ROWS[index]
}

/// The embedded catalog's rows, in code order.
#[cfg(test)]
pub(crate) fn rows() -> &'static [Row] {
    &ROWS
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §AR-config.9.1: the embedded rows are the handles', one each, in order.
    #[test]
    fn every_handle_has_its_row_in_order() {
        let codes: Vec<&str> = rows().iter().map(|row| row.code.as_str()).collect();
        let handles: Vec<&str> = Code::ALL.iter().map(|code| code.as_str()).collect();
        assert_eq!(codes, handles);
        assert!(rows().iter().all(|row| !row.meaning.is_empty()));
    }
}
