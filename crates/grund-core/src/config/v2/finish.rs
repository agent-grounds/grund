//! What only the whole v2 file can say (§FS-config-v2.reader.4): the checks that
//! read more than one table — a row's keys together, one row's place against
//! another's, a ladder or a resolution entry against the row it names — and the
//! rows' lowering into the schema. Every refusal here is located at the line its
//! value was written at, and the first one in the file is the one reported.

use anyhow::Result;
use std::path::{Component, Path};

use super::tables::not_yet;
use super::walk::Reader;
use crate::config::kind::{KindIndex, KindResolution};
use crate::config::project::Project;
use crate::config::record::CODE_SOURCE_KIND;
use crate::config::rows::{Extent, Form, Kind, Origin, Place, Row};

use super::schema::RowDraft;

/// §FS-config-v2.reader.4: judge the file whole, then lower its rows.
pub(super) fn finish(mut r: Reader) -> Result<Project> {
    let mut late: Vec<(usize, String)> = Vec::new();
    // §FS-config-v2.defaults.2: the epoch's language set is not executed yet.
    if !r.languages {
        late.push((
            r.version_line,
            "v2 needs an explicit [schema.sources] languages: this grund does not support the v2 default language set yet".to_string(),
        ));
    }
    for row in &r.rows {
        row_checks(row, &mut late);
    }
    nested_places(&r.rows, &mut late);
    named_rows(&r, &mut late);
    if let Some((line, message)) = late.into_iter().min_by_key(|(line, _)| *line) {
        return r.fail(line, message);
    }
    lower(&mut r);
    Ok(r.project)
}

/// §FS-config-v2.schema.places: what one row's keys must say together, each
/// combination v1 refuses on its `[[kinds]]` row (§FS-config.3.4), and the
/// rows #457 owns (§FS-config-v2.rollout).
fn row_checks(row: &RowDraft, late: &mut Vec<(usize, String)>) {
    let name = &row.name;
    // §FS-config-v2.rollout: the complement's identity is #457's, whether the
    // reserved name or a non-citable row with no place spells it.
    if name == CODE_SOURCE_KIND || (!row.citable && row.places.is_empty()) {
        late.push((row.line, not_yet(&format!("[schema.kinds.{name}]"))));
    }
    if let Some((_, line)) = &row.id_format
        && !row.citable
    {
        let message = format!(
            "`[schema.kinds.{name}] id_format` needs a citable row (`citable = false` declares no IDs)"
        );
        late.push((*line, message));
    }
    if let Some((_, line)) = &row.fetch {
        if !row.citable {
            let message = format!(
                "`[schema.kinds.{name}] fetch` needs a citable row (`citable = false` declares no IDs)"
            );
            late.push((*line, message));
        } else if row.places.len() != 1 {
            let message =
                format!("`[schema.kinds.{name}] fetch` needs one place to hold its snapshots");
            late.push((*line, message));
        }
    }
    if let Some((_, line)) = &row.index {
        let folder = row
            .places
            .iter()
            .any(|(extent, _)| matches!(extent, Extent::Folder(_)));
        if !row.citable || !folder {
            let message = format!(
                "`[schema.kinds.{name}] index` needs a citable row with a folder place (an index lists a folder's declarations)"
            );
            late.push((*line, message));
        }
    }
    if let Some(line) = row.scan_line
        && !row.scan
    {
        if row.citable {
            let message = format!(
                "`[schema.kinds.{name}] scan = false` needs `citable = false` (an unwalked row's declarations would be invisible)"
            );
            late.push((line, message));
        } else if row.places.is_empty() {
            let message =
                format!("`[schema.kinds.{name}] scan = false` needs a place to leave unwalked");
            late.push((line, message));
        }
    }
}

/// §FS-config-v2.rollout: a place at or inside another row's place is #457's,
/// refused at the inner place's key.
fn nested_places(rows: &[RowDraft], late: &mut Vec<(usize, String)>) {
    for (index, inner) in rows.iter().enumerate() {
        for (extent, line) in &inner.places {
            let inside = rows.iter().enumerate().any(|(other, outer)| {
                other != index
                    && outer
                        .places
                        .iter()
                        .any(|(around, _)| within(path_of(extent), path_of(around)))
            });
            if inside {
                late.push((
                    *line,
                    not_yet(&format!("[schema.kinds.{}] {}", inner.name, key_of(extent))),
                ));
            }
        }
    }
}

