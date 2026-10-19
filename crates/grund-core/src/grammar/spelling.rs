//! Presentation belongs to a source record, never numeric identity
//! (§FS-config.3.2.8). All catalog consumers select through the same pairing.

use std::collections::BTreeMap;

use super::{Grammar, render_id};
use crate::model::{Catalog, Citation, Declaration, Id, ValueBinding, paired_declarations};

/// The selected declaration's spelling, including a stub's real target
/// (§FS-config.3.2.8). Duplicate identities retain their ordinary ambiguity;
/// enumeration uses the first record in the pairing's established site order.
pub(crate) fn render_catalog_id(
    grammar: &Grammar,
    declarations: &BTreeMap<Id, Vec<Declaration>>,
    id: &Id,
) -> String {
    declarations
        .get(id)
        .and_then(|decls| paired_declarations(decls).records().next())
        .map_or_else(
            || render_id(grammar, id),
            |decl| render_declaration_id(grammar, decl),
        )
}

/// A particular report row or heading uses that record's own source spelling
/// (§FS-config.3.2.8, §FS-fmt.6.2.2).
pub(crate) fn render_declaration_id(grammar: &Grammar, declaration: &Declaration) -> String {
    declaration
        .authored_id
        .clone()
        .unwrap_or_else(|| render_id(grammar, &declaration.id))
}

/// A full source citation retains its own digits, even when unresolved
/// (§FS-config.3.2.8). Local/shorthand expansions use their resolved identity.
pub(crate) fn render_citation_id(
    grammar: &Grammar,
    declarations: &BTreeMap<Id, Vec<Declaration>>,
    citation: &Citation,
    marker: &str,
) -> String {
    if citation.local_section || citation.shorthand {
        return render_catalog_id(grammar, declarations, &citation.id);
    }
    let raw = citation
        .text
        .strip_prefix('<')
        .and_then(|raw| raw.strip_prefix(marker)?.strip_prefix('>'))
        .or_else(|| citation.text.strip_prefix(marker))
        .unwrap_or(&citation.text);
    let raw = citation
        .namespace
        .as_ref()
        .and_then(|alias| raw.strip_prefix(alias)?.strip_prefix('/'))
        .unwrap_or(raw);
    render_input_id(grammar, raw, &citation.id)
}

/// Preserve the ID part of a parsed source argument (§FS-config.3.2.8).
pub(crate) fn render_input_id(grammar: &Grammar, raw: &str, id: &Id) -> String {
    grammar
        .id_input_re
        .captures(raw)
        .and_then(|caps| caps.name("id").map(|id| id.as_str().to_string()))
        .unwrap_or_else(|| render_id(grammar, id))
}

/// Binding reports borrow their original citation's digits without changing
/// the public binding record (§FS-config.3.2.8). Index once per report; trees
/// without bindings need no index (§GOAL-fast-feedback).
pub(crate) struct BindingSpellings<'a>(BTreeMap<(&'a std::path::Path, usize, usize), &'a Citation>);

impl<'a> BindingSpellings<'a> {
    pub(crate) fn new(catalog: &'a Catalog) -> Self {
        let citations = if catalog.value_bindings.is_empty() {
            &[]
        } else {
            catalog.citations.as_slice()
        };
        Self(
            citations
                .iter()
                .map(|cite| ((cite.file.as_path(), cite.line, cite.column), cite))
                .collect(),
        )
    }

    pub(crate) fn render(
        &self,
        grammar: &Grammar,
        declarations: &BTreeMap<Id, Vec<Declaration>>,
        binding: &ValueBinding,
        marker: &str,
    ) -> String {
        self.0
            .get(&(
                binding.file.as_path(),
                binding.line,
                binding.citation_column,
            ))
            .map_or_else(
                || render_catalog_id(grammar, declarations, &binding.id),
                |cite| render_citation_id(grammar, declarations, cite, marker),
            )
    }
}
