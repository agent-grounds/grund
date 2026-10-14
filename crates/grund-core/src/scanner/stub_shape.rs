//! What a stub looks like (§FS-declarations.stubs.shape, §AR-scanner.4.7): the
//! one predicate the file pass asks of a heading's file, and the note on every
//! stub-shaped heading the home test of grund 0.19.0 reads differently, which
//! `check` reports as `stub-shape` (§FS-declarations.checks.stub-shape).

use std::path::Path;

use super::context::in_some_home;
use crate::config::{Frame, Schema};
use crate::grammar::STUB_LINK_HEADING;
use crate::model::{Catalog, StubShapeNote, StubShapeReading};

/// Whether a stub-shaped heading in `path` is a stub until 0.19.0: a directory
/// named `docs` anywhere on the whole path (§FS-declarations.stubs.shape). The
/// walk's pass and `scan_unwalked_file` both ask it through the file pass.
pub(super) fn stub_by_docs(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "docs")
}

/// Note every heading of the stub shape in this file's `findings` that the home
/// test reads differently from `stub_by_docs` (§AR-scanner.4.7). The home test is
/// asked only of a Markdown file holding such a heading, and reads nothing the
/// scan has not read: the kind table and the path.
pub(super) fn note_stub_shapes(
    path: &Path,
    schema: &Schema,
    frame: Frame<'_>,
    findings: &mut Catalog,
) {
    // §FS-config-v2.schema.sources.extensions: a stub is Markdown by the file's language.
    if !frame.compiled.reading(schema, path).markdown {
        return;
    }
    // A heading of the stub shape is a stub where the `docs` test holds, and a
    // declaration whose title, as the pass trimmed it, is still one link where
    // it does not.
    let by_docs = stub_by_docs(path);
    let shaped = findings
        .declarations
        .values()
        .flatten()
        .filter_map(|decl| {
            if by_docs {
                return decl
                    .is_stub
                    .then_some((decl, StubShapeReading::StubOnlyByDocs));
            }
            let title = decl.title.as_deref()?;
            let tail = format!(":{title}");
            let path = STUB_LINK_HEADING.captures(&tail)?["path"].to_string();
            Some((decl, StubShapeReading::StubByHome { path }))
        })
        .collect::<Vec<_>>();
    if shaped.is_empty() || by_docs == in_some_home(path, schema, frame) {
        return;
    }
    let notes = shaped
        .into_iter()
        .map(|(decl, reading)| StubShapeNote {
            file: decl.file.clone(),
            line: decl.line,
            id: decl.id.clone(),
            reading,
        })
        .collect::<Vec<_>>();
    findings.stub_shape_notes.extend(notes);
}
