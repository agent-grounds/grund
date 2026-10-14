//! The v1 file's keys as the reader's line pass recorded them (§FS-config-migrate.4.3):
//! where each authored key sits, and which `[[kinds]]` row each row key belongs
//! to. Migration reads lines from here and never parses the file again.

use crate::config::v1::{LineKind, LineRecord};

/// One `[[kinds]]` table: its header line, its row name and its keys.
pub(super) struct KindsBlock {
    pub(super) header: usize,
    pub(super) name: String,
    pub(super) keys: Vec<(usize, String)>,
}

/// The authored keys of one v1 file, by section.
pub(super) struct Keys<'a> {
    pub(super) lines: &'a [LineRecord],
    pub(super) blocks: Vec<KindsBlock>,
}

impl<'a> Keys<'a> {
    pub(super) fn of(lines: &'a [LineRecord]) -> Self {
        let mut blocks: Vec<KindsBlock> = Vec::new();
        let mut open = false;
        for record in lines {
            match &record.kind {
                LineKind::Header { name, array, .. } => {
                    open = *array && name == "kinds";
                    if open {
                        blocks.push(KindsBlock {
                            header: record.line,
                            name: String::new(),
                            keys: Vec::new(),
                        });
                    }
                }
                LineKind::Key { key, value, .. } if open => {
                    let block = blocks.last_mut().expect("a kinds block is open");
                    if key == "kind" {
                        block.name = unquote(value);
                    }
                    block.keys.push((record.line, key.clone()));
                }
                _ => {}
            }
        }
        Self { lines, blocks }
    }

    /// The line `key` was written at in `section` (`""` before any header), or
    /// `None` when the file omits it.
    pub(super) fn line(&self, section: &str, key: &str) -> Option<usize> {
        self.lines.iter().find_map(|record| match &record.kind {
            LineKind::Key {
                section: s, key: k, ..
            } if s == section && k == key => Some(record.line),
            _ => None,
        })
    }

    /// The line of `key` in the `[[kinds]]` row named `row`.
    pub(super) fn row_line(&self, row: &str, key: &str) -> Option<usize> {
        let block = self.blocks.iter().find(|block| block.name == row)?;
        block
            .keys
            .iter()
            .find(|(_, k)| k == key)
            .map(|(line, _)| *line)
    }

    /// The row a key on `line` belongs to, when it sits in a `[[kinds]]` table.
    pub(super) fn row_at(&self, line: usize) -> Option<&KindsBlock> {
        self.blocks
            .iter()
            .find(|block| block.keys.iter().any(|(at, _)| *at == line) || block.header == line)
    }
}

/// A basic TOML string's contents, the quotes and escapes taken off.
pub(super) fn unquote(value: &str) -> String {
    let inner = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value);
    inner.replace("\\\"", "\"").replace("\\\\", "\\")
}
