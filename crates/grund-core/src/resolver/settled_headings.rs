//! The heading lines a `fmt` run leaves on disk where they differ from the lines
//! it read (§FS-fmt.6.3.3): a run that expands a citation inside a section heading
//! moves that heading's anchor, so every link the same run writes to it is
//! derived from the line it writes, not from the scan's record of the line it
//! read. The rewriting is the writer's; this is only the record the anchor
//! derivation reads, which is why it sits beside it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::grammar::{atx_heading_text, reduce_heading_text, section_anchor_text};
use crate::model::{FileRecord, RecordedHeading, SectionInfo};

/// One file's rewritten heading lines, by 1-indexed line.
pub(crate) type SettledLines = BTreeMap<usize, String>;

/// The rewritten heading lines of a run, by the path the scan read each file
/// under. Empty for every reader that is not `fmt`: the tree as it stands is
/// then the tree the anchor names.
#[derive(Default)]
pub(crate) struct SettledHeadings {
    files: BTreeMap<PathBuf, SettledLines>,
}

impl SettledHeadings {
    /// Record that the run leaves `line` of `file` as `text`.
    pub(crate) fn insert(&mut self, file: &Path, line: usize, text: String) {
        self.files
            .entry(file.to_path_buf())
            .or_default()
            .insert(line, text);
    }

    /// Fold another project's record in: a workspace run links across projects.
    pub(crate) fn extend(&mut self, other: SettledHeadings) {
        for (file, lines) in other.files {
            self.files.entry(file).or_default().extend(lines);
        }
    }

    /// The settled lines of one file, if the run rewrites any of its headings.
    pub(crate) fn of_file(&self, file: &Path) -> Option<&SettledLines> {
        self.files.get(file)
    }

    /// The anchor text of section `sec`, recorded as `info` in `file`, as the run
    /// leaves its heading — derived as the scanner derives it from a line it
    /// reads (§AR-scanner.2.2) — or `None` where the run leaves the heading as is.
    pub(crate) fn section_title(
        &self,
        file: &Path,
        sec: &str,
        info: &SectionInfo,
    ) -> Option<String> {
        let line = self.of_file(file)?.get(&info.line)?;
        Some(if info.relative {
            // §FS-declarations.section.relative.shown: anchored as written and rendered.
            reduce_heading_text(atx_heading_text(line.trim_start(), info.heading_level))
        } else {
            section_anchor_text(line, sec)
        })
    }
}

/// `heading`'s rendered text in `record` as the run leaves it: the settled line's
/// where the run rewrites it, the scan's record otherwise
/// (§FS-declarations.heading.rendered-text).
pub(crate) fn settled_heading_text<'a>(
    record: &'a FileRecord,
    settled: Option<&'a SettledLines>,
    heading: &RecordedHeading,
) -> &'a str {
    match settled.and_then(|lines| lines.get(&heading.line)) {
        Some(line) if heading.atx => atx_heading_text(line.trim_start(), heading.level),
        _ => record.heading_text(heading),
    }
}
