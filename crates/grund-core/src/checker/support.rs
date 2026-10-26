use crate::config::{Frame, Schema};
#[cfg(test)]
use crate::grammar::render_id;
use crate::grammar::{is_inside_inline_code, render_catalog_id};
use crate::model::remedies;
use crate::model::{Catalog, Citation, Diagnostic, Id, sort_path_key};

/// §FS-check.3.1: the dangling message. A near same-kind ID is a likely typo; a
/// Markdown inline-code context is a likely illustration. Offer whichever
/// applies — and both when a dangling citation in backticks also has a near
/// match. Outside inline code the escape hint is withheld so a prose typo is
/// nudged toward the near ID, not toward escaping.
#[cfg(test)]
pub(crate) fn dangling_message(
    schema: &Schema,
    frame: Frame<'_>,
    namespace: Option<&str>,
    findings: &Catalog,
    missing: &Id,
    in_inline_code: bool,
) -> String {
    dangling_message_spelled(
        schema,
        frame,
        namespace,
        findings,
        missing,
        in_inline_code,
        &render_id(frame.grammar(), missing),
    )
}

/// The source citation supplies the spelling of an unresolved number
/// (§FS-config.3.2.8); identity still selects same-kind suggestions.
pub(super) fn dangling_message_spelled(
    schema: &Schema,
    frame: Frame<'_>,
    namespace: Option<&str>,
    findings: &Catalog,
    missing: &Id,
    in_inline_code: bool,
    spelling: &str,
) -> String {
    let unknown = qualify(namespace, spelling);
    let near = nearest_declared_id(frame, namespace, findings, missing, spelling);
    let escape = in_inline_code.then(|| format!("<{}>{unknown}", schema.citation.marker));
    // §FS-errors.5.5.1: the case of dangling's Fix the search and the site pick.
    match (near, escape) {
        (Some(near), Some(escape)) => format!(
            "unknown reference {unknown}; {}",
            remedies::dangling_near_escape(near, escape)
        ),
        (Some(near), None) => {
            format!(
                "unknown reference {unknown}; {}",
                remedies::dangling_near(near)
            )
        }
        (None, Some(escape)) => {
            format!(
                "unknown reference {unknown}; {}",
                remedies::dangling_escape(escape)
            )
        }
        (None, None) => format!("unknown reference {unknown}"),
    }
}

/// §FS-check.3.1.3 / §FS-check.4.12.1: a missing declaration in a fetch-enabled
/// home. Existing typo/illustration hints replace the fetch action while the
/// snapshot-specific base and fixed finding class remain.
pub(super) fn missing_snapshot_message(
    schema: &Schema,
    frame: Frame<'_>,
    namespace: Option<&str>,
    findings: &Catalog,
    missing: &Id,
    in_inline_code: bool,
    home: &str,
    must: bool,
    spelling: &str,
) -> String {
    let rendered = qualify(namespace, spelling);
    let base = if must {
        format!("unknown reference {rendered}; no snapshot in {home}")
    } else {
        format!("no snapshot for {rendered} in {home}")
    };
    let near = nearest_declared_id(frame, namespace, findings, missing, spelling);
    let escape = in_inline_code.then(|| format!("<{}>{rendered}", schema.citation.marker));
    // §FS-errors.5.5.1: `must` reports dangling, otherwise missing-snapshot, and each
    // message carries its own code's case.
    let remedy = match (near, escape, must) {
        (Some(near), Some(escape), true) => remedies::dangling_near_escape(near, escape),
        (Some(near), Some(escape), false) => remedies::missing_snapshot_near_escape(near, escape),
        (Some(near), None, true) => remedies::dangling_near(near),
        (Some(near), None, false) => remedies::missing_snapshot_near(near),
        (None, Some(escape), true) => remedies::dangling_escape(escape),
        (None, Some(escape), false) => remedies::missing_snapshot_escape(escape),
        (None, None, true) => return format!("{base} — {}", remedies::dangling_fetch(rendered)),
        (None, None, false) => {
            return format!("{base} — {}", remedies::missing_snapshot_fetch(rendered));
        }
    };
    format!("{base}; {remedy}")
}

