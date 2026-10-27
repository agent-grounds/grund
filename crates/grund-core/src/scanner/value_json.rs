//! Discovery and one-read enrollment of a value home's value sources
//! (§FS-values.2.2, §AR-scanner.2.1.3, §AR-scanner.3): a JSON source is read
//! here, a YAML one by `value_yaml.rs`, and both enroll through
//! `value_json_enrollment.rs`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::json::{JsonNode, JsonReader};
use super::tree::ScanError;
use super::value_json_enrollment::{enroll_json_member, push_json_invalid};
use super::value_yaml::enroll_yaml_sources;
use crate::config::{Frame, Schema};
use crate::model::sort_path_key;
use crate::model::{Catalog, DeclarationSource, ValueFormat, normalize_path_lexically};
use crate::sources::{paths_same_location, physical_path_key};

pub(super) fn scan_value_sources(
    schema: &Schema,
    frame: Frame<'_>,
    findings: &mut Catalog,
    errors: &mut Vec<ScanError>,
) {
    let sources = match value_sources(schema, frame) {
        Ok(sources) => sources,
        Err((path, message)) => {
            errors.push((path, message));
            return;
        }
    };
    let mut grouped = BTreeMap::<PathBuf, (Vec<PathBuf>, Vec<&str>)>::new();
    let mut formats = BTreeMap::new();
    for (path, kind, format) in sources {
        formats.insert(path.clone(), format);
        let (paths, owners) = grouped.entry(physical_path_key(&path)).or_default();
        if !paths.contains(&path) {
            paths.push(path);
        }
        if !owners.contains(&kind) {
            owners.push(kind);
        }
    }
    let mut grouped = grouped.into_values().collect::<Vec<_>>();
    for (paths, owners) in &mut grouped {
        paths.sort_by_key(|path| sort_path_key(path));
        owners.sort();
    }
    grouped.sort_by_key(|(paths, _)| sort_path_key(&paths[0]));
    let mut yaml = Vec::new();
    for (paths, owners) in grouped {
        let path = &paths[0];
        if formats.get(path) == Some(&ValueFormat::Yaml) {
            yaml.push((path.clone(), owners));
            continue;
        }
        // §AR-system.2.13: one location, one text, whichever spelling holds it.
        let text = match frame.sources().text(path) {
            Ok(text) => text.into_owned(),
            Err(error) => {
                errors.push((
                    path.clone(),
                    format!("could not read value JSON source: {error}"),
                ));
                continue;
            }
        };
        match JsonReader::parse(&text) {
            Ok(root) => enroll_value_root(
                frame,
                path,
                &owners,
                &text,
                root,
                ValueFormat::Json,
                findings,
            ),
            Err(message) => errors.push((path.clone(), format!("invalid JSON: {message}"))),
        }
        if owners.len() > 1 {
            duplicate_owned_source(path, findings);
        }
    }
    enroll_yaml_sources(schema, frame, yaml, findings);
}

/// A source owned by two homes is ambiguous even after one read; duplicate each
/// site so the ordinary duplicate rule stays the single winner (§FS-values.2.3,
/// §FS-declarations.checks.duplicate).
pub(super) fn duplicate_owned_source(path: &Path, findings: &mut Catalog) {
    let duplicated = findings
        .declarations
        .iter()
        .flat_map(|(id, declarations)| {
            declarations
                .iter()
                .filter(|declaration| {
                    matches!(declaration.source, DeclarationSource::Json { .. })
                        && paths_same_location(&declaration.file, path)
                })
                .cloned()
                .map(|declaration| (id.clone(), declaration))
        })
        .collect::<Vec<_>>();
    for (id, declaration) in duplicated {
        findings
            .declarations
            .entry(id)
            .or_default()
            .push(declaration);
    }
}

/// Every value source of every value home, with its owning kind and format
/// (§FS-values.terms, §FS-values.2.2), in one bytewise path order across formats.
pub(super) fn value_sources<'a>(
    schema: &'a Schema,
    frame: Frame<'a>,
) -> std::result::Result<Vec<(PathBuf, &'a str, ValueFormat)>, (PathBuf, String)> {
    let mut sources = Vec::new();
    // §FS-config-v2.schema.sources.values: the home picks the files, their language the reader.
    let format_of = |path: &Path| frame.compiled.reading(schema, path).value_format();
    for row in schema.value_rows() {
        let kind = row.name.as_str();
        // §FS-config-v2.schema.places: every file and folder of the row is a value home.
        for file in row.files() {
            let path = normalize_path_lexically(&frame.root().join(file));
            if let Some(format) = format_of(&path) {
                sources.push((path, kind, format));
            }
        }
        for folder in row.folders() {
            let folder = normalize_path_lexically(&frame.root().join(folder));
            // §FS-check.6.1.1: cover this effective input before its shared read.
            let disk = frame.sources();
            let entries = disk
                .read_dir(&folder)
                .map_err(|error| (folder.clone(), format!("read value home: {error}")))?;
            for entry in entries {
                let entry =
                    entry.map_err(|error| (folder.clone(), format!("read value home: {error}")))?;
                let path = normalize_path_lexically(&entry.path());
                if let Some(format) = format_of(&path)
                    && disk.is_file(&path)
                {
                    sources.push((path, kind, format));
                }
            }
            for path in disk.buffer_only_files() {
                if path
                    .parent()
                    .is_some_and(|parent| paths_same_location(parent, &folder))
                    && let Some(format) = format_of(&path)
                {
                    sources.push((path, kind, format));
                }
            }
        }
    }
    sources.sort_by(|(left, left_kind, _), (right, right_kind, _)| {
        (sort_path_key(left), left_kind).cmp(&(sort_path_key(right), right_kind))
    });
    sources.dedup_by(|left, right| left.0 == right.0 && left.1 == right.1);
    Ok(sources)
}

/// The noun with its indefinite article, for a reason naming a format's word.
pub(super) fn a(noun: &str) -> String {
    let article = if noun.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    format!("{article} {noun}")
}

/// Enroll one source's root, read by either reader (§FS-values.2.2.1).
pub(super) fn enroll_value_root(
    frame: Frame<'_>,
    path: &Path,
    owners: &[&str],
    text: &str,
    root: JsonNode,
    format: ValueFormat,
    findings: &mut Catalog,
) {
    let (name, (object, _)) = (format.name(), format.collections());
    let (members, span) = match root {
        JsonNode::Object(members, span) => (members, span),
        // §FS-values.2.2.3: a construct the YAML subset refuses, named.
        JsonNode::Refused(span, reason) => {
            push_json_invalid(findings, None, path, text, span, &reason, format);
            return;
        }
        root => {
            let message = format!("value {name} root must be {}", a(object));
            push_json_invalid(findings, None, path, text, root.span(), &message, format);
            return;
        }
    };
    if members.is_empty() {
        let message = format!("value {name} {object} must not be empty");
        push_json_invalid(findings, None, path, text, span, &message, format);
    }
    for member in members {
        enroll_json_member(frame, path, owners, text, member, format, findings);
    }
}
