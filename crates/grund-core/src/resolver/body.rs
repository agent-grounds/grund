//! Declaration-body extraction (§AR-system.2.10): where a declaration's body
//! begins and ends in the file that holds it, across Markdown and every
//! supported comment dialect (§AR-scanner.2.4.1, §AR-scanner.2.2, §FS-show.2.3).
//!
//! This is a question about *source structure* asked of what a scan already
//! recorded — the same question §AR-scanner.2.4.1 answers for the citing side — so
//! it is a function of the loaded findings rather than of any one command's
//! rendering (§AR-resolver.placement). `queries/show.rs` keeps what is genuinely
//! rendering: the entry points, the JSON shapes, and the refusals.
//!
//! It reads the file record the scan kept (§AR-scanner.2.8), never the file: the
//! slice is today's, walked once over the record for every view a mode asks for
//! (`scanner::today_slices`, §DF-one-parse-answers-from-0-19). What is left here
//! is choosing the views and composing them; the text helpers that reshape a
//! finished body sit in `body_lines.rs`.

use anyhow::{Result, anyhow};

use crate::config::{Frame, Schema};
use crate::grammar::render_id;
use crate::model::{Declaration, FileRecords, Id, ShowOutput, ShowRenderMode};
use crate::scanner::{TodayMiss, TodaySink, TodaySite, today_slices};

use super::body_lines::{join_with_blank, truncate_to_first_paragraph};

/// Pull the body text of a declaration out of its file: the lines under the
/// `# <ID>: …` heading down to the next same-or-shallower heading (§AR-scanner.2.4.1),
/// optionally just one citable subsection (§AR-scanner.2.2) or just the lead
/// paragraph (§AR-system.2.10). For an inline declaration in a code/`"""` doc-comment
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
    let section_line =
        section.and_then(|path| declaration.sections.get(path).map(|info| info.line));
    let site = site_of(declaration, section.is_some(), section_line);
    let views = match mode {
        // `--toc` = the default lead, then a blank line, then the nested section
        // headings (§AR-system.2.10), both out of the one walk.
        ShowRenderMode::Toc => vec![
            TodaySink::new(ShowRenderMode::Default, include_heading),
            TodaySink::new(ShowRenderMode::Outline, false),
        ],
        // `--brief` = heading + first paragraph (§AR-system.2.10); the H1 is
        // suppressed when a section is selected.
        ShowRenderMode::Brief => vec![TodaySink::new(ShowRenderMode::Default, section.is_none())],
        mode => vec![TodaySink::new(mode, include_heading)],
    };
    let (line, views) = slice_site(files, id, declaration, site, section, views, schema, frame)?;
    let mut views = views.into_iter().map(TodaySink::finish);
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

/// The scanner-recorded site a slice is pinned to. `show` supplies it after
/// establishing uniqueness, so an assigned scanner body span bounds the slice too;
/// size rows additionally use it to keep duplicate homes and duplicate section
/// coordinates site-local (§AR-scanner.2.2.5, §FS-list.3.4). The body
/// end is the catalog's only where it published one: a read-only scan without
/// sections keeps its single-line placeholder and the slicer derives that
/// declaration's boundary itself (§AR-benchmarks).
pub(super) fn site_of(
    declaration: &Declaration,
    sectioned: bool,
    section_line: Option<usize>,
) -> TodaySite {
    TodaySite {
        declaration_line: declaration.line,
        declaration_body_end: (sectioned || declaration.body_end > declaration.line)
            .then_some(declaration.body_end),
        section_line,
    }
}

/// One walk of the declaration's record into `sinks` (§AR-system.2.10), with the
/// line the slice starts at, or the refusal a missing declaration or section is.
#[allow(clippy::too_many_arguments)]
pub(super) fn slice_site(
    files: &FileRecords,
    id: &Id,
    declaration: &Declaration,
    site: TodaySite,
    section: Option<&str>,
    mut sinks: Vec<TodaySink>,
    schema: &Schema,
    frame: Frame<'_>,
) -> Result<(usize, Vec<TodaySink>)> {
    let record = files
        .get(&declaration.file)
        .ok_or_else(|| anyhow!("read {}: not scanned", declaration.file.display()))?;
    let lexical = frame.compiled.lexical(schema);
    match today_slices(record, site, section, &mut sinks, lexical) {
        Ok(line) => Ok((line, sinks)),
        Err(TodayMiss::Declaration) => {
            Err(anyhow!("ID not found: {}", render_id(frame.grammar(), id)))
        }
        Err(TodayMiss::Section) => Err(anyhow!(
            "section not found: {}{}{}",
            render_id(frame.grammar(), id),
            schema.ids.section_separator,
            section.unwrap_or("")
        )),
    }
}
