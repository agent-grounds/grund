//! A v2 kind's fields (§FS-fields.declare.field): each
//! `[schema.kinds.<KIND>.fields.<name>]` read into a draft at its keys' lines,
//! the refusals one table can make as soon as its key is met
//! (§FS-fields.content.one-of, §FS-fields.identity.match), and the ones that
//! need every field of the kind once the file is read (§FS-fields.order.refusals).

use anyhow::Result;
use std::collections::BTreeSet;

use super::front_matter::KeyDraft;
use super::walk::Reader;
use crate::config::fields::{FRONT_MATTER, Field, FieldContent, FrontMatterShape, Placement};
use crate::config::project::Strength;
use crate::config::rows::Form;
use crate::config::v1::{parse_string, parse_string_list};
use crate::grammar::named_section_component;

/// One field table as written, each key at its line.
pub(super) struct FieldDraft {
    pub(super) kind: String,
    pub(super) name: String,
    pub(super) presence: Option<Strength>,
    pub(super) title: Option<(String, usize)>,
    pub(super) position: Option<(Placement, usize)>,
    pub(super) after: Option<(String, usize)>,
    pub(super) content: Option<(bool, usize)>,
    pub(super) one_of: Option<(Vec<String>, usize)>,
    /// The front-matter field's `closed` (§FS-fields.front-matter.config).
    pub(super) closed: Option<Strength>,
    /// The front-matter field's keys, in header order.
    pub(super) keys: Vec<KeyDraft>,
}

impl FieldDraft {
    fn label(&self) -> String {
        format!("{}.{}", self.kind, self.name)
    }

    /// §FS-fields.declare.field: the field as the record holds it.
    pub(super) fn lower(&self) -> Field {
        let placement = match (&self.position, &self.after) {
            (Some((position, _)), _) => Some(position.clone()),
            (None, Some((after, _))) => Some(Placement::After(after.clone())),
            (None, None) => None,
        };
        let content = match (&self.one_of, self.content) {
            // §FS-fields.front-matter.record: the slot carries its closed and its keys.
            _ if self.name == FRONT_MATTER => FieldContent::FrontMatter(FrontMatterShape {
                closed: self.closed.unwrap_or(Strength::May),
                keys: self.keys.iter().map(KeyDraft::lower).collect(),
            }),
            (Some((values, _)), _) => FieldContent::OneOf(values.clone()),
            (None, Some((true, _))) => FieldContent::Values,
            (None, _) => FieldContent::Prose,
        };
        Field {
            name: self.name.clone(),
            title: self.title.as_ref().map(|(title, _)| title.clone()),
            presence: self.presence.unwrap_or(Strength::May),
            placement,
            content,
        }
    }
}

/// §FS-config-v2.reader.2, §FS-fields.declare.field: open the field table
/// `[schema.kinds.<kind>.fields.<name>]` at `line`, under a row already written,
/// whose declarations have a body to hold it (§FS-fields.forms).
pub(super) fn open_field(r: &mut Reader, kind: &str, name: &str, line: usize) -> Result<usize> {
    let header = format!("[schema.kinds.{kind}.fields.{name}]");
    if !named_section_component(name) {
        let message =
            format!("`{header}`: a field name is a section name matching `[a-z][a-z0-9-]*`");
        return r.fail(line, message);
    }
    let Some(row) = r.rows.iter().find(|row| row.name == kind) else {
        let message = format!("`{header}` names no row: write [schema.kinds.{kind}] above it");
        return r.fail(line, message);
    };
    if let Some(why) = bodiless(row) {
        return r.fail(line, format!("{why}, so it takes no fields"));
    }
    r.fields.push(FieldDraft {
        kind: kind.to_string(),
        name: name.to_string(),
        presence: None,
        title: None,
        position: None,
        after: None,
        content: None,
        one_of: None,
        closed: None,
        keys: Vec::new(),
    });
    Ok(r.fields.len() - 1)
}

