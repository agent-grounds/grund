//! First-class value inputs: Markdown component validation, exact authored
//! binding recognition, and home-derived JSON catalog enrollment
//! (§FS-values.2, §FS-values.3, §AR-scanner.2.1–§AR-scanner.3).

use std::collections::BTreeSet;
use std::path::Path;

use super::citation_line::CitationLine;
use super::tree::path_starts_with;
use super::value_binding_attempts::{
    invalid_value_binding_site, scan_noncanonical_value_binding_attempts,
};
use super::value_context::{binding_span_is_inside, value_binding_context};
use crate::config::{FileReading, Frame, Schema};
use crate::grammar::{Grammar, expand_relative_heading, value_binding_section_shape_is_valid};
use crate::model::{
    Catalog, Id, InvalidValueSite, ValueBinding, authored_component, component_text_is_valid,
};
use crate::sources::paths_same_location;
use crate::workspace::WorkspaceCitationTarget;

pub(super) fn value_declaration_is_in_home(
    schema: &Schema,
    frame: Frame<'_>,
    path: &Path,
    id: &Id,
) -> bool {
    // §FS-config-v2.schema.places: any file or folder of the row.
    schema.value_rows().any(|row| {
        row.name == id.kind
            && (row
                .files()
                .any(|file| paths_same_location(path, &frame.root().join(file)))
                || row
                    .folders()
                    .any(|folder| path_starts_with(path, &frame.root().join(folder))))
    })
}

/// Validate opted-in Markdown declarations after body spans are known, so a
/// later sibling heading cannot accidentally become a value field
/// (§FS-values.2.1).
pub(super) fn validate_markdown_value_declarations(
    path: &Path,
    text: &str,
    reading: &FileReading<'_>,
    schema: &Schema,
    frame: Frame<'_>,
    findings: &mut Catalog,
) {
    let is_md = reading.markdown;
    if schema.value_rows().next().is_none() {
        return;
    }
    let lines = text.lines().collect::<Vec<_>>();
    let mut invalid = Vec::new();
    for decl in findings.declarations.values_mut().flatten().filter(|decl| {
        paths_same_location(&decl.file, path)
            && value_declaration_is_in_home(schema, frame, path, &decl.id)
    }) {
        let invalid_id = decl.id.clone();
        let invalid_file = decl.file.clone();
        let invalid_source = decl.source.clone();
        let invalid_for = |line, reason: &str| InvalidValueSite {
            id: Some(invalid_id.clone()),
            file: invalid_file.clone(),
            line,
            column: None,
            message: reason.to_string(),
            source: invalid_source.clone(),
            binding_namespace: None,
            binding_section: None,
        };
        let mut valid = true;
        if !is_md {
            valid = false;
            invalid.push(invalid_for(
                decl.line,
                "value declarations in text homes must be Markdown headings",
            ));
        }
        for line_no in decl.body_start.saturating_add(1)..=decl.body_end {
            if lines
                .get(line_no.saturating_sub(1))
                .is_some_and(|line| empty_citable_value_heading(line, decl.heading_level, frame))
            {
                valid = false;
                invalid.push(invalid_for(
                    line_no,
                    "value component heading must contain a nonempty component",
                ));
            }
        }
        let mut fields = decl
            .sections
            .iter_mut()
            .filter(|(_, info)| (decl.body_start..=decl.body_end).contains(&info.line))
            .collect::<Vec<_>>();
        fields.sort_by_key(|(_, info)| info.line);
        if fields.is_empty() {
            valid = false;
            invalid.push(invalid_for(
                decl.line,
                "value declaration must define contiguous numbered components starting at `.1`",
            ));
        }
        for (index, (coordinate, info)) in fields.into_iter().enumerate() {
            let expected = index + 1;
            if coordinate != &expected.to_string() || info.heading_level != decl.heading_level + 1 {
                valid = false;
                invalid.push(invalid_for(
                    info.line,
                    &format!(
                        "value components must be immediate contiguous headings `.1` through `.N` (expected `.{expected}`)"
                    ),
                ));
                continue;
            }
            let Some(line) = lines.get(info.line.saturating_sub(1)) else {
                valid = false;
                continue;
            };
            let recorded = info.relative.then_some(coordinate.as_str());
            match recorded_component(line, recorded, reading.lexical.grammar) {
                Some((component, column)) if component_text_is_valid(component) => {
                    info.value = Some(authored_component(component, column));
                }
                _ => {
                    valid = false;
                    invalid.push(invalid_for(
                        info.line,
                        "value component must be nonempty, single-line, edge-unspaced, and contain no backtick or control character",
                    ));
                }
            }
        }
        for (_, duplicate) in decl
            .duplicate_sections
            .iter()
            .filter(|(_, info)| (decl.body_start..=decl.body_end).contains(&info.line))
        {
            valid = false;
            invalid.push(invalid_for(
                duplicate.line,
                "value component coordinate is duplicated",
            ));
        }
        decl.value_valid = Some(valid);
    }
    findings.invalid_value_declarations.extend(invalid);
}

