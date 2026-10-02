//! What classifies a delimited form as an attempted value binding: the record
//! an invalid attempt leaves, and the recovery pass that finds the attempts the
//! ordinary citation scanner did not produce a binding record for
//! (§FS-values.3.1.1, §AR-scanner.3).

use std::collections::BTreeSet;
use std::path::Path;

use super::citation_line::CitationLine;
use super::value_context::binding_span_is_inside;
use crate::config::Config;
use crate::grammar::{QUALIFIED_CITATION_PREFIX, parse_id_arg, parse_longest_id_prefix};
use crate::model::{DeclarationSource, Id, InvalidValueSite};
use crate::workspace::WorkspaceCitationTarget;

/// Find a backtick-delimited literal followed immediately by a reference to an
/// opted-in value kind even when punctuation, spacing, marker, or field syntax
/// kept the ordinary citation scanner from producing the exact binding record.
/// Unbackticked adjacent prose and bare citations deliberately never enter this
/// pass (§FS-values.3.1.1).
pub(super) fn scan_noncanonical_value_binding_attempts(
    line: &CitationLine<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
    context: (usize, usize),
    classified_openings: &BTreeSet<usize>,
    invalid: &mut Vec<InvalidValueSite>,
) {
    for (close_tick, _) in line.scan_line.match_indices('`') {
        if close_tick < context.0 || close_tick >= context.1 {
            continue;
        }
        let Some(tail) = line.scan_line.get(close_tick + 1..context.1) else {
            continue;
        };
        let Some(target) = attempted_value_target(tail, line.config, workspace_targets) else {
            continue;
        };
        // Without an opener on this physical line, this is the closing half of
        // a multiline literal—the binding is still an invalid attempted
        // delimited form (§FS-values.3.1.1).
        let open_tick = line.scan_line[..close_tick]
            .rfind('`')
            .unwrap_or(close_tick);
        if !binding_span_is_inside(context, open_tick, close_tick + 1) {
            continue;
        }
        if classified_openings.contains(&open_tick) {
            continue;
        }
        invalid.push(invalid_value_binding_site(
            line,
            target.namespace,
            Some(target.id),
            target.section,
            open_tick,
        ));
    }
}

pub(super) fn invalid_value_binding_site(
    line: &CitationLine<'_>,
    namespace: Option<String>,
    id: Option<Id>,
    section: Option<String>,
    open_tick: usize,
) -> InvalidValueSite {
    InvalidValueSite {
        id,
        file: line.path.to_path_buf(),
        line: line.lineno,
        column: Some(line.column_offset + open_tick + 1),
        message: "value binding must be exactly `literal` (marker-prefixed full value ID with one positive numeric field)"
            .to_string(),
        source: DeclarationSource::Text,
        binding_namespace: namespace,
        binding_section: section,
    }
}

/// The record a binding split by a line break leaves: an attempt like every
/// other, located at the literal on the line before its citation rather than on
/// the line being scanned, and asking for the two to be joined instead of
/// quoting the grammar (§FS-values.3.1.1.1).
pub(super) fn split_value_binding_site(
    path: &Path,
    (line, column): (usize, usize),
    target: AttemptedValueTarget,
) -> InvalidValueSite {
    InvalidValueSite {
        id: Some(target.id),
        file: path.to_path_buf(),
        line,
        column: Some(column),
        message: "value binding must be on one physical line; join the literal and its citation"
            .to_string(),
        source: DeclarationSource::Text,
        binding_namespace: target.namespace,
        binding_section: target.section,
    }
}

pub(super) struct AttemptedValueTarget {
    namespace: Option<String>,
    id: Id,
    section: Option<String>,
}

pub(super) fn attempted_value_target(
    tail: &str,
    local: &Config,
    workspace_targets: &[WorkspaceCitationTarget],
) -> Option<AttemptedValueTarget> {
    let mut rest = tail;
    if rest.starts_with(|ch: char| ch.is_whitespace()) {
        rest = rest.trim_start_matches(|ch: char| ch.is_whitespace());
    }
    if let Some(trimmed) = rest.strip_prefix('(') {
        rest = trimmed;
    }
    rest = rest.trim_start_matches(|ch: char| ch.is_whitespace());
    rest = rest.strip_prefix(&local.marker).unwrap_or(rest);

    if let Some(prefix) = QUALIFIED_CITATION_PREFIX.captures(rest) {
        let alias = prefix.name("namespace")?.as_str();
        let target = workspace_targets
            .iter()
            .find(|target| target.alias == alias)?;
        let id_rest = &rest[prefix.get(0)?.end()..];
        let (id, section) = attempted_value_id(id_rest, &target.config)?;
        return Some(AttemptedValueTarget {
            namespace: Some(alias.to_string()),
            id,
            section,
        });
    }

    let (id, section) = attempted_value_id(rest, local)?;
    Some(AttemptedValueTarget {
        namespace: None,
        id,
        section,
    })
}

fn attempted_value_id(raw: &str, config: &Config) -> Option<(Id, Option<String>)> {
    if let Some(parsed) = parse_longest_id_prefix(raw, &config.grammar) {
        return Some((parsed.id, parsed.section));
    }
    // A named address is deliberately outside the ordinary numeric-only
    // grammar, so recover only the complete ID before the separator to classify
    // the delimited form as an attempted value binding (§FS-values.3.1.1).
    raw.match_indices(&config.section_separator)
        .map(|(index, _)| index)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .find_map(|index| match parse_id_arg(&raw[..index], &config.grammar) {
            Ok((id, None)) => Some((id, None)),
            _ => None,
        })
}
