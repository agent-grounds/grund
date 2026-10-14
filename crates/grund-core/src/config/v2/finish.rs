//! What only the whole v2 file can say (§FS-config-v2.reader.4): the checks that
//! read more than one table — a row's keys together, one row's place against
//! another's, a ladder or a resolution entry against the row it names — and the
//! rows' lowering into the schema. Every refusal here is located at the line its
//! value was written at, and the first one in the file is the one reported.

use anyhow::Result;
use std::path::Path;

use super::fields::{FieldDraft, kind_checks};
use super::walk::Reader;
use crate::config::kind::{KindConfig, KindIndex, KindResolution};
use crate::config::project::{Project, Strength};
use crate::config::record::{CODE_SOURCE_KIND, ConfigLocation};
use crate::config::rows::{Extent, Form, Kind, Origin, Place, Row};
use crate::config::v1::{ValueLines, rule_kind_refusal, validate_kind_value_keys};

use super::schema::RowDraft;
use super::{places, sources, sources_model};

/// §FS-config-v2.reader.4: judge the file whole, then lower its rows. `root` is
/// the config root a value home must exist under (§FS-fields.forms).
pub(super) fn finish(mut r: Reader, root: &Path) -> Result<Project> {
    let mut late: Vec<(usize, String)> = Vec::new();
    sources::judge(&r.sources, &mut late);
    for row in &r.rows {
        row_checks(row, &mut late);
        let fields: Vec<&FieldDraft> = r.fields.iter().filter(|f| f.kind == row.name).collect();
        kind_checks(&row.name, &fields, &mut late);
    }
    places::judge(&r.rows, root, &r.project.workspace, &mut late);
    named_rows(&r, &mut late);
    if let Some((line, message)) = late.into_iter().min_by_key(|(line, _)| *line) {
        return r.fail(line, message);
    }
    for row in &r.rows {
        form_prerequisites(&r, row, root)?;
    }
    lower(&mut r);
    sources_model::lower(&mut r);
    Ok(r.project)
}

/// §FS-fields.forms, §FS-fields.content.values: `form = "value"`, `form =
/// "rule"` and a values field ask what v1's `values`, `rules` and
/// `value_chapter` ask of their row, with v1's messages, at the v2 key.
fn form_prerequisites(r: &Reader, row: &RowDraft, root: &Path) -> Result<()> {
    let values_field = r
        .fields
        .iter()
        .find(|field| field.kind == row.name && matches!(field.content, Some((true, _))));
    let form = row.form.map(|(form, _)| form);
    let place = |file: bool| {
        row.places
            .iter()
            .find_map(|place| match (&place.extent, file) {
                (Extent::File(path), true) | (Extent::Folder(path), false) => Some(path.clone()),
                _ => None,
            })
    };
    let k = KindConfig {
        kind: row.name.clone(),
        folder: place(false),
        file: place(true),
        title: None,
        index: KindIndex::Default,
        citable: row.citable,
        scan: row.scan,
        require_grounding: None,
        grounding_level: None,
        values: form == Some(Form::Value),
        value_chapter: values_field.map(|field| field.name.clone()),
        rules: form == Some(Form::Rule),
        format: None,
        resolve: None,
        fetch: row.fetch.as_ref().map(|(fetch, _)| fetch.clone()),
    };
    let form_line = row.form.map_or(row.line, |(_, line)| line);
    if let Some(refusal) = rule_kind_refusal(&k) {
        return r.fail(form_line, refusal);
    }
    let lines = ValueLines {
        header: row.line,
        values: Some(form_line),
        value_chapter: values_field.and_then(|field| field.content.map(|(_, line)| line)),
    };
    let named_sections = r.project.schema.ids.named_sections;
    validate_kind_value_keys(r.path, &k, &lines, root, named_sections)
}

/// §FS-config-v2.schema.places: what one row's keys must say together, each
/// combination v1 refuses on its `[[kinds]]` row (§FS-config.3.4), and what the
/// complement and a row with no place may say
/// (§FS-config-v2.schema.places.complement).
fn row_checks(row: &RowDraft, late: &mut Vec<(usize, String)>) {
    let name = &row.name;
    let complement = name == CODE_SOURCE_KIND;
    if complement && row.citable {
        let line = row.citable_line.unwrap_or(row.line);
        let message = format!("`[schema.kinds.{name}]` is the complement and declares no IDs");
        late.push((line, message));
    }
    if !complement && !row.citable && row.places.is_empty() {
        let message = format!(
            "`[schema.kinds.{name}]` has no place: give it `folders` or `files`, or name the complement `code`"
        );
        late.push((row.line, message));
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
            .any(|place| matches!(place.extent, Extent::Folder(_)));
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
        if complement {
            // §FS-config-v2.schema.places.boundaries.6
            let message = format!(
                "`[schema.kinds.{name}] scan = false` cannot unwalk the complement; narrow its places or use `[schema.sources] exclude`"
            );
            late.push((line, message));
        } else if row.citable {
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
                .any(|place| matches!(place.extent, Extent::File(_)))
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
/// order; the scan as the union of their scanned places and the complement's
/// extent, the config root unless `code` lists places
/// (§FS-config-v2.schema.places.complement, §FS-config-v2.defaults.1); and a fetched kind's resolution, `must`
/// unless `[rules.resolution]` says `warn`.
fn lower(r: &mut Reader) {
    let rows: Vec<Row> = r
        .rows
        .iter()
        .map(|row| lower_row(row, &r.fields, r.path))
        .collect();
    let narrowed = r
        .rows
        .iter()
        .any(|row| row.name == CODE_SOURCE_KIND && !row.places.is_empty());
    let include = rows
        .iter()
        .flat_map(|row| row.places.iter())
        .filter(|place| place.scanned)
        .filter_map(|place| place.path().map(str::to_string))
        .collect();
    let schema = &mut r.project.schema;
    schema.sources.include = narrowed.then_some(include);
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

fn lower_row(draft: &RowDraft, fields: &[FieldDraft], path: &Path) -> Row {
    // §FS-config-v2.schema.places.complement: `code` is the complement place, and
    // its own places, after it, are the complement's extent.
    let complement = (draft.name == CODE_SOURCE_KIND).then_some(Place {
        extent: Extent::Complement,
        scanned: true,
        source: None,
    });
    // Files, then folders, each in written order: the one place order, which
    // `config show` prints and so loads back to these rows (§FS-config-v2.schema.places).
    let (files, folders): (Vec<_>, Vec<_>) = draft
        .places
        .iter()
        .partition(|place| matches!(place.extent, Extent::File(_)));
    let places = files.into_iter().chain(folders).map(|place| Place {
        extent: place.extent.clone(),
        scanned: draft.scan,
        source: Some(ConfigLocation {
            path: path.to_path_buf(),
            line: place.line,
        }),
    });
    Row {
        name: draft.name.clone(),
        places: complement.into_iter().chain(places).collect(),
        kind: draft.citable.then(|| Kind {
            id_format: draft.id_format.as_ref().map(|(format, _)| format.clone()),
            form: draft.form.map_or(Form::Prose, |(form, _)| form),
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
            // §FS-fields.declare.field: in header order (§FS-fields.record).
            fields: fields
                .iter()
                .filter(|field| field.kind == draft.name)
                .map(FieldDraft::lower)
                .collect(),
            closed: draft.closed.map_or(Strength::May, |(closed, _)| closed),
        }),
    }
}
