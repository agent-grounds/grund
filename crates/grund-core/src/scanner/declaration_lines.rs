//! The declaration lines of a file as the scan reads them, for the readers that
//! hold its text but not its record: the formatter's protection of declaration
//! lines and the editor's on-type rule (§FS-fmt.2.3, §FS-lsp.1.4.3). Each line
//! is asked the scan's own questions in the scan's order: whether the gate lets
//! it declare (§FS-config-v2.schema.sources.declarations.gate), and whether it is
//! a bare line of a closed `/* … */` block (§FS-declarations.line.block-comment).
//! Both are read off the one block walk the scan records, so a line the scan
//! declares nothing on is never protected as a declaration.

use super::record::walk_blocks;
use crate::config::FileReading;
use crate::grammar::{declaration_id_on_source_line, is_bare_block_line};
use crate::model::{BlockForm, Id};

/// For every line of one file, `(may declare, bare block line)`.
pub(crate) struct DeclarationLines<'a> {
    reading: FileReading<'a>,
    lines: Vec<(bool, bool)>,
}

impl<'a> DeclarationLines<'a> {
    pub(crate) fn of(text: &str, reading: FileReading<'a>) -> Self {
        let lines = text.lines().collect::<Vec<_>>();
        let mut flags = vec![(reading.line_may_declare(None), false); lines.len()];
        if !reading.markdown {
            for block in walk_blocks(&lines, &reading) {
                for index in block.start - 1..block.end {
                    let bare = block.form == BlockForm::Block
                        && is_bare_block_line(lines[index], block.closed);
                    flags[index] = (reading.line_may_declare(Some(block.declares)), bare);
                }
            }
        }
        Self {
            reading,
            lines: flags,
        }
    }

    /// The ID line `index` (0-based) declares as the scan reads it, `text` being
    /// that line's content (a docstring line's interior): `None` on a line the
    /// gate keeps from declaring, whatever it looks like.
    pub(crate) fn declared(
        &self,
        index: usize,
        text: &str,
        in_py_docstring: bool,
    ) -> Option<(Id, usize)> {
        let (may_declare, bare) = self.lines.get(index).copied().unwrap_or_default();
        may_declare
            .then(|| {
                declaration_id_on_source_line(
                    self.reading.lexical.grammar,
                    text,
                    in_py_docstring,
                    bare,
                    self.reading.markdown,
                )
            })
            .flatten()
    }
}
