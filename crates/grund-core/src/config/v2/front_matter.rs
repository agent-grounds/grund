//! The front-matter field's tables (§FS-fields.front-matter.config):
//! `[schema.kinds.<KIND>.fields.front-matter]`, whose keys are `presence` and
//! `closed`, and each `[…fields.front-matter.keys.<key>]` under it, whose keys
//! are `presence` and `one_of`. The field's draft is an ordinary
//! [`FieldDraft`], so the row checks and the lowering read it as any field.

use anyhow::Result;

use super::fields::{FieldDraft, one_of_error};
use super::walk::Reader;
use crate::config::fields::{FRONT_MATTER, FrontMatterKey};
use crate::config::project::Strength;
use crate::grammar::named_section_component;

/// One key table as written (§FS-fields.front-matter.config).
pub(super) struct KeyDraft {
    pub(super) name: String,
    pub(super) presence: Option<Strength>,
    pub(super) one_of: Option<Vec<String>>,
}

impl KeyDraft {
    /// §FS-fields.front-matter.record: the key as the record holds it.
    pub(super) fn lower(&self) -> FrontMatterKey {
        FrontMatterKey {
            name: self.name.clone(),
            presence: self.presence.unwrap_or(Strength::May),
            one_of: self.one_of.clone(),
        }
    }
}

/// §FS-fields.front-matter.config: open `[schema.kinds.<kind>.fields.front-matter.keys.<key>]`
/// at `line`, a section name under its field's table already written; the
/// field's draft index and the key's index in it.
pub(super) fn open_key_table(
    r: &mut Reader,
    kind: &str,
    key: &str,
    line: usize,
) -> Result<(usize, usize)> {
    let header = format!("[schema.kinds.{kind}.fields.{FRONT_MATTER}.keys.{key}]");
    if !named_section_component(key) {
        let message =
            format!("`{header}`: a key name is a section name matching `[a-z][a-z0-9-]*`");
        return r.fail(line, message);
    }
    let field = r
        .fields
        .iter()
        .position(|field| field.kind == kind && field.name == FRONT_MATTER);
    let Some(field) = field else {
        let message = format!(
            "`{header}` names no field: write [schema.kinds.{kind}.fields.{FRONT_MATTER}] above it"
        );
        return r.fail(line, message);
    };
    let keys = &mut r.fields[field].keys;
    keys.push(KeyDraft {
        name: key.to_string(),
        presence: None,
        one_of: None,
    });
    Ok((field, keys.len() - 1))
}

/// §FS-fields.front-matter.config: one key of the field's own table.
pub(super) fn front_matter_key(
    r: &mut Reader,
    index: usize,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let field = &r.fields[index];
    let clause = format!("[schema.kinds.{}.fields.{}]", field.kind, field.name);
    let strength = strength(r, &clause, key, value, line)?;
    match key {
        "presence" => r.fields[index].presence = Some(strength),
        "closed" => r.fields[index].closed = Some(strength),
        _ => return Ok(false),
    }
    Ok(true)
}

/// §FS-fields.front-matter.config: one key of a key's table, its `one_of`
/// refused as a field's is, in the key's name.
pub(super) fn key_key(
    r: &mut Reader,
    field: usize,
    index: usize,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let draft: &FieldDraft = &r.fields[field];
    let name = &draft.keys[index].name;
    let clause = format!(
        "[schema.kinds.{}.fields.{FRONT_MATTER}.keys.{name}]",
        draft.kind
    );
    let label = format!("key {}.{FRONT_MATTER}.{name}", draft.kind);
    match key {
        "presence" => {
            let presence = strength(r, &clause, key, value, line)?;
            r.fields[field].keys[index].presence = Some(presence);
        }
        "one_of" => {
            let values = r.string_list(line, value)?;
            if let Some(message) = one_of_error(&label, &values) {
                return r.fail(line, message);
            }
            r.fields[field].keys[index].one_of = Some(values);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// A strength-valued key of `clause`, refused outside its four words as a
/// field's `presence` is (§FS-fields.front-matter.config).
fn strength(r: &mut Reader, clause: &str, key: &str, value: &str, line: usize) -> Result<Strength> {
    let written = r.string(line, value)?;
    match Strength::parse(&written) {
        Some(strength) => Ok(strength),
        None => r.fail(
            line,
            format!("unknown {clause} {key} `{written}` (expected must, warn, should, or may)"),
        ),
    }
}
