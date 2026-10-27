//! The 0.21 window over YAML value sources (§FS-values.2.2.4): every error a
//! run locates at a YAML value source, and every finding the scan held back from
//! one, is one `yaml-value-source-deferred` warning until grund 0.22.0. Which
//! findings are wrapped is decided by where they are located, never by how a
//! citation resolves (§FS-values.requirements.origin-blind).

use std::path::Path;

use once_cell::sync::Lazy;
use regex::Regex;

use super::support::sort_diagnostics;
use crate::config::{Frame, Schema};
use crate::grammar::render_catalog_id;
use crate::model::remedies;
use crate::model::{Catalog, CheckReport, Code, Diagnostic, Site, Switch, YamlDeferred};
use crate::sources::paths_same_location;

/// The release that ends the window: a YAML source's findings carry their own
/// codes and severities from it on (§FS-values.2.2.4, §DF-yaml-value-sources).
pub(crate) const YAML_WINDOW_RELEASE: &str = "0.22.0";

/// A wrapped finding's own release clause, which its words drop
/// (§FS-values.2.2.4): ` — became an error in grund X`, or `; this became …`.
static RELEASE_CLAUSE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?: —|;)(?: this)? became an error in grund \d+\.\d+\.\d+").unwrap());

/// §FS-values.2.2.4: move each error located at a YAML value source to the
/// warnings as the window's code, and report what the scan held back.
pub(super) fn defer_yaml_findings(
    catalog: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    // §FS-check.1.4.1: a whole-value kind is what turns the window on.
    if schema.value_rows().next().is_some() {
        report.armed.arm(Switch::ValueHome);
    }
    if catalog.yaml_value_sources.is_empty() {
        return;
    }
    let at_yaml = |path: &Path| {
        catalog
            .yaml_value_sources
            .iter()
            .any(|source| paths_same_location(source, path))
    };
    let (deferred, kept) = std::mem::take(&mut report.errors)
        .into_iter()
        .partition::<Vec<_>, _>(|error| error.path.as_deref().is_some_and(at_yaml));
    report.errors = kept;
    for error in deferred {
        let file = frame.display_path(error.path.as_deref().unwrap_or(Path::new("")));
        let (words, remedy) = words_and_remedy(&error.message);
        let message = wrapped(&words, remedy, &file);
        report.warnings.push(Diagnostic {
            code: Code::YAML_VALUE_SOURCE_DEFERRED,
            message,
            ..error
        });
    }
    for record in &catalog.yaml_deferred {
        report.warnings.push(held_back(record, catalog, frame));
    }
    sort_diagnostics(&mut report.warnings);
}

/// A wrapped finding's message split into its words and its remedy: the
/// message without its own release clause, cut at its first `; `.
fn words_and_remedy(message: &str) -> (String, Option<String>) {
    let message = RELEASE_CLAUSE.replace(message, "");
    match message.split_once("; ") {
        Some((words, remedy)) => (words.to_string(), Some(remedy.to_string())),
        None => (message.into_owned(), None),
    }
}

/// The window's message: the wrapped words, the release clause, then the row's
/// *Fix* (§FS-errors.5.5).
fn wrapped(words: &str, remedy: Option<String>, file: &str) -> String {
    let fix = match remedy {
        Some(remedy) => remedies::yaml_value_source_deferred_wrapped(remedy, file),
        None => remedies::yaml_value_source_deferred_move(file),
    };
    format!(
        "{words} — a YAML value source's findings are warnings until grund {YAML_WINDOW_RELEASE}; {fix}"
    )
}

fn held_back(record: &YamlDeferred, catalog: &Catalog, frame: Frame<'_>) -> Diagnostic {
    let diagnostic = |path: &Path, line, column, message, sites| Diagnostic {
        code: Code::YAML_VALUE_SOURCE_DEFERRED,
        path: Some(path.to_path_buf()),
        line: Some(line),
        column,
        message,
        sites,
        authority: Vec::new(),
    };
    match record {
        YamlDeferred::Unread {
            file,
            line,
            column,
            message,
        } => {
            let shown = frame.display_path(file);
            diagnostic(
                file,
                *line,
                *column,
                wrapped(message, None, &shown),
                Vec::new(),
            )
        }
        YamlDeferred::Duplicate { id, sites } => {
            let (file, line) = &sites[0];
            let others = sites[1..]
                .iter()
                .map(|(path, line)| format!("{}:{line}", frame.display_path(path)))
                .collect::<Vec<_>>()
                .join(", ");
            let rendered = render_catalog_id(frame.grammar(), &catalog.declarations, id);
            let words = format!("duplicate declaration of {rendered} (also declared at {others})");
            let message = wrapped(&words, None, &frame.display_path(file));
            let sites = sites
                .iter()
                .map(|(path, line)| Site {
                    path: path.clone(),
                    line: *line,
                })
                .collect();
            diagnostic(file, *line, None, message, sites)
        }
        // §FS-values.2.2.4: no finding to wrap, so a sentence of its own.
        YamlDeferred::Fetched { file, kind } => {
            let shown = frame.display_path(file);
            let message = format!(
                "{shown} is not read, because {kind} sets fetch; grund {YAML_WINDOW_RELEASE} reads a YAML value source in a fetched kind's home; {}",
                remedies::yaml_value_source_deferred_move(&shown)
            );
            diagnostic(file, 1, None, message, Vec::new())
        }
    }
}
