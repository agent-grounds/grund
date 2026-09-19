//! The inline citation style rule (§AR-checker.2.14, §FS-inline-citation-style.4):
//! one pass over `findings.citations`, deduplicated by enclosing comment block,
//! judging each block against `[reference] inline_style`, the `inline_note_*`
//! budgets, and `inline_note_layout`.
//!
//! It sat in `grammar/inline_note_layout.rs` while both were file-name
//! categories, which left a checker rule inside the component §AR-system.2.1
//! says holds no rule. What stayed there is the classifier the scanner
//! annotates a site from — the layout a value selects, which lines are judged,
//! and whether a block carries a note at all — so the two stages still read one
//! answer and this file only turns the recorded verdicts into findings. Nothing
//! here reads a file: the span, the widest column, the note verdict and the
//! deviating lines all arrive on the site (§AR-scanner.3.1).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::config::Config;
use crate::grammar::{CITATION_RUN_SEPARATOR, LayoutChannel, layout_channel};
use crate::model::{CheckReport, Citation, Diagnostic, Findings, InlineCitationSite, plural};

/// The citation tokens of one inline citation site, for the message a budget
/// finding names (§FS-inline-citation-style.4.1.2, §4.2): each citation's `text`
/// exactly as written — marker, qualifier, section — in source order,
/// duplicates dropped after the first, chain-spelled with
/// `CITATION_RUN_SEPARATOR` the way §3.3 already joins a citation run.
fn site_citation_texts(findings: &Findings) -> BTreeMap<(&Path, usize), String> {
    let mut per_site: BTreeMap<(&Path, usize), Vec<&str>> = BTreeMap::new();
    for cite in &findings.citations {
        let Some(site) = &cite.inline_site else {
            continue;
        };
        let texts = per_site
            .entry((cite.file.as_path(), site.first_line))
            .or_default();
        if !texts.contains(&cite.text.as_str()) {
            texts.push(cite.text.as_str());
        }
    }
    per_site
        .into_iter()
        .map(|(key, texts)| (key, texts.join(CITATION_RUN_SEPARATOR)))
        .collect()
}

/// The site clause a budget finding appends to name what it measured
/// (§FS-inline-citation-style.4.1.2, §4.2): the block's line span and the
/// citations that made it a site, as written. A one-line site — only possible
/// for the column cap — reads `line N cites`; a longer one `lines A-B cite`.
fn site_clause(first_line: usize, last_line: usize, citations: &str) -> String {
    if first_line == last_line {
        format!("line {first_line} cites {citations}")
    } else {
        format!("lines {first_line}-{last_line} cite {citations}")
    }
}

/// §FS-inline-citation-style.4.1.3: the fix-it clause a line-count finding
/// carries — the block-splitting rule (§1) the author needs to act on the site
/// clause above. The column cap omits it: a wide line is fixed by wrapping,
/// not by splitting.
const BLOCK_SPLIT_CLAUSE: &str = "; a blank line splits a note, an empty comment line does not";

