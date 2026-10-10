//! Declaration-body extraction (§AR-system.2.10): where a declaration's body
//! begins and ends in the file that holds it, across Markdown and every
//! supported comment dialect (§FS-show.2.1, §FS-show.2.2, §FS-show.2.3).
//!
//! This is a question about *source structure* asked of the spans a scan already
//! recorded — the same question §AR-scanner.2.4.1 answers for the citing side — so
//! it is a function of the loaded findings rather than of any one command's
//! rendering (§AR-resolver.placement). `queries/show.rs` keeps what is genuinely
//! rendering: the entry points, the JSON shapes, and the refusals.
//!
//! It sat in `queries/body.rs` while the queries were the only component that
//! asked, which had the checker's lead-budget rule reading the point-body pair
//! upward out of a sibling's answer (§FS-declarations.checks.oversized-lead, §AR-system.4). The
//! text helpers that came with it out of `queries/show.rs` now sit in `body_lines.rs`
//! and `comment_envelope.rs`.

use anyhow::{Context, Result, anyhow};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config::{Frame, Schema};
use crate::grammar::{
    MarkdownBlocks, PythonDocstringScanState, declaration_id_on_line, render_id, section_path,
    source_scan_line,
};
use crate::model::{Declaration, Id, ShowOutput, ShowRenderMode, TextOverlays};

use crate::scanner::overlay_text;

use super::body_lines::{
    closes_comment_block, join_with_blank, push_outline_section, truncate_to_first_paragraph,
};
use super::comment_envelope::{
    clean_envelope_line, continues_comment_body, is_line_style_comment_line, own_line_marker,
};

/// Pull the body text of a declaration out of its file: the lines under the
/// `# <ID>: …` heading down to the next same-or-shallower heading (§FS-show.2.1),
/// optionally just one citable subsection (§FS-show.2.2) or just the lead
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
/// not a section heading, not a declaration heading. Fences are tracked exactly as
/// §AR-scanner.2.2's scan tracks them, Markdown only, so the section map `check`
/// reads and the body this returns are bounded by the same lines.
///
/// Why a repeated section path terminates the section: a *second* heading claiming
/// the requested path does not continue the section, it ends it the way a sibling
/// heading does, so two bodies are never merged into a slice no heading spans. Such
/// a query does not reach here at all — it is refused from the scanner's record
/// before the body is read (`ambiguous_section_refusal`). Before the target section
/// is found, unrelated headings are scanned past rather than ending anything.
pub(crate) fn extract_declaration_body(
    path: &Path,
    id: &Id,
    declaration: &Declaration,
    section: Option<&str>,
    mode: ShowRenderMode,
    include_heading: bool,
    schema: &Schema,
    frame: Frame<'_>,
    overlays: &TextOverlays,
) -> Result<ShowOutput> {
    let mut cache = PointBodyCache::new(overlays);
    extract_declaration_body_cached(
        &mut cache,
        path,
        id,
        section,
        mode,
        include_heading,
        schema,
        frame,
        Some(PointBodySite {
            declaration_line: declaration.line,
            declaration_body_end: (section.is_some() || declaration.body_end > declaration.line)
                .then_some(declaration.body_end),
            section_line: section
                .and_then(|path| declaration.sections.get(path).map(|info| info.line)),
        }),
    )
}

/// The exact scanner-recorded site whose body is being sliced. `show` supplies
/// it after establishing uniqueness so an assigned scanner body span bounds the
/// source slice too; size rows additionally use it to keep duplicate homes and
/// duplicate section coordinates site-local (§FS-show.2.1.2.1, §FS-list.3.4).
#[derive(Clone, Copy)]
pub(super) struct PointBodySite {
    pub(super) declaration_line: usize,
    /// Present when the scanner computed a meaningful body span. A read-only
    /// scan without sections retains its lazy single-line placeholder and lets
    /// the slicer derive that declaration's boundary itself (§AR-benchmarks).
    pub(super) declaration_body_end: Option<usize>,
    pub(super) section_line: Option<usize>,
}

/// Per-operation source cache shared by list and check point measurements. It
/// owns no parsing rules: the cached bytes still flow through the exact show
/// slicer below (§FS-list.3.4.1, §FS-declarations.checks.oversized-lead).
pub(crate) struct PointBodyCache<'a> {
    overlays: &'a TextOverlays,
    text: BTreeMap<PathBuf, String>,
}

impl<'a> PointBodyCache<'a> {
    pub(crate) fn new(overlays: &'a TextOverlays) -> Self {
        Self {
            overlays,
            text: BTreeMap::new(),
        }
    }

