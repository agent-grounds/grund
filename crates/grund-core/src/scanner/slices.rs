//! A declaration's or section's text, sliced out of the file record (§AR-scanner.2.8.2):
//! the lead, the whole body and the section map `show` and `list --size` print, each
//! bounded where the record's extent rules end it (`extents.rs`) and each line read
//! through the envelope the record reads it with (§FS-languages.comment-wrapper). There
//! is no second reading of where a body ends: the record's is the one every command
//! gives (§DF-one-parse-answers-from-0-19).

use super::embedded_value_context::without_value_tag;
use crate::grammar::{
    comment_envelope, expand_relative_heading, reduce_heading_text, relative_heading_form,
    section_display_name,
};
use crate::model::{
    Declaration, FileRecord, FileRecords, HeadingKind, RecordedHeading, SectionInfo,
    ShowRenderMode, ShowSection, named_section_path,
};

/// One view a slice fills (§FS-show.2.1): the lead (`Default`), the whole body
/// (`Full`) or the section map (`Outline`), with or without the heading line.
pub(crate) struct SliceSink {
    view: ShowRenderMode,
    include_heading: bool,
    canonical: bool,
    lines: Vec<String>,
    sections: Vec<ShowSection>,
}

impl SliceSink {
    pub(crate) fn new(view: ShowRenderMode, include_heading: bool) -> Self {
        Self {
            view,
            include_heading,
            canonical: false,
            lines: Vec::new(),
            sections: Vec::new(),
        }
    }

    /// This sink, printing a relative map heading at its canonical path, as `--toc`
    /// does (§FS-declarations.section.relative.shown, §FS-show.2.1.2).
    pub(crate) fn canonical(mut self) -> Self {
        self.canonical = true;
        self
    }

    /// `line`, a section map heading the slice takes, as this sink prints it.
    fn heading_line(&self, line: String, heading: &RecordedHeading) -> String {
        if self.canonical {
            canonical_heading_line(line, heading)
        } else {
            line
        }
    }

    /// The view's text, blank lines trimmed from both ends and `\n`-terminated,
    /// and the section map an `Outline` view filled.
    pub(crate) fn finish(self) -> (String, Vec<ShowSection>) {
        let first = self.lines.iter().position(|line| !line.trim().is_empty());
        let last = self.lines.iter().rposition(|line| !line.trim().is_empty());
        let body = match (first, last) {
            (Some(first), Some(last)) => format!("{}\n", self.lines[first..=last].join("\n")),
            _ => String::new(),
        };
        (body, self.sections)
    }
}

/// What a slice could not find.
pub(crate) enum SliceMiss {
    Declaration,
    Section,
}

/// Slice `decl`, or its section `section`, into every sink (§FS-show.2.1): the
/// heading the record holds for it, its lead down to the recorded lead end
/// (§FS-declarations.lead), its body down to the recorded end (§FS-declarations.body,
/// §FS-declarations.section), and its section map, the sections and duplicate
/// claimants of `decl` inside that span (§FS-declarations.section.map). `claimant` is
/// the heading line of the claimant of `section` to slice, so a duplicate's size row is
/// its own (§FS-list.3.4); without it the path's first claimant is. Answers the line
/// the slice starts at.
pub(crate) fn record_slices(
    record: &FileRecord,
    decl: &Declaration,
    section: Option<&str>,
    claimant: Option<usize>,
    sinks: &mut [SliceSink],
) -> Result<usize, SliceMiss> {
    let declared = |kind: HeadingKind| move |heading: &&RecordedHeading| heading.kind == kind;
    let decl_heading = record
        .heading_at(decl.line)
        .filter(declared(HeadingKind::Declaration))
        .ok_or(SliceMiss::Declaration)?;
    let top = match section {
        Some(path) => claimant
            .or_else(|| decl.sections.get(path).map(|info| info.line))
            .and_then(|line| record.heading_at(line))
            .filter(|heading| matches!(heading.kind, HeadingKind::Section | HeadingKind::Duplicate))
            .filter(|heading| heading.path.as_deref() == Some(path))
            .ok_or(SliceMiss::Section)?,
        None => decl_heading,
    };
    let base_depth = section.map_or(0, |path| path.split('.').count());
    // §FS-show.2.1.2: the map holds this declaration's own claimants, not a nested one's.
    let own = |line: usize| {
        decl.sections.values().any(|info| info.line == line)
            || decl
                .duplicate_sections
                .iter()
                .any(|(_, info)| info.line == line)
    };
    let map: Vec<&RecordedHeading> = record
        .headings_between(top.line + 1, top.end)
        .iter()
        .filter(|heading| {
            matches!(heading.kind, HeadingKind::Section | HeadingKind::Duplicate)
                && heading.path.is_some()
                && own(heading.line)
        })
        .collect();
    // §FS-values.2.4.2: a marked root's title drops its tag; its raw heading keeps it.
    let marked = |heading: &RecordedHeading, path: &str| {
        decl.sections
            .get(path)
            .is_some_and(|info| info.line == heading.line && info.marked())
    };
    for sink in sinks.iter_mut() {
        // §FS-show.3.1.2: the heading-carrying form opens on the declaration's heading line.
        if sink.include_heading {
            sink.lines.push(body_line(record, decl_heading.line));
        }
        // §FS-show.2.1.1: a selected section always opens on its own heading, outside the map,
        // at its canonical path in every view (§FS-declarations.section.relative.shown.selected).
        if section.is_some() && sink.view != ShowRenderMode::Outline {
            let line = canonical_heading_line(body_line(record, top.line), top);
            sink.lines.push(line);
        }
        match sink.view {
            ShowRenderMode::Outline => {
                for heading in &map {
                    let path = heading.path.as_deref().unwrap_or_default();
                    let line = sink.heading_line(body_line(record, heading.line), heading);
                    sink.sections.push(ShowSection {
                        path: path.to_string(),
                        title: section_title(record, heading, path, marked(heading, path)),
                        depth: path.split('.').count() - base_depth,
                    });
                    sink.lines.push(line);
                }
            }
            view => {
                let last = if view == ShowRenderMode::Default {
                    top.lead_end
                } else {
                    top.end
                };
                sink.lines
                    .extend((top.line + 1..=last).map(|lineno| body_line(record, lineno)));
            }
        }
    }
    Ok(top.line)
}

