//! The stub-shape warning (§FS-declarations.checks.stub-shape): a heading of the
//! stub shape that the `docs` test of today and the home test of grund 0.19.0
//! read differently, reported at its line in one of two messages. The scanner
//! notes the heading where it read it (§AR-scanner.4.7); this reports the note.

use crate::config::Frame;
use crate::grammar::render_id;
use crate::model::{Catalog, CheckReport, Diagnostic, StubShapeReading};

/// §FS-declarations.checks.stub-shape: one warning per noted heading. It leaves the
/// exit status where the rest of the run put it (§FS-check.2), and nothing else of
/// the heading moves until 0.19.0.
pub(super) fn check_stub_shapes(findings: &Catalog, frame: Frame<'_>, report: &mut CheckReport) {
    report
        .warnings
        .extend(findings.stub_shape_notes.iter().map(|note| {
            let id = render_id(frame.grammar(), &note.id);
            let message = match &note.reading {
                StubShapeReading::StubOnlyByDocs => format!(
                    "{id} is read as a stub only because a directory on its path is named docs, \
                     and that reading is removed in grund 0.19.0, which reads a stub only in a \
                     kind's home — move the file into a kind's folder, or name its folder in \
                     [[kinds]], to keep it a stub"
                ),
                StubShapeReading::StubByHome { path } => format!(
                    "{id} is read as a declaration titled with a link, and that reading is \
                     removed in grund 0.19.0, which reads it as a stub to {path} — give the \
                     heading a title that is not one link to keep it a declaration"
                ),
            };
            Diagnostic {
                code: "stub-shape",
                path: Some(note.file.clone()),
                line: Some(note.line),
                column: None,
                message,
                sites: Vec::new(),
                authority: Vec::new(),
            }
        }));
}