    fn read(&mut self, path: &Path) -> Result<&str> {
        if !self.text.contains_key(path) {
            let text = read_text_with_overlays(path, self.overlays)?;
            self.text.insert(path.to_path_buf(), text);
        }
        Ok(self.text.get(path).expect("cached point source"))
    }
}

/// Shared show slicer, optionally pinned to one scanner-recorded site so the
/// size catalog can expose duplicates without making them resolvable
/// (§FS-show.2.1, §FS-list.3.4).
pub(super) fn extract_declaration_body_cached(
    cache: &mut PointBodyCache<'_>,
    path: &Path,
    id: &Id,
    section: Option<&str>,
    mode: ShowRenderMode,
    include_heading: bool,
    schema: &Schema,
    frame: Frame<'_>,
    site: Option<PointBodySite>,
) -> Result<ShowOutput> {
    // `--toc` = the default lead, then a blank line, then the nested section
    // headings (§FS-show.2.1.2). Internally: compose the Default body with an
    // Outline-only scan, sharing the same `(path, id, section)` resolution.
    if mode == ShowRenderMode::Toc {
        let mut default_output = extract_declaration_body_cached(
            cache,
            path,
            id,
            section,
            ShowRenderMode::Default,
            include_heading,
            schema,
            frame,
            site,
        )?;
        let outline_output = extract_declaration_body_cached(
            cache,
            path,
            id,
            section,
            ShowRenderMode::Outline,
            false,
            schema,
            frame,
            site,
        )?;
        default_output.body = join_with_blank(&default_output.body, &outline_output.body);
        default_output.sections = outline_output.sections;
        return Ok(default_output);
    }

    // `--brief` = heading + first paragraph (§FS-show.2.1.1); the H1 is
    // suppressed when a section is selected.
    if mode == ShowRenderMode::Brief {
        let want_h1_for_default = section.is_none();
        let mut output = extract_declaration_body_cached(
            cache,
            path,
            id,
            section,
            ShowRenderMode::Default,
            want_h1_for_default,
            schema,
            frame,
            site,
        )?;
        output.body = truncate_to_first_paragraph(&output.body);
        return Ok(output);
    }

    let text = cache.read(path)?;
    let is_md = path.extension().and_then(|e| e.to_str()) == Some("md");
    let is_py = path.extension().and_then(|e| e.to_str()) == Some("py");
    let mut in_decl = false;
    let mut line_style_comment = false;
    let mut own_marker: Option<String> = None;
    let mut block_closed = false;
    let mut py_docstring = PythonDocstringScanState::default();
    let mut found_section = section.is_none();
    let mut target_depth = usize::MAX;
    let mut lines = Vec::new();
    let mut sections = Vec::new();
    let mut output_line = 1;
    let mut markdown_blocks = MarkdownBlocks::default();

    for (idx, line) in text.lines().enumerate() {
        let lineno = idx + 1;
        // §FS-show.2.3.1.2: the `*/` closing the declaration's own block ends it.
        if in_decl && block_closed {
            break;
        }
        if in_decl
            && site
                .and_then(|site| site.declaration_body_end)
                .is_some_and(|body_end| lineno > body_end)
        {
            break;
        }
        let scan = source_scan_line(
            line,
            is_py,
            schema.sources.docstring_python,
            &mut py_docstring,
        );
        let scan_line = scan.text.as_ref();
        if in_decl
            && scan.in_py_docstring
            && scan.closed_py_docstring
            && scan_line.trim().is_empty()
        {
            break;
        }
        // §FS-show.2.5: inside a Markdown fence or a raw-text HTML block nothing is
        // structure (§FS-check.1.1.5.1). Such lines are body text and still reach
        // `lines` below.
        let fenced = is_md && !markdown_blocks.line(line).may_be_heading();
        if !fenced
            && let Some((found, _)) =
                declaration_id_on_line(frame.grammar(), scan_line, scan.in_py_docstring, is_md)
        {
            if in_decl && (site.is_some() || &found != id) {
                break;
            }
            let selected_declaration = site
                .map(|site| site.declaration_line == lineno)
                .unwrap_or(true);
            if &found == id && selected_declaration {
                in_decl = true;
                line_style_comment = is_line_style_comment_line(scan_line);
                // §FS-show.2.3.1.1: a `--` or `;` body is read behind its own marker.
                own_marker = (!is_md && !scan.in_py_docstring)
                    .then(|| own_line_marker(scan_line, frame.compiled.lexical(schema)))
                    .flatten();
                block_closed = closes_comment_block(
                    scan_line,
                    line_style_comment,
                    scan.in_py_docstring,
                    is_md,
                );
                output_line = lineno;
                // `md` format keeps the heading verbatim — including for `--brief`,
                // which then prints heading + first paragraph (§FS-show.3.1.2).
                if include_heading {
                    lines.push(clean_envelope_line(
                        scan_line,
                        is_md || scan.in_py_docstring,
                        own_marker.as_deref(),
                    ));
                }
                if scan.closed_py_docstring {
                    break;
                }
                continue;
            }
        }
        if !in_decl {
            continue;
        }
        if !is_md {
            let blank = line.trim().is_empty();
            if scan.in_py_docstring {
                // Python docstring content is plain Markdown; delimiter-only
                // triple-quote lines are skipped by `source_scan_line`
                // (§AR-scanner.4.4, §FS-show.2.3.2).
            } else if blank {
                // A blank line ends a line-style comment block (`//`, `#`, …);
                // inside a `/* … */` block or a docstring it is part of the body
                // (§FS-show.2.3.1.2).
                if line_style_comment {
                    break;
                }
            } else if !continues_comment_body(
                scan_line,
                own_marker.as_deref(),
                frame.compiled.lexical(schema),
            ) {
                break;
            }
            block_closed =
                closes_comment_block(scan_line, line_style_comment, scan.in_py_docstring, is_md);
        }
        if !fenced && let Some(caps) = frame.grammar().section_re.captures(scan_line) {
            let sec = section_path(&caps).unwrap_or("");
            let depth = sec.split('.').count();
            match section {
                // Whole-declaration lead: stop at the first citable subsection.
                None => {
                    if mode == ShowRenderMode::Default {
                        break;
                    }
                    if mode == ShowRenderMode::Outline {
                        push_outline_section(
                            &mut lines,
                            &mut sections,
                            scan_line,
                            sec,
                            depth,
                            is_md || scan.in_py_docstring,
                            own_marker.as_deref(),
                        );
                        continue;
                    }
                }
                Some(target) => {
                    // `!found_section`: a *second* heading claiming the requested
                    // path terminates the section rather than continuing it
                    // (§DF-duplicate-section-path.1, §FS-show.2.2.2).
                    let selected_section = site
                        .and_then(|site| site.section_line)
                        .map(|line| line == lineno)
                        .unwrap_or(true);
                    if sec == target && !found_section && selected_section {
                        found_section = true;
                        target_depth = depth;
                        output_line = lineno;
                        if mode != ShowRenderMode::Outline {
                            lines.push(clean_envelope_line(
                                scan_line,
                                is_md || scan.in_py_docstring,
                                own_marker.as_deref(),
                            ));
                        }
                        continue;
                    }
                    // Inside the target section: a sibling-or-shallower heading ends
                    // it, and in default mode any further numbered heading — including
                    // a child — ends the section's lead prose (§FS-show.2.2).
                    if found_section && (mode == ShowRenderMode::Default || depth <= target_depth) {
                        break;
                    }
                    if found_section && mode == ShowRenderMode::Outline {
                        push_outline_section(
                            &mut lines,
                            &mut sections,
                            scan_line,
                            sec,
                            depth - target_depth,
                            is_md || scan.in_py_docstring,
                            own_marker.as_deref(),
                        );
                        continue;
                    }
                }
            }
        }
        if found_section && mode != ShowRenderMode::Outline {
            lines.push(clean_envelope_line(
                scan_line,
                is_md || scan.in_py_docstring,
                own_marker.as_deref(),
            ));
        }
        if in_decl && scan.closed_py_docstring {
            break;
        }
    }

    if !in_decl {
        return Err(anyhow!("ID not found: {}", render_id(frame.grammar(), id)));
    }
    if !found_section {
        return Err(anyhow!(
            "section not found: {}{}{}",
            render_id(frame.grammar(), id),
            schema.ids.section_separator,
            section.unwrap_or("")
        ));
    }
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    let body = if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    };
    Ok(ShowOutput {
        body,
        path: path.to_path_buf(),
        line: output_line,
        json: None,
        sections,
    })
}

fn read_text_with_overlays(path: &Path, overlays: &TextOverlays) -> Result<String> {
    if let Some(text) = overlay_text(overlays, path) {
        Ok(text.to_string())
    } else {
        // §FS-check.6.1.1: cover this effective input before its shared read.
        crate::config::input_read_to_string(path)
            .with_context(|| format!("read {}", path.display()))
    }
}
