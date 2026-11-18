//! Formatter protection from the scanner's binding classifications and check's
//! authority predicates, never a separate binding grammar (§FS-values.8).

use std::collections::BTreeSet;
use std::path::Path;

use crate::checker::{
    binding_target_has_any_value_authority, binding_target_reports_invalid_attempt,
};
use crate::config::Config;
use crate::grammar::{LineLinks, MarkdownLineCitation};
use crate::model::Catalog;
use crate::resolver::WorkspaceContext;
use crate::scanner::markdown_value_binding_candidates;

/// Marker offsets whose citation bytes the link pass must preserve (§FS-values.8).
/// Same-line forms are read by the scanner on the current normalized bytes,
/// so earlier rewrites cannot move protection onto another occurrence. Split
/// attempts come from the whole-file scan, which already judged adjacency and
/// text-run boundaries. Their citation line starts with only continuation
/// prefixes and `(` before the marker, so normalization cannot move that marker.
pub(super) fn protected_value_citations(
    line: &str,
    links: LineLinks<'_>,
    path: &Path,
    citations: &[MarkdownLineCitation],
    config: &Config,
    findings: &Catalog,
    workspace: Option<&WorkspaceContext>,
) -> BTreeSet<usize> {
    if citations.is_empty() {
        return BTreeSet::new();
    }
    let target = |namespace: Option<&str>| match namespace {
        Some(alias) => workspace
            .and_then(|workspace| workspace.project_by_alias(alias))
            .map(|project| (&project.findings, project.config.schema())),
        None => Some((findings, config.schema())),
    };
    let mut protected = BTreeSet::new();
    // §FS-values.8: a split refusal's diagnostic points at the earlier literal,
    // while its citation address identifies just the occurrence to preserve.
    for site in &findings.invalid_value_bindings {
        if site.file != path {
            continue;
        }
        let Some((citation_line, column)) = site.binding_citation else {
            continue;
        };
        if links.line_number() == Some(citation_line)
            && site.line < citation_line
            && let Some(id) = &site.id
            && target(site.binding_namespace.as_deref()).is_some_and(|(catalog, schema)| {
                binding_target_reports_invalid_attempt(
                    catalog,
                    schema,
                    id,
                    site.binding_section.as_deref(),
                )
            })
        {
            protected.insert(column - 1);
        }
    }
    if !line.contains('`') {
        return protected;
    }
    let workspace_targets = workspace
        .map(WorkspaceContext::citation_targets)
        .unwrap_or_default();
    let candidates = markdown_value_binding_candidates(
        line,
        links,
        path,
        config.schema(),
        config.frame(),
        config.frame().reading(config.schema(), path).lexical,
        citations,
        &workspace_targets,
    );
    for binding in &candidates.value_bindings {
        if target(binding.namespace.as_deref()).is_some_and(|(catalog, schema)| {
            binding_target_has_any_value_authority(
                catalog,
                schema,
                &binding.id,
                binding.section.as_deref(),
            ) || binding_target_reports_invalid_attempt(
                catalog,
                schema,
                &binding.id,
                binding.section.as_deref(),
            )
        }) {
            protected.insert(binding.citation_column - 1);
        }
    }
    for site in &candidates.invalid_value_bindings {
        if let Some(id) = &site.id
            && target(site.binding_namespace.as_deref()).is_some_and(|(catalog, schema)| {
                binding_target_reports_invalid_attempt(
                    catalog,
                    schema,
                    id,
                    site.binding_section.as_deref(),
                )
            })
            && let Some((_, column)) = site.binding_citation
        {
            protected.insert(column - 1);
        }
    }
    protected
}
