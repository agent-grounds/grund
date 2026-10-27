//! A value home's YAML sources through the 0.21 window (§FS-values.2.2.4): each
//! is read into the span tree, held to the value subset, and enrolled through
//! the one path JSON takes, or held back whole as a deferred finding.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::value_json::{duplicate_owned_source, enroll_value_root};
use super::value_yaml_subset::yaml_value_root;
use super::yaml::parse_yaml;
use crate::config::{Frame, Schema};
use crate::model::{Catalog, Declaration, Id, ValueFormat, YamlDeferred, sort_path_key};

/// Enroll every YAML source a value home holds, in path order. A source any
/// finding keeps from being enrolled declares nothing, so a JSON value it would
/// duplicate stays the one declared (§FS-values.2.2.4).
pub(super) fn enroll_yaml_sources(
    schema: &Schema,
    frame: Frame<'_>,
    sources: Vec<(PathBuf, Vec<&str>)>,
    findings: &mut Catalog,
) {
    let mut readable: Vec<(PathBuf, BTreeMap<Id, Vec<Declaration>>)> = Vec::new();
    for (path, owners) in sources {
        findings.yaml_value_sources.push(path.clone());
        let fetched = owners.iter().find(|owner| {
            schema
                .value_rows()
                .any(|row| row.name == **owner && row.fetches())
        });
        if let Some(kind) = fetched {
            findings.yaml_deferred.push(YamlDeferred::Fetched {
                file: path,
                kind: kind.to_string(),
            });
            continue;
        }
        let text = match frame.sources().text(&path) {
            Ok(text) => text.into_owned(),
            Err(error) => {
                findings.yaml_deferred.push(YamlDeferred::Unread {
                    file: path,
                    line: 1,
                    column: None,
                    message: format!("could not read value source: {error}"),
                });
                continue;
            }
        };
        let stream = match parse_yaml(&text) {
            Ok(stream) => stream,
            Err(error) => {
                findings.yaml_deferred.push(YamlDeferred::Unread {
                    file: path,
                    line: error.line,
                    column: Some(error.column),
                    message: format!("invalid YAML: {}", error.message),
                });
                continue;
            }
        };
        let mut own = Catalog::default();
        let root = yaml_value_root(&text, &stream);
        enroll_value_root(
            frame,
            &path,
            &owners,
            &text,
            root,
            ValueFormat::Yaml,
            &mut own,
        );
        if owners.len() > 1 {
            duplicate_owned_source(&path, &mut own);
        }
        if own.invalid_value_declarations.is_empty() {
            readable.push((path, own.declarations));
        } else {
            findings
                .invalid_value_declarations
                .append(&mut own.invalid_value_declarations);
        }
    }
    let party = record_duplicates(&readable, findings);
    for (path, declarations) in readable {
        if party.contains(&path) {
            continue;
        }
        for (id, mut declarations) in declarations {
            findings
                .declarations
                .entry(id)
                .or_default()
                .append(&mut declarations);
        }
    }
}

/// Every ID a readable YAML source declares that is declared more than once,
/// across the YAML sources and what the catalog already holds, is one deferred
/// duplicate at its first YAML site (§FS-values.2.3, §FS-values.2.2.4). Returns
/// the sources party to one.
fn record_duplicates(
    readable: &[(PathBuf, BTreeMap<Id, Vec<Declaration>>)],
    findings: &mut Catalog,
) -> BTreeSet<PathBuf> {
    let mut yaml_sites = BTreeMap::<&Id, Vec<(PathBuf, usize)>>::new();
    for (_, declarations) in readable {
        for (id, declarations) in declarations {
            let sites = yaml_sites.entry(id).or_default();
            sites.extend(declarations.iter().map(|d| (d.file.clone(), d.line)));
        }
    }
    let mut party = BTreeSet::new();
    for (id, sites) in yaml_sites {
        let mut others = findings
            .declarations
            .get(id)
            .into_iter()
            .flatten()
            .filter(|declaration| !declaration.is_stub)
            .map(|declaration| (declaration.file.clone(), declaration.line))
            .chain(sites[1..].iter().cloned())
            .collect::<Vec<_>>();
        if others.is_empty() {
            continue;
        }
        party.extend(sites.iter().map(|(path, _)| path.clone()));
        others.sort_by(|(left, left_line), (right, right_line)| {
            (sort_path_key(left), left_line).cmp(&(sort_path_key(right), right_line))
        });
        let mut all = vec![sites[0].clone()];
        all.extend(others);
        findings.yaml_deferred.push(YamlDeferred::Duplicate {
            id: id.clone(),
            sites: all,
        });
    }
    party
}
