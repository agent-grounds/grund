//! Which of the per-file pass's closing steps a file needs (§AR-scanner.2.4):
//! asked once every line is read, of the declarations the pass found and the
//! kinds the schema declares, so a file holding none of what a step reads never
//! pays for it (§AR-benchmarks), and the value validations among them, run here.

use std::path::Path;

use super::chapter_values::validate_declared_value_chapters;
use super::embedded_values::validate_embedded_value_roots;
use super::value_context::SourceValueLineContext;
use super::values::validate_markdown_value_declarations;
use crate::config::{Frame, Schema, kind_uses_values, kind_value_chapter};
use crate::model::Catalog;

/// The closing steps `scan_file_text` runs for one file.
pub(super) struct AfterPass {
    /// A declaration in the file holds a section.
    pub(super) text_sections: bool,
    /// A section in the file roots embedded values (§FS-values.2.4).
    pub(super) embedded_roots: bool,
    /// A declaration in the file is of a kind that names a value chapter.
    pub(super) declared_chapters: bool,
    /// A declaration in the file is of a `values = true` kind (§FS-values.2.1).
    pub(super) value_declarations: bool,
}

impl AfterPass {
    /// What `findings` — the catalog the per-file pass just filled — asks of
    /// the closing steps, under this `schema`. `scan_values` is the pass's own
    /// answer to whether value authority is on at all (§FS-values.1).
    pub(super) fn of(findings: &Catalog, schema: &Schema, scan_values: bool) -> Self {
        let declarations = || findings.declarations.values().flatten();
        Self {
            text_sections: declarations()
                .any(|decl| !decl.sections.is_empty() || !decl.duplicate_sections.is_empty()),
            embedded_roots: declarations()
                .flat_map(|decl| decl.sections.values())
                .any(|section| section.value_root.is_some()),
            // §FS-values.2.5: the declared chapter is strict at its own level whether
            // or not it managed to hold a single root, so this is asked of the kind
            // rather than of what enrollment found.
            declared_chapters: declarations()
                .any(|decl| kind_value_chapter(schema, &decl.id.kind).is_some()),
            value_declarations: scan_values
                && declarations().any(|decl| kind_uses_values(schema, &decl.id.kind)),
        }
    }
    /// Run the value validations this file is due: declared value kinds
    /// (§FS-values.2.1), declared value chapters (§FS-values.2.5) and embedded
    /// value roots (§FS-values.2.4.3).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn validate_values(
        &self,
        path: &Path,
        text: &str,
        is_md: bool,
        is_py: bool,
        schema: &Schema,
        frame: Frame<'_>,
        source_contexts: Option<&[Option<SourceValueLineContext>]>,
        findings: &mut Catalog,
    ) {
        if self.value_declarations {
            validate_markdown_value_declarations(path, text, is_md, schema, frame, findings);
        }
        if self.declared_chapters {
            validate_declared_value_chapters(
                path,
                text,
                is_md,
                is_py,
                schema,
                frame,
                source_contexts,
                findings,
            );
        }
        if self.embedded_roots {
            validate_embedded_value_roots(
                path,
                text,
                is_md,
                is_py,
                schema,
                frame,
                source_contexts,
                findings,
            );
        }
    }
}
