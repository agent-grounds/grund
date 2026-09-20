//! The spans and targets one snapshot row carries (§AR-system.2.9,
//! §AR-lsp.5.1): the query ID an editor navigates by, where a citation or a stub
//! resolves to, and the exact column and text of a declaration, section or
//! heading title (§FS-lsp.1.1, §FS-lsp.1.3).
//!
//! Beside `lsp_snapshot.rs` rather than in it, per §AR-core-module-layout.3's
//! file budget: the walk there decides *which* rows exist, and each answer here
//! is a function of one row.

use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{
    Citation, Declaration, DeclarationSource, SectionInfo, TextOverlays,
    canonicalize_existing_prefix, is_stub_for_inline_decl, paths_same_location,
    resolve_stub_target, sort_path_key,
};
use crate::resolver::{WorkspaceContext, WorkspaceProject};
use crate::scanner::{EMBEDDED_VALUE_MARKER, overlay_text};

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
    let mut homes: Vec<&Declaration> = decls
        .iter()
        .filter(|decl| !is_stub_for_inline_decl(&target_project.config.root, decl, decls))
        .collect();
    homes.sort_by(|a, b| (sort_path_key(&a.file), a.line).cmp(&(sort_path_key(&b.file), b.line)));
    let home = homes.first().copied().or_else(|| decls.first())?;
    Some((home.file.clone(), home.line))
}

pub(super) fn lsp_target_for_stub(
    project: &WorkspaceProject,
    stub: &Declaration,
    decls: &[Declaration],
) -> Option<(PathBuf, usize)> {
    let target = stub.defined_in.as_ref()?;
    let resolved = resolve_stub_target(&project.config.root, &stub.file, target);
    let inline = decls
        .iter()
        .find(|decl| paths_same_location(&decl.file, &resolved) && decl.file != stub.file)?;
    Some((inline.file.clone(), inline.line))
}

pub(super) fn declaration_range_parts(
    decl: &Declaration,
    rendered_id: &str,
    overlays: &TextOverlays,
) -> (usize, String) {
    match &decl.source {
        DeclarationSource::Json {
            key_column,
            key_text,
            ..
        } => (*key_column, key_text.clone()),
        DeclarationSource::Text => heading_span_parts(&decl.file, decl.line, rendered_id, overlays),
    }
}

pub(super) fn section_range_parts(
    decl: &Declaration,
    info: &SectionInfo,
    section: &str,
    overlays: &TextOverlays,
) -> (usize, String) {
    if matches!(decl.source, DeclarationSource::Json { .. })
        && let Some(value) = &info.value
    {
        return (value.column, value.source_slice.clone());
    }
    let (column, mut text) = heading_span_parts(&decl.file, info.line, section, overlays);
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
/// section heading) to the end of the trimmed line. Falls back to column 1 and
/// the bare `needle` when the line cannot be read.
pub(super) fn heading_span_parts(
    file: &Path,
    line: usize,
    needle: &str,
    overlays: &TextOverlays,
) -> (usize, String) {
    overlay_text(overlays, file)
        .map(str::to_string)
        .or_else(|| fs::read_to_string(file).ok())
        .and_then(|text| text.lines().nth(line.saturating_sub(1)).map(str::to_string))
        .and_then(|line| {
            let start = line.find(needle)?;
            let end = line.trim_end().len();
            let text = line
                .get(start..end)
                .filter(|text| !text.is_empty())
                .unwrap_or(needle)
                .to_string();
            Some((start + 1, text))
        })
        .unwrap_or_else(|| (1, needle.to_string()))
}

pub(super) fn absolutize_path(path: &Path) -> PathBuf {
    canonicalize_existing_prefix(path)
}
