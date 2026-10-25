//! §FS-config-migrate.2.2: what the report says about each authored key — where
//! it went (`mapped:`), or why it was dropped or changed (§FS-config-migrate.3.2) —
//! and each v1 default the candidate writes out (§FS-config-migrate.3.3).

use super::candidate::may_default;
use super::keys::{Keys, unquote};
use super::places::Include;
use super::record::{MappedKey, MigrateNotice};
use crate::config::citations::CitationLevel;
use crate::config::project::Project;
use crate::config::v1::LineKind;

const ALWAYS: &str = "nothing, version 2 always means it";

/// `A`, `A and B`, `A, B and C`.
pub(super) fn joined(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// §FS-config-migrate.2.2: both reports name the same source definitions;
/// comment guidance spells their full table addresses for manual placement.
fn definitions(v2: &Project, full_addresses: bool) -> String {
    let names: Vec<String> = v2
        .schema
        .sources
        .model
        .iter()
        .flat_map(|model| model.definitions.iter())
        .map(|def| {
            if full_addresses {
                format!("[schema.sources.definitions.{}]", def.name)
            } else {
                def.name.clone()
            }
        })
        .collect();
    let word = if names.len() == 1 {
        "definition"
    } else {
        "definitions"
    };
    format!("{word} {}", joined(&names))
}

/// §FS-config-migrate.mapping `[scan] include`.
fn include(plan: &Include) -> String {
    let mut parts = Vec::new();
    if !plan.folders.is_empty() {
        parts.push(format!("folders, {}", joined(&plan.folders)));
    }
    if !plan.files.is_empty() {
        parts.push(format!("files, {}", joined(&plan.files)));
    }
    let duplicates: Vec<String> = plan
        .duplicates
        .iter()
        .map(|(entry, row)| format!("{entry} is a place of [schema.kinds.{row}] already"))
        .collect();
    let mut out = if parts.is_empty() {
        "nothing".to_string()
    } else {
        format!("[schema.kinds.code] {}", parts.join("; "))
    };
    for duplicate in duplicates {
        out.push_str(if out == "nothing" { ", " } else { "; " });
        out.push_str(&duplicate);
    }
    out
}

/// `[scan] docstring_python`'s spelling: the definition `.py` maps to, which
/// reads docstrings only when the key was on, or nothing when `.py` is not read.
fn python_docstrings(v2: &Project, full_addresses: bool) -> String {
    let Some(model) = &v2.schema.sources.model else {
        return ALWAYS.into();
    };
    let definition = model
        .extensions
        .iter()
        .find(|(suffix, _)| suffix == "py")
        .and_then(|(_, name)| model.definitions.iter().find(|def| def.name == *name));
    match definition {
        Some(def) => {
            let name = if full_addresses {
                format!("[schema.sources.definitions.{}]", def.name)
            } else {
                def.name.clone()
            };
            let state = if def.docstrings { "" } else { " (off)" };
            format!("docstrings of definition {name}{state}")
        }
        None if full_addresses => ALWAYS.into(),
        None => "nothing, [scan] extensions does not read `py`".into(),
    }
}

/// The v2 spelling of one authored v1 key (§FS-config-migrate.mapping).
/// §FS-config-migrate.2.2: shared mapping facts for mapped lines and comments.
fn spelling(
    section: &str,
    key: &str,
    row: &str,
    v2: &Project,
    plan: &Include,
    full_addresses: bool,
) -> String {
    let notes = &v2.schema.notes;
    let guidance = if notes.warn_on_suggested {
        "warn"
    } else {
        "may"
    };
    let grounding = &v2.rules.grounding;
    match (section, key) {
        ("", "grund_config_version") => "grund_config_version = 2".into(),
        ("", "project_name") => "project_name".into(),
        ("", "project_description") => "[presentation] description".into(),
        ("", "kinds") => "no [schema.kinds] row".into(),
        ("reference", "conversation" | "trigger") => format!("[presentation] {key}"),
        ("reference", "marker" | "shorthand") => format!("[schema] {key}"),
        ("reference", "grounding_level" | "require_grounding") if grounding.ladder.is_some() => {
            "[rules.citations.grounding] must".into()
        }
        ("reference", "inline_note_layout" | "inline_note_layout_check") => {
            if notes.layout == "any" && notes.layout_check == "off" {
                ALWAYS.into()
            } else {
                "[schema.notes.layout]".into()
            }
        }
        ("reference", "inline_note_max_columns") => "[schema.notes.columns] must".into(),
        ("reference", "inline_note_max_lines") => "[schema.notes.lines] must".into(),
        ("reference", "inline_note_suggested_lines" | "warn_on_suggested") => {
            format!("[schema.notes.lines] {guidance}")
        }
        ("reference", "inline_style") => "[schema.notes.text] must".into(),
        ("reference", "lead_size_warning") => "[schema.leads.words] warn".into(),
        ("id", "format") => "[schema] id_format".into(),
        ("id", "section_heading_levels") => "[schema] heading_depth".into(),
        ("id", "number_pattern" | "section_separator" | "slug_pattern") => {
            format!("[schema] {key}")
        }
        ("kinds", "kind") => format!("[schema.kinds.{row}]"),
        ("kinds", "file") => format!("[schema.kinds.{row}] files"),
        ("kinds", "folder") => format!("[schema.kinds.{row}] folders"),
        ("kinds", "format") => format!("[schema.kinds.{row}] id_format"),
        ("kinds", "rules" | "values") => format!("[schema.kinds.{row}] form"),
        ("kinds", "citable" | "scan" | "index" | "fetch") => format!("[schema.kinds.{row}] {key}"),
        ("kinds", "title") => format!("[presentation.kinds.{row}] title"),
        ("kinds", "resolve") if v2.rules.resolution.contains_key(row) => {
            format!("[rules.resolution] {row}")
        }
        ("kinds", "grounding_level" | "require_grounding") => match grounding.kinds.get(row) {
            Some(_) => format!("[rules.citations.{row}.grounding]"),
            None if grounding.ladder.is_some() => {
                "[rules.citations.grounding], which the row takes".into()
            }
            None => ALWAYS.into(),
        },
        ("scan", "include") => include(plan),
        ("scan", "extensions") => format!(
            "[schema.sources] languages and extensions, {}",
            definitions(v2, full_addresses)
        ),
        ("scan", "comment_prefixes") => {
            format!("comment_prefixes of {}", definitions(v2, full_addresses))
        }
        ("scan", "docstring_python") => python_docstrings(v2, full_addresses),
        ("scan", "exclude" | "respect_gitignore") => format!("[schema.sources] {key}"),
        ("fmt.cross_refs", "anchor_format") => "[presentation.fmt] anchors".into(),
        ("fmt.cross_refs", "enabled") => "[presentation.fmt] links".into(),
        ("fmt", "exclude") => "[presentation.fmt] exclude".into(),
        ("workspace", _) => format!("[workspace] {key}"),
        ("citations", "default") => "[rules.citations] default".into(),
        (section, key) if section.starts_with("citations.") => format!("[rules.{section}] {key}"),
        _ => ALWAYS.into(),
    }
}

/// The key map and the notices, both in file order.
pub(super) fn map(
    keys: &Keys,
    v2: &Project,
    plan: &Include,
) -> (Vec<MappedKey>, Vec<MigrateNotice>) {
    map_with_addresses(keys, v2, plan, false)
}

/// §FS-config-migrate.2.2: reuse the key map with fully addressed definitions.
pub(super) fn comment_map(keys: &Keys, v2: &Project, plan: &Include) -> Vec<MappedKey> {
    map_with_addresses(keys, v2, plan, true).0
}

/// §FS-config-migrate.2.2: one authored-key mapping supplies both presentations.
fn map_with_addresses(
    keys: &Keys,
    v2: &Project,
    plan: &Include,
    full_addresses: bool,
) -> (Vec<MappedKey>, Vec<MigrateNotice>) {
    let mut mapped = Vec::new();
    let mut notices = Vec::new();
    for record in keys.lines {
        let LineKind::Key {
            section,
            key,
            value,
            ..
        } = &record.kind
        else {
            continue;
        };
        let line = record.line;
        let shown = match section.as_str() {
            "" => key.clone(),
            "kinds" => format!("[[kinds]] {key}"),
            section => format!("[{section}] {key}"),
        };
        if section == "output" {
            notices.push(output_notice(line, key, value));
            continue;
        }
        if (section == "citations" || section.starts_with("citations.")) && key == "default" {
            let level = parse_level(&unquote(value));
            if level.is_some() && may_default(level) != level {
                notices.push(MigrateNotice {
                    line,
                    dropped: false,
                    key: format!("{shown} = {value}"),
                    why: "version 1 gives a positive default no force, so it is written as may"
                        .into(),
                });
            }
        }
        let row = keys.row_at(line).map_or("", |block| block.name.as_str());
        mapped.push(MappedKey {
            line,
            key: shown,
            spelling: spelling(section, key, row, v2, plan, full_addresses),
        });
    }
    (mapped, notices)
}

fn parse_level(value: &str) -> Option<CitationLevel> {
    match value {
        "must" => Some(CitationLevel::Must),
        "should" => Some(CitationLevel::Should),
        _ => None,
    }
}

/// §FS-config-migrate.3.2: `[output]` is dropped, naming the flag that keeps it.
fn output_notice(line: usize, key: &str, value: &str) -> MigrateNotice {
    let (key, why) = match (key, value) {
        ("format", "\"text\"") => (
            format!("[output] format = {value}"),
            "the default of --format, nothing to add".to_string(),
        ),
        ("format", _) => (
            format!("[output] format = {value}"),
            format!("pass --format {} to keep it", unquote(value)),
        ),
        ("relative_paths", "true") => (
            format!("[output] relative_paths = {value}"),
            "the default of --path-base, nothing to add".to_string(),
        ),
        ("relative_paths", _) => (
            format!("[output] relative_paths = {value}"),
            "pass --path-base=invocation to keep it".to_string(),
        ),
        (key, _) => (format!("[output] {key}"), "nothing reads it".to_string()),
    };
    MigrateNotice {
        line,
        dropped: true,
        key,
        why,
    }
}

/// §FS-config-migrate.3.3: each default the candidate writes out that the v1
/// file did not author, as `<v2 key> = <value>` read off `body`, in its order.
pub(super) fn written(keys: &Keys, v1: &Project, body: &str) -> Vec<String> {
    let authored = |section: &str, key: &str| keys.line(section, key).is_some();
    let mut wanted: Vec<(String, String)> = Vec::new();
    if !authored("id", "format") {
        wanted.push(("schema".into(), "id_format".into()));
    }
    if !authored("id", "slug_pattern") {
        wanted.push(("schema".into(), "slug_pattern".into()));
    }
    if !authored("scan", "extensions") {
        wanted.push(("schema.sources".into(), "languages".into()));
    }
    wanted.push(("schema.sources".into(), "declarations".into()));
    if !authored("scan", "exclude") {
        wanted.push(("schema.sources".into(), "exclude".into()));
    }
    let mut lines = Vec::new();
    let mut table = String::new();
    for line in body.lines() {
        if line.starts_with('[') {
            table = line.trim_matches(['[', ']']).to_string();
            if keys.blocks.is_empty()
                && !v1.schema.rows.is_empty()
                && keys.line("", "kinds").is_none()
                && let Some(row) = table.strip_prefix("schema.kinds.")
                && !row.contains('.')
                && v1.schema.rows.iter().any(|r| r.name == row)
            {
                lines.push(format!("[schema.kinds.{row}]"));
            }
            continue;
        }
        let Some((key, value)) = line.split_once(" = ") else {
            continue;
        };
        let definition = table.starts_with("schema.sources.definitions.");
        let take = wanted.iter().any(|(t, k)| *t == table && k == key)
            || (definition && key == "comment_prefixes" && !authored("scan", "comment_prefixes"))
            || (table == "schema.sources.definitions.v1_python"
                && key == "docstrings"
                && !authored("scan", "docstring_python"))
            || (table == "schema.notes.lines"
                && key == "may"
                && !authored("reference", "inline_note_suggested_lines"))
            || (table == "presentation" && key == "rules");
        if take {
            lines.push(format!("[{table}] {key} = {value}"));
        }
    }
    lines
}