fn empty_citable_value_heading(line: &str, declaration_level: usize, frame: Frame<'_>) -> bool {
    let trimmed = line.trim_start();
    let level = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    if level <= declaration_level
        || !trimmed[level..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
    {
        return false;
    }
    let rest = trimmed[level..].trim_start();
    let token = rest.split_whitespace().next().unwrap_or("");
    let numeric = token
        .strip_suffix('.')
        .unwrap_or(token)
        .split('.')
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
    let named = frame.grammar().named_sections
        && token.strip_suffix(':').is_some_and(|coordinate| {
            coordinate.split('.').all(|part| {
                !part.is_empty()
                    && part.bytes().all(|byte| {
                        // coordinate-lexer: looser on purpose: `## 3a:` stays a heading
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                    })
            })
        });
    (numeric || named) && rest[token.len()..].trim().is_empty()
}

pub(crate) fn markdown_component<'a>(line: &'a str, grammar: &Grammar) -> Option<(&'a str, usize)> {
    let captures = grammar.section_re.captures(line)?;
    let coordinate = captures.name("sec")?;
    // Numeric heading punctuation is optional and intentionally sits outside
    // the `sec` capture; it delimits the title but is not part of it
    // (§FS-values.2.1).
    let tail = line[coordinate.end()..]
        .strip_prefix('.')
        .unwrap_or(&line[coordinate.end()..]);
    let component = tail.trim_start_matches([' ', '\t']);
    let start = line.len() - component.len();
    Some((component, start + 1))
}

/// [`markdown_component`] of a heading, read at `recorded` — the canonical path
/// the record holds for it — when it is written in the relative form, so its
/// component is its complete form's; the column stays on the line as written
/// (§FS-values.2.4). A doc-comment line is expanded after its comment prefix, so
/// its relative form reads as its complete form does
/// (§FS-declarations.section.relative.where).
pub(crate) fn recorded_component<'a>(
    line: &'a str,
    recorded: Option<&str>,
    grammar: &Grammar,
) -> Option<(&'a str, usize)> {
    let Some(path) = recorded else {
        return markdown_component(line, grammar);
    };
    // The heading's `#` run starts the line or follows whitespace after the prefix.
    let heading = line
        .char_indices()
        .filter(|&(at, ch)| ch == '#' && (at == 0 || line[..at].ends_with([' ', '\t'])))
        .map(|(at, _)| at)
        .find(|&at| expand_relative_heading(&line[at..], path) != line[at..])
        .unwrap_or(0);
    let complete = format!(
        "{}{}",
        &line[..heading],
        expand_relative_heading(&line[heading..], path)
    );
    // The expansion keeps everything after the path, so the component ends `line`.
    let start = line.len() - markdown_component(&complete, grammar)?.0.len();
    Some((&line[start..], start + 1))
}

