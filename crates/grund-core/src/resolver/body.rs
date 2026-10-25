//! Declaration-body extraction (§AR-system.2.10): where a declaration's body
//! begins and ends in the file that holds it, across Markdown and every
//! supported comment dialect (§FS-declarations.body, §FS-declarations.section, §FS-show.2.3).
//!
//! This is a question about *source structure* asked of what a scan already
//! recorded — the same question §AR-scanner.2.4.1 answers for the citing side — so
//! it is a function of the loaded findings rather than of any one command's
//! rendering (§AR-resolver.placement). `queries/show.rs` keeps what is genuinely
//! rendering: the entry points, the JSON shapes, and the refusals.
//!
//! It reads the file record the scan kept (§AR-scanner.2.8), never the file: every
//! view a mode asks for is sliced at the extents the record holds
//! (`scanner::record_slices`, §DF-one-parse-answers-from-0-19). What is left here
//! is choosing the views and composing them; the text helpers that reshape a
//! finished body sit in `body_lines.rs`.

use anyhow::{Result, anyhow};

use crate::config::{Frame, Schema};
use crate::grammar::render_declaration_id;
use crate::model::{Declaration, FileRecords, Id, ShowOutput, ShowRenderMode};
use crate::scanner::{SliceMiss, SliceSink, record_slices};

use super::body_lines::{join_with_blank, truncate_to_first_paragraph};

/// Pull the body text of a declaration out of its file: the lines under the
/// `# <ID>: …` heading down to the next same-or-shallower heading (§FS-declarations.body),
/// optionally just one citable subsection (§FS-declarations.section) or just the lead
/// paragraph (§FS-show.2.1.1). For an inline declaration in a code/`"""` doc-comment
/// this walks the comment block (§FS-show.2.3.1) and strips comment markers
/// (§FS-show.2.3.2) before returning the text.
///
/// Why `--brief` always keeps a heading: the slice has to be self-labeled whatever
/// the output format, `text` or `md`, so it carries the H1 for a whole declaration
/// and the section heading for a selected section. When a section is selected the
/// H1 is suppressed — only the most specific heading is kept.
///
/// Why a fence suspends structure: inside a Markdown fence nothing is structure —
/// not a section heading, not a declaration heading. The record holds only the
/// headings §AR-scanner.2.2's scan met outside a fence, so the section map `check`
/// reads and the body this returns are bounded by the same lines.
///
/// Why a repeated section path terminates the section: a *second* heading claiming
/// the requested path does not continue the section, it ends it the way a sibling
/// heading does, so two bodies are never merged into a slice no heading spans. Such
/// a query does not reach here at all — it is refused from the scanner's record
/// before the body is read (`ambiguous_section_refusal`). Before the target section
/// is found, unrelated headings are scanned past rather than ending anything.
pub(crate) fn extract_declaration_body(
    files: &FileRecords,
    id: &Id,
    declaration: &Declaration,
    section: Option<&str>,
    mode: ShowRenderMode,
    include_heading: bool,
    schema: &Schema,
    frame: Frame<'_>,
) -> Result<ShowOutput> {
    let views = match mode {
        // `--toc` = the default lead, then a blank line, then the nested section
        // headings (§FS-show.2.1.2), both out of the one walk.
        ShowRenderMode::Toc => vec![
            SliceSink::new(ShowRenderMode::Default, include_heading),
            SliceSink::new(ShowRenderMode::Outline, false).canonical(),
        ],
        // `--brief` = heading + first paragraph (§FS-show.2.1.1); the H1 is
        // suppressed when a section is selected.
        ShowRenderMode::Brief => vec![SliceSink::new(ShowRenderMode::Default, section.is_none())],
        mode => vec![SliceSink::new(mode, include_heading)],
    };
    let (line, views) = slice_site(files, id, declaration, section, None, views, schema, frame)?;
    let mut views = views.into_iter().map(SliceSink::finish);
    let (body, sections) = views.next().unwrap_or_default();
    let (body, sections) = match mode {
        ShowRenderMode::Toc => {
            let (outline, sections) = views.next().unwrap_or_default();
            (join_with_blank(&body, &outline), sections)
        }
        ShowRenderMode::Brief => (truncate_to_first_paragraph(&body), sections),
        _ => (body, sections),
    };
    Ok(ShowOutput {
        body,
        path: declaration.file.clone(),
        line,
        json: None,
        sections,
    })
}

/// One slice of the declaration's record into `sinks` (§FS-show.2.1), with the
/// line the slice starts at, or the refusal a missing declaration or section is.
/// `claimant` pins which claimant of `section` is sliced (§FS-list.3.4).
#[allow(clippy::too_many_arguments)]
pub(super) fn slice_site(
    files: &FileRecords,
    _id: &Id,
    declaration: &Declaration,
    section: Option<&str>,
    claimant: Option<usize>,
    mut sinks: Vec<SliceSink>,
    schema: &Schema,
    frame: Frame<'_>,
) -> Result<(usize, Vec<SliceSink>)> {
    let record = files
        .get(&declaration.file)
        .ok_or_else(|| anyhow!("read {}: not scanned", declaration.file.display()))?;
    match record_slices(record, declaration, section, claimant, &mut sinks) {
        Ok(line) => Ok((line, sinks)),
        Err(SliceMiss::Declaration) => Err(anyhow!(
            "ID not found: {}",
            render_declaration_id(frame.grammar(), declaration)
        )),
        Err(SliceMiss::Section) => Err(anyhow!(
            "section not found: {}{}{}",
            render_declaration_id(frame.grammar(), declaration),
            schema.ids.section_separator,
            section.unwrap_or("")
        )),
    }
}