/// `line`, the heading line the record holds as `heading`, with a relative path
/// replaced by the canonical one the scan recorded for it, never inferred again
/// (§FS-declarations.section.relative.shown): `### .2: Ceiling` at `retry.2` is
/// `### retry.2: Ceiling`. A heading written with its complete path comes back as is.
fn canonical_heading_line(line: String, heading: &RecordedHeading) -> String {
    match heading.path.as_deref() {
        Some(path) if heading.relative => expand_relative_heading(&line, path),
        _ => line,
    }
}

/// Line `lineno` as `show` prints it (§FS-languages.comment-wrapper): a Markdown line
/// as written; a docstring line inside its delimiters; a comment line with the envelope
/// its block's form gives, a C-family closer on a heading line included
/// (§FS-languages.comment-wrapper.headings).
fn body_line(record: &FileRecord, lineno: usize) -> String {
    let line = record.line(lineno).unwrap_or("");
    if record.markdown {
        return line.to_string();
    }
    let scanned = match record.zone_at(lineno) {
        Some(zone) => &line[zone.start..zone.end],
        None => line,
    };
    match record.block_at(lineno) {
        Some(block) => comment_envelope(scanned, &block.form).to_string(),
        None => scanned.to_string(),
    }
}

/// A listed section's title (§FS-rules.8, §FS-coordinate-globs.matching.catalog). A named
/// path is a chapter, titled as the rule unit a selector row shares it with
/// (`section_display_name`). A numbered path is titled by the one rule its `--toc` entry is,
/// `section_title`, over the heading the record holds at its line, reduced as list text is
/// (`reduce_heading_text`), so `### .1. Child` under `## 2. Parent` lists as `Child`.
pub(crate) fn section_row_title(
    files: &FileRecords,
    decl: &Declaration,
    path: &str,
    info: &SectionInfo,
) -> String {
    let display = || section_display_name(&info.title, path, info.relative).into_owned();
    if named_section_path(path) {
        return display();
    }
    let Some(record) = files.get(&decl.file) else {
        return display();
    };
    match record
        .heading_at(info.line)
        .filter(|heading| heading.path.as_deref() == Some(path))
    {
        Some(heading) => reduce_heading_text(&section_title(record, heading, path, info.marked())),
        None => display(),
    }
}

/// A map entry's title: the heading's rendered text, which has no closing `#` run
/// (§FS-declarations.heading), after its path (§FS-show.2.1.2), the canonical one
/// for a relative heading (§FS-declarations.section.relative.shown), without the tag
/// of a `marked` value root (§FS-values.2.4.2).
fn section_title(
    record: &FileRecord,
    heading: &RecordedHeading,
    path: &str,
    marked: bool,
) -> String {
    let text = record.heading_text(heading);
    // The tag goes from the whole heading text, path included, as the editor's
    // range drops it, so a title that is only the tag is empty on both.
    let text = if marked {
        without_value_tag(text)
    } else {
        text
    };
    let rest = match relative_heading_form(text).filter(|_| heading.relative) {
        Some(form) => text
            .trim_start()
            .strip_prefix('.')
            .and_then(|rest| rest.strip_prefix(form.component)),
        None => text.trim_start().strip_prefix(path),
    };
    rest.unwrap_or(text)
        .trim_start_matches(['.', ':'])
        .trim()
        .to_string()
}