/// Record only exact authored bindings in Markdown or a recognized whole-line
/// source comment. The citation remains in `Catalog::citations`; this record
/// adds the component and binding span without a second resolver
/// (§FS-values.3, §DA-explicit-value-bindings.2).
pub(super) fn scan_value_bindings(
    line: &CitationLine<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
    citation_start: usize,
    findings: &mut Catalog,
) {
    let Some(context) = value_binding_context(line) else {
        return;
    };
    let mut bindings = Vec::new();
    let mut invalid = Vec::new();
    let mut classified_openings = BTreeSet::new();
    for citation in &findings.citations[citation_start..] {
        if !citation.has_marker {
            continue;
        }
        let marker_start = citation
            .column
            .saturating_sub(line.column_offset)
            .saturating_sub(1);
        let token_end = marker_start.saturating_add(citation.text.len());
        let Some(prefix) = line.scan_line.get(..marker_start) else {
            continue;
        };
        let Some(before_marker) = prefix.strip_suffix(" (") else {
            continue;
        };
        if !before_marker.ends_with('`') {
            if let Some(open_tick) = unmatched_open_tick(before_marker) {
                if !binding_span_is_inside(context, open_tick, token_end) {
                    continue;
                }
                classified_openings.insert(open_tick);
                invalid.push(invalid_value_binding_site(
                    line,
                    citation.namespace.clone(),
                    Some(citation.id.clone()),
                    citation.section.clone(),
                    open_tick,
                ));
            }
            continue;
        }
        let Some(close_tick) = before_marker.len().checked_sub(1) else {
            continue;
        };
        if before_marker.as_bytes().get(close_tick) != Some(&b'`') {
            continue;
        }
        let Some(open_tick) = before_marker[..close_tick].rfind('`') else {
            continue;
        };
        let literal = &before_marker[open_tick + 1..close_tick];
        let closes = line
            .scan_line
            .get(token_end..)
            .is_some_and(|tail| tail.starts_with(')'));
        let binding_end = token_end.saturating_add(usize::from(closes));
        if !binding_span_is_inside(context, open_tick, binding_end) {
            continue;
        }
        let section = citation.section.as_deref();
        // §FS-values.3.1: a value root (a bare ID or a valid root path), then an optional
        // positive numeric immediate-component coordinate. Whether the path is a root,
        // and so which reading it takes, is the checker's question (§FS-values.5.1).
        let valid_section = section.is_none_or(value_binding_section_shape_is_valid);
        if citation.shorthand {
            // The ordinary noncanonical-shorthand finding owns this site and
            // suppresses value handling (§FS-values.5.1).
            classified_openings.insert(open_tick);
            continue;
        }
        if !closes || !valid_section || !component_text_is_valid(literal) {
            classified_openings.insert(open_tick);
            invalid.push(invalid_value_binding_site(
                line,
                citation.namespace.clone(),
                Some(citation.id.clone()),
                citation.section.clone(),
                open_tick,
            ));
            continue;
        }
        classified_openings.insert(open_tick);
        bindings.push(ValueBinding {
            namespace: citation.namespace.clone(),
            id: citation.id.clone(),
            section: citation.section.clone(),
            authored: authored_component(literal, line.column_offset + open_tick + 2),
            file: citation.file.clone(),
            line: citation.line,
            column: line.column_offset + open_tick + 1,
            citation_column: citation.column,
        });
    }
    scan_noncanonical_value_binding_attempts(
        line,
        workspace_targets,
        context,
        &classified_openings,
        &mut invalid,
    );
    findings.value_bindings.extend(bindings);
    findings.invalid_value_bindings.extend(invalid);
}

fn unmatched_open_tick(prefix: &str) -> Option<usize> {
    prefix.match_indices('`').fold(None, |opening, (index, _)| {
        opening.map_or(Some(index), |_| None)
    })
}
