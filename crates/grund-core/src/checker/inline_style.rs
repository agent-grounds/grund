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

use crate::config::{Frame, Schema, Strength, Threshold};
use crate::grammar::{CITATION_RUN_SEPARATOR, LayoutChannel, layout_channel};
use crate::model::{Catalog, CheckReport, Citation, Diagnostic, InlineCitationSite, plural};

/// The citation tokens of one inline citation site, for the message a budget
/// finding names (§FS-inline-citation-style.4.1.2, §FS-inline-citation-style.4.2): each citation's `text`
/// exactly as written — marker, qualifier, section — in source order,
/// duplicates dropped after the first, chain-spelled with
/// `CITATION_RUN_SEPARATOR` the way §FS-inline-citation-style.3.3 already joins a citation run.
fn site_citation_texts(findings: &Catalog) -> BTreeMap<(&Path, usize), String> {
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
/// (§FS-inline-citation-style.4.1.2, §FS-inline-citation-style.4.2): the block's line span and the
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
/// carries — the block-splitting rule (§FS-inline-citation-style.1) the author needs to act on the site
/// clause above. The column cap omits it: a wide line is fixed by wrapping,
/// not by splitting.
const BLOCK_SPLIT_CLAUSE: &str = "; a blank line splits a note, an empty comment line does not";

/// §AR-checker.2.14: the inline citation style rule, a pure pass over
/// `findings.citations` deduplicated by site. The budgets and the note-presence
/// verdict are read off the site the scanner recorded, and so are the per-line
/// layout deviations, so nothing here re-reads a file
/// (§FS-inline-citation-style.4).
pub(super) fn check_inline_citation_style(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    // A site is identified by the file and the line it opens on — two blocks in
    // one file cannot share an opener — so the key stays two cheap fields rather
    // than a clone of the whole recorded site.
    let mut seen = BTreeSet::new();
    let layout_message = layout_violation_message(schema);
    let citation_texts = site_citation_texts(findings);
    // §FS-inline-citation-style.4.2: v1's opted-in soft cap is a `warn` budget;
    // v2 adds its own `warn` and `should` budgets (§FS-config-v2.schema.measures).
    let notes = &schema.notes;
    let mut soft_lines: Vec<(Strength, usize)> = notes
        .warn_on_suggested
        .then_some((Strength::Warn, notes.suggested_lines))
        .into_iter()
        .collect();
    soft_lines.extend(soft_thresholds(&notes.extra_lines));
    let soft_columns = soft_thresholds(&notes.extra_columns);
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
        match schema.notes.inline_style.as_str() {
            "citation-only" => {
                if site.has_note {
                    report.errors.push(Diagnostic {
                        code: "inline-citation-style",
                        path: Some(cite.file.clone()),
                        line: Some(site.first_line),
                        column: None,
                        message: "inline citation must carry no prose".to_string(),
                        sites: Vec::new(),
                        authority: Vec::new(),
                    });
                }
            }
            _ => {
                let lines = site.last_line - site.first_line + 1;
                if lines > schema.notes.max_lines {
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
                            schema.notes.max_lines,
                            site_clause(site.first_line, site.last_line, citations),
                        ),
                        sites: Vec::new(),
                    authority: Vec::new(),});
                }
                let columns_over_cap = site.max_columns > schema.notes.max_columns;
                if columns_over_cap {
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
                            schema.notes.max_columns,
                            site_clause(site.first_line, site.last_line, citations),
                        ),
                        sites: Vec::new(),
                        authority: Vec::new(),
                    });
                }
                if lines <= schema.notes.max_lines
                    && let Some((channel, limit)) = soft_limit_crossed(&soft_lines, lines)
                {
                    channel_of(report, channel).push(Diagnostic {
                        code: "inline-citation-style",
                        path: Some(cite.file.clone()),
                        line: Some(site.first_line),
                        column: None,
                        // §FS-inline-citation-style.4.2: names the measured size,
                        // the site, and the block-splitting rule
                        message: format!(
                            "inline note is {lines} line{}, over the {limit}-line preferred limit: {}{BLOCK_SPLIT_CLAUSE}",
                            plural(lines),
                            site_clause(site.first_line, site.last_line, citations),
                        ),
                        sites: Vec::new(),
                    authority: Vec::new(),});
                }
                // §FS-config-v2.schema.measures: a v2 column budget below the cap is
                // the cap's finding on its own strength's channel.
                if !columns_over_cap
                    && let Some((channel, limit)) =
                        soft_limit_crossed(&soft_columns, site.max_columns)
                {
                    channel_of(report, channel).push(Diagnostic {
                        code: "inline-citation-style",
                        path: Some(cite.file.clone()),
                        line: Some(site.first_line),
                        column: None,
                        message: format!(
                            "inline note is {} column{}, over the {limit}-column maximum: {}",
                            site.max_columns,
                            plural(site.max_columns),
                            site_clause(site.first_line, site.last_line, citations),
                        ),
                        sites: Vec::new(),
                        authority: Vec::new(),
                    });
                }
                report_layout_deviations(cite, site, schema, frame, &layout_message, report);
            }
        }
    }
}

/// The `warn` and `should` thresholds of one v2 measure (§FS-config-v2.schema.measures);
/// `must` is the cap the caller already reads, and `may` checks nothing.
fn soft_thresholds(thresholds: &[Threshold]) -> Vec<(Strength, usize)> {
    thresholds
        .iter()
        .filter(|threshold| matches!(threshold.strength, Strength::Warn | Strength::Should))
        .map(|threshold| (threshold.strength, threshold.value))
        .collect()
}

/// The strongest soft budget `measured` is over, with its threshold: one finding
/// per site, on the strongest channel it reaches (§FS-config-v2.rules.strengths).
fn soft_limit_crossed(limits: &[(Strength, usize)], measured: usize) -> Option<(Strength, usize)> {
    [Strength::Warn, Strength::Should]
        .into_iter()
        .find_map(|strength| {
            limits
                .iter()
                .filter(|(at, limit)| *at == strength && measured > *limit)
                .map(|(_, limit)| (strength, *limit))
                .max_by_key(|(_, limit)| *limit)
        })
}

/// The report channel a strength reaches (§FS-config-v2.rules.strengths).
fn channel_of(report: &mut CheckReport, strength: Strength) -> &mut Vec<Diagnostic> {
    match strength {
        Strength::Must => &mut report.errors,
        Strength::Warn => &mut report.warnings,
        Strength::Should | Strength::May => &mut report.suggestions,
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
    schema: &Schema,
    frame: Frame<'_>,
    message: &str,
    report: &mut CheckReport,
) {
    let channel = match layout_channel(frame.compiled.lexical(schema)) {
        Some(LayoutChannel::Warn) => &mut report.warnings,
        Some(LayoutChannel::Error) => &mut report.errors,
        // §FS-config-v2.schema.measures: `[schema.notes.layout] should`.
        Some(LayoutChannel::Suggest) => &mut report.suggestions,
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
            authority: Vec::new(),
        });
    }
}

/// The one message this rule emits, built with the configured marker so the form
/// it names is the form the project writes (§FS-inline-citation-style.4.4.2).
fn layout_violation_message(schema: &Schema) -> String {
    format!(
        "inline note must open with its citations and a colon ({}<ID>: note)",
        schema.citation.marker
    )
}