/// §FS-check.3.1.2: whether a citation site sits inside a Markdown inline-code
/// span, the signal that a dangling `§`-citation may be an illustration. Only
/// the rare dangling path asks, so this reads the one line out of the file record
/// the scan kept (§AR-scanner.2.8) rather than widening every `Citation`; source
/// files (columns shifted by stripped comment prefixes) never qualify, and
/// `markdown` is whether the citing file reads as Markdown. Any bounds failure
/// yields `false` — no hint.
pub(super) fn citation_in_markdown_inline_code(
    findings: &Catalog,
    frame: Frame<'_>,
    cite: &Citation,
    markdown: bool,
) -> bool {
    if !markdown {
        return false;
    }
    // §AR-system.2.13: the record holds what a save would write, an open
    // buffer's included; a file without one is read as the run reads it.
    let line = match findings.files.get(&cite.file) {
        Some(record) => record.line(cite.line).map(str::to_string),
        // §FS-check.6.1.1: cover this effective input before its shared read.
        None => frame.sources().text(&cite.file).ok().and_then(|text| {
            let line = text.lines().nth(cite.line.saturating_sub(1))?;
            Some(line.to_string())
        }),
    };
    let Some(line) = line else {
        return false;
    };
    let pos = cite.column.saturating_sub(1);
    pos <= line.len() && is_inside_inline_code(&line, pos)
}

fn nearest_declared_id(
    frame: Frame<'_>,
    namespace: Option<&str>,
    findings: &Catalog,
    missing: &Id,
    missing_text: &str,
) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    for candidate in findings.declarations.keys() {
        if candidate.kind != missing.kind {
            continue;
        }
        let candidate_text = render_catalog_id(frame.grammar(), &findings.declarations, candidate);
        let distance = edit_distance(missing_text, &candidate_text);
        if !close_enough_for_hint(
            distance,
            missing_text.chars().count(),
            candidate_text.chars().count(),
        ) {
            continue;
        }
        let rendered = qualify(namespace, &candidate_text);
        match &best {
            Some((best_distance, best_rendered))
                if distance > *best_distance
                    || (distance == *best_distance && rendered >= *best_rendered) => {}
            _ => best = Some((distance, rendered)),
        }
    }
    best.map(|(_, rendered)| rendered)
}

/// Namespace qualification does not change authored digits (§FS-config.3.2.8).
pub(super) fn qualify(namespace: Option<&str>, spelling: &str) -> String {
    namespace.map_or_else(
        || spelling.to_string(),
        |alias| format!("{alias}/{spelling}"),
    )
}

pub(super) fn close_enough_for_hint(distance: usize, left_len: usize, right_len: usize) -> bool {
    distance > 0 && distance <= 3 && distance * 3 <= left_len.max(right_len)
}

pub(super) fn edit_distance(left: &str, right: &str) -> usize {
    let right_chars: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right_chars.len()).collect();
    let mut current = vec![0; right_chars.len() + 1];

    for (i, left_char) in left.chars().enumerate() {
        current[0] = i + 1;
        for (j, right_char) in right_chars.iter().enumerate() {
            let substitution = previous[j] + usize::from(left_char != *right_char);
            let insertion = current[j] + 1;
            let deletion = previous[j + 1] + 1;
            current[j + 1] = substitution.min(insertion).min(deletion);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right_chars.len()]
}

pub(super) fn section_depth(section_path: &str) -> usize {
    section_path.split('.').count()
}

pub(super) fn heading_marks(level: usize) -> String {
    "#".repeat(level)
}

/// Put diagnostics in the one fixed order `grund` ever prints them in — by path, then
/// line, then message text — so two runs over the same tree agree byte-for-byte
/// (§FS-errors.4.1) and ordering is not a knob (§FS-non-goals.9).
pub(crate) fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(diagnostic_cmp);
}

pub(crate) fn diagnostic_cmp(a: &Diagnostic, b: &Diagnostic) -> std::cmp::Ordering {
    (
        a.path.as_ref().map(|p| sort_path_key(p)),
        a.line.unwrap_or(0),
        a.message.as_str(),
    )
        .cmp(&(
            b.path.as_ref().map(|p| sort_path_key(p)),
            b.line.unwrap_or(0),
            b.message.as_str(),
        ))
}
