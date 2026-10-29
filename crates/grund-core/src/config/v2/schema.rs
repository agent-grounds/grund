//! The schema concern's keys (§FS-config-v2.schema): `[schema]`'s grammar keys
//! and `heading_depth`, and each `[schema.kinds.<NAME>]` row, read into a draft
//! that `finish.rs` judges and lowers once every row is known. The measure tables
//! are `measures.rs`'s, and `[schema.sources]` is `sources.rs`'s.

use anyhow::Result;
use std::path::{Component, Path};

use super::fields::bodiless;
use super::walk::Reader;
use crate::config::kind::KindIndex;
use crate::config::project::Strength;
use crate::config::record::CODE_SOURCE_KIND;
use crate::config::record::ShorthandPolicy;
use crate::config::rows::{Extent, Form};
use crate::config::v1::parse_bool;
use crate::grammar::{id_grammar_literal_slash_error, id_grammar_v2_slash_error};

/// One `[schema.kinds.<NAME>]` row as written, each key at its line, before the
/// checks that need every row (§FS-config-v2.schema.places).
pub(super) struct RowDraft {
    pub(super) name: String,
    /// The row's header line.
    pub(super) line: usize,
    /// Its places, normalized, each at the line of the key that wrote it.
    pub(super) places: Vec<PlaceDraft>,
    pub(super) citable: bool,
    /// The line `citable` was written at, which the complement's refusal names.
    pub(super) citable_line: Option<usize>,
    pub(super) scan: bool,
    pub(super) scan_line: Option<usize>,
    pub(super) index: Option<(KindIndex, usize)>,
    pub(super) id_format: Option<(String, usize)>,
    pub(super) fetch: Option<(String, usize)>,
    /// §FS-fields.declare.form: `None` is `prose`.
    pub(super) form: Option<(Form, usize)>,
    /// §FS-fields.declare.closed: `None` is `may`.
    pub(super) closed: Option<(Strength, usize)>,
}

/// One place as written (§FS-config-v2.schema.places.boundaries): its extent
/// under the normalized path, the line of its key, and the entry as written,
/// which a refusal quotes.
pub(super) struct PlaceDraft {
    pub(super) extent: Extent,
    pub(super) line: usize,
    pub(super) written: String,
}

impl PlaceDraft {
    /// The normalized path the place names.
    pub(super) fn path(&self) -> &str {
        match &self.extent {
            Extent::Folder(path) | Extent::File(path) => path,
            Extent::Complement => "",
        }
    }

    /// The key that wrote the place.
    pub(super) fn key(&self) -> &'static str {
        match self.extent {
            Extent::File(_) => "files",
            _ => "folders",
        }
    }
}

impl RowDraft {
    pub(super) fn new(name: String, line: usize) -> Self {
        // §FS-config-v2.schema.places.complement: `code` declares no IDs.
        let citable = name != CODE_SOURCE_KIND;
        Self {
            name,
            line,
            places: Vec::new(),
            citable,
            citable_line: None,
            scan: true,
            scan_line: None,
            index: None,
            id_format: None,
            fetch: None,
            form: None,
            closed: None,
        }
    }
}

