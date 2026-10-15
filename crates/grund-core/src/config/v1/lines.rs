//! The v1 line pass's own record of the file (§AR-config.8.1): each line as the
//! section walk in `parse.rs` met it — blank, a comment, a table header, or a key
//! with its section, its value and any trailing comment. Migration reads the key
//! lines and the comments from here rather than parsing the file a second time
//! (§FS-config-migrate.4.3); ordinary readers skip capture entirely.
//!
//! The pass also carries the one tolerance only migration asks for: a
//! path-shaped `[scan] exclude` entry is read rather than refused, so it can be
//! diagnosed (§FS-config-migrate.3.1.4).

use super::values::strip_comment;

/// What one line of a v1 file is (§FS-config-migrate.terms).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LineKind {
    Blank,
    /// A line holding only a `#` comment.
    Comment,
    /// `[name]`, or `[[name]]` when `array`, with any trailing comment.
    Header {
        name: String,
        array: bool,
        trailing: Option<String>,
    },
    /// `key = value` inside `section` (`""` before any header), with the
    /// trailing `#` comment the line carries, if any.
    Key {
        section: String,
        key: String,
        value: String,
        trailing: Option<String>,
    },
}

/// One line of the file, 1-based, with its raw text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LineRecord {
    pub(crate) line: usize,
    pub(crate) raw: String,
    pub(crate) kind: LineKind,
}

/// The shared capture policy (§AR-config.8.1): the default pass neither
/// constructs nor retains migration's records and reads strictly.
#[derive(Clone, Debug, Default)]
pub(crate) struct LinePass {
    capture: bool,
    /// Read a path-shaped `[scan] exclude` entry instead of refusing it.
    pub(crate) tolerant: bool,
    pub(crate) lines: Vec<LineRecord>,
}

impl LinePass {
    /// Migration explicitly requests complete records (§AR-config.8.1) and
    /// tolerant exclusion reads (§FS-config-migrate.3.1.4) in the same pass.
    pub(crate) fn tolerant() -> Self {
        Self {
            capture: true,
            tolerant: true,
            lines: Vec::new(),
        }
    }

    /// Record line `line`, `raw`, met inside `section`, only when requested;
    /// ordinary reads skip all record construction (§AR-config.8.1).
    pub(super) fn see(&mut self, line: usize, raw: &str, section: &str) {
        if !self.capture {
            return;
        }
        let code = strip_comment(raw);
        let trimmed = code.trim();
        let trailing = raw[code.len()..].trim();
        let trailing = (!trailing.is_empty()).then(|| trailing.to_string());
        let kind = if trimmed.is_empty() {
            if raw.trim().is_empty() {
                LineKind::Blank
            } else {
                LineKind::Comment
            }
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            LineKind::Header {
                name: trimmed.trim_matches(['[', ']']).to_string(),
                array: trimmed.starts_with("[[") && trimmed.ends_with("]]"),
                trailing,
            }
        } else {
            let (key, value) = trimmed.split_once('=').unwrap_or((trimmed, ""));
            LineKind::Key {
                section: section.to_string(),
                key: key.trim().to_string(),
                value: value.trim().to_string(),
                trailing,
            }
        };
        self.lines.push(LineRecord {
            line,
            raw: raw.to_string(),
            kind,
        });
        #[cfg(test)]
        super::tests_capture_contract::record_created();
    }
}
