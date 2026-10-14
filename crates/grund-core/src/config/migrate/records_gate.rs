//! §FS-config-migrate.5.1: the records gate. The candidate, loaded through the
//! v2 reader, and the v1 file's project are each reduced to the meanings the
//! gate compares, one section at a time, ignoring only what §FS-config-migrate.5.1
//! lists; a section that differs prints its two readings.

use std::collections::BTreeSet;
use std::fmt::Debug;
use std::path::Path;

use super::candidate::may_default;
use crate::config::grounding::configured_row_grounding;
use crate::config::kind::KindResolution;
use crate::config::project::{Project, Strength};
use crate::config::rows::{Extent, Place, Row};
use crate::config::sources::DeclarationMode;
use crate::grammar::{DocCommentRule, extension_rule};

/// One section of the comparison, as a line per reading.
type Section = (&'static str, String);

fn line(name: &'static str, value: impl Debug) -> Section {
    (name, format!("{value:?}"))
}

/// §FS-config-migrate.5.4: the differing sections as `- `/`+ ` lines, empty
/// when the records are equal. It runs only when no two v1 homes nest, which
/// refuses, so the nesting rule decides nothing and is not compared.
pub(super) fn compare(v1: &Project, v2: &Project, root: &Path) -> Vec<String> {
    let (old, new) = (reduce(v1, root), reduce(v2, root));
    let mut out = Vec::new();
    for ((name, before), (_, after)) in old.iter().zip(&new) {
        if before != after {
            out.push(format!("- {name}: {before}"));
            out.push(format!("+ {name}: {after}"));
        }
    }
    out
}

/// A row without the place sources, which are where it was written.
fn bare(row: &Row) -> Row {
    let mut row = row.clone();
    row.places = row
        .places
        .iter()
        .map(|place| Place {
            source: None,
            ..place.clone()
        })
        .collect();
    row
}

fn reduce(p: &Project, root: &Path) -> Vec<Section> {
    let schema = &p.schema;
    let workspace = &p.workspace;
    let notes = &schema.notes;
    let mut lines: BTreeSet<(u8, usize)> = BTreeSet::new();
    lines.insert((Strength::Must as u8, notes.max_lines));
    let guidance = if notes.warn_on_suggested || p.version == 1 {
        let strength = if notes.warn_on_suggested {
            Strength::Warn
        } else {
            Strength::May
        };
        Some((strength as u8, notes.suggested_lines))
    } else {
        None
    };
    lines.extend(guidance);
    lines.extend(
        notes
            .extra_lines
            .iter()
            .map(|t| (t.strength as u8, t.value)),
    );
    let mut columns: BTreeSet<(u8, usize)> = BTreeSet::new();
    columns.insert((Strength::Must as u8, notes.max_columns));
    columns.extend(
        notes
            .extra_columns
            .iter()
            .map(|t| (t.strength as u8, t.value)),
    );
    let rows: Vec<Row> = schema
        .rows
        .iter()
        .filter(|row| !row.is_complement())
        .map(bare)
        .collect();
    let mut citations = p.rules.citations.clone();
    citations.global_default = may_default(citations.global_default);
    for rules in citations.per_kind.values_mut() {
        rules.default = may_default(rules.default);
    }
    let mut names: Vec<&str> = schema.rows.iter().map(|row| row.name.as_str()).collect();
    if !names.contains(&schema.complement_name()) {
        names.push(schema.complement_name());
    }
    let grounding: Vec<_> = names
        .iter()
        .map(|name| (name.to_string(), configured_row_grounding(&p.rules, name)))
        .collect();
    let resolution: Vec<_> = p
        .rules
        .resolution
        .iter()
        .filter(|(_, r)| **r != KindResolution::Must)
        .collect();
    let presentation = &p.presentation;
    vec![
        line("project_name", &p.name),
        line(
            "workspace",
            (
                workspace.declared,
                &workspace.members,
                &workspace.optional_members,
                workspace.include_root,
            ),
        ),
        line(
            "citation syntax",
            (&schema.citation.marker, schema.citation.shorthand),
        ),
        line("ids", &schema.ids),
        line("sources", sources(p)),
        line("scan roots", scan_roots(p, root)),
        line(
            "notes",
            (
                &notes.inline_style,
                &notes.layout,
                &notes.layout_check,
                lines,
                columns,
            ),
        ),
        line("leads", (&schema.leads, &schema.lead_thresholds)),
        line("rows", rows),
        line("complement", schema.complement_name()),
        line("citations", citations),
        line(
            "grounding",
            (
                p.rules.grounding.require,
                p.rules.grounding.level,
                grounding,
            ),
        ),
        line("resolution", resolution),
        line(
            "presentation",
            (
                &presentation.kinds,
                &presentation.description,
                &presentation.conversation,
                &presentation.trigger,
                presentation.rules_at_root,
                &presentation.fmt,
            ),
        ),
    ]
}

/// The source model suffix by suffix: reader class, comment prefixes, docstring
/// reading, note-measurement rule and declaration eligibility, with the exclusions
/// as globs and the gitignore switch.
fn sources(
    p: &Project,
) -> (
    Vec<(String, bool, Vec<String>, bool, DocCommentRule, bool)>,
    Vec<String>,
    bool,
) {
    let sources = &p.schema.sources;
    let mut suffixes = Vec::new();
    let exclude = match &sources.model {
        None => {
            for suffix in &sources.extensions {
                let python = suffix == "py" && sources.docstring_python;
                let rule = match extension_rule(suffix) {
                    DocCommentRule::Docstring if !python => DocCommentRule::None,
                    rule => rule,
                };
                suffixes.push((
                    suffix.clone(),
                    suffix == "md",
                    sources.comment_prefixes.clone(),
                    python,
                    rule,
                    true,
                ));
            }
            sources
                .exclude
                .iter()
                .map(|name| format!("{name}/"))
                .collect()
        }
        Some(model) => {
            for suffix in model.text_suffixes() {
                let probe = format!("x.{suffix}");
                let Some(language) = model.language_of(Path::new(&probe)) else {
                    continue;
                };
                suffixes.push((
                    suffix,
                    language.class == crate::config::sources::LanguageClass::Document,
                    language.comment_prefixes.clone(),
                    language.docstrings,
                    language.measurement_rule(),
                    model.declarations == DeclarationMode::AnyComment,
                ));
            }
            model.exclude.clone()
        }
    };
    suffixes.sort_by(|a, b| a.0.cmp(&b.0));
    (suffixes, exclude, sources.respect_gitignore)
}

/// `include` as the derived scan roots: v1's entries outside every home that
/// exist under `root`, v2's complement places.
fn scan_roots(p: &Project, root: &Path) -> BTreeSet<String> {
    let schema = &p.schema;
    match &schema.sources.model {
        None => {
            let homes: Vec<&str> = schema
                .places()
                .filter_map(|(_, place)| match &place.extent {
                    Extent::Folder(path) | Extent::File(path) => Some(path.as_str()),
                    Extent::Complement => None,
                })
                .collect();
            let inside = |entry: &str, home: &str| {
                entry == home
                    || entry
                        .strip_prefix(home)
                        .is_some_and(|rest| rest.starts_with('/'))
            };
            schema
                .sources
                .include
                .iter()
                .flatten()
                .map(|entry| {
                    entry
                        .trim_start_matches("./")
                        .trim_end_matches('/')
                        .to_string()
                })
                .filter(|entry| !homes.iter().any(|home| inside(entry, home)))
                .filter(|entry| root.join(entry).exists())
                .collect()
        }
        Some(_) => schema
            .rows
            .iter()
            .filter(|row| row.is_complement())
            .flat_map(|row| row.places.iter())
            .filter_map(|place| match &place.extent {
                Extent::Folder(path) | Extent::File(path) => Some(path.clone()),
                Extent::Complement => None,
            })
            .collect(),
    }
}