/// §FS-config-v2.schema.1: `[schema]`. The grammar keys mean their v1
/// counterparts; `heading_depth` is a strength over the depth check.
pub(super) fn schema_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    match key {
        "marker" => {
            let marker = r.string(line, value)?;
            if marker.is_empty() {
                // §FS-config-v2.defaults.1: citations are marked, with no opt-out.
                return r.fail(line, "[schema] marker must not be empty".to_string());
            }
            r.project.schema.citation.marker = marker;
        }
        "shorthand" => {
            let policy = r.string(line, value)?;
            r.project.schema.citation.shorthand = match policy.as_str() {
                "canonical" => ShorthandPolicy::Canonical,
                "accepted" => ShorthandPolicy::Accepted,
                _ => {
                    return r.fail(
                        line,
                        format!(
                            "unknown [schema] shorthand `{policy}` (expected canonical or accepted)"
                        ),
                    );
                }
            };
        }
        "id_format" | "slug_pattern" | "number_pattern" | "section_separator" => {
            let parsed = r.string(line, value)?;
            if let Some(message) = id_grammar_v2_slash_error(key, &parsed) {
                return r.fail(line, message);
            }
            let ids = &mut r.project.schema.ids;
            match key {
                "id_format" => ids.format = parsed,
                "slug_pattern" => ids.slug_pattern = parsed,
                "number_pattern" => ids.number_pattern = parsed,
                _ => ids.section_separator = parsed,
            }
        }
        // §FS-config-v2.schema.1: `may` recognizes sections unchecked, v1's `loose`.
        "heading_depth" => {
            let strength = r.string(line, value)?;
            let mode = match strength.as_str() {
                "must" => "strict",
                "warn" => "warn",
                "should" => "suggest",
                "may" => "loose",
                _ => {
                    return r.fail(
                        line,
                        format!(
                            "unknown [schema] heading_depth `{strength}` (expected must, warn, should, or may)"
                        ),
                    );
                }
            };
            r.project.schema.ids.section_heading_levels = mode.to_string();
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// §FS-config-v2.schema.places: one key of the row at `index`.
pub(super) fn row_key(
    r: &mut Reader,
    index: usize,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let clause = format!("[schema.kinds.{}] {key}", r.rows[index].name);
    match key {
        "files" | "folders" => {
            for written in r.string_list(line, value)? {
                if written.is_empty() {
                    return r.fail(line, format!("`{clause}` names an empty path"));
                }
                // §FS-config-v2.schema.places.boundaries.1: normalized, and inside the root.
                let Some(path) = normalize_place(&written) else {
                    let message =
                        format!("place `{written}` in `{clause}` is outside the config root");
                    return r.fail(line, message);
                };
                if path.is_empty() {
                    let message = format!(
                        "place `{written}` in `{clause}` names the config root, \
                         which is the complement's extent"
                    );
                    return r.fail(line, message);
                }
                let extent = if key == "files" {
                    Extent::File(path)
                } else {
                    Extent::Folder(path)
                };
                let place = PlaceDraft {
                    extent,
                    line,
                    written,
                };
                r.rows[index].places.push(place);
            }
        }
        "citable" => {
            r.rows[index].citable = parse_bool(r.path, line, value)?;
            r.rows[index].citable_line = Some(line);
        }
        "scan" => {
            r.rows[index].scan = parse_bool(r.path, line, value)?;
            r.rows[index].scan_line = Some(line);
        }
        "index" => {
            let index_value = match value {
                "false" => KindIndex::Disabled,
                "true" => {
                    let message = format!("`{clause}` takes a file name or `false`");
                    return r.fail(line, message);
                }
                _ => {
                    let name = r.string(line, value)?;
                    if let Some(why) = index_name_error(&name) {
                        return r.fail(line, format!("`{clause}` {why}"));
                    }
                    KindIndex::Named(name)
                }
            };
            r.rows[index].index = Some((index_value, line));
        }
        "id_format" => {
            let format = r.string(line, value)?;
            if let Some(message) = id_grammar_literal_slash_error(&clause, &format) {
                return r.fail(line, message);
            }
            r.rows[index].id_format = Some((format, line));
        }
        "fetch" => {
            let fetch = r.string(line, value)?;
            r.rows[index].fetch = Some((fetch, line));
        }
        // §FS-fields.declare.form
        "form" => {
            let form = r.string(line, value)?;
            let parsed = match form.as_str() {
                "prose" => Form::Prose,
                "value" => Form::Value,
                "rule" => Form::Rule,
                _ => {
                    let message = format!(
                        "unknown [schema.kinds.{}] form `{form}` (expected prose, value, or rule)",
                        r.rows[index].name
                    );
                    return r.fail(line, message);
                }
            };
            r.rows[index].form = Some((parsed, line));
        }
        // §FS-fields.declare.closed
        "closed" => {
            let closed = r.string(line, value)?;
            let Some(strength) = Strength::parse(&closed) else {
                let message = format!(
                    "unknown [schema.kinds.{}] closed `{closed}` (expected must, warn, should, or may)",
                    r.rows[index].name
                );
                return r.fail(line, message);
            };
            r.rows[index].closed = Some((strength, line));
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// §FS-fields.forms: once a row's keys are all read, a kind with no authored
/// body cannot be closed, refused at `closed`.
pub(super) fn close_row(r: &Reader, index: usize) -> Result<()> {
    let row = &r.rows[index];
    if let (Some((_, line)), Some(why)) = (row.closed, bodiless(row)) {
        return r.fail(line, format!("{why}, so it cannot be closed"));
    }
    Ok(())
}

/// §FS-config-v2.schema.places.boundaries.1: a place as one lexical path from
/// the config root, `./` and a trailing `/` folded away; `None` for a place that
/// is absolute or climbs out of the root with `..`.
pub(super) fn normalize_place(written: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for component in Path::new(written).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::Normal(part) => parts.push(part.to_str()?),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(parts.join("/"))
}

/// Why an `index` value cannot name the folder's index: it must be a Markdown
/// file inside the folder, as v1's is (§FS-config.3.4).
fn index_name_error(name: &str) -> Option<&'static str> {
    let inside = !name.is_empty()
        && !name.contains('\\')
        && Path::new(name)
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if !inside {
        return Some("must name a file inside the folder");
    }
    if Path::new(name).extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Some("must name a Markdown file");
    }
    None
}