fn path_of(extent: &Extent) -> &str {
    match extent {
        Extent::Folder(path) | Extent::File(path) => path,
        Extent::Complement => "",
    }
}

fn key_of(extent: &Extent) -> &'static str {
    match extent {
        Extent::File(_) => "files",
        _ => "folders",
    }
}

/// Whether `inner` is `outer` or under it, component by component.
fn within(inner: &str, outer: &str) -> bool {
    let normal = |path: &str| -> Vec<String> {
        Path::new(path)
            .components()
            .filter(|component| !matches!(component, Component::CurDir))
            .map(|component| component.as_os_str().to_string_lossy().into_owned())
            .collect()
    };
    let (inner, outer) = (normal(inner), normal(outer));
    !outer.is_empty() && inner.starts_with(&outer)
}

/// The tables that name a row: a row ladder (§FS-config-v2.rules.grounding), a
/// resolution entry (§FS-config-v2.rules.resolution), and a presentation title.
fn named_rows(r: &Reader, late: &mut Vec<(usize, String)>) {
    let row = |name: &str| r.rows.iter().find(|row| row.name == name);
    for (place, line) in &r.row_ladders {
        let table = format!("`[rules.citations.{place}.grounding]`");
        let Some(row) = row(place) else {
            // The undeclared homeless kind takes a ladder by its name.
            if place != CODE_SOURCE_KIND {
                late.push((*line, format!("{table} names an unknown kind `{place}`")));
            }
            continue;
        };
        if !row.scan {
            let message = format!(
                "{table} names an unwalked row (its place is `scan = false`, so the ladder could never fire)"
            );
            late.push((*line, message));
        } else if row.citable
            && row
                .places
                .iter()
                .any(|(extent, _)| matches!(extent, Extent::File(_)))
        {
            let message = format!(
                "{table} names a citable single-file row (one declaration document, which the grounding rule leaves alone)"
            );
            late.push((*line, message));
        }
    }
    for (kind, _, line) in &r.resolution {
        if row(kind).is_none_or(|row| row.fetch.is_none()) {
            let message =
                format!("`[rules.resolution] {kind}` needs `fetch` on [schema.kinds.{kind}]");
            late.push((*line, message));
        }
    }
    for (kind, line) in &r.presentation_kinds {
        if row(kind).is_none() {
            let message = format!("`[presentation.kinds.{kind}]` names an unknown kind `{kind}`");
            late.push((*line, message));
        }
    }
}

/// §FS-config-v2.schema.places: the drafts as the schema's rows, in authored
/// order; the scan as the union of their scanned places, or the config root
/// when no row has one (§FS-config-v2.defaults.1); and a fetched kind's
/// resolution, `must` unless `[rules.resolution]` says `warn`.
fn lower(r: &mut Reader) {
    let rows: Vec<Row> = r.rows.iter().map(lower_row).collect();
    let any_place = rows.iter().any(|row| !row.places.is_empty());
    let include = rows
        .iter()
        .flat_map(|row| row.places.iter())
        .filter(|place| place.scanned)
        .map(|place| path_of(&place.extent).to_string())
        .collect();
    let schema = &mut r.project.schema;
    schema.sources.include = any_place.then_some(include);
    schema.rows = rows;
    let resolution = &mut r.project.rules.resolution;
    for row in &r.rows {
        if row.fetch.is_some() {
            resolution.insert(row.name.clone(), KindResolution::Must);
        }
    }
    for (kind, strength, _) in &r.resolution {
        resolution.insert(kind.clone(), *strength);
    }
}

fn lower_row(draft: &RowDraft) -> Row {
    Row {
        name: draft.name.clone(),
        places: draft
            .places
            .iter()
            .map(|(extent, _)| Place {
                extent: extent.clone(),
                scanned: draft.scan,
            })
            .collect(),
        kind: draft.citable.then(|| Kind {
            id_format: draft.id_format.as_ref().map(|(format, _)| format.clone()),
            form: Form::Prose,
            index: draft
                .index
                .as_ref()
                .map_or(KindIndex::Default, |(index, _)| index.clone()),
            origin: match &draft.fetch {
                Some((fetch, _)) => Origin::External {
                    fetch: fetch.clone(),
                },
                None => Origin::Local,
            },
        }),
    }
}
