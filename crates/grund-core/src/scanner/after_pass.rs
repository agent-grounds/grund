//! Which of the per-file pass's closing steps a file needs (§AR-scanner.2.4):
//! asked once every line is read, of the declarations the pass found and the
//! kinds the schema declares, so a file holding none of what a step reads never
//! pays for it (§AR-benchmarks).

use crate::config::{Schema, kind_uses_values, kind_value_chapter};
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
}
