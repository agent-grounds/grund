//! The spans and targets one snapshot row carries (§AR-system.2.9,
//! §AR-lsp.5.1): the query ID an editor navigates by, where a citation or a stub
//! resolves to, and the exact column and text of a declaration, section or
//! heading title (§FS-lsp.1.1, §FS-lsp.1.3).
//!
//! Beside `lsp_snapshot.rs` rather than in it, per §AR-core-module-layout.3's
//! file budget: the walk there decides *which* rows exist, and each answer here
//! is a function of one row.

use std::path::{Path, PathBuf};

use crate::model::{
    Citation, Declaration, DeclarationSource, FileRecords, SectionInfo, StubResolution,
    paired_declarations,
};
use crate::resolver::{WorkspaceContext, WorkspaceProject};
use crate::scanner::EMBEDDED_VALUE_MARKER;
use crate::sources::canonicalize_existing_prefix;

pub(super) fn lsp_query_id(
    context: &WorkspaceContext,
    project: &WorkspaceProject,
    rendered_id: &str,
    section: Option<&str>,
) -> String {
    let mut query = if context.workspace_loaded {
        format!("{}/{}", project.alias, rendered_id)
    } else {
        rendered_id.to_string()
    };
    if let Some(section) = section {
        query.push_str(&project.config.section_separator);
        query.push_str(section);
    }
    query
}

pub(super) fn lsp_target_for_citation(
    target_project: &WorkspaceProject,
    citation: &Citation,
) -> Option<(PathBuf, usize)> {
    let decls = target_project.findings.declarations.get(&citation.id)?;
    if let Some(section) = citation.section.as_deref()
        && let Some(decl) = decls
            .iter()
            .find(|decl| decl.sections.contains_key(section))
        && let Some(info) = decl.sections.get(section)
    {
        return Some((decl.file.clone(), info.line));
    }
    // A missing local coordinate has an owner for graph and diagnostic
    // purposes, but no editor destination: falling back to the declaration
    // heading would invent a target for the unresolved token (§FS-lsp.1.3).
    if citation.local_section && citation.section.is_some() {
        return None;
    }
    // §AR-scanner.4.6: the first declaration's record, where the
    // scan found it, whether or not the walk reached its file.
    let first = paired_declarations(decls).iter().next()?.record;
    Some((first.file.clone(), first.line))
}

/// Where a stub navigates: the first record its verdict found at the target
/// (§AR-scanner.4.6); a stub whose verdict holds no record has no
/// destination.
pub(super) fn lsp_target_for_stub(stub: &Declaration) -> Option<(PathBuf, usize)> {
    let Some(StubResolution::Declares(records)) = &stub.stub_resolution else {
        return None;
    };
    let first = records.first()?;
    Some((first.file.clone(), first.line))
}

pub(super) fn declaration_range_parts(
    files: &FileRecords,
    decl: &Declaration,
    rendered_id: &str,
) -> (usize, String) {
    match &decl.source {
        DeclarationSource::Json {
            key_column,
            key_text,
            ..
        } => (*key_column, key_text.clone()),
        DeclarationSource::Text => heading_span_parts(files, &decl.file, decl.line, rendered_id),
    }
}

pub(super) fn section_range_parts(
    files: &FileRecords,
    decl: &Declaration,
    info: &SectionInfo,
    section: &str,
) -> (usize, String) {
    if matches!(decl.source, DeclarationSource::Json { .. })
        && let Some(value) = &info.value
    {
        return (value.column, value.source_slice.clone());
    }
    // §FS-lsp.1.2.4: a relative heading's range is the `.<component>` it writes.
    let needle = match section.rsplit_once('.') {
        Some((_, last)) if info.relative => format!(".{last}"),
        _ if info.relative => format!(".{section}"),
        _ => section.to_string(),
    };
    let (column, mut text) = heading_span_parts(files, &decl.file, info.line, &needle);
    if info.value_root.is_some()
        && let Some(marker) = text.rfind(&format!(" {EMBEDDED_VALUE_MARKER}"))
    {
        // The marker is authored raw source but not part of the section's
        // semantic title or editor selection (§FS-values.2.4.2, §FS-lsp.1.3).
        text.truncate(marker);
    }
    (column, text)
}

/// The 1-based start column and title text of a heading-line token: the span
/// from `needle` (the rendered ID for a declaration, the section number for a
/// section heading) to where the heading's rendered text ends — the `--toc`
/// title's end, read off the heading the scan recorded rather than the line
/// again, so a C-family `*/` and a Markdown heading's closing hashes are not
/// part of it (§FS-lsp.1.3.4). Falls back to column 1 and the bare `needle`
/// when the scan recorded no heading there.
pub(super) fn heading_span_parts(
    files: &FileRecords,
    file: &Path,
    line: usize,
    needle: &str,
) -> (usize, String) {
    files
        .get(file)
        .and_then(|record| {
            let heading = record.heading_at(line)?;
            let line_start = record.line_start(line)?;
            let text = record.text();
            let rendered = &text[heading.text.clone()];
            let start = match rendered.find(needle) {
                Some(at) => heading.text.start + at,
                None => {
                    heading.span.start + text[heading.span.start..heading.text.end].find(needle)?
                }
            };
            let title = text
                .get(start..heading.text.end)
                .filter(|title| !title.is_empty())
                .unwrap_or(needle)
                .to_string();
            Some((start - line_start + 1, title))
        })
        .unwrap_or_else(|| (1, needle.to_string()))
}

pub(super) fn absolutize_path(path: &Path) -> PathBuf {
    canonicalize_existing_prefix(path)
}
