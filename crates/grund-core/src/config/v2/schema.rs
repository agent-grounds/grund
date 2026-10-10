//! The schema concern's keys (§FS-config-v2.schema): `[schema]`'s grammar keys
//! and `heading_depth`, `[schema.sources]`, and each `[schema.kinds.<NAME>]`
//! row, read into a draft that `finish.rs` judges and lowers once every row is
//! known. The measure tables are `measures.rs`'s.

use anyhow::Result;
use std::path::{Component, Path};

use super::tables::not_yet;
use super::walk::Reader;
use crate::config::kind::KindIndex;
use crate::config::record::ShorthandPolicy;
use crate::config::rows::Extent;
use crate::config::v1::{parse_bool, parse_string, parse_string_list};
use crate::grammar::{id_grammar_literal_slash_error, id_grammar_v2_slash_error};

/// One `[schema.kinds.<NAME>]` row as written, each key at its line, before the
/// checks that need every row (§FS-config-v2.schema.places).
pub(super) struct RowDraft {
    pub(super) name: String,
    /// The row's header line.
    pub(super) line: usize,
    /// Its places, each at the line of the key that wrote it.
    pub(super) places: Vec<(Extent, usize)>,
    pub(super) citable: bool,
    pub(super) scan: bool,
    pub(super) scan_line: Option<usize>,
    pub(super) index: Option<(KindIndex, usize)>,
    pub(super) id_format: Option<(String, usize)>,
    pub(super) fetch: Option<(String, usize)>,
}

impl RowDraft {
    pub(super) fn new(name: String, line: usize) -> Self {
        Self {
            name,
            line,
            places: Vec::new(),
            citable: true,
            scan: true,
            scan_line: None,
            index: None,
            id_format: None,
            fetch: None,
        }
    }
}

/// §FS-config-v2.schema.1: `[schema]`. The grammar keys mean their v1
/// counterparts; `heading_depth` is a strength over the depth check.
pub(super) fn schema_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    match key {
        "marker" => {
            let marker = parse_string(r.path, line, value)?;
            if marker.is_empty() {
                // §FS-config-v2.defaults.1: citations are marked, with no opt-out.
                return r.fail(line, "[schema] marker must not be empty".to_string());
            }
            r.project.schema.citation.marker = marker;
        }
        "shorthand" => {
            let policy = parse_string(r.path, line, value)?;
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
            let parsed = parse_string(r.path, line, value)?;
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
            let strength = parse_string(r.path, line, value)?;
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

/// §FS-config-v2.schema.sources: `[schema.sources]`. Markdown alone is executed
/// (§FS-config-v2.defaults.2); the rest of #456's clauses are refused.
pub(super) fn sources_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    match key {
        "languages" => {
            let languages = parse_string_list(r.path, line, value)?;
            if languages != ["markdown"] {
                return r.fail(line, not_yet("[schema.sources] languages"));
            }
            r.languages = true;
        }
        "exclude" => return r.fail(line, not_yet("[schema.sources] exclude")),
        "respect_gitignore" => {
            r.project.schema.sources.respect_gitignore = parse_bool(r.path, line, value)?;
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
            for path in parse_string_list(r.path, line, value)? {
                if path.is_empty() {
                    return r.fail(line, format!("`{clause}` names an empty path"));
                }
                let extent = if key == "files" {
                    Extent::File(path)
                } else {
                    Extent::Folder(path)
                };
                r.rows[index].places.push((extent, line));
            }
            // §FS-config-v2.rollout: a row with more than one place is #457's.
            if r.rows[index].places.len() > 1 {
                return r.fail(line, not_yet(&clause));
            }
        }
        "citable" => r.rows[index].citable = parse_bool(r.path, line, value)?,
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
                    let name = parse_string(r.path, line, value)?;
                    if let Some(why) = index_name_error(&name) {
                        return r.fail(line, format!("`{clause}` {why}"));
                    }
                    KindIndex::Named(name)
                }
            };
            r.rows[index].index = Some((index_value, line));
        }
        "id_format" => {
            let format = parse_string(r.path, line, value)?;
            if let Some(message) = id_grammar_literal_slash_error(&clause, &format) {
                return r.fail(line, message);
            }
            r.rows[index].id_format = Some((format, line));
        }
        "fetch" => {
            let fetch = parse_string(r.path, line, value)?;
            r.rows[index].fetch = Some((fetch, line));
        }
        // §FS-config-v2.schema.fields: #458's row keys.
        "form" | "closed" | "one_of" => return r.fail(line, not_yet(&clause)),
        _ => return Ok(false),
    }
    Ok(true)
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