/// §AR-checker.2.14: the inline citation style rule, a pure pass over
/// `findings.citations` deduplicated by site. The budgets and the note-presence
/// verdict are read off the site the scanner recorded, and so are the per-line
/// layout deviations, so nothing here re-reads a file
/// (§FS-inline-citation-style.4).
pub(super) fn check_inline_citation_style(
    findings: &Findings,
    config: &Config,
    report: &mut CheckReport,
) {
    // A site is identified by the file and the line it opens on — two blocks in
    // one file cannot share an opener — so the key stays two cheap fields rather
    // than a clone of the whole recorded site.
    let mut seen = BTreeSet::new();
    let layout_message = layout_violation_message(config);
    let citation_texts = site_citation_texts(findings);
    for cite in &findings.citations {
        let Some(site) = &cite.inline_site else {
            continue;
        };
        if !seen.insert((cite.file.as_path(), site.first_line)) {
            continue;
        }
        let citations = citation_texts
            .get(&(cite.file.as_path(), site.first_line))
            .map(String::as_str)
            .unwrap_or_default();
        match config.inline_style.as_str() {
            "citation-only" => {
                if site.has_note {
                    report.errors.push(Diagnostic {
                        code: "inline-citation-style",
                        path: Some(cite.file.clone()),
                        line: Some(site.first_line),
                        column: None,
                        message: "inline citation must carry no prose".to_string(),
                        sites: Vec::new(),
                    });
                }
            }
            _ => {
                let lines = site.last_line - site.first_line + 1;
                if lines > config.inline_note_max_lines {
                    report.errors.push(Diagnostic {
                        code: "inline-citation-style",
                        path: Some(cite.file.clone()),
                        line: Some(site.first_line),
                        column: None,
                        // §FS-inline-citation-style.4.1: names the measured size,
                        // the site, and the block-splitting rule
                        message: format!(
                            "inline note is {lines} line{}, over the {}-line maximum: {}{BLOCK_SPLIT_CLAUSE}",
                            plural(lines),
                            config.inline_note_max_lines,
                            site_clause(site.first_line, site.last_line, citations),
                        ),
                        sites: Vec::new(),
                    });
                }
                if site.max_columns > config.inline_note_max_columns {
                    report.errors.push(Diagnostic {
                        code: "inline-citation-style",
                        path: Some(cite.file.clone()),
                        line: Some(site.first_line),
                        column: None,
                        // §FS-inline-citation-style.4.1: names the measured size
                        // and the site
                        message: format!(
                            "inline note is {} column{}, over the {}-column maximum: {}",
                            site.max_columns,
                            plural(site.max_columns),
                            config.inline_note_max_columns,
                            site_clause(site.first_line, site.last_line, citations),
                        ),
                        sites: Vec::new(),
                    });
                }
                if config.warn_on_suggested
                    && lines > config.inline_note_suggested_lines
                    && lines <= config.inline_note_max_lines
                {
                    report.warnings.push(Diagnostic {
                        code: "inline-citation-style",
                        path: Some(cite.file.clone()),
                        line: Some(site.first_line),
                        column: None,
                        // §FS-inline-citation-style.4.2: names the measured size,
                        // the site, and the block-splitting rule
                        message: format!(
                            "inline note is {lines} line{}, over the {}-line preferred limit: {}{BLOCK_SPLIT_CLAUSE}",
                            plural(lines),
                            config.inline_note_suggested_lines,
                            site_clause(site.first_line, site.last_line, citations),
                        ),
                        sites: Vec::new(),
                    });
                }
                report_layout_deviations(cite, site, config, &layout_message, report);
            }
        }
    }
}

/// §FS-inline-citation-style.4.4: one finding per nonconforming line, anchored at
/// that line rather than at the site's opener — the only member of this rule that
/// does, because a layout deviation is a property of the line an author edits. The
/// level picks the channel and nothing else: the message is identical under `warn`
/// and `error`, so migrating a project between them re-reads nothing.
fn report_layout_deviations(
    cite: &Citation,
    site: &InlineCitationSite,
    config: &Config,
    message: &str,
    report: &mut CheckReport,
) {
    let channel = match layout_channel(config.lexical()) {
        Some(LayoutChannel::Warn) => &mut report.warnings,
        Some(LayoutChannel::Error) => &mut report.errors,
        None => return,
    };
    for line in &site.layout_violations {
        channel.push(Diagnostic {
            code: "inline-citation-style",
            path: Some(cite.file.clone()),
            line: Some(*line),
            column: None,
            message: message.to_string(),
            sites: Vec::new(),
        });
    }
}

/// The one message this rule emits, built with the configured marker so the form
/// it names is the form the project writes (§FS-inline-citation-style.4.4.2).
fn layout_violation_message(config: &Config) -> String {
    format!(
        "inline note must open with its citations and a colon ({}<ID>: note)",
        config.marker
    )
}