/// §FS-fields.forms: why a row's declarations have no authored body to hold a
/// field, if they have none — the start of the refusal of a field or `closed`.
pub(super) fn bodiless(row: &super::schema::RowDraft) -> Option<String> {
    let name = &row.name;
    if !row.citable {
        return Some(format!("{name} is not citable: it declares nothing"));
    }
    if matches!(row.form, Some((Form::Value, _))) {
        return Some(format!(
            "{name} is form = \"value\": its children are numbered components"
        ));
    }
    if row.fetch.is_some() {
        return Some(format!(
            "{name} is fetched: its declarations are a snapshot"
        ));
    }
    None
}

/// Whether a configured display name would never equal a parsed one
/// (§FS-fields.identity.match).
fn untrimmed(value: &str) -> bool {
    value.is_empty() || value.trim() != value
}

/// §FS-fields.declare.field: one key of the field table at `index`.
pub(super) fn field_key(
    r: &mut Reader,
    index: usize,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let field = &r.fields[index];
    let (label, clause) = (
        field.label(),
        format!("[schema.kinds.{}.fields.{}]", field.kind, field.name),
    );
    // §FS-fields.content.one-of: a scalar's title is its value, and its content its title.
    let scalar_title =
        format!("field {label} sets one_of and title; a scalar's title is its value");
    let scalar_content =
        format!("field {label} sets one_of and content; a scalar's content is its title");
    let both = format!("field {label} sets both position and after; a field takes one placement");
    match key {
        "title" => {
            let title = parse_string(r.path, line, value)?;
            if field.one_of.is_some() {
                return r.fail(line, scalar_title);
            }
            if untrimmed(&title) {
                let message = format!(
                    "field {label} has title \"{title}\", which is empty or has surrounding whitespace"
                );
                return r.fail(line, message);
            }
            r.fields[index].title = Some((title, line));
        }
        "presence" => {
            let presence = parse_string(r.path, line, value)?;
            let Some(strength) = Strength::parse(&presence) else {
                let message = format!(
                    "unknown {clause} presence `{presence}` (expected must, warn, should, or may)"
                );
                return r.fail(line, message);
            };
            r.fields[index].presence = Some(strength);
        }
        "position" => {
            let position = parse_string(r.path, line, value)?;
            let placement = match position.as_str() {
                "first" => Placement::First,
                "last" => Placement::Last,
                _ => {
                    let message =
                        format!("unknown {clause} position `{position}` (expected first or last)");
                    return r.fail(line, message);
                }
            };
            // §FS-fields.order.refusals: one placement, refused at the later key.
            if field.after.is_some() {
                return r.fail(line, both);
            }
            r.fields[index].position = Some((placement, line));
        }
        "after" => {
            let after = parse_string(r.path, line, value)?;
            if field.position.is_some() {
                return r.fail(line, both);
            }
            r.fields[index].after = Some((after, line));
        }
        "content" => {
            let content = parse_string(r.path, line, value)?;
            let values = match content.as_str() {
                "prose" => false,
                "values" => true,
                _ => {
                    let message =
                        format!("unknown {clause} content `{content}` (expected prose or values)");
                    return r.fail(line, message);
                }
            };
            if field.one_of.is_some() {
                return r.fail(line, scalar_content);
            }
            r.fields[index].content = Some((values, line));
        }
        "one_of" => {
            let values = parse_string_list(r.path, line, value)?;
            if field.title.is_some() {
                return r.fail(line, scalar_title);
            }
            if field.content.is_some() {
                return r.fail(line, scalar_content);
            }
            if let Some(message) = one_of_error(&format!("field {label}"), &values) {
                return r.fail(line, message);
            }
            r.fields[index].one_of = Some((values, line));
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// §FS-fields.content.one-of, §FS-fields.identity.match: what a `one_of` list
/// cannot be, `label` naming whose it is: `field ADR.status` or, for a
/// front-matter key, `key ADR.front-matter.status` (§FS-fields.front-matter.config).
pub(super) fn one_of_error(label: &str, values: &[String]) -> Option<String> {
    if values.is_empty() {
        return Some(format!(
            "{label} has an empty one_of; a scalar needs at least one value"
        ));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if untrimmed(value) {
            return Some(format!(
                "{label} has one_of entry \"{value}\", which is empty or has surrounding whitespace"
            ));
        }
        if !seen.insert(value.as_str()) {
            return Some(format!("{label} lists \"{value}\" twice in one_of"));
        }
    }
    None
}

/// §FS-fields.order.refusals, §FS-fields.content.values: what one kind's
/// fields must say together, each kept at the line it is located at.
pub(super) fn kind_checks(kind: &str, fields: &[&FieldDraft], late: &mut Vec<(usize, String)>) {
    let names: Vec<&str> = fields.iter().map(|field| field.name.as_str()).collect();
    let find = |name: &str| fields.iter().find(|field| field.name == name);
    let mut ends: Vec<(&Placement, &str)> = Vec::new();
    let mut values: Option<&str> = None;
    for field in fields {
        if let Some((position, line)) = &field.position {
            if let Some((_, first)) = ends.iter().find(|(end, _)| *end == position) {
                let end = if *position == Placement::First {
                    "first"
                } else {
                    "last"
                };
                let message = format!(
                    "fields {kind}.{first} and {} are both {end}; only one field can be",
                    field.label()
                );
                late.push((*line, message));
            }
            ends.push((position, &field.name));
        }
        if let Some((true, line)) = field.content {
            if values.is_some() {
                // §FS-fields.content.values: v1's one-chapter refusal.
                late.push((line, "[[kinds]] sets `value_chapter` twice".to_string()));
            }
            values.get_or_insert(&field.name);
        }
    }
    let mut chained: Vec<(&str, &str)> = Vec::new();
    for field in fields {
        let Some((target, line)) = &field.after else {
            continue;
        };
        let refusal = if *target == field.name {
            Some(format!("{} comes after itself", field.label()))
        } else if target == FRONT_MATTER && find(target).is_some() {
            // §FS-fields.front-matter.config: the block is not among the chapters.
            Some(format!(
                "{} comes after {FRONT_MATTER}, which is the file's front matter and has no place among the chapters",
                field.label()
            ))
        } else if let Some(predecessor) = find(target) {
            if matches!(predecessor.position, Some((Placement::Last, _))) {
                Some(format!(
                    "{kind}.{target} is last; nothing can come after it"
                ))
            } else if let Some((other, _)) = chained.iter().find(|(_, to)| to == target) {
                Some(format!(
                    "fields {kind}.{other} and {} both come directly after {kind}.{target}; chain one after the other",
                    field.label()
                ))
            } else {
                chained.push((&field.name, target));
                None
            }
        } else {
            Some(format!(
                "after = \"{target}\" names no field of {kind}; its fields: {}",
                listed(&names)
            ))
        };
        if let Some(message) = refusal {
            late.push((*line, message));
        }
    }
    cycles(fields, &chained, late);
}

/// A list of field names as a message spells it, or `none`.
pub(crate) fn listed(names: &[&str]) -> String {
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(", ")
    }
}

/// §FS-fields.order.refusals: each cycle of `after` links once, its fields in
/// header order, located at the latest one's `after`. `chained` holds only
/// links that name another field and no field's successor twice, so each field
/// has at most one link out and one in.
fn cycles(fields: &[&FieldDraft], chained: &[(&str, &str)], late: &mut Vec<(usize, String)>) {
    let position = |name: &str| fields.iter().position(|field| field.name == name);
    let mut reported: BTreeSet<usize> = BTreeSet::new();
    for (start, _) in chained {
        let mut walk: Vec<&str> = vec![start];
        let mut at = *start;
        while let Some((_, next)) = chained.iter().find(|(from, _)| *from == at) {
            if *next == *start {
                let mut members: Vec<usize> =
                    walk.iter().filter_map(|name| position(name)).collect();
                members.sort_unstable();
                if reported.insert(members[0]) {
                    let latest = fields[members[members.len() - 1]];
                    let labels: Vec<String> =
                        members.iter().map(|index| fields[*index].label()).collect();
                    let message = match labels.as_slice() {
                        [a, b] => format!(
                            "{a} and {b} each come after the other; one of them needs position or no after"
                        ),
                        [init @ .., last] => format!(
                            "{} and {last} come after one another in a cycle; one of them needs position or no after",
                            init.join(", ")
                        ),
                        [] => unreachable!("a cycle has members"),
                    };
                    let line = latest.after.as_ref().map_or(0, |(_, line)| *line);
                    late.push((line, message));
                }
                break;
            }
            if walk.contains(next) {
                break;
            }
            walk.push(next);
            at = next;
        }
    }
}
