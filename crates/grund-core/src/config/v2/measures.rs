//! The measure tables (§FS-config-v2.schema.measures): the measure names the
//! table and each strength key is a threshold. A table is ordered once its keys
//! are read — a stronger strength takes the looser threshold — and lowered into
//! v1's budgets where v1 has one, so the checker measures what it always did and
//! only the channel moves with the strength (§FS-config-v2.mapping).

use anyhow::Result;

use super::defaults::{MAX_COLUMNS, MAX_LINES};
use super::tables::{Measure, Written};
use super::walk::Reader;
use crate::config::point_sizes::{LeadSizeWarning, PointSizeUnit};
use crate::config::project::{Strength, Threshold};
use crate::config::record::ConfigLocation;
use crate::config::v1::{parse_bool, parse_string, parse_usize};

/// One strength key of a measure table. Measures take positive strengths only
/// (§FS-config-v2.rules.strengths), so a prohibition is an unknown key.
pub(super) fn measure_key(
    r: &mut Reader,
    written: &mut Vec<Written>,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let Some(strength) = Strength::parse(key) else {
        return Ok(false);
    };
    let value = parse_usize(r.path, line, value)?;
    written.push(Written {
        strength,
        value,
        line,
    });
    Ok(true)
}

/// §FS-config-v2.schema.measures: order the table's thresholds, then lower them.
pub(super) fn close_measure(r: &mut Reader, measure: Measure, written: &[Written]) -> Result<()> {
    let mut ranked = written.to_vec();
    ranked.sort_by_key(|written| written.strength as u8);
    for (index, stronger) in ranked.iter().enumerate() {
        if let Some(weaker) = ranked[index + 1..]
            .iter()
            .find(|weaker| stronger.value < weaker.value)
        {
            return r.fail(stronger.line, ordering(stronger, weaker, ""));
        }
    }
    // A softer threshold written alone is still ordered against the `must` the
    // epoch fixes, at the key that crosses it.
    let default_must = match measure {
        Measure::Lines => Some(MAX_LINES),
        Measure::Columns => Some(MAX_COLUMNS),
        Measure::Words => None,
    };
    if let Some(value) = default_must
        && !ranked
            .iter()
            .any(|written| written.strength == Strength::Must)
        && let Some(weaker) = ranked.iter().find(|weaker| weaker.value > value)
    {
        let must = Written {
            strength: Strength::Must,
            value,
            line: weaker.line,
        };
        return r.fail(weaker.line, ordering(&must, weaker, ", the v2 default"));
    }
    lower(r, measure, &ranked);
    Ok(())
}

fn ordering(stronger: &Written, weaker: &Written, note: &str) -> String {
    format!(
        "threshold ordering: `{}` ({}{note}) is stricter than `{}` ({}); a stronger strength takes the looser threshold",
        stronger.strength.as_str(),
        stronger.value,
        weaker.strength.as_str(),
        weaker.value
    )
}

/// The thresholds into the budgets, strongest first. `must` is v1's hard cap;
/// for lines the guidance is the weakest of `may`, `should`, `warn`, and a
/// `warn` guidance is v1's `warn_on_suggested`; a lead's `warn` is v1's
/// `lead_size_warning`. Every other threshold is kept beside them.
fn lower(r: &mut Reader, measure: Measure, ranked: &[Written]) {
    let path = r.path;
    let threshold = |written: &Written| Threshold {
        strength: written.strength,
        value: written.value,
        source: Some(ConfigLocation {
            path: path.to_path_buf(),
            line: written.line,
        }),
    };
    let must = ranked.iter().find(|w| w.strength == Strength::Must);
    let rest: Vec<&Written> = ranked
        .iter()
        .filter(|w| w.strength != Strength::Must)
        .collect();
    let schema = &mut r.project.schema;
    match measure {
        Measure::Lines => {
            let notes = &mut schema.notes;
            if let Some(must) = must {
                notes.max_lines = must.value;
                notes.max_lines_source = threshold(must).source;
            }
            let guidance = rest.last().copied();
            if let Some(guidance) = guidance {
                notes.suggested_lines = guidance.value;
                notes.suggested_lines_source = threshold(guidance).source;
                notes.warn_on_suggested = guidance.strength == Strength::Warn;
            }
            let warned = notes.warn_on_suggested;
            notes.extra_lines = rest
                .iter()
                .filter(|w| w.strength != Strength::Warn || !warned)
                .map(|w| threshold(w))
                .collect();
        }
        Measure::Columns => {
            if let Some(must) = must {
                schema.notes.max_columns = must.value;
            }
            schema.notes.extra_columns = rest.iter().map(|w| threshold(w)).collect();
        }
        Measure::Words => {
            schema.leads = ranked
                .iter()
                .find(|w| w.strength == Strength::Warn)
                .map(|warn| LeadSizeWarning {
                    max: warn.value,
                    unit: PointSizeUnit::Words,
                });
            schema.lead_thresholds = ranked
                .iter()
                .filter(|w| w.strength != Strength::Warn)
                .map(&threshold)
                .collect();
        }
    }
}

/// `[schema.notes.text]`: `must = false` is v1's `citation-only`.
pub(super) fn text_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    if key != "must" {
        return Ok(false);
    }
    let text = parse_bool(r.path, line, value)?;
    r.project.schema.notes.inline_style = if text {
        "citation-with-note"
    } else {
        "citation-only"
    }
    .to_string();
    Ok(true)
}

/// `[schema.notes.layout]`: one strength, whose value is the layout and whose
/// channel is v1's `inline_note_layout_check`; `may` states it unchecked.
pub(super) fn layout_key(
    r: &mut Reader,
    first: &mut Option<usize>,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let Some(strength) = Strength::parse(key) else {
        return Ok(false);
    };
    if let Some(first) = first {
        return r.fail(
            line,
            format!(
                "[schema.notes.layout] takes one strength (`{key}` beside the one at line {first})"
            ),
        );
    }
    *first = Some(line);
    let layout = parse_string(r.path, line, value)?;
    if !matches!(layout.as_str(), "any" | "citation-first-colon") {
        return r.fail(
            line,
            format!(
                "unknown [schema.notes.layout] layout `{layout}` (expected any or citation-first-colon)"
            ),
        );
    }
    let notes = &mut r.project.schema.notes;
    notes.layout = layout;
    notes.layout_check = match strength {
        Strength::Must => "error",
        Strength::Warn => "warn",
        Strength::Should => "suggest",
        Strength::May => "off",
    }
    .to_string();
    Ok(true)
}
